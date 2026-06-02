use rand::Rng;
use rayon::prelude::*;
use serde_json::Value;
use dashmap::DashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct SearchHit {
    pub text: String,
    pub score: f32,
}

pub struct Net {
    agent: ureq::Agent,
    cache: DashMap<String, String>,
}

impl Net {
    pub fn new() -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(12))
            .build();
        Self {
            agent,
            cache: DashMap::new(),
        }
    }

    pub fn cached(&self, query: &str) -> Option<String> {
        self.cache
            .get(&query.to_ascii_lowercase())
            .map(|r| r.value().clone())
    }

    pub fn search(&self, query: &str) -> Result<String, String> {
        let key = query.to_ascii_lowercase();
        if let Some(hit) = self.cached(query) {
            return Ok(hit);
        }

        let url = format!(
            "https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
            urlencoding::encode(query)
        );

        let body = self
            .agent
            .get(&url)
            .call()
            .map_err(|e| e.to_string())?
            .into_string()
            .map_err(|e| e.to_string())?;

        let v: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
        let info = extract_ddg(&v).unwrap_or_else(|| crate::wired::EMPTY_NET.to_string());

        if self.cache.len() > 500 {
            self.cache.clear();
        }
        self.cache.insert(key, info.clone());
        Ok(info)
    }

    /// Multi-source fetch with relevance scoring (rayon over candidates).
    pub fn search_many(&self, budget: &NetBudget, query: &str, max: usize) -> Vec<SearchHit> {
        let body = match budget.try_fetch_raw(self, query) {
            Some(b) => b,
            None => return Vec::new(),
        };
        let Ok(v) = serde_json::from_str::<Value>(&body) else {
            return Vec::new();
        };

        let mut raw = extract_all_topics(&v);
        if let Some(primary) = extract_ddg(&v) {
            raw.insert(0, primary);
        }
        if raw.is_empty() {
            raw.push(crate::wired::EMPTY_NET.to_string());
        }

        let qtok: Vec<String> = tokenize(query);
        let scored: Vec<SearchHit> = raw
            .par_iter()
            .map(|text| SearchHit {
                score: relevance(&qtok, text),
                text: text.clone(),
            })
            .collect();

        let mut scored = scored;
        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        scored.truncate(max);
        scored
    }

    fn fetch_json(&self, query: &str) -> Result<String, String> {
        if let Some(hit) = self.cached(query) {
            return Ok(serde_json::json!({"cached": hit}).to_string());
        }
        let url = format!(
            "https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
            urlencoding::encode(query)
        );
        self.agent
            .get(&url)
            .call()
            .map_err(|e| e.to_string())?
            .into_string()
            .map_err(|e| e.to_string())
    }
}

fn extract_ddg(v: &Value) -> Option<String> {
    let mut out = String::new();
    if let Some(s) = v.get("Abstract").and_then(|x| x.as_str()) {
        if !s.is_empty() {
            out.push_str(s);
        }
    }
    if let Some(s) = v.get("Definition").and_then(|x| x.as_str()) {
        if !s.is_empty() {
            if !out.is_empty() {
                out.push_str(" | ");
            }
            out.push_str(s);
        }
    }
    if out.is_empty() {
        if let Some(topics) = v.get("RelatedTopics").and_then(|x| x.as_array()) {
            for t in topics {
                if let Some(text) = t.get("Text").and_then(|x| x.as_str()) {
                    out.push_str(text);
                    break;
                }
            }
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn extract_all_topics(v: &Value) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(topics) = v.get("RelatedTopics").and_then(|x| x.as_array()) {
        for t in topics {
            if let Some(text) = t.get("Text").and_then(|x| x.as_str()) {
                if !text.is_empty() {
                    out.push(text.to_string());
                }
            }
            if let Some(subs) = t.get("Topics").and_then(|x| x.as_array()) {
                for st in subs {
                    if let Some(text) = st.get("Text").and_then(|x| x.as_str()) {
                        if !text.is_empty() {
                            out.push(text.to_string());
                        }
                    }
                }
            }
        }
    }
    out
}

fn tokenize(s: &str) -> Vec<String> {
    s.to_ascii_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 3)
        .map(String::from)
        .collect()
}

fn relevance(query: &[String], text: &str) -> f32 {
    if query.is_empty() {
        return 0.1;
    }

    // Convert text to lower case once
    let text_lower = text.to_ascii_lowercase();

    // Tokenize as slices instead of allocating Strings
    let ttok: Vec<&str> = text_lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 3)
        .collect();

    let hits = query
        .iter()
        .filter(|t| ttok.iter().any(|x| x.contains(t.as_str())))
        .count();
    hits as f32 / query.len() as f32
}

/// Lock-free shared budget for rayon swarm ticks.
pub struct NetBudget {
    remaining: AtomicUsize,
}

impl NetBudget {
    pub fn new(per_tick: usize) -> Self {
        Self {
            remaining: AtomicUsize::new(per_tick),
        }
    }

    pub fn reset(&self, per_tick: usize) {
        self.remaining.store(per_tick, Ordering::Relaxed);
    }

    pub fn try_fetch(&self, net: &Net, query: &str) -> Option<String> {
        self.try_fetch_raw(net, query)
            .and_then(|body| {
                serde_json::from_str::<Value>(&body)
                    .ok()
                    .and_then(|v| extract_ddg(&v))
            })
            .or_else(|| net.cached(query))
    }

    pub fn try_fetch_raw(&self, net: &Net, query: &str) -> Option<String> {
        loop {
            let r = self.remaining.load(Ordering::Relaxed);
            if r == 0 {
                return net
                    .cached(query)
                    .map(|s| serde_json::json!({"Abstract": s}).to_string());
            }
            if self
                .remaining
                .compare_exchange_weak(r, r - 1, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
            {
                return net.fetch_json(query).ok();
            }
        }
    }
}

#[allow(dead_code)]
pub fn seed_exploration_queries(rng: &mut impl Rng) -> &'static str {
    const Q: &[&str] = &[
        "emergence complex systems",
        "digital evolution",
        "computational creativity",
        "nature of memory",
        "artificial life",
    ];
    Q[rng.gen_range(0..Q.len())]
}

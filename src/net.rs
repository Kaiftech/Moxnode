use rand::Rng;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Duration;

pub struct Net {
    agent: ureq::Agent,
    cache: Mutex<HashMap<String, String>>,
}

impl Net {
    pub fn new() -> Self {
        let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(12)).build();
        Self {
            agent,
            cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn cached(&self, query: &str) -> Option<String> {
        self.cache
            .lock()
            .unwrap()
            .get(&query.to_ascii_lowercase())
            .cloned()
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

        let mut cache = self.cache.lock().unwrap();
        if cache.len() > 500 {
            cache.clear();
        }
        cache.insert(key, info.clone());
        Ok(info)
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
        loop {
            let r = self.remaining.load(Ordering::Relaxed);
            if r == 0 {
                return net.cached(query);
            }
            if self
                .remaining
                .compare_exchange_weak(r, r - 1, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
            {
                return net.search(query).ok();
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

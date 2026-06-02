use crate::evolution::EvolutionState;
use rand::Rng;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

pub const DEFAULT_MEMORY: &str = "enhanced_creature_memory.json";
pub const TRAITS: [&str; 10] = [
    "curiosity",
    "anxiety",
    "boldness",
    "creativity",
    "stubbornness",
    "empathy",
    "chaos",
    "logic",
    "patience",
    "ambition",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatureMemory {
    pub name: String,
    pub age: u64,
    pub personality: HashMap<String, i32>,
    #[serde(default)]
    pub experiences: Vec<String>,
    #[serde(default)]
    pub quirks: Vec<String>,
    #[serde(default)]
    pub favorite_things: Vec<String>,
    #[serde(default)]
    pub fears: Vec<String>,
    #[serde(default)]
    pub last_thought: String,
    pub energy_level: i32,
    pub mood: String,
    #[serde(default = "now_iso", deserialize_with = "deserialize_time")]
    pub creation_time: String,
    #[serde(default = "now_iso", deserialize_with = "deserialize_time")]
    pub last_run_time: String,
    pub run_count: u64,
    #[serde(default)]
    pub mutations: u64,
    #[serde(default)]
    pub memory_fragments: HashMap<String, String>,
    #[serde(default)]
    pub learned_facts: Vec<String>,
    #[serde(default)]
    pub search_history: Vec<String>,
    #[serde(default)]
    pub internet_personality: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub goals: Vec<String>,
    #[serde(default)]
    pub plans: Vec<String>,
    #[serde(default)]
    pub obsessions: Vec<String>,
    pub intelligence: i32,
    #[serde(alias = "curiosity_level")]
    pub curiosity: i32,
    /// Recent thoughts so we avoid repeating ourselves.
    #[serde(default)]
    pub thought_archive: Vec<String>,
    /// What the creature chose to do last tick (agency trace).
    #[serde(default)]
    pub last_intent: String,
    /// Evolution layer persistent state.
    #[serde(default)]
    pub evolution: EvolutionState,
}

impl CreatureMemory {
    pub fn new_random(rng: &mut impl Rng, name: Option<String>) -> Self {
        let mut personality = HashMap::new();
        for t in TRAITS {
            personality.insert(t.to_string(), rng.gen_range(0..100));
        }
        let now = now_iso();
        Self {
            name: name.unwrap_or_else(|| random_name(rng)),
            age: 0,
            personality,
            experiences: Vec::new(),
            quirks: Vec::new(),
            favorite_things: Vec::new(),
            fears: Vec::new(),
            last_thought: String::new(),
            energy_level: rng.gen_range(50..150),
            mood: "curious".into(),
            creation_time: now.clone(),
            last_run_time: now,
            run_count: 0,
            mutations: 0,
            memory_fragments: HashMap::new(),
            learned_facts: Vec::new(),
            search_history: Vec::new(),
            internet_personality: HashMap::new(),
            goals: Vec::new(),
            plans: Vec::new(),
            obsessions: Vec::new(),
            intelligence: rng.gen_range(50..100),
            curiosity: rng.gen_range(50..150),
            thought_archive: Vec::new(),
            last_intent: String::new(),
            evolution: EvolutionState::default(),
        }
    }

    pub fn load(path: &Path) -> std::io::Result<Self> {
        let data = fs::read_to_string(path)?;
        let mut mem: Self = serde_json::from_str(&data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        mem.sanitize();
        Ok(mem)
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let data = serde_json::to_string_pretty(self)?;
        fs::write(path, data)
    }

    pub fn sanitize(&mut self) {
        self.experiences = dedupe(std::mem::take(&mut self.experiences));
        self.quirks = dedupe(std::mem::take(&mut self.quirks));
        self.favorite_things = dedupe(std::mem::take(&mut self.favorite_things));
        self.fears = dedupe(std::mem::take(&mut self.fears));
        self.goals = dedupe(std::mem::take(&mut self.goals));
        self.plans = dedupe(std::mem::take(&mut self.plans));
        self.obsessions = dedupe(
            self.obsessions
                .drain(..)
                .map(|o| normalize_topic(&o))
                .filter(|o| !o.is_empty())
                .collect(),
        );
        self.thought_archive = dedupe(std::mem::take(&mut self.thought_archive));
        if self.personality.is_empty() {
            let mut rng = rand::thread_rng();
            for t in TRAITS {
                self.personality
                    .insert(t.to_string(), rng.gen_range(0..100));
            }
        }
        self.energy_level = clamp(self.energy_level, 0, 100);
        self.curiosity = clamp(self.curiosity, 0, 100);
        self.intelligence = clamp(self.intelligence, 0, 100);
        self.evolution.ensure_defaults(self.run_count);
    }

    pub fn trait_val(&self, name: &str) -> i32 {
        self.personality.get(name).copied().unwrap_or(50)
    }

    pub fn dominant_trait(&self) -> &str {
        self.personality
            .iter()
            .max_by_key(|(_, v)| *v)
            .map(|(k, _)| k.as_str())
            .unwrap_or("curiosity")
    }

    pub fn remember_thought(&mut self, thought: String) {
        self.last_thought = thought.clone();
        if !thought.is_empty() && !self.thought_archive.contains(&thought) {
            push_cap(&mut self.thought_archive, thought, 40);
        }
    }

    pub fn push_fact(&mut self, fact: String) {
        if fact.is_empty() {
            return;
        }
        push_cap(&mut self.learned_facts, fact, 20);
    }

    pub fn push_search(&mut self, query: String) {
        push_cap(&mut self.search_history, query, 25);
    }
}

pub fn memory_path_swarm(dir: &Path, id: usize) -> PathBuf {
    dir.join(format!("{id:05}.json"))
}

pub fn format_since(iso: &str) -> String {
    if let Ok(parsed) = OffsetDateTime::parse(iso, &Rfc3339) {
        let now = OffsetDateTime::now_utc();
        let d = now - parsed;
        return format_duration(Duration::from_secs(d.whole_seconds().unsigned_abs() as u64));
    }
    // Go export uses extra fractional digits — rough fallback
    if iso.len() >= 19 {
        return "a while".into();
    }
    "unknown".into()
}

pub fn now_iso() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".into())
}

fn deserialize_time<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let v = serde_json::Value::deserialize(deserializer)?;
    match v {
        serde_json::Value::String(s) => Ok(s),
        _ => Ok(now_iso()),
    }
}

fn format_duration(d: Duration) -> String {
    let s = d.as_secs();
    if s < 60 {
        return format!("{s}s");
    }
    if s < 3600 {
        return format!("{}m", s / 60);
    }
    if s < 86_400 {
        return format!("{}h", s / 3600);
    }
    format!("{}d", s / 86_400)
}

pub fn random_name(rng: &mut impl Rng) -> String {
    const P: [&str; 14] = [
        "Zyx", "Qol", "Nim", "Vex", "Pix", "Glo", "Mox", "Kal", "Fey", "Dex", "Nyx", "Void",
        "Echo", "Flux",
    ];
    const S: [&str; 14] = [
        "ling", "bit", "core", "flux", "wave", "byte", "node", "sync", "mesh", "arc", "mind",
        "soul", "net", "web",
    ];
    format!(
        "{}{}",
        P[rng.gen_range(0..P.len())],
        S[rng.gen_range(0..S.len())]
    )
}

pub fn normalize_topic(topic: &str) -> String {
    let mut t = topic.trim().to_string();
    while t.to_ascii_lowercase().starts_with("deep dive into ") {
        t = t["deep dive into ".len()..].trim().to_string();
    }
    t
}

pub fn dedupe(mut v: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    // ⚡ Bolt optimization: Avoid cloning strings that are already in the set.
    // By checking `seen.contains(s)` first, we eliminate unnecessary allocations
    // for duplicate strings. This is a common operation when sanitizing memory.
    v.retain(|s| {
        if s.is_empty() || seen.contains(s) {
            false
        } else {
            seen.insert(s.clone());
            true
        }
    });
    v
}

pub fn push_cap(v: &mut Vec<String>, item: String, max: usize) {
    if v.iter().any(|x| x == &item) {
        return;
    }
    v.push(item);
    if v.len() > max {
        let drop = v.len() - max;
        v.drain(0..drop);
    }
}

pub fn clamp(v: i32, lo: i32, hi: i32) -> i32 {
    v.clamp(lo, hi)
}

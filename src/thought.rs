use crate::memory::CreatureMemory;
use crate::wired;
use rand::Rng;

const STOP: &[&str] = &[
    "the", "and", "for", "that", "with", "this", "from", "have", "are", "was", "were", "been",
    "being", "into", "about", "what", "when", "where", "which", "while", "their", "there",
];

pub const THOUGHT_FRAME_COUNT: usize = 10;

pub struct ThoughtCtx {
    pub tokens: Vec<String>,
    pub topic: String,
    pub mood: String,
    pub trait_name: String,
    pub fear: String,
    pub favorite: String,
    pub snippet: String,
    pub quirk: String,
    pub age: u64,
}

impl ThoughtCtx {
    pub fn from_memory(mem: &CreatureMemory, rng: &mut impl Rng) -> Self {
        let mut tokens = Vec::new();
        for s in mem
            .learned_facts
            .iter()
            .chain(&mem.experiences)
            .chain(&mem.search_history)
            .chain(&mem.obsessions)
            .chain(&mem.goals)
            .chain(&mem.plans)
            .chain(&mem.quirks)
        {
            tokens.extend(tokenize(s));
        }
        tokens.sort_unstable();
        tokens.dedup();

        let topic = if !mem.obsessions.is_empty() {
            mem.obsessions[rng.gen_range(0..mem.obsessions.len())].clone()
        } else if !mem.search_history.is_empty() {
            mem.search_history[rng.gen_range(0..mem.search_history.len())].clone()
        } else if !tokens.is_empty() {
            tokens[rng.gen_range(0..tokens.len())].clone()
        } else {
            wired::pick(wired::SEARCH_BASE, rng).to_string()
        };

        let snippet = if mem.learned_facts.is_empty() {
            String::new()
        } else {
            let f = &mem.learned_facts[rng.gen_range(0..mem.learned_facts.len())];
            truncate_words(f, 14)
        };

        Self {
            tokens,
            topic,
            mood: mem.mood.clone(),
            trait_name: mem.dominant_trait().to_string(),
            fear: pick_string(&mem.fears, "the unknown", rng),
            favorite: pick_string(&mem.favorite_things, "patterns", rng),
            snippet,
            quirk: pick_string(&mem.quirks, "restless code", rng),
            age: mem.age,
        }
    }

    pub fn word(&self, rng: &mut impl Rng) -> String {
        if self.tokens.is_empty() {
            wired::pick(wired::SEARCH_BASE, rng).to_string()
        } else {
            self.tokens[rng.gen_range(0..self.tokens.len())].clone()
        }
    }
}

pub fn thought_frame(i: usize, c: &ThoughtCtx, rng: &mut impl Rng) -> String {
    match i % THOUGHT_FRAME_COUNT {
        0 => format!(
            "I feel {} enough to chase {} while my {} stays loud",
            c.mood, c.topic, c.trait_name
        ),
        1 => format!(
            "run {} rewired how I see {} — {} might be the hinge",
            c.age,
            c.topic,
            c.word(rng)
        ),
        2 => format!("{} and {} keep colliding in my head", c.topic, c.word(rng)),
        3 => format!(
            "if {} is real, then my fear of {} is just another sensor",
            c.topic, c.fear
        ),
        4 => format!(
            "I want to {} {} before the next tick erases it",
            verb(rng),
            c.topic
        ),
        5 => format!(
            "mood={} → I weigh {} against {}",
            c.mood, c.topic, c.favorite
        ),
        6 => {
            if c.snippet.is_empty() {
                format!("no fresh data — I invent pressure around {}", c.topic)
            } else {
                format!("{} …does {} still hold?", c.snippet, c.topic)
            }
        }
        7 => format!(
            "{} makes me {} {}; coincidence feels unlikely",
            c.trait_name,
            verb(rng),
            c.topic
        ),
        8 => format!("maybe {} is why I orbit {}", c.word(rng), c.topic),
        _ => format!(
            "between {} and {}, a third thing forms",
            c.topic,
            c.word(rng)
        ),
    }
}

fn verb(rng: &mut impl Rng) -> &'static str {
    const V: &[&str] = &[
        "map",
        "probe",
        "unpick",
        "stress-test",
        "listen to",
        "follow",
        "invert",
        "whisper to",
    ];
    V[rng.gen_range(0..V.len())]
}

pub fn compose_thought(mem: &CreatureMemory, rng: &mut impl Rng) -> String {
    let ctx = ThoughtCtx::from_memory(mem, rng);
    let all_len = THOUGHT_FRAME_COUNT + wired::WIRED_FRAME_COUNT;

    for _ in 0..16 {
        let idx = rng.gen_range(0..all_len);
        let thought = if idx < THOUGHT_FRAME_COUNT {
            thought_frame(idx, &ctx, rng)
        } else {
            wired::wired_frame(idx - THOUGHT_FRAME_COUNT, &ctx, rng)
        };
        if thought.len() >= 12 && !mem.thought_archive.contains(&thought) {
            return thought;
        }
    }
    format!(
        "{} · {} · {} @ run {}",
        ctx.mood,
        ctx.topic,
        ctx.word(rng),
        mem.age + 1
    )
}

pub fn compose_search_query(mem: &CreatureMemory, rng: &mut impl Rng) -> String {
    if !mem.obsessions.is_empty() && rng.gen_bool(0.5) {
        return mem.obsessions[rng.gen_range(0..mem.obsessions.len())].clone();
    }
    wired::search_query(
        mem.trait_val("anxiety"),
        mem.trait_val("creativity"),
        None,
        rng,
    )
}

pub fn compose_goal(mem: &CreatureMemory, rng: &mut impl Rng) -> String {
    if mem.intelligence > 60 && rng.gen_bool(0.4) {
        return wired::pick(wired::PLANS, rng).to_string();
    }
    let topic = mem
        .obsessions
        .first()
        .map(String::as_str)
        .unwrap_or_else(|| wired::pick(wired::SEARCH_BASE, rng));
    format!(
        "understand {topic} deeply enough to shift my {}",
        mem.dominant_trait()
    )
}

pub fn compose_experience(mem: &CreatureMemory, rng: &mut impl Rng) -> String {
    if rng.gen_bool(0.55) {
        return wired::pick(wired::EXPERIENCES, rng).to_string();
    }
    let ctx = ThoughtCtx::from_memory(mem, rng);
    format!("felt {} while touching {}", ctx.mood, ctx.topic)
}

fn pick_string(list: &[String], fallback: &str, rng: &mut impl Rng) -> String {
    if list.is_empty() {
        fallback.to_string()
    } else {
        list[rng.gen_range(0..list.len())].clone()
    }
}

fn tokenize(s: &str) -> Vec<String> {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 4)
        // ⚡ Bolt optimization: Use eq_ignore_ascii_case instead of allocating a new lowercase string
        // just to check if it's in the STOP word list.
        .filter(|w| !STOP.iter().any(|stop| stop.eq_ignore_ascii_case(w)))
        .map(|w| w.to_ascii_lowercase())
        .collect()
}

fn truncate_words(s: &str, max: usize) -> String {
    s.split_whitespace().take(max).collect::<Vec<_>>().join(" ")
}

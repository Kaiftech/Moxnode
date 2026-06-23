use crate::evolution::config::EvolutionConfig;
use crate::evolution::state::EvolutionState;
use crate::memory::CreatureMemory;
use rand::Rng;
const DECAY: f32 = 0.4;
const COMPETE: f32 = 0.15;

pub fn tick(mem: &mut CreatureMemory, cfg: &EvolutionConfig, rng: &mut impl Rng) {
    if !cfg.priorities {
        return;
    }

    let state = &mut mem.evolution;
    state.ensure_defaults(mem.run_count);

    let keys: Vec<String> = state.priorities.keys().cloned().collect();
    for k in keys {
        if let Some(v) = state.priorities.get_mut(&k) {
            *v -= DECAY * 0.01;
        }
    }

    if !mem.obsessions.is_empty() {
        let top = normalize_key(&mem.obsessions[0]);
        *state.priorities.entry(top).or_insert(50.0) += 2.5;
    }

    if mem.mood == "confused" {
        *state.priorities.entry("patterns".into()).or_insert(40.0) += 1.2;
    }

    if rng.gen_bool(0.08) {
        mutate_priority(state, rng);
    }

    compete(state);

    for v in state.priorities.values_mut() {
        *v = v.clamp(5.0, 100.0);
    }
}

pub fn top_topic(state: &EvolutionState) -> String {
    state
        .priorities
        .iter()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(k, _)| k.clone())
        .unwrap_or_else(|| "memory".into())
}

pub fn shift(state: &mut EvolutionState, topic: &str, delta: f32) {
    let k = normalize_key(topic);
    let e = state.priorities.entry(k).or_insert(50.0);
    *e = (*e + delta).clamp(5.0, 100.0);
}

pub fn print(state: &EvolutionState, cfg: &EvolutionConfig) {
    if !cfg.priorities {
        return;
    }
    let mut items: Vec<_> = state.priorities.iter().collect();
    items.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
    println!("[PRIORITIES]");
    for (k, v) in items.iter().take(6) {
        println!("  {:16} {:.0}", k, v);
    }
}

fn compete(state: &mut EvolutionState) {
    let sum: f32 = state.priorities.values().sum();
    if sum < 1.0 {
        return;
    }
    let keys: Vec<String> = state.priorities.keys().cloned().collect();
    for k in keys {
        if let Some(v) = state.priorities.get_mut(&k) {
            *v -= COMPETE * (*v / sum);
        }
    }
}

fn mutate_priority(state: &mut EvolutionState, rng: &mut impl Rng) {
    let keys: Vec<String> = state.priorities.keys().cloned().collect();
    if keys.is_empty() {
        return;
    }
    let k = keys[rng.gen_range(0..keys.len())].clone();
    if let Some(v) = state.priorities.get_mut(&k) {
        *v += rng.gen_range(-8..9) as f32;
    }
}

fn normalize_key(s: &str) -> String {
    match s.char_indices().nth(24) {
        None => s.to_string(),
        Some((idx, _)) => s[..idx].to_string(),
    }
}

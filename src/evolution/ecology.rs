use crate::evolution::state::{EvolutionState, LivingMemory};
use crate::evolution::config::EvolutionConfig;
use crate::memory::CreatureMemory;
use rand::Rng;
use rayon::prelude::*;

const DECAY: f32 = 0.012;
const REINFORCE: f32 = 0.08;
const RESURFACE_CHANCE: f32 = 0.04;

pub fn tick(mem: &mut CreatureMemory, cfg: &EvolutionConfig, rng: &mut impl Rng) {
    if !cfg.memory_decay {
        return;
    }

    sync_from_legacy(mem);
    let state = &mut mem.evolution;

    state.living_memories.par_iter_mut().for_each(|m| {
        let age = mem.run_count.saturating_sub(m.last_touch_run) as f32;
        m.strength -= DECAY * (1.0 + age * 0.01);
        if m.recalls > 2 {
            m.strength += REINFORCE * 0.5;
        }
        m.distortion += 0.002 * m.recalls as f32;
        m.strength = m.strength.clamp(0.02, 1.0);
        m.distortion = m.distortion.clamp(0.0, 0.45);
    });

    cluster_memories(state);

    if rng.gen::<f32>() < RESURFACE_CHANCE {
        if let Some((id, content, distortion)) = weak_memory(state, rng) {
            let distorted = distort_text(&content, distortion, rng);
            state.contradictions.push(format!("resurfaced: {distorted}"));
            if let Some(lm) = state.living_memories.iter_mut().find(|x| x.id == id) {
                lm.strength += 0.15;
                lm.recalls += 1;
                lm.last_touch_run = mem.run_count;
            }
        }
    }

    state.living_memories.retain(|m| m.strength > 0.04);
    if state.living_memories.len() > 80 {
        state.living_memories.sort_by(|a, b| b.strength.partial_cmp(&a.strength).unwrap());
        state.living_memories.truncate(80);
    }
}

fn sync_from_legacy(mem: &mut CreatureMemory) {
    for fact in mem.learned_facts.iter().rev().take(3) {
        if mem.evolution.living_memories.iter().any(|m| &m.content == fact) {
            continue;
        }
        let id = mem.evolution.next_memory_id;
        mem.evolution.next_memory_id += 1;
        mem.evolution.living_memories.push(LivingMemory {
            id,
            content: fact.clone(),
            strength: 0.75,
            distortion: 0.0,
            cluster_id: 0,
            recalls: 0,
            born_run: mem.run_count,
            last_touch_run: mem.run_count,
        });
    }
}

fn cluster_memories(state: &mut EvolutionState) {
    let tokens: Vec<Vec<String>> = state
        .living_memories
        .par_iter()
        .map(|m| tokenize(&m.content))
        .collect();

    for (i, mem) in state.living_memories.iter_mut().enumerate() {
        let mut best = mem.cluster_id;
        let mut best_score = 0usize;
        for (j, other) in tokens.iter().enumerate() {
            if i == j {
                continue;
            }
            let score = overlap(&tokens[i], other);
            if score > best_score {
                best_score = score;
                best = j as u32;
            }
        }
        if best_score >= 2 {
            mem.cluster_id = best;
        }
    }
}

fn weak_memory(state: &EvolutionState, rng: &mut impl Rng) -> Option<(u64, String, f32)> {
    let weak: Vec<_> = state
        .living_memories
        .iter()
        .filter(|m| m.strength < 0.35)
        .map(|m| (m.id, m.content.clone(), m.distortion))
        .collect();
    if weak.is_empty() {
        return None;
    }
    Some(weak[rng.gen_range(0..weak.len())].clone())
}

pub fn reinforce_recall(state: &mut EvolutionState, content: &str, run: u64) {
    if let Some(m) = state
        .living_memories
        .iter_mut()
        .find(|m| m.content == content)
    {
        m.strength = (m.strength + REINFORCE).min(1.0);
        m.recalls += 1;
        m.last_touch_run = run;
    }
}

pub fn ingest_experience(state: &mut EvolutionState, text: &str, run: u64) {
    let id = state.next_memory_id;
    state.next_memory_id += 1;
    state.living_memories.push(LivingMemory {
        id,
        content: text.to_string(),
        strength: 0.85,
        distortion: 0.0,
        cluster_id: id as u32,
        recalls: 1,
        born_run: run,
        last_touch_run: run,
    });
}

fn distort_text(s: &str, amount: f32, rng: &mut impl Rng) -> String {
    if amount < 0.08 || s.len() < 20 {
        return s.to_string();
    }
    let words: Vec<&str> = s.split_whitespace().collect();
    if words.len() < 4 {
        return s.to_string();
    }
    let i = rng.gen_range(0..words.len());
    let mut out = words.to_vec();
    out[i] = "[...]";
    out.join(" ")
}

fn tokenize(s: &str) -> Vec<String> {
    s.to_ascii_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 4)
        .map(String::from)
        .collect()
}

fn overlap(a: &[String], b: &[String]) -> usize {
    a.iter().filter(|t| b.contains(t)).count()
}

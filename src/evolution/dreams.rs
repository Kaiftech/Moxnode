use crate::evolution::config::EvolutionConfig;
use crate::evolution::ecology;
use crate::memory::CreatureMemory;
use rand::Rng;

pub fn maybe_dream(
    mem: &mut CreatureMemory,
    cfg: &EvolutionConfig,
    rng: &mut impl Rng,
    verbose: bool,
) -> bool {
    if !cfg.dreams || mem.evolution.dream_cooldown > 0 {
        if mem.evolution.dream_cooldown > 0 {
            mem.evolution.dream_cooldown -= 1;
        }
        return false;
    }

    let pressure = mem.trait_val("creativity") + mem.trait_val("anxiety");
    let trigger = mem.run_count % 17 == 0
        || !mem.evolution.contradictions.is_empty()
        || mem.mood == "restless";

    if !trigger || rng.gen_range(0..100) > pressure / 2 {
        return false;
    }

    if verbose {
        println!("[DREAM]");
        println!(
            "  inputs: {} living memories, {} contradictions",
            mem.evolution.living_memories.len(),
            mem.evolution.contradictions.len()
        );
    }

    let thought = compose_dream(mem, rng);
    if verbose {
        println!("  fragment: \"{}\"", thought);
    }

    mem.remember_thought(thought.clone());
    ecology::ingest_experience(
        &mut mem.evolution,
        &format!("dream: {thought}"),
        mem.run_count,
    );

    if rng.gen_bool(0.3) {
        let trait_name = mem.dominant_trait().to_string();
        if let Some(v) = mem.personality.get_mut(&trait_name) {
            *v = (*v + rng.gen_range(-3..4)).clamp(0, 100);
        }
    }

    mem.evolution.dream_cooldown = 8;
    mem.evolution.contradictions.clear();
    true
}

fn compose_dream(mem: &CreatureMemory, rng: &mut impl Rng) -> String {
    let state = &mem.evolution;
    let a = state
        .living_memories
        .get(rng.gen_range(0..state.living_memories.len().max(1)))
        .map(|m| ecology_distort(&m.content, m.distortion))
        .unwrap_or_else(|| "static".into());

    let b = mem
        .obsessions
        .first()
        .cloned()
        .unwrap_or_else(|| "the void".into());

    let c = state
        .contradictions
        .first()
        .cloned()
        .unwrap_or_else(|| "unfinished logic".into());

    let frames = [
        format!("in sleep, {a} argued with {b}"),
        format!("a broken loop: {c} became {b}"),
        format!("I tasted {a} but it meant {b}"),
        format!("{b} wore the mask of {a}"),
    ];
    frames[rng.gen_range(0..frames.len())].clone()
}

fn ecology_distort(s: &str, d: f32) -> String {
    if d > 0.2 {
        match s.char_indices().nth(40) {
            Some((idx, _)) => format!("{}~", &s[..idx]),
            None => format!("{}~", s),
        }
    } else {
        match s.char_indices().nth(50) {
            Some((idx, _)) => s[..idx].to_string(),
            None => s.to_string(),
        }
    }
}

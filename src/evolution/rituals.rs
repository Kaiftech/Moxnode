use crate::decide::Intent;
use crate::evolution::config::EvolutionConfig;
use crate::evolution::state::{EvolutionState, Ritual};
use crate::memory::CreatureMemory;
use rand::Rng;

pub fn tick(mem: &mut CreatureMemory, cfg: &EvolutionConfig, rng: &mut impl Rng) {
    if !cfg.rituals {
        return;
    }

    let run = mem.run_count;
    detect_ritual(&mut mem.evolution, run, rng);

    for r in &mut mem.evolution.rituals {
        if mem.run_count.saturating_sub(r.last_run) >= r.cadence_runs {
            r.strength += 0.05;
        }
    }
}

pub fn ritual_intent(state: &EvolutionState, cfg: &EvolutionConfig, run: u64) -> Option<Intent> {
    if !cfg.rituals {
        return None;
    }
    for r in &state.rituals {
        if run.saturating_sub(r.last_run) >= r.cadence_runs {
            return match r.name.as_str() {
                "memory_revisit" => Some(Intent::Think),
                "search_again" => Some(Intent::Search),
                "reflection" => Some(Intent::Write),
                _ => Some(Intent::Think),
            };
        }
    }
    None
}

pub fn mark_done(state: &mut EvolutionState, name: &str, run: u64) {
    if let Some(r) = state.rituals.iter_mut().find(|r| r.name == name) {
        r.last_run = run;
    }
}

fn detect_ritual(state: &mut EvolutionState, run: u64, rng: &mut impl Rng) {
    if state.last_intents.len() < 5 {
        return;
    }
    let last = &state.last_intents[state.last_intents.len() - 1];
    let same = state.last_intents.iter().rev().take(5).all(|x| x == last);
    if !same {
        return;
    }

    let name = match last.as_str() {
        "search" => "search_again",
        "think" => "memory_revisit",
        "write" => "reflection",
        _ => return,
    };

    if state.rituals.iter().any(|r| r.name == name) {
        return;
    }

    if rng.gen_bool(0.4) {
        state.rituals.push(Ritual {
            name: name.into(),
            cadence_runs: rng.gen_range(8..24),
            last_run: run,
            strength: 0.6,
        });
        println!("[RITUAL] emerged: {name}");
    }
}

use crate::decide::Intent;
use crate::evolution::config::EvolutionConfig;
use crate::evolution::state::EvolutionState;
use crate::memory::CreatureMemory;
use rand::Rng;

#[derive(Debug, Clone, Copy, Default)]
pub struct IntentInfluence {
    pub think: f64,
    pub search: f64,
    pub rest: f64,
    pub obsess: f64,
    pub plan: f64,
    pub mutate: f64,
    pub write: f64,
}

pub fn tick(mem: &mut CreatureMemory, cfg: &EvolutionConfig, rng: &mut impl Rng) {
    if !cfg.society {
        return;
    }

    let bold = mem.trait_val("boldness") as f32;
    let patience = mem.trait_val("patience") as f32;
    let logic = mem.trait_val("logic") as f32;
    let creativity = mem.trait_val("creativity") as f32;
    let empathy = mem.trait_val("empathy") as f32;

    let v = &mut mem.evolution.voices;
    v.explorer = drift(v.explorer, bold, 0.02);
    v.archivist = drift(v.archivist, patience, 0.02);
    v.skeptic = drift(v.skeptic, logic, 0.02);
    v.dreamer = drift(v.dreamer, creativity, 0.02);
    v.philosopher = drift(v.philosopher, empathy, 0.02);

    if rng.gen_bool(0.12) {
        let conflict = format!(
            "{} wants search, {} wants preservation",
            voice_label(max_voice(v)),
            if v.archivist > v.explorer {
                "Archivist"
            } else {
                "Explorer"
            }
        );
        mem.evolution.voices.last_conflict = conflict;
    }
}

pub fn influence(state: &EvolutionState, cfg: &EvolutionConfig) -> IntentInfluence {
    if !cfg.society {
        return IntentInfluence::default();
    }
    let v = &state.voices;
    IntentInfluence {
        think: (v.philosopher * 0.15) as f64,
        search: (v.explorer * 0.2 + v.skeptic * 0.05) as f64,
        rest: (v.archivist * 0.08) as f64,
        obsess: (v.dreamer * 0.12) as f64,
        plan: (v.philosopher * 0.1) as f64,
        mutate: (v.explorer * 0.06) as f64,
        write: (v.archivist * 0.1) as f64,
    }
}

pub fn search_query_bias(state: &EvolutionState, cfg: &EvolutionConfig, base: &str) -> String {
    if !cfg.society {
        return base.to_string();
    }
    let v = &state.voices;
    if v.skeptic > v.explorer {
        format!("evidence for {base}")
    } else if v.explorer > v.philosopher {
        format!("novel perspectives on {base}")
    } else if v.dreamer > 55.0 {
        format!("{base} symbolic meaning")
    } else {
        base.to_string()
    }
}

pub fn print_council(state: &EvolutionState, intent: Intent) {
    let v = &state.voices;
    println!("[SOCIETY]");
    println!(
        "  Explorer {:.0} | Archivist {:.0} | Skeptic {:.0}",
        v.explorer, v.archivist, v.skeptic
    );
    println!(
        "  Dreamer {:.0} | Philosopher {:.0} → intent {}",
        v.dreamer,
        v.philosopher,
        intent.label()
    );
    if !v.last_conflict.is_empty() && intent == Intent::Search {
        println!("  tension: {}", v.last_conflict);
    }
}

fn drift(current: f32, target: f32, rate: f32) -> f32 {
    current + (target - current) * rate
}

fn max_voice(v: &crate::evolution::state::SocietyState) -> usize {
    let arr = [v.explorer, v.archivist, v.skeptic, v.dreamer, v.philosopher];
    arr.iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(i, _)| i)
        .unwrap_or(0)
}

fn voice_label(i: usize) -> &'static str {
    match i {
        0 => "Explorer",
        1 => "Archivist",
        2 => "Skeptic",
        3 => "Dreamer",
        _ => "Philosopher",
    }
}

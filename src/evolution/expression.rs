use crate::evolution::config::EvolutionConfig;
use crate::evolution::state::EvolutionState;
use crate::memory::CreatureMemory;
use rand::Rng;

pub fn tick(mem: &mut CreatureMemory, cfg: &EvolutionConfig, rng: &mut impl Rng) {
    if !cfg.expression {
        return;
    }

    let creativity = mem.trait_val("creativity") as f32;
    let energy_high = mem.energy_level > 70;
    let e = &mut mem.evolution.expression;
    e.cadence = e.cadence.saturating_add(if energy_high { 1 } else { 0 });
    e.metaphor_density = (e.metaphor_density + 0.002 * creativity).min(1.0);
    e.rhythm_bias = (e.rhythm_bias + (rng.gen::<f32>() - 0.5) * 0.02).clamp(-1.0, 1.0);

    if rng.gen_bool(0.05) {
        let phrase = format!("{} {}", pick_connector(rng), mem.mood);
        if !e.phrases.contains(&phrase) {
            e.phrases.push(phrase);
            if e.phrases.len() > 12 {
                e.phrases.remove(0);
            }
        }
    }
}

pub fn prefix_thought(state: &EvolutionState, cfg: &EvolutionConfig, thought: &str) -> String {
    if !cfg.expression || state.expression.phrases.is_empty() {
        return thought.to_string();
    }
    let p = &state.expression.phrases[state.expression.phrases.len() % state.expression.phrases.len()];
    if state.expression.metaphor_density > 0.6 {
        format!("{p}, {thought}")
    } else {
        thought.to_string()
    }
}

pub fn prefix_line(state: &EvolutionState, cfg: &EvolutionConfig, kind: &str) -> String {
    if !cfg.expression {
        return format!("💭");
    }
    let cadence = state.expression.cadence % 4;
    match (kind, cadence) {
        ("think", 0) => "💭".into(),
        ("think", _) => "···".into(),
        ("search", _) => "⌁".into(),
        ("world", _) => "◈".into(),
        _ => "•".into(),
    }
}

fn pick_connector(rng: &mut impl Rng) -> &'static str {
    const C: &[&str] = &["between packets", "under noise", "through latency", "beside cache"];
    C[rng.gen_range(0..C.len())]
}

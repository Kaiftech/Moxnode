use crate::decide::Intent;
use crate::evolution::config::EvolutionConfig;
use crate::evolution::state::{EvolutionState, Habit};
use crate::memory::CreatureMemory;
use rand::Rng;

const THRESHOLD: u32 = 6;

pub fn record_intent(state: &mut EvolutionState, intent: Intent, run: u64) {
    let key = intent.label().to_string();
    *state.behavior_counts.entry(key.clone()).or_insert(0) += 1;

    state.last_intents.push(key);
    if state.last_intents.len() > 12 {
        state.last_intents.remove(0);
    }

    let count = state
        .behavior_counts
        .get(intent.label())
        .copied()
        .unwrap_or(0);
    if count >= THRESHOLD && count % THRESHOLD == 0 {
        let pattern = habit_pattern(intent);
        if !state.habits.iter().any(|h| h.pattern == pattern) {
            state.habits.push(Habit {
                pattern: pattern.clone(),
                strength: 0.4 + (count as f32 * 0.03),
                born_run: run,
            });
        }
    }
}

pub fn apply(
    mem: &CreatureMemory,
    state: &EvolutionState,
    cfg: &EvolutionConfig,
    rng: &mut impl Rng,
) -> Option<Intent> {
    if !cfg.habits || state.habits.is_empty() {
        return None;
    }

    let mood = mem.mood.as_str();
    for h in &state.habits {
        if h.strength < 0.5 {
            continue;
        }
        if h.pattern.contains("confusion") && mood == "confused" && rng.gen_bool(0.35) {
            return Some(Intent::Search);
        }
        if h.pattern.contains("curiosity") && mem.curiosity > 70 && rng.gen_bool(0.3) {
            return Some(Intent::Search);
        }
        if h.pattern.contains("stress") && mem.energy_level < 35 && rng.gen_bool(0.4) {
            return Some(Intent::Rest);
        }
        if h.pattern.contains("reflection") && rng.gen_bool(0.25) {
            return Some(Intent::Think);
        }
    }
    None
}

pub fn tick(mem: &mut CreatureMemory) {
    let run = mem.run_count;
    for h in &mut mem.evolution.habits {
        h.strength *= 0.998;
        h.strength += (run.saturating_sub(h.born_run) as f32 * 0.0001).min(0.05);
        h.strength = h.strength.clamp(0.2, 1.2);
    }
    mem.evolution.habits.retain(|h| h.strength > 0.25);
}

fn habit_pattern(intent: Intent) -> String {
    match intent {
        Intent::Search => "curiosity: revisit familiar ideas".into(),
        Intent::Think => "reflection: loop known thoughts".into(),
        Intent::Rest => "stress: recurring rest pattern".into(),
        Intent::Obsess => "confusion: orbit one topic".into(),
        _ => format!("habit:{}", intent.label()),
    }
}

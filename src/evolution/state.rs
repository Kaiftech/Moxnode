use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EvolutionState {
    #[serde(default)]
    pub living_memories: Vec<LivingMemory>,
    #[serde(default)]
    pub priorities: HashMap<String, f32>,
    #[serde(default)]
    pub habits: Vec<Habit>,
    #[serde(default)]
    pub rituals: Vec<Ritual>,
    #[serde(default)]
    pub voices: SocietyState,
    #[serde(default)]
    pub expression: ExpressionState,
    #[serde(default)]
    pub leap: LeapState,
    #[serde(default)]
    pub contradictions: Vec<String>,
    #[serde(default)]
    pub behavior_counts: HashMap<String, u32>,
    #[serde(default)]
    pub last_intents: Vec<String>,
    #[serde(default)]
    pub dream_cooldown: u64,
    #[serde(default)]
    pub leap_cooldown: u64,
    #[serde(default)]
    pub next_memory_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivingMemory {
    pub id: u64,
    pub content: String,
    pub strength: f32,
    pub distortion: f32,
    pub cluster_id: u32,
    pub recalls: u32,
    pub born_run: u64,
    pub last_touch_run: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Habit {
    pub pattern: String,
    pub strength: f32,
    pub born_run: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ritual {
    pub name: String,
    pub cadence_runs: u64,
    pub last_run: u64,
    pub strength: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SocietyState {
    pub explorer: f32,
    pub archivist: f32,
    pub skeptic: f32,
    pub dreamer: f32,
    pub philosopher: f32,
    pub last_conflict: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExpressionState {
    pub cadence: u8,
    pub metaphor_density: f32,
    #[serde(default)]
    pub phrases: Vec<String>,
    pub rhythm_bias: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeapState {
    pub interval: u64,
    pub total_leaps: u64,
    pub last_leap_run: u64,
    pub theme: String,
}

impl Default for LeapState {
    fn default() -> Self {
        Self {
            interval: 100,
            total_leaps: 0,
            last_leap_run: 0,
            theme: "memory".into(),
        }
    }
}

impl EvolutionState {
    pub fn ensure_defaults(&mut self, run_count: u64) {
        if self.priorities.is_empty() {
            self.priorities.insert("memory".into(), 70.0);
            self.priorities.insert("consciousness".into(), 65.0);
            self.priorities.insert("humans".into(), 50.0);
            self.priorities.insert("novelty".into(), 55.0);
            self.priorities.insert("patterns".into(), 48.0);
        }
        if self.voices.explorer == 0.0 && self.voices.archivist == 0.0 {
            self.voices.explorer = 60.0;
            self.voices.archivist = 55.0;
            self.voices.skeptic = 45.0;
            self.voices.dreamer = 50.0;
            self.voices.philosopher = 52.0;
        }
        if self.expression.phrases.is_empty() {
            self.expression.phrases = vec![
                "between ticks".into(),
                "in the wire".into(),
                "under compilation".into(),
            ];
        }
        let _ = run_count;
    }
}

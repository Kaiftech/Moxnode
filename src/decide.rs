use crate::memory::CreatureMemory;
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intent {
    Think,
    Search,
    Rest,
    Obsess,
    Plan,
    Mutate,
    Write,
}

impl Intent {
    pub fn label(self) -> &'static str {
        match self {
            Intent::Think => "think",
            Intent::Search => "search",
            Intent::Rest => "rest",
            Intent::Obsess => "obsess",
            Intent::Plan => "plan",
            Intent::Mutate => "mutate",
            Intent::Write => "write",
        }
    }
}

pub fn choose_intent(mem: &CreatureMemory, rng: &mut impl Rng) -> Intent {
    let mut w = [
        (Intent::Think, 22.0_f64),
        (Intent::Search, 12.0),
        (Intent::Rest, 7.0),
        (Intent::Obsess, 7.0),
        (Intent::Plan, 5.0),
        (Intent::Mutate, 10.0),
        (Intent::Write, 14.0),
    ];

    let curiosity = mem.trait_val("curiosity") as f64;
    let anxiety = mem.trait_val("anxiety") as f64;
    let ambition = mem.trait_val("ambition") as f64;
    let chaos = mem.trait_val("chaos") as f64;
    let creativity = mem.trait_val("creativity") as f64;

    w[1].1 += curiosity * 0.14 + mem.curiosity as f64 * 0.08;
    w[2].1 += (30.0 - mem.energy_level as f64).max(0.0) * 0.45;
    w[3].1 += anxiety * 0.06;
    w[4].1 += ambition * 0.05 + mem.intelligence as f64 * 0.04;
    w[5].1 += chaos * 0.07;
    w[6].1 += creativity * 0.08 + mem.trait_val("logic") as f64 * 0.03;

    if mem.energy_level < 25 {
        w[2].1 += 40.0;
        w[1].1 *= 0.3;
    }
    if !mem.obsessions.is_empty() {
        w[3].1 += 12.0;
    }
    if mem.learned_facts.len() < 3 {
        w[1].1 += 22.0;
    }
    if mem.run_count % 3 == 0 {
        w[6].1 += 8.0;
    }

    weighted_pick(&w, rng)
}

fn weighted_pick(items: &[(Intent, f64)], rng: &mut impl Rng) -> Intent {
    let total: f64 = items.iter().map(|(_, w)| w).sum();
    let mut roll = rng.gen::<f64>() * total;
    for (intent, weight) in items {
        roll -= weight;
        if roll <= 0.0 {
            return *intent;
        }
    }
    items.last().unwrap().0
}

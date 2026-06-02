pub mod config;
pub mod dreams;
pub mod ecology;
pub mod expression;
pub mod habits;
pub mod leaps;
pub mod priorities;
pub mod rituals;
pub mod society;
pub mod state;
pub mod world;

pub use config::EvolutionConfig;
pub use state::EvolutionState;

use crate::decide::Intent;
use crate::memory::CreatureMemory;
use crate::net::{Net, NetBudget};
use rand::Rng;

/// Pre-tick: ecology, priorities, habits, expression drift.
pub fn pre_tick(mem: &mut CreatureMemory, cfg: &EvolutionConfig, rng: &mut impl Rng) {
    if !cfg.any() {
        return;
    }
    mem.evolution.ensure_defaults(mem.run_count);
    ecology::tick(mem, cfg, rng);
    priorities::tick(mem, cfg, rng);
    society::tick(mem, cfg, rng);
    if cfg.habits {
        habits::tick(mem);
    }
    rituals::tick(mem, cfg, rng);
    expression::tick(mem, cfg, rng);
}

/// Choose intent — society + habits + rituals layer on top of base decide.
pub fn pick_intent(mem: &mut CreatureMemory, cfg: &EvolutionConfig, rng: &mut impl Rng) -> Intent {
    if !cfg.any() {
        return crate::decide::choose_intent(mem, rng, None);
    }

    let run = mem.run_count;
    if let Some(intent) = rituals::ritual_intent(&mem.evolution, cfg, run) {
        if cfg.habits {
            habits::record_intent(&mut mem.evolution, intent, run);
        }
        return intent;
    }

    let evo_ref = &mem.evolution;
    if let Some(intent) = habits::apply(mem, evo_ref, cfg, rng) {
        if cfg.habits {
            habits::record_intent(&mut mem.evolution, intent, run);
        }
        return intent;
    }

    let influence = society::influence(&mem.evolution, cfg);
    let intent = crate::decide::choose_intent(mem, rng, Some(influence));

    if cfg.society {
        society::print_council(&mem.evolution, intent);
    }

    if cfg.habits {
        habits::record_intent(&mut mem.evolution, intent, run);
    }
    intent
}

/// Post-tick: dreams, continuity hooks.
pub fn post_tick(
    mem: &mut CreatureMemory,
    cfg: &EvolutionConfig,
    rng: &mut impl Rng,
    verbose: bool,
) {
    if !cfg.any() {
        return;
    }

    if cfg.priorities {
        priorities::print(&mem.evolution, cfg);
    }

    dreams::maybe_dream(mem, cfg, rng, verbose);

    for c in mem.evolution.contradictions.drain(..).take(2) {
        if verbose {
            println!("[CONTINUITY] {c}");
        }
    }
}

pub fn try_leap(
    mem: &mut CreatureMemory,
    net: &Net,
    budget: &NetBudget,
    cfg: &EvolutionConfig,
    rng: &mut impl Rng,
    verbose: bool,
) -> bool {
    leaps::maybe_leap(mem, net, budget, cfg, rng, verbose)
}

pub fn try_world_search(
    mem: &mut CreatureMemory,
    net: &Net,
    budget: &NetBudget,
    cfg: &EvolutionConfig,
    rng: &mut impl Rng,
    verbose: bool,
) -> bool {
    let Some(raw) = world::should_search(mem, cfg, rng) else {
        return false;
    };
    let (trigger, query) = raw.split_once('|').unwrap_or(("internal state", &raw));
    world::explore(mem, net, budget, trigger, query, 6, verbose, rng);
    true
}

pub fn style_thought(mem: &CreatureMemory, cfg: &EvolutionConfig, thought: String) -> String {
    expression::prefix_thought(&mem.evolution, cfg, &thought)
}

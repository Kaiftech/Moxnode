use crate::evolution::config::EvolutionConfig;
use crate::evolution::priorities;
use crate::evolution::world;
use crate::memory::{push_cap, CreatureMemory};
use crate::net::{Net, NetBudget};
use rand::Rng;

pub fn maybe_leap(
    mem: &mut CreatureMemory,
    net: &Net,
    budget: &NetBudget,
    cfg: &EvolutionConfig,
    rng: &mut impl Rng,
    verbose: bool,
) -> bool {
    if !cfg.leaps || mem.evolution.leap_cooldown > 0 {
        if mem.evolution.leap_cooldown > 0 {
            mem.evolution.leap_cooldown -= 1;
        }
        return false;
    }

    let due = mem.run_count > 0 && mem.run_count % mem.evolution.leap.interval == 0;
    if !due {
        return false;
    }

    mem.age += 1;
    mem.evolution.leap.theme = priorities::top_topic(&mem.evolution);
    let theme = mem.evolution.leap.theme.clone();

    if verbose {
        println!();
        println!("══════════════════════════════════════");
        println!("RUN {} — LEAP EVENT", mem.run_count + 1);
        println!("══════════════════════════════════════");
        println!();
        println!("Theme:");
        println!("{theme}");
        println!();
        println!("Search depth:");
        println!("high");
        println!();
    }

    let topics = [
        format!("{theme} forgetting"),
        format!("{theme} cognition"),
        format!("{theme} nostalgia"),
        format!("digital {theme} evolution"),
    ];

    for q in &topics {
        let r = world::explore(mem, net, budget, "evolutionary leap", q, 8, false, rng);
        if verbose && !r.snippets.is_empty() {
            println!("Topics discovered:");
            for s in r.snippets.iter().take(1) {
                let line: String = s.chars().take(60).collect();
                println!("- {line}");
            }
        }
    }

    if verbose {
        println!();
        println!("Effects:");
        println!("new obsession candidate");
        println!("priority mutation");
        println!("dream trigger");
        println!();
        println!(
            "Next leap interval: {} runs",
            mem.evolution.leap.interval.saturating_sub(1).max(10)
        );
        println!("Cooldown: reflection period (12 runs)");
        println!();
    }

    push_cap(&mut mem.obsessions, theme.clone(), 8);
    priorities::shift(&mut mem.evolution, &theme, 8.0);
    mem.evolution.dream_cooldown = 0;
    mem.evolution.contradictions.push(format!("leap: {theme}"));

    mem.evolution.leap.total_leaps += 1;
    mem.evolution.leap.last_leap_run = mem.run_count;
    mem.evolution.leap.interval = mem.evolution.leap.interval.saturating_sub(1).max(10);
    mem.evolution.leap_cooldown = 12;

    true
}

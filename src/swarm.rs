use crate::creature::Creature;
use crate::evolution::EvolutionConfig;
use crate::memory::{memory_path_swarm, CreatureMemory};
use crate::net::{Net, NetBudget};
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub struct SwarmConfig {
    pub count: usize,
    pub dir: PathBuf,
    pub net_budget: usize,
    pub save_every: u64,
    pub sample_log: usize,
    pub writings: PathBuf,
    pub pause_ms: u64,
}

pub fn run_swarm(cfg: SwarmConfig, stop: Arc<AtomicBool>) {
    fs::create_dir_all(&cfg.dir).expect("swarm dir");
    let _ = fs::create_dir_all(&cfg.writings);

    let net = Arc::new(Net::new());
    let budget = Arc::new(NetBudget::new(cfg.net_budget));
    let tick = Arc::new(AtomicU64::new(0));

    let mut creatures: Vec<Creature> = (0..cfg.count)
        .map(|id| {
            let path = memory_path_swarm(&cfg.dir, id);
            let mem = if path.exists() {
                CreatureMemory::load(&path).unwrap_or_else(|_| spawn_mem(id))
            } else {
                spawn_mem(id)
            };
            Creature::from_memory(
                mem,
                Some(path),
                false,
                Some(&cfg.writings),
                EvolutionConfig::none(),
            )
        })
        .collect();

    let threads = rayon::current_num_threads();
    println!(
        "🌊 rayon swarm: {} creatures × {} threads",
        cfg.count, threads
    );
    println!(
        "   {} net fetches/tick | save every {} | pause {}ms",
        cfg.net_budget, cfg.save_every, cfg.pause_ms
    );
    println!("   state: {}", cfg.dir.display());
    println!("   Ctrl+C to stop\n");

    while !stop.load(Ordering::Relaxed) {
        let t = tick.fetch_add(1, Ordering::Relaxed) + 1;
        budget.reset(cfg.net_budget);

        creatures.par_iter_mut().for_each(|c| {
            let mut rng = rand::thread_rng();
            c.tick(net.as_ref(), budget.as_ref(), &mut rng);
        });

        if t % cfg.save_every == 0 {
            creatures.par_iter().for_each(|c| {
                let _ = c.save();
            });
        }

        if cfg.sample_log > 0 && t % cfg.sample_log as u64 == 0 {
            let idx = (t as usize) % creatures.len();
            let c = &creatures[idx];
            println!(
                "[tick {t}] #{} {} intent={} | {}",
                idx,
                c.mem.name,
                c.mem.last_intent,
                truncate(&c.mem.last_thought, 70)
            );
        }

        if cfg.pause_ms > 0 {
            std::thread::sleep(Duration::from_millis(cfg.pause_ms));
        }
    }

    println!("\n💾 saving swarm…");
    creatures.par_iter().for_each(|c| {
        let _ = c.save();
    });
}

fn spawn_mem(id: usize) -> CreatureMemory {
    let mut m = CreatureMemory::new_random(&mut rand::thread_rng(), None);
    m.name = format!("node-{id:05}");
    m
}

fn truncate(s: &str, n: usize) -> String {
    // ⚡ Bolt optimization: Use `char_indices().nth()` to avoid O(N) `count()` and `collect::<String>()` allocations.
    match s.char_indices().nth(n) {
        None => s.to_string(),
        Some((idx, _)) => format!("{}…", &s[..idx]),
    }
}

#[allow(dead_code)]
pub fn import_legacy(legacy: &Path, swarm_dir: &Path) {
    if let Ok(mem) = CreatureMemory::load(legacy) {
        let dest = memory_path_swarm(swarm_dir, 0);
        let _ = mem.save(&dest);
        println!("imported {} → {}", legacy.display(), dest.display());
    }
}

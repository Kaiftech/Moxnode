mod creature;
mod decide;
mod evolution;
mod memory;
mod net;
mod selfwrite;
mod swarm;
mod thought;
mod wired;

use clap::Parser;
use creature::{load_or_create, Creature};
use evolution::EvolutionConfig;
use memory::DEFAULT_MEMORY;
use net::{Net, NetBudget};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(
    name = "moxnode",
    about = "Digital creatures — rayon-fast, runs forever, writes itself"
)]
struct Cli {
    #[arg(short, long, default_value = DEFAULT_MEMORY)]
    memory: PathBuf,

    #[arg(short, long, default_value = "writings")]
    writings: PathBuf,

    #[arg(long, help = "One tick then exit")]
    once: bool,

    #[arg(long, help = "Show state only")]
    status: bool,

    /// Parallel creatures with rayon, e.g. --swarm 1000
    #[arg(long)]
    swarm: Option<usize>,

    #[arg(long, default_value = "swarm")]
    swarm_dir: PathBuf,

    #[arg(long, default_value = "12")]
    net_budget: usize,

    #[arg(long, default_value = "25")]
    save_every: u64,

    #[arg(long, default_value = "50")]
    sample_every: u64,

    /// Rayon worker threads (default = all CPU cores).
    #[arg(long)]
    threads: Option<usize>,

    /// Shorthand: swarm of N×CPU cores, minimal pause — maximum throughput.
    #[arg(long)]
    fast: bool,

    /// Single-creature mode: ~80ms between ticks instead of ~3s.
    #[arg(long)]
    turbo: bool,

    /// Swarm pause between ticks in ms (0 = peg CPU).
    #[arg(long, default_value = "2")]
    pause_ms: u64,

    /// Enable all evolution subsystems.
    #[arg(long)]
    evolution: bool,

    #[arg(long)]
    enable_internet: bool,

    #[arg(long)]
    enable_memory_decay: bool,

    #[arg(long)]
    enable_society: bool,

    #[arg(long)]
    enable_habits: bool,

    #[arg(long)]
    enable_priorities: bool,

    #[arg(long)]
    enable_dreams: bool,

    #[arg(long)]
    enable_rituals: bool,

    #[arg(long)]
    enable_expression: bool,

    #[arg(long)]
    enable_leaps: bool,
}

fn main() {
    let cli = Cli::parse();
    init_rayon(cli.threads);
    let evo = EvolutionConfig::from_cli(
        cli.evolution,
        cli.enable_internet,
        cli.enable_memory_decay,
        cli.enable_society,
        cli.enable_habits,
        cli.enable_priorities,
        cli.enable_dreams,
        cli.enable_rituals,
        cli.enable_expression,
        cli.enable_leaps,
    );
    print_banner();

    let stop = Arc::new(AtomicBool::new(false));
    install_ctrlc(Arc::clone(&stop));

    if cli.status || cli.once {
        return run_single(&cli, &stop, evo);
    }

    if cli.fast || cli.swarm.is_some() && cli.swarm != Some(0) {
        let count = resolve_swarm_count(&cli);
        swarm::run_swarm(
            swarm::SwarmConfig {
                count,
                dir: cli.swarm_dir,
                net_budget: cli.net_budget,
                save_every: cli.save_every,
                sample_log: cli.sample_every as usize,
                writings: cli.writings,
                pause_ms: if cli.fast { 0 } else { cli.pause_ms },
            },
            stop,
        );
        return;
    }

    run_single_loop(&cli, stop, evo);
}

fn resolve_swarm_count(cli: &Cli) -> usize {
    if let Some(n) = cli.swarm {
        return n;
    }
    if cli.fast {
        return num_cpus::get().saturating_mul(8).max(32);
    }
    num_cpus::get().saturating_mul(4).max(16)
}

fn init_rayon(threads: Option<usize>) {
    let n = threads.unwrap_or_else(num_cpus::get).max(1);
    rayon::ThreadPoolBuilder::new()
        .num_threads(n)
        .build_global()
        .expect("rayon thread pool");
}

fn install_ctrlc(stop: Arc<AtomicBool>) {
    ctrlc::set_handler(move || {
        stop.store(true, Ordering::Relaxed);
    })
    .expect("ctrl+c");
}

/// Default: one Moxnode, infinite loop, saves every tick.
fn run_single_loop(cli: &Cli, stop: Arc<AtomicBool>, evo: EvolutionConfig) {
    let mut creature = load_or_create(&cli.memory, true, Some(&cli.writings), evo);
    let net = Net::new();
    let budget = NetBudget::new(cli.net_budget);

    println!(
        "🔄 {} — infinite loop (rayon pool: {} threads, Ctrl+C saves)\n",
        creature.mem.name,
        rayon::current_num_threads()
    );

    while !stop.load(Ordering::Relaxed) {
        tick_once(&mut creature, &net, &budget, cli);
        let _ = creature.save();
        let mut rng = rand::thread_rng();
        thread::sleep(Duration::from_millis(
            creature.sleep_ms(&mut rng, cli.turbo),
        ));
    }

    println!("\n🌙 goodnight");
    let _ = creature.save();
}

fn run_single(cli: &Cli, _stop: &Arc<AtomicBool>, evo: EvolutionConfig) {
    let mut creature = load_or_create(&cli.memory, true, Some(&cli.writings), evo);
    if cli.status {
        creature.reflect();
        return;
    }
    let net = Net::new();
    let budget = NetBudget::new(cli.net_budget);
    tick_once(&mut creature, &net, &budget, cli);
    creature.reflect();
    let _ = creature.save();
}

fn tick_once(creature: &mut Creature, net: &Net, budget: &NetBudget, cli: &Cli) {
    println!("{}", "─".repeat(48));
    println!("🌟 run #{}", creature.mem.run_count + 1);
    budget.reset(cli.net_budget);
    let mut rng = rand::thread_rng();
    creature.tick(net, budget, &mut rng);
}

fn print_banner() {
    println!();
    println!("  ╭──────────────────────────────────────────╮");
    println!("  │  MOXNODE — rayon · binary · forever loop │");
    println!("  ╰──────────────────────────────────────────╯");
    println!("  moxnode.exe              # your creature, loops forever");
    println!("  moxnode.exe --fast       # rayon swarm, max speed");
    println!("  moxnode.exe --swarm 1000 # 1000 parallel creatures");
    println!("  moxnode.exe --evolution   # full evolution layer");
    println!();
}

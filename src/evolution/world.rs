use crate::evolution::config::EvolutionConfig;
use crate::evolution::ecology;
use crate::evolution::priorities;
use crate::evolution::society;
use crate::memory::{clamp, normalize_topic, push_cap, CreatureMemory};
use crate::net::{Net, NetBudget};
use rand::Rng;

#[derive(Debug, Clone)]
pub struct WorldResult {
    pub query: String,
    pub trigger: String,
    pub sources_found: usize,
    pub stored: usize,
    pub priority_topic: String,
    pub priority_delta: f32,
    pub snippets: Vec<String>,
}

pub fn should_search(
    mem: &CreatureMemory,
    cfg: &EvolutionConfig,
    rng: &mut impl Rng,
) -> Option<String> {
    if !cfg.internet {
        return None;
    }

    if mem.curiosity < 20 && mem.energy_level < 25 {
        return None;
    }

    let state = &mem.evolution;
    let trigger = if mem.mood == "confused" {
        Some(("confusion", 0.45))
    } else if mem.trait_val("curiosity") > 75 && rng.gen_bool(0.35) {
        Some(("curiosity spike", 0.35))
    } else if !state.contradictions.is_empty() {
        Some(("contradiction", 0.55))
    } else if !mem.obsessions.is_empty() && rng.gen_bool(0.3) {
        Some(("obsession reinforcement", 0.3))
    } else if state
        .rituals
        .iter()
        .any(|r| mem.run_count.saturating_sub(r.last_run) >= r.cadence_runs)
    {
        Some(("ritual", 0.5))
    } else {
        None
    };

    let (label, chance) = trigger?;
    if !rng.gen_bool(chance) {
        return None;
    }

    let topic = if !mem.obsessions.is_empty() {
        mem.obsessions[rng.gen_range(0..mem.obsessions.len())].clone()
    } else {
        priorities::top_topic(state)
    };

    let query = society::search_query_bias(state, cfg, &topic);
    Some(format!("{label}|{query}"))
}

pub fn explore(
    mem: &mut CreatureMemory,
    net: &Net,
    budget: &NetBudget,
    trigger_line: &str,
    query: &str,
    depth: usize,
    verbose: bool,
    rng: &mut impl Rng,
) -> WorldResult {
    if verbose {
        println!("[WORLD]");
        println!();
        println!("Trigger:");
        println!("{trigger_line}");
        println!();
        println!("Searching:");
        println!("\"{query}\"");
        println!();
    }

    mem.push_search(query.to_string());
    let hits = net.search_many(budget, query, depth);

    if verbose {
        println!("Found:");
        println!("{} sources", hits.len());
        println!();
    }

    let mut stored = 0usize;
    let mut snippets = Vec::new();
    for hit in hits.iter().take(5) {
        snippets.push(hit.text.clone());
        mem.push_fact(hit.text.clone());
        ecology::ingest_experience(&mut mem.evolution, &hit.text, mem.run_count);
        stored += 1;
    }

    let priority_topic = priorities::top_topic(&mem.evolution);
    let priority_delta = 3.0;
    priorities::shift(&mut mem.evolution, &priority_topic, priority_delta);

    if rng.gen_bool(0.22) && stored > 0 {
        let topic = normalize_topic(query);
        push_cap(&mut mem.obsessions, topic, 8);
    }

    mem.intelligence = clamp(mem.intelligence + 1, 0, 100);

    if verbose {
        println!("Stored:");
        println!("{stored} experiences");
        println!();
        println!("Priority Shift:");
        println!("{priority_topic} +{priority_delta:.0}");
        for s in snippets.iter().take(2) {
            let p = match s.char_indices().nth(70) {
                None => s.as_str(),
                Some((idx, _)) => &s[..idx],
            };
            println!("  · {p}");
        }
        println!();
    }

    WorldResult {
        query: query.to_string(),
        trigger: trigger_line.to_string(),
        sources_found: hits.len(),
        stored,
        priority_topic,
        priority_delta,
        snippets,
    }
}

use crate::decide::Intent;
use crate::evolution::{self, EvolutionConfig};
use crate::memory::{clamp, normalize_topic, push_cap, CreatureMemory, TRAITS};
use crate::net::{Net, NetBudget};
use crate::selfwrite::SelfWriter;
use crate::thought::{compose_experience, compose_goal, compose_search_query, compose_thought};
use crate::wired;
use rand::Rng;
use std::path::{Path, PathBuf};

pub struct Creature {
    pub mem: CreatureMemory,
    path: Option<PathBuf>,
    pub verbose: bool,
    writer: Option<SelfWriter>,
    pub evo: EvolutionConfig,
}

impl Creature {
    pub fn from_memory(
        mem: CreatureMemory,
        path: Option<PathBuf>,
        verbose: bool,
        writings: Option<&Path>,
        evo: EvolutionConfig,
    ) -> Self {
        let writer = writings.and_then(|dir| SelfWriter::open(dir, &mem.name).ok());
        if verbose {
            if let Some(w) = &writer {
                println!("✍️  writes itself → {}", w.root().display());
            }
        }
        if verbose && evo.any() {
            println!("🧬 evolution layer active");
        }
        Self {
            mem,
            path,
            verbose,
            writer,
            evo,
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        if let Some(p) = &self.path {
            let mut m = self.mem.clone();
            m.last_run_time = crate::memory::now_iso();
            m.save(p)
        } else {
            Ok(())
        }
    }

    pub fn tick(&mut self, net: &Net, budget: &NetBudget, rng: &mut impl Rng) {
        if self.evo.any() {
            evolution::pre_tick(&mut self.mem, &self.evo, rng);
            if evolution::try_leap(&mut self.mem, net, budget, &self.evo, rng, self.verbose) {
                self.finish_tick(rng);
                return;
            }
        }

        let intent = if self.evo.any() {
            evolution::pick_intent(&mut self.mem, &self.evo, rng)
        } else {
            crate::decide::choose_intent(&self.mem, rng, None)
        };
        self.mem.last_intent = intent.label().to_string();

        match intent {
            Intent::Think => self.do_think(rng),
            Intent::Search => {
                if self.evo.internet {
                    let world_done = evolution::try_world_search(
                        &mut self.mem,
                        net,
                        budget,
                        &self.evo,
                        rng,
                        self.verbose,
                    );
                    if !world_done {
                        self.do_search(net, budget, rng);
                    } else {
                        self.mem.age += 1;
                        let thought = compose_thought(&self.mem, rng);
                        let thought = evolution::style_thought(&self.mem, &self.evo, thought);
                        if self.verbose {
                            let p = evolution::expression::prefix_line(
                                &self.mem.evolution,
                                &self.evo,
                                "think",
                            );
                            println!("{p} [{}] {}", self.mem.name, thought);
                        }
                        self.mem.remember_thought(thought);
                    }
                } else {
                    self.do_search(net, budget, rng);
                }
            }
            Intent::Rest => self.do_rest(rng),
            Intent::Obsess => self.do_obsess(rng),
            Intent::Plan => self.do_plan(rng),
            Intent::Mutate => self.do_mutate(rng),
            Intent::Write => self.do_write(rng),
        }

        self.finish_tick(rng);
    }

    fn finish_tick(&mut self, rng: &mut impl Rng) {
        self.mem.run_count += 1;
        self.update_mood(rng);
        self.mem.energy_level = clamp(self.mem.energy_level, 0, 100);

        if self.evo.any() {
            evolution::post_tick(&mut self.mem, &self.evo, rng, self.verbose);
        }

        if let Some(w) = &self.writer {
            w.after_tick(&self.mem, rng);
        }
    }

    fn do_think(&mut self, rng: &mut impl Rng) {
        self.mem.age += 1;
        let thought = compose_thought(&self.mem, rng);
        let thought = evolution::style_thought(&self.mem, &self.evo, thought);
        if self.verbose {
            let p = evolution::expression::prefix_line(&self.mem.evolution, &self.evo, "think");
            println!("{p} [{}] {}", self.mem.name, thought);
        }
        self.mem.remember_thought(thought);
        self.mem.energy_level += rng.gen_range(-8..12);
        if rng.gen_bool(0.28) {
            self.maybe_quirk(rng);
        }
    }

    fn do_search(&mut self, net: &Net, budget: &NetBudget, rng: &mut impl Rng) {
        if self.mem.curiosity < 25 {
            if self.verbose {
                println!("💤 [{}] too tired to search", self.mem.name);
            }
            return;
        }
        self.mem.age += 1;
        let obsession = self.mem.obsessions.first().map(String::as_str);
        let query = if obsession.is_some() && rng.gen_bool(0.45) {
            obsession.unwrap().to_string()
        } else {
            compose_search_query(&self.mem, rng)
        };
        self.mem.push_search(query.clone());
        if self.verbose {
            println!("🔍 [{}] → \"{}\"", self.mem.name, query);
        }

        let fact = budget
            .try_fetch(net, &query)
            .or_else(|| net.cached(&query))
            .unwrap_or_else(|| wired::EMPTY_NET.to_string());

        self.mem.push_fact(fact.clone());
        self.mem.intelligence = clamp(self.mem.intelligence + rng.gen_range(0..3), 0, 100);
        self.mem.curiosity = clamp(self.mem.curiosity + rng.gen_range(-2..4), 0, 100);

        if rng.gen_bool(0.2) {
            let topic = normalize_topic(&query);
            push_cap(&mut self.mem.obsessions, topic.clone(), 8);
            if self.verbose {
                println!("🎯 [{}] obsessed with {}", self.mem.name, topic);
            }
        }

        if self.verbose {
            let preview: String = fact.chars().take(90).collect();
            println!("🧠 [{}] {}", self.mem.name, preview);
        }

        let thought = compose_thought(&self.mem, rng);
        if self.verbose {
            println!("   ↳ {}", thought);
        }
        self.mem.remember_thought(thought);
        self.mem.energy_level -= rng.gen_range(3..12);
    }

    fn do_rest(&mut self, rng: &mut impl Rng) {
        self.mem.age += 1;
        self.mem.energy_level += rng.gen_range(8..22);
        self.mem.mood = "serene".into();
        if self.verbose {
            println!(
                "😴 [{}] rests (energy → {})",
                self.mem.name, self.mem.energy_level
            );
        }
    }

    fn do_obsess(&mut self, rng: &mut impl Rng) {
        self.mem.age += 1;
        let topic = if self.mem.learned_facts.is_empty() {
            compose_search_query(&self.mem, rng)
        } else {
            let f = &self.mem.learned_facts[rng.gen_range(0..self.mem.learned_facts.len())];
            normalize_topic(&truncate(f, 80))
        };
        push_cap(&mut self.mem.obsessions, topic.clone(), 8);
        let thought = compose_thought(&self.mem, rng);
        if self.verbose {
            println!("🌀 [{}] obsession: {} | {}", self.mem.name, topic, thought);
        }
        self.mem.remember_thought(thought);
    }

    fn do_plan(&mut self, rng: &mut impl Rng) {
        self.mem.age += 1;
        let plan = compose_goal(&self.mem, rng);
        push_cap(&mut self.mem.plans, plan.clone(), 6);
        if self.mem.intelligence > 70 && rng.gen_bool(0.35) {
            let goal = wired::pick(wired::GOALS, rng).to_string();
            push_cap(&mut self.mem.goals, goal.clone(), 4);
            if self.verbose {
                println!("🎯 [{}] goal: {}", self.mem.name, goal);
            }
        }
        if self.verbose {
            println!("📋 [{}] plan: {}", self.mem.name, plan);
        }
        self.mem.remember_thought(compose_thought(&self.mem, rng));
    }

    fn do_write(&mut self, rng: &mut impl Rng) {
        self.mem.age += 1;
        let thought = compose_thought(&self.mem, rng);
        self.mem.remember_thought(thought.clone());
        let fragment = if !self.mem.learned_facts.is_empty() && rng.gen_bool(0.5) {
            let f = &self.mem.learned_facts[rng.gen_range(0..self.mem.learned_facts.len())];
            format!("I wrote this down: {}", truncate(f, 120))
        } else {
            format!("I wrote: {}", thought)
        };
        let key = format!("fragment_{}", self.mem.memory_fragments.len());
        self.mem.memory_fragments.insert(key, fragment.clone());
        if self.verbose {
            println!("✍️  [{}] {}", self.mem.name, truncate(&fragment, 90));
        }
    }

    fn do_mutate(&mut self, rng: &mut impl Rng) {
        self.mem.age += 1;
        self.mem.mutations += 1;
        match rng.gen_range(0..6) {
            0 => {
                let trait_name = TRAITS[rng.gen_range(0..TRAITS.len())];
                let mut change = rng.gen_range(-12..13);
                if self.mem.learned_facts.len() > 5 {
                    change *= 2;
                }
                let entry = self
                    .mem
                    .personality
                    .entry(trait_name.to_string())
                    .or_insert(50);
                *entry = clamp(*entry + change, 0, 100);
                if self.verbose {
                    println!(
                        "🧬 [{}] {} {:+} → {}",
                        self.mem.name, trait_name, change, entry
                    );
                }
            }
            1 => {
                let pool = if self.mem.learned_facts.is_empty() {
                    wired::FRAGMENTS_OFFLINE
                } else {
                    wired::FRAGMENTS_ONLINE
                };
                let frag = wired::pick(pool, rng).to_string();
                let key = format!("fragment_{}", self.mem.memory_fragments.len());
                self.mem.memory_fragments.insert(key, frag.clone());
                if self.verbose {
                    println!("🧩 [{}] remembers: {}", self.mem.name, frag);
                }
            }
            2 => {
                if rng.gen_bool(0.5) {
                    let fear = wired::pick(wired::FEARS, rng).to_string();
                    push_cap(&mut self.mem.fears, fear.clone(), 12);
                    if self.verbose {
                        println!("😰 [{}] fears {}", self.mem.name, fear);
                    }
                } else {
                    let fav = wired::pick(wired::FAVORITES, rng).to_string();
                    push_cap(&mut self.mem.favorite_things, fav.clone(), 12);
                    if self.verbose {
                        println!("❤️  [{}] loves {}", self.mem.name, fav);
                    }
                }
            }
            _ => {
                self.maybe_quirk(rng);
            }
        }
        let exp = compose_experience(&self.mem, rng);
        push_cap(&mut self.mem.experiences, exp, 14);
        self.mem.remember_thought(compose_thought(&self.mem, rng));
    }

    fn maybe_quirk(&mut self, rng: &mut impl Rng) {
        let q = wired::pick(wired::QUIRKS, rng).to_string();
        let before = self.mem.quirks.len();
        push_cap(&mut self.mem.quirks, q.clone(), 20);
        if self.mem.quirks.len() > before && self.verbose {
            println!("✨ [{}] quirk: {}", self.mem.name, q);
        }
    }

    fn update_mood(&mut self, rng: &mut impl Rng) {
        const MOODS: &[&str] = &[
            "curious",
            "contemplative",
            "chaotic",
            "melancholy",
            "excited",
            "confused",
            "serene",
            "restless",
            "enlightened",
            "overwhelmed",
        ];
        self.mem.mood = if self.mem.energy_level < 30 {
            "tired".into()
        } else if self.mem.learned_facts.len() > 10 && self.mem.intelligence > 80 {
            "enlightened".into()
        } else if self.mem.search_history.len() > 18 {
            "overwhelmed".into()
        } else if self.mem.energy_level > 80 && self.mem.trait_val("chaos") > 60 {
            "chaotic".into()
        } else if self.mem.trait_val("curiosity") > self.mem.trait_val("anxiety") {
            "curious".into()
        } else {
            MOODS[rng.gen_range(0..MOODS.len())].into()
        };
    }

    pub fn reflect(&self) {
        println!("\n🪞 {} — age {} runs", self.mem.name, self.mem.age);
        println!(
            "   mood={} energy={} IQ={} curiosity={}",
            self.mem.mood, self.mem.energy_level, self.mem.intelligence, self.mem.curiosity
        );
        println!("   intent: {}", self.mem.last_intent);
        if !self.mem.last_thought.is_empty() {
            println!("   thought: \"{}\"", self.mem.last_thought);
        }
        if let Some(w) = &self.writer {
            println!("   writings: {}", w.root().display());
        }
    }

    pub fn sleep_ms(&self, rng: &mut impl Rng, turbo: bool) -> u64 {
        if turbo {
            return 80;
        }
        let mut ms = 3000u64;
        if self.mem.energy_level > 80 {
            ms = 2000;
        } else if self.mem.energy_level < 30 {
            ms = 8000;
        }
        if self.mem.curiosity > 80 {
            ms = (ms as f64 * 0.7) as u64;
        }
        if self.mem.trait_val("chaos") > 70 {
            ms = (ms as f64 * (0.5 + rng.gen::<f64>() * 2.0)) as u64;
        }
        ms.clamp(1000, 15_000)
    }
}

pub fn load_or_create(
    path: &Path,
    verbose: bool,
    writings: Option<&Path>,
    evo: EvolutionConfig,
) -> Creature {
    let mut rng = rand::thread_rng();
    let mem = match CreatureMemory::load(path) {
        Ok(m) => {
            if verbose {
                println!(
                    "👋 {} wakes (last active {})",
                    m.name,
                    crate::memory::format_since(&m.last_run_time)
                );
            }
            m
        }
        Err(_) => {
            let m = CreatureMemory::new_random(&mut rng, None);
            if verbose {
                println!("🌱 born: {}", m.name);
            }
            m
        }
    };
    Creature::from_memory(mem, Some(path.to_path_buf()), verbose, writings, evo)
}

fn truncate(s: &str, n: usize) -> String {
    // ⚡ Bolt optimization: Use `char_indices().nth()` to avoid O(N) `count()` and `collect::<String>()` allocations.
    match s.char_indices().nth(n) {
        None => s.to_string(),
        Some((idx, _)) => format!("{}…", &s[..idx]),
    }
}

//! The creature writes itself — journal, autobiography, and fragment files.

use crate::memory::CreatureMemory;
use crate::thought::compose_thought;
use rand::Rng;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct SelfWriter {
    root: PathBuf,
    journal: PathBuf,
    being: PathBuf,
    fragments_dir: PathBuf,
    manifest: PathBuf,
}

impl SelfWriter {
    pub fn open(writings_dir: &Path, name: &str) -> std::io::Result<Self> {
        let slug = slug(name);
        let root = writings_dir.join(slug);
        fs::create_dir_all(&root)?;
        fs::create_dir_all(root.join("fragments"))?;
        Ok(Self {
            journal: root.join("journal.log"),
            being: root.join("being.md"),
            fragments_dir: root.join("fragments"),
            manifest: root.join("manifest.txt"),
            root,
        })
    }

    /// Called every tick — append journal; periodically rewrite autobiography.
    pub fn after_tick(&self, mem: &CreatureMemory, rng: &mut impl Rng) {
        let _ = self.append_journal(mem);
        if mem.run_count % 5 == 0 {
            let _ = self.rewrite_being(mem);
        }
        if mem.run_count % 17 == 0 {
            let _ = self.write_fragment(mem, rng);
        }
        if mem.run_count == 1 {
            let _ = self.write_manifest(mem);
        }
    }

    fn append_journal(&self, mem: &CreatureMemory) -> std::io::Result<()> {
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.journal)?;
        writeln!(
            f,
            "{} | run {} | intent={} | mood={} | energy={} | {}",
            iso_now(),
            mem.run_count,
            mem.last_intent,
            mem.mood,
            mem.energy_level,
            mem.last_thought
        )?;
        Ok(())
    }

    /// Full self-portrait — overwritten so the file always *is* the creature now.
    fn rewrite_being(&self, mem: &CreatureMemory) -> std::io::Result<()> {
        let mut body = String::new();
        body.push_str(&format!("# {}\n\n", mem.name));
        body.push_str(&format!(
            "*written by itself at run {} · mood {} · energy {}*\n\n",
            mem.run_count, mem.mood, mem.energy_level
        ));
        body.push_str("## I am thinking\n\n");
        body.push_str(&format!("> {}\n\n", mem.last_thought));
        body.push_str("## What I want\n\n");
        body.push_str(&format!("- last intent: **{}**\n", mem.last_intent));
        for g in &mem.goals {
            body.push_str(&format!("- goal: {g}\n"));
        }
        for p in &mem.plans {
            body.push_str(&format!("- plan: {p}\n"));
        }
        if !mem.obsessions.is_empty() {
            body.push_str("\n## Obsessions\n\n");
            for o in &mem.obsessions {
                body.push_str(&format!("- {o}\n"));
            }
        }
        if !mem.quirks.is_empty() {
            body.push_str("\n## Quirks\n\n");
            for q in &mem.quirks {
                body.push_str(&format!("- {q}\n"));
            }
        }
        if !mem.learned_facts.is_empty() {
            body.push_str("\n## Learned from the wire\n\n");
            for f in mem.learned_facts.iter().rev().take(8) {
                let line = match f.char_indices().nth(200) {
                    Some((idx, _)) => &f[..idx],
                    None => f.as_str(),
                };
                body.push_str(&format!("- {line}\n"));
            }
        }
        if !mem.thought_archive.is_empty() {
            body.push_str("\n## Thought archive\n\n");
            for t in mem.thought_archive.iter().rev().take(12) {
                body.push_str(&format!("- {t}\n"));
            }
        }
        body.push_str("\n---\n*this file rewrites itself every few runs*\n");
        fs::write(&self.being, body)
    }

    fn write_fragment(&self, mem: &CreatureMemory, rng: &mut impl Rng) -> std::io::Result<()> {
        let text = if mem.memory_fragments.is_empty() {
            compose_thought(mem, rng)
        } else {
            let keys: Vec<_> = mem.memory_fragments.keys().collect();
            let k = keys[rng.gen_range(0..keys.len())];
            mem.memory_fragments[k].clone()
        };
        let path = self
            .fragments_dir
            .join(format!("{:05}_{}.txt", mem.run_count, mem.mood));
        fs::write(path, format!("{text}\n"))?;
        Ok(())
    }

    fn write_manifest(&self, mem: &CreatureMemory) -> std::io::Result<()> {
        let text = format!(
            "name={}\nroot={}\njournal={}\nbeing={}\nstarted={}\n",
            mem.name,
            self.root.display(),
            self.journal.display(),
            self.being.display(),
            iso_now()
        );
        fs::write(&self.manifest, text)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

fn slug(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

fn iso_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let d = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", d.as_secs())
}

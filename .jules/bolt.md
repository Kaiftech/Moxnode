## 2024-05-18 - Rayon Mutex Contention
**Learning:** In a Rayon swarm of fast-running creatures, `src/net.rs` uses a single `Mutex<HashMap<String, String>>` for its cache. This shared `Mutex` becomes a massive bottleneck for multi-threaded operation since many creatures try to acquire it simultaneously.
**Action:** Replace `Mutex<HashMap>` with `dashmap::DashMap` or another lock-free/sharded concurrent data structure to eliminate thread contention on the cache in swarm mode.

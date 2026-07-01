## 2024-05-24 - Hidden string allocations in hot Rayon paths
**Learning:** Found multiple places (e.g. `normalize_topic`, `relevance`) doing string allocations (`to_ascii_lowercase()`, `to_string()`, `.collect::<Vec<String>>`) inside tight loops run by Rayon worker threads. These hidden allocations cause significant thread contention over the shared global allocator.
**Action:** Always favor `&str` slicing and `.eq_ignore_ascii_case()` over allocating new strings for comparison, especially inside multi-threaded contexts like Rayon.

## 2026-06-03 - Hidden String Allocations in `to_ascii_lowercase`
**Learning:** In moxnode's multi-threaded `rayon` environment, hidden string allocations from methods like `.to_ascii_lowercase()` inside tight loops cause significant thread contention over the shared global allocator. This degrades performance.
**Action:** Favor `&str` slicing and `.eq_ignore_ascii_case()` over allocating new lowercased `String`s, especially in tight loops or checks.
## 2024-06-04 - Hidden String Allocations in Tight Loops Cause Thread Contention
**Learning:** In moxnode's multi-threaded `rayon` environment, hidden string allocations (e.g., `to_ascii_lowercase()`, `to_string()`, `.collect::<Vec<String>>`) inside tight loops cause significant thread contention over the shared global allocator.
**Action:** Favor `&str` slicing and `.eq_ignore_ascii_case()` over allocating new strings for comparisons or normalizations. Avoid direct indexing like `t[..len]` which can panic on non-UTF-8 boundaries; instead, use safe methods like `t.get(..len).is_some_and(...)`.
## 2023-10-27 - Zero-Allocation Byte Windowing for Case-Insensitive Substring Search
**Learning:** In highly concurrent environments, generating full lowercased text strings just for `.contains()` checks introduces massive contention. Using byte slices and `.windows(len)` combined with `eq_ignore_ascii_case` avoids allocation entirely.
**Action:** When doing case-insensitive substring search in Rust tight loops, use `.as_bytes().windows(t.len()).any(|w| w.eq_ignore_ascii_case(t.as_bytes()))` instead of allocating strings, ensuring `len` > 0 to prevent panics.
## 2024-06-05 - Inverted Indexing for Multi-Document Clustering
**Learning:** In text processing or token overlap routines (like `cluster_memories` in ecology), using an $O(N^2)$ nested loop to count intersecting tokens creates a massive CPU bottleneck for large document sets.
**Action:** Replace nested overlap loops with an inverted index (`HashMap<&Token, Vec<usize>>`). This reduces intersection complexity to $O(N \times M)$ on average. Note: Because `HashMap` iteration order is non-deterministic, explicitly restore original tie-breaking logic (e.g. preferring the smaller `index`) when selecting best matches.

## 2024-05-24 - Hidden string allocations in hot Rayon paths
**Learning:** Found multiple places (e.g. `normalize_topic`, `relevance`) doing string allocations (`to_ascii_lowercase()`, `to_string()`, `.collect::<Vec<String>>`) inside tight loops run by Rayon worker threads. These hidden allocations cause significant thread contention over the shared global allocator.
**Action:** Always favor `&str` slicing and `.eq_ignore_ascii_case()` over allocating new strings for comparison, especially inside multi-threaded contexts like Rayon.

## 2026-06-03 - Hidden String Allocations in `to_ascii_lowercase`
**Learning:** In moxnode's multi-threaded `rayon` environment, hidden string allocations from methods like `.to_ascii_lowercase()` inside tight loops cause significant thread contention over the shared global allocator. This degrades performance.
**Action:** Favor `&str` slicing and `.eq_ignore_ascii_case()` over allocating new lowercased `String`s, especially in tight loops or checks.
## 2024-06-04 - Hidden String Allocations in Tight Loops Cause Thread Contention
**Learning:** In moxnode's multi-threaded `rayon` environment, hidden string allocations (e.g., `to_ascii_lowercase()`, `to_string()`, `.collect::<Vec<String>>`) inside tight loops cause significant thread contention over the shared global allocator.
**Action:** Favor `&str` slicing and `.eq_ignore_ascii_case()` over allocating new strings for comparisons or normalizations. Avoid direct indexing like `t[..len]` which can panic on non-UTF-8 boundaries; instead, use safe methods like `t.get(..len).is_some_and(...)`.
## 2024-06-07 - Rayon Thread Contention from `to_ascii_lowercase()` in Loops
**Learning:** Found another instance in `src/net.rs` where `text.to_ascii_lowercase()` was allocating a new string on every iteration inside a `par_iter()` loop. This reinforces that hidden allocations inside Rayon loops are a major source of thread contention and performance degradation.
**Action:** Always replace `to_ascii_lowercase()` with zero-allocation `&str` slicing and byte window scanning using `haystack.as_bytes().windows(needle.len()).any(|w| w.eq_ignore_ascii_case(needle.as_bytes()))`. Ensure to check `needle.is_empty()` and `haystack.len() < needle.len()` first to avoid panics or extra work.

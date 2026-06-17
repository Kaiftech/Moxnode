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
## 2024-06-17 - Zero-Allocation Byte Windowing for Case-Insensitive Substring Search inside O(N^2) loops
**Learning:** The `overlap` function in `src/evolution/ecology.rs` is called inside an O(N^2) loop (`cluster_memories`). Initially replacing `a.iter().filter(|t| b.contains(t)).count()` with `a.iter().filter(|t| b.iter().any(|x| x.eq_ignore_ascii_case(t))).count()` resulted in a small performance regression. This is because `b.contains(t)` checks pointer equality quickly for `String` references if they are the exact same allocation or length. Iterating and doing `eq_ignore_ascii_case` on every element is slower. To fix this, always check if lengths match (`x.len() != t.len()`) before calling `eq_ignore_ascii_case`.
**Action:** When doing case-insensitive substring search in Rust tight loops, especially O(N^2), use `.as_bytes().windows(t.len()).any(|w| w.eq_ignore_ascii_case(t.as_bytes()))` or length checks first, instead of allocating strings, ensuring `len` > 0 to prevent panics.

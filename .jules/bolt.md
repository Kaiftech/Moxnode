## 2024-05-24 - Hidden string allocations in hot Rayon paths
**Learning:** Found multiple places (e.g. `normalize_topic`, `relevance`) doing string allocations (`to_ascii_lowercase()`, `to_string()`, `.collect::<Vec<String>>`) inside tight loops run by Rayon worker threads. These hidden allocations cause significant thread contention over the shared global allocator.
**Action:** Always favor `&str` slicing and `.eq_ignore_ascii_case()` over allocating new strings for comparison, especially inside multi-threaded contexts like Rayon.

## 2026-06-03 - Hidden String Allocations in `to_ascii_lowercase`
**Learning:** In moxnode's multi-threaded `rayon` environment, hidden string allocations from methods like `.to_ascii_lowercase()` inside tight loops cause significant thread contention over the shared global allocator. This degrades performance.
**Action:** Favor `&str` slicing and `.eq_ignore_ascii_case()` over allocating new lowercased `String`s, especially in tight loops or checks.
## 2024-06-04 - Hidden String Allocations in Tight Loops Cause Thread Contention
**Learning:** In moxnode's multi-threaded `rayon` environment, hidden string allocations (e.g., `to_ascii_lowercase()`, `to_string()`, `.collect::<Vec<String>>`) inside tight loops cause significant thread contention over the shared global allocator.
**Action:** Favor `&str` slicing and `.eq_ignore_ascii_case()` over allocating new strings for comparisons or normalizations. Avoid direct indexing like `t[..len]` which can panic on non-UTF-8 boundaries; instead, use safe methods like `t.get(..len).is_some_and(...)`.
## 2024-06-12 - Refactoring iterator `.collect()` with `String::with_capacity`
**Learning:** For iterating over characters to form strings where size could be known upfront, using `String::with_capacity` in a `for` loop avoids internal buffer re-allocations that can occur when calling `.collect()` on an iterator chain. This is especially true for functions converting characters one-by-one.
**Action:** When replacing string transformations that collect character-by-character, explicitly pre-allocate the final string buffer with `.len()` if the output byte size is known or safely upper-bounded.

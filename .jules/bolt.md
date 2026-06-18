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

## 2024-05-24 - Rust String Truncation Allocation Overhead
**Learning:** `s.chars().take(n).collect::<String>()` allocates memory for every truncated string character. More crucially, performing an explicit bounds check like `s.chars().count() <= n` traverses the entire string $O(N)$, which is drastically inefficient for very large strings (e.g., memory payloads of thousands of chars truncated to the first 100 chars).
**Action:** Always favor slicing via `.char_indices().nth(n)` over `chars().take(n)`. This yields the precise byte boundary and avoids any heap allocation, simply slicing the original reference as `&s[..idx]`.

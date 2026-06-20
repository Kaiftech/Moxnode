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
## 2024-06-05 - Optimize string truncation to avoid allocations
**Learning:** Found multiple places using `.chars().count() <= n` and `.chars().take(n).collect::<String>()` for truncating strings. This is extremely inefficient because counting characters forces an $O(N)$ traversal over the whole string, decoding every single UTF-8 character even if only retaining the first few. Furthermore, creating a new `String` allocates memory on the heap.
**Action:** Use `.char_indices().nth(n)` to lazily and correctly stop decoding at the requested character index. Once found, use `&s[..idx]` to perform a zero-allocation string slice, preventing overhead. This reduces algorithmic complexity to $O(n)$ where $n$ is the target substring length, and strictly avoids heap allocations.

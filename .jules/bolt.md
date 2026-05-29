## 2024-05-29 - Unnecessary String Cloning in dedupe
**Learning:** In `src/memory.rs`, `dedupe` uses `seen.insert(s.clone())` inside `v.retain()`, causing an unnecessary clone for every string checked, even if it is already in the HashSet. `dedupe` is called multiple times on multiple vectors during `sanitize()`, which runs every time a memory is loaded.
**Action:** Modify `dedupe` to check `seen.contains(s)` before cloning and inserting, avoiding allocations for duplicates.

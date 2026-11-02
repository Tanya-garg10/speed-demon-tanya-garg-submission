# Track A: Ingest

The task, the input files and the exact output format are in the [spec](https://github.com/hackathon-raptors/speed-demon-2026/blob/main/specs/a-ingest.md).

## Approach

**Step 1: Simple single-threaded Rust implementation** following the reference algorithm:
1. Read file line by line with `BufReader`
2. Parse each line: split on spaces, parse timestamp to hour, parse latency and status
3. Store latencies in `HashMap<Key, Vec<u32>>` where Key = (hour_string, endpoint)
4. Count 5xx errors in separate `HashMap<Key, u32>`
5. After reading all lines: sort keys, sort each latency vector, compute percentiles
6. Write output with proper formatting

**Step 2: mmap/one big read optimization** (current):
1. Read entire file into memory with `fs::read()` (239 MB fits in 8 GB limit)
2. Parse lines by scanning for `\n` bytes directly in the byte buffer
3. Extract fields by finding space bytes without creating string allocations
4. Parse numbers from byte slices with custom `parse_u32_from_bytes()` function
5. Same data structures and processing as Step 1, but without per-line string allocations

**Benefits of mmap approach:**
- No per-line `String` allocations (each line in Python/naive Rust creates a new string)
- No `split()` overhead (scans bytes directly)
- No `parse()` allocations (parses integers directly from bytes)
- Single memory allocation for entire file vs many small allocations

## Dependencies

None (pure Rust stdlib).

## With another 24 hours

1. Replace line-by-line reading with mmap/whole file reading
2. Hand-written parser for fields (avoid `split()` and `parse()` allocations)
3. Custom hash table with open addressing for endpoints
4. Per-thread local hash tables with 12-way parallel processing
5. Exact percentiles via counting histograms instead of full sort
6. Memory tuning and PGO optimization

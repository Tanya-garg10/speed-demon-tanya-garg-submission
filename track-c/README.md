# Track C: Join

The task, the input files and the exact output format are in the [spec](https://github.com/hackathon-raptors/speed-demon-2026/blob/main/specs/c-join.md).

## Approach

Rust implementation with custom CSV parser that handles all edge cases: BOM handling, quoted fields with embedded commas/newlines, escaped quotes, and mixed CRLF/LF terminators. Uses hashbrown HashMap for fast region lookup and country aggregation. Reads entire CSV into memory for sequential parsing.

## Dependencies

- hashbrown: Fast HashMap implementation
- memchr: Optimized byte searching

## With another 24 hours

Would explore streaming CSV parsing to reduce memory usage for very large datasets.

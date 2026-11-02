# Track B: Index

The task, the input files and the exact output format are in the [spec](https://github.com/hackathon-raptors/speed-demon-2026/blob/main/specs/b-index.md).

## Approach

Rust implementation with an efficient inverted index using hashbrown HashMap for O(1) term lookups. Boolean queries use sorted set operations (intersection, union, difference) on pre-sorted posting lists. Phrase queries intersect posting lists first, then verify token sequences by re-tokenizing only candidate documents. Tokenization is byte-based ASCII-only as specified.

## Dependencies

- hashbrown: Fast HashMap implementation
- memchr: Optimized byte searching

## With another 24 hours

Would explore phrase indexing with position postings to avoid re-tokenization for phrase queries.

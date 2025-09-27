# Performance Optimization Plan

Status after optimization #1 (two-pass estimation + ASCII run batching):

`bench_compare.py` sample shows ~8x speedup vs Python (0.079s -> 0.0096s) on mixed text.

## Next Steps (in order)

1. Bitset Pre-Check per 256-codepoint Block
   - Generate in build.rs a 256-bit bitmap per Unicode block present in Python Unidecode.
   - Before hashing into a PHF map, test bit. If unset, skip lookup.
   - Memory: ~0x110 blocks * 256 bits ≈ 8 KB.
   - Expected gain: reduces hash work on sparse blocks.

2. Dense Block Direct Arrays
   - For blocks with high fill ratio (e.g. >40% mapped), generate an array `&'static [&'static str; 256]` (null/empty for unmapped) instead of PHF.
   - Replace match arm to branch either into array indexing or PHF.
   - Avoid hashing cost entirely on dense blocks.

3. SIMD / memchr Scanning for Non-ASCII Boundaries
   - Replace manual per-byte ASCII run loop with a vectorized scanner (e.g. `memchr::memchr_iter` or custom `std::arch` implementation) to find >=0x80 bytes.
   - Upside: lower branch mispredict rate and faster segmentation on large mostly-ASCII inputs.

4. SmallVec / Stack Buffer for Short Inputs
   - Use `smallvec::SmallVec<[u8; 128]>` to avoid heap alloc for common short strings.
   - Convert to `unsafe { String::from_utf8_unchecked(buf) }` (all output is ASCII) after fill.

5. Optional Parallel Chunking
   - Only if large file transliteration emerges as a use case.

## Bench Targets
Add criterion benchmarks for:
 - ascii_short / ascii_long
 - latin_mixed
 - cjk_sentence
 - emoji_mix
 - random_unicode (sparse mappings)

## Guardrails
 - Golden test must continue to assert bit-for-bit equality vs Python output for every sampled codepoint range.
 - Add unit tests for blocks with both dense and sparse strategies.

## Rollout Strategy
Implement 1 -> measure; commit.
Implement 2 -> measure; commit.
Gate 3 & 4 behind feature flags initially if binary size impact becomes noticeable.

# NOTES — o3 debt cleanup follow-ups

Branch work tracked against `/workspace/Core_engine/o3.md` (2026-09-04 review).

## Done in this pass (`fix/o3-debt-cleanup`)

- **B-1** `Span::checked_shift` → `Option<Span>`; reparse call sites updated.
- **B-2** `Span::try_new` / `Span::new` (panic in all builds); parser sentinel no longer uses `start > end`.
- **B-3** Documented `&str` vs bytes; added `lex_bytes`; Bad tokens never split UTF-8 scalars; CJK/emoji/invalid UTF-8 tests.
- **B-4** Extra depth gates on `parse_expr` / `parse_block` / `parse_stmt`; unary prefixes already iterative; stress tests at 256 / 1k / 10k.
- **B-5** Measured child fanout on a representative program (see `children_fanout_measurement` test). Average fanout ≪ 4; `attach_leaf` is amortized O(1) push — **SmallVec not adopted** (no invented speedup claim). Revisit after a dedicated allocation profile.
- **TD-1** `maude_engine` / `rocq_export` were not present on `C1`; added as **stubs** behind `feature = "experimental"` (not in default API).

## Deferred (explicitly out of scope this pass)

- **TD-2** Full crate split (`core-ast`, `core-lexer`, …) — run `cargo build --timings` first.
- **TD-3** Unified `EngineError` hierarchy / `thiserror`.
- **TD-4** `cargo-fuzz` targets + corpus layout.
- **§6.3** Parser rewrite with LALRPOP / Pest / Chumsky / Tree-sitter — hand-written parser retained for error recovery & span control; evaluate only with measured benefit.
- **B-5** SmallVec adoption pending real alloc/latency numbers on pathological trivia inputs.
- CI already enforces `fmt` + `clippy -D warnings`; optional `cargo audit` / `cargo deny` / Miri / sanitizers still follow-ups.

## Feature flag

```bash
cargo test --features experimental
```

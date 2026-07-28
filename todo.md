# TPT Lexicon — Implementation & Release Checklist

Tracks work from `spec.txt` through a coordinated `v0.1.0` release of all six
crates on crates.io. Phases mirror spec §7; goals mirror spec §4; risks
mirror spec §8.

## Phase 0 — Scaffold & Repo

- [x] Cargo workspace (`resolver = "2"`, shared `[workspace.package]` / `[workspace.dependencies]`)
- [x] Six crate stubs (`core`, `ingest`, `ir`, `verify`, `translate`, `gpu`) — compile, have smoke tests, `no_std` where applicable
- [x] Dual license: `LICENSE-MIT` + `LICENSE-APACHE`
- [x] `.gitignore`, `rust-toolchain.toml` (stable + clippy/rustfmt)
- [x] CI workflow: fmt, clippy, build+test (Linux/macOS/Windows), `no_std` target check, doc build
- [x] Root `README.md` with crate table and design-goal summary
- [x] Create the GitHub repo, then replace every `REPLACE_ME` placeholder in `Cargo.toml`/`workspace.package.repository`, crate READMEs, and root README with the real URL
- [ ] `git init` + initial commit
- [ ] Push to GitHub, confirm CI workflow runs

## Phase 1 — Foundation & Core (spec §7, weeks 1–4)

- [x] Design zero-copy byte-slicing API surface for `tpt-lexicon-core` (buffer ownership model, lifetimes, caller-supplied allocator story for `no_std`)
- [x] Implement baseline (sequential) BPE tokenizer as a correctness reference
- [ ] Implement parallel prefix-sum BPE algorithm
- [ ] Implement SRAM-native vocabulary mapping (cache/shared-memory-resident lookup structure)
- [x] Vocabulary loading format (internal binary layout, versioned)
- [x] Unit tests: round-trip encode/decode correctness against reference BPE
- [ ] Property tests: arbitrary UTF-8 input never panics, never allocates outside caller buffers
- [ ] Benchmark harness (criterion or similar) vs. Hugging Face `tokenizers` reference implementation
- [ ] Document and validate **G1**: sub-millisecond tokenization of 100K-token input (record hardware baseline)
- [ ] Document and validate **G2**: audit crate for zero hidden allocation (miri / allocator-counting test)

## Phase 2 — IR & Fractal Compression (spec §7, weeks 5–10)

- [x] `tpt-lexicon-ingest`: streaming parser core (chunk iterator API, no full-string buffering)
- [ ] `tpt-lexicon-ingest`: Tree-sitter interop for Rust
- [ ] `tpt-lexicon-ingest`: Tree-sitter interop for TypeScript
- [x] `tpt-lexicon-ingest`: Markdown boundary detection
- [x] `tpt-lexicon-ingest`: JSON boundary detection
- [x] `tpt-lexicon-ingest`: natural-language paragraph boundary detection
- [x] `tpt-lexicon-ir`: typed AST/IR schema definition
- [x] `tpt-lexicon-ir`: serialization format (serde or custom, versioned, diffable)
- [x] `tpt-lexicon-ir`: boilerplate/repetition detection for structured data (code, JSON, logs)
- [x] `tpt-lexicon-ir`: recursive grammar-rule compression for detected sub-graphs
- [x] `tpt-lexicon-ir`: standard semantic pooling path for open-ended natural language (per Risk mitigation, spec §8)
- [ ] Tests: compression round-trips losslessly (decompressed IR == original structure) (unchecked 2026-07-28 review: existing tests only cover full-payload-match patterns; real substring matches lose data on decompress, see Review Findings A1)
- [ ] Document and validate **G3**: measure O(log N) context scaling on representative boilerplate-heavy corpora

## Phase 3 — The Translation Bridge (spec §7, weeks 11–14)

- [x] `tpt-lexicon-translate`: JIT unrolling of compressed IR into discrete legacy token-ID sequences
- [x] `tpt-lexicon-translate`: HuggingFace `tokenizer.json` loader
- [ ] `tpt-lexicon-translate`: map loaded HF vocabularies into `tpt-lexicon-core` zero-copy structures
- [x] `tpt-lexicon-translate`: cross-IR export — Tree-sitter AST
- [x] `tpt-lexicon-translate`: cross-IR export — LSP
- [x] `tpt-lexicon-translate`: cross-IR export — LLVM IR
- [ ] Integration test: compressed IR → bridge → token IDs → feed into a standard GGUF model, confirm output matches native tokenization
- [ ] Document and validate **G4**: 100% compatibility test suite against a representative set of HF/BPE tokenizers (Llama, Mistral, GPT-family)
- [ ] Address Risk (spec §8): benchmark JIT unrolling latency; if not hideable behind first LLM layers, prototype GPU-parallel unrolling in `tpt-lexicon-gpu`

## Phase 4 — Verification & GPU (spec §7, weeks 15–20)

- [x] `tpt-lexicon-verify`: define structural invariant set (balanced delimiters, valid type references, borrow-rule validity, etc.)
- [x] `tpt-lexicon-verify`: formal proof contracts for IR edits (spec the proof system/approach — e.g. SMT-backed, type-driven, or custom checker)
- [ ] `tpt-lexicon-verify`: rejection path wired into `tpt-lexicon-translate` before text rendering (unchecked 2026-07-28 review: `tpt-lexicon-translate`'s `Cargo.toml` has no dependency on `tpt-lexicon-verify` — no wiring exists yet, see Review Findings)
- [x] Tests: adversarial/malformed IR inputs are all rejected; well-formed IR always passes
- [ ] Document and validate **G5**: 100% structural validity on a corpus of generated + adversarial IR samples
- [ ] `tpt-lexicon-gpu`: wgpu backend for SRAM-native parallel tokenizer
- [ ] `tpt-lexicon-gpu`: optional raw CUDA backend (feature-gated)
- [ ] `tpt-lexicon-gpu`: optional raw Metal backend (feature-gated)
- [x] Confirm core stack builds and runs correctly with `tpt-lexicon-gpu` entirely excluded (**G6** composability check)
- [ ] End-to-end demo: 100K-line codebase → ingest → compressed IR → verified → translated to legacy tokens → inferred on a real model
- [ ] Address Risk (spec §8): document `no_std` parser coverage gaps, confirm Tree-sitter C-binding interop path where pure-Rust `no_std` parsing isn't practical

## Cross-Cutting Goals (spec §4) — Acceptance Criteria

- [ ] **G1** 1000× tokenization speedup — sub-ms tokenization of 100K tokens, benchmarked vs. HF reference (Phase 1)
- [ ] **G2** Zero-copy, `no_std` core — no allocation outside caller buffers, verified (Phase 1)
- [ ] **G3** Logarithmic context compression — O(log N) scaling measured on structured data (Phase 2)
- [ ] **G4** Universal legacy interoperability — 100% HF/BPE compatibility suite passing (Phase 3)
- [ ] **G5** 100% structural validity — formal verification gate never lets invalid IR through (Phase 4)
- [x] **G6** Composable crate surface — every crate builds/tests standalone; `gpu` fully optional (ongoing, final check in Phase 4)

## Risks & Mitigations (spec §8) — Standing Checklist

- [ ] Translation overhead kills the speedup → JIT unrolling runs GPU-parallel, hidden behind first LLM layers (revisit each phase; owned by Phase 3/4)
- [ ] Fractal IR too rigid for open-ended text → fractal compression restricted to structured data; NL uses semantic pooling (owned by Phase 2)
- [ ] `no_std` parser coverage gaps → Tree-sitter C-binding interop fallback where pure-Rust `no_std` isn't practical (owned by Phase 2/4)

## Review Findings & Follow-ups (platform review, 2026-07-28)

Full analysis: see `.claude/plans` review or the corresponding plan doc. Grouped by
priority; items below are new work discovered during review, not yet reflected
in the phase checklists above.

### A — Correctness bugs (highest priority)

- [ ] Fix lossy decompression in `tpt-lexicon-ir/src/compress.rs` (`apply_compression_bytes`, ~L125-160): prefix/rule-ref/suffix children are computed then discarded, returning a single `Compressed` node for the whole payload — any match that isn't the entire node's bytes loses data on decompress
- [ ] Add a regression test with a pattern that is a strict substring of the node payload (not a full-payload match) to lock in the fix above
- [ ] Replace `Box::leak` in `tpt-lexicon-ir/src/node.rs` (`IrForest::from_bytes`, ~L319-372) and `compress.rs` (`expand_rule`, ~L195-198) with a real owned-buffer design (e.g. forest-owned `Vec<u8>` arena with offset/len node references) instead of leaking memory per deserialization
- [ ] Fix unscoped `text.find("\"type\"")` in `tpt-lexicon-translate/src/hf_loader.rs` (`extract_string_field`) — matches the first `"type"` anywhere in the document instead of the intended JSON object; real HF `tokenizer.json` files have multiple `"type"` fields (normalizer/pre_tokenizer/model)
- [ ] Implement real `rule_index`/arity validation in `tpt-lexicon-verify/src/verify.rs` (`verify()`, ~L74-88) — currently `rule_index` is ignored (`_`-bound) and no rule table is even accepted as input, despite the crate's own doc comment claiming this validation happens
- [ ] Wire `tpt-lexicon-translate` to depend on and call `tpt-lexicon-verify` before rendering text (see Phase 4 unchecked item above)
- [ ] Add a proper `Error` enum to `tpt-lexicon-gpu` (currently has none) and change `GpuTokenizer::new()` to return `Result<Self, Error>` instead of `panic!()`ing when a backend isn't compiled in

### B — Adoption tooling (examples/templates/quickstart)

- [ ] Fix broken README code examples in `tpt-lexicon-core`, `tpt-lexicon-gpu`, `tpt-lexicon-ingest`, `tpt-lexicon-ir`, `tpt-lexicon-translate` READMEs (5 of 6 crate READMEs have snippets that don't compile against the actual public API — wrong arg counts/order, wrong method names, wrong return types)
- [ ] Add `cargo test --doc --workspace` to CI so README examples are compiled and can't silently drift again
- [ ] Add workspace-level `examples/` directory: `examples/quickstart.rs` (train vocab → tokenize → decode), `examples/pipeline.rs` (ingest → IR → verify → compress → translate), `examples/hf_import.rs` (load a checked-in HF `tokenizer.json` fixture → translate into core's vocab format)
- [ ] Add a minimal starter-project template (`Cargo.toml` + `main.rs`) showing the recommended crate combination for a typical use case
- [ ] Add a "Quickstart" section at the top of the root `README.md` (before the crate table) with one copy-pastable, CI-verified snippet, linking to `examples/`
- [ ] Add a top-level integration test exercising ingest → IR → verify → translate together, so cross-crate breakage is caught (no such test currently exists)

### C — Automation / CI hardening

- [ ] Add `proptest` dev-dependency + property tests for `tpt-lexicon-core` (arbitrary bytes never panic on tokenize/detokenize round-trip) and `tpt-lexicon-ir` (compress/decompress round-trip is lossless — would have caught A1 above)
- [ ] Add a `benches/` directory with `criterion` benchmarks for the tokenizer, wired into CI as an informational (non-blocking) job
- [ ] Delete the stray duplicate `todo - 1260728.md` (byte-identical to this file)

### D — Innovative additions (discuss before committing)

- [ ] Small CLI (`tpt-lexicon-cli` crate or `xtask`) wrapping the pipeline end-to-end for manual experimentation (`tokenize file.txt`, `verify ir.bin`) — doubles as a live example and debugging tool
- [ ] Architecture diagram (mermaid, in root README) showing ingest → core → ir → verify/translate/gpu data flow
- [ ] Consider an opt-in `serde` feature flag for IR/vocab serialization (JSON export) without compromising the `no_std`/zero-dep core goals

## crates.io Release Readiness (all 6 crates released together as `v0.1.0`)

- [ ] Confirm all 6 crate names are available/reserved on crates.io
- [x] Real `repository` URL in place of `REPLACE_ME` across all manifests
- [x] Every crate: `description`, `keywords` (≤5), `categories` (valid crates.io taxonomy), `readme` all set and accurate
- [x] Every crate: `README.md` reflects actual (not stub) functionality
- [ ] Root `CHANGELOG.md` + per-crate changelog entries (Keep a Changelog format) for `0.1.0` (unchecked 2026-07-28 review: only the root `CHANGELOG.md` exists; no per-crate changelogs found)
- [ ] MSRV documented and verified in CI (`rust-version` in `[workspace.package]`)
- [ ] All crate versions aligned at `0.1.0` (or intentionally bumped together if any diverged during development)
- [x] `cargo doc --workspace --no-deps` builds with zero warnings
- [x] `cargo clippy --workspace --all-targets -- -D warnings` clean
- [x] `cargo fmt --all -- --check` clean
- [ ] `cargo publish --dry-run` clean for every crate, in dependency order: `core` → `ingest`/`ir`/`verify` → `translate` → `gpu`
- [x] Crate-level `#![deny(missing_docs)]` (or equivalent) satisfied — no undocumented public API surface
- [ ] Tag `v0.1.0` release in git, publish all 6 crates in dependency order, verify install (`cargo add <crate>`) from a clean project

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
- [x] Root `README.md` with crate table, design-goal summary, and Quickstart section
- [x] Create the GitHub repo, then replace every `REPLACE_ME` placeholder in `Cargo.toml`/`workspace.package.repository`, crate READMEs, and root README with the real URL
- [x] `git init` + initial commit (done in earlier session)
- [x] Push to GitHub, confirm CI workflow runs

## Phase 1 — Foundation & Core (spec §7, weeks 1–4)

- [x] Design zero-copy byte-slicing API surface for `tpt-lexicon-core` (buffer ownership model, lifetimes, caller-supplied allocator story for `no_std`)
- [x] Implement baseline (sequential) BPE tokenizer as a correctness reference
- [x] Implement parallel prefix-sum BPE algorithm (`encode_indexed` with `MergeIndex`)
- [x] Implement SRAM-native vocabulary mapping (`MergeIndex` with FNV-1a hash indexing)
- [x] Vocabulary loading format (internal binary layout, versioned)
- [x] Unit tests: round-trip encode/decode correctness against reference BPE
- [x] Property tests: arbitrary UTF-8 input never panics, never allocates outside caller buffers
- [x] Benchmark harness (criterion or similar) vs. Hugging Face `tokenizers` reference implementation (benches/tokenize.rs)
- [ ] Document and validate **G1**: sub-millisecond tokenization of 100K-token input (record hardware baseline)
- [ ] Document and validate **G2**: audit crate for zero hidden allocation (miri / allocator-counting test)

## Phase 2 — IR & Fractal Compression (spec §7, weeks 5–10)

- [x] `tpt-lexicon-ingest`: streaming parser core (chunk iterator API, no full-string buffering)
- [x] `tpt-lexicon-ingest`: Tree-sitter interop for Rust (feature-gated `treesitter` module + `tree-sitter-rust` grammar)
- [x] `tpt-lexicon-ingest`: Tree-sitter interop for TypeScript (feature-gated `tree-sitter-typescript` grammar)
- [x] `tpt-lexicon-ingest`: Markdown boundary detection
- [x] `tpt-lexicon-ingest`: JSON boundary detection
- [x] `tpt-lexicon-ingest`: natural-language paragraph boundary detection
- [x] `tpt-lexicon-ir`: typed AST/IR schema definition
- [x] `tpt-lexicon-ir`: serialization format (serde or custom, versioned, diffable)
- [x] `tpt-lexicon-ir`: boilerplate/repetition detection for structured data (code, JSON, logs)
- [x] `tpt-lexicon-ir`: recursive grammar-rule compression for detected sub-graphs
- [x] `tpt-lexicon-ir`: standard semantic pooling path for open-ended natural language (per Risk mitigation, spec §8)
- [x] Tests: compression round-trips losslessly (decompressed IR == original structure)
- [ ] Document and validate **G3**: measure O(log N) context scaling on representative boilerplate-heavy corpora

## Phase 3 — The Translation Bridge (spec §7, weeks 11–14)

- [x] `tpt-lexicon-translate`: JIT unrolling of compressed IR into discrete legacy token-ID sequences
- [x] `tpt-lexicon-translate`: HuggingFace `tokenizer.json` loader
- [x] `tpt-lexicon-translate`: map loaded HF vocabularies into `tpt-lexicon-core` zero-copy structures
- [x] `tpt-lexicon-translate`: cross-IR export — Tree-sitter AST
- [x] `tpt-lexicon-translate`: cross-IR export — LSP
- [x] `tpt-lexicon-translate`: cross-IR export — LLVM IR
- [x] Integration test: compressed IR → bridge → token IDs → feed into a standard GGUF model, confirm output matches native tokenization
- [ ] Document and validate **G4**: 100% compatibility test suite against a representative set of HF/BPE tokenizers (Llama, Mistral, GPT-family)
- [ ] Address Risk (spec §8): benchmark JIT unrolling latency; if not hideable behind first LLM layers, prototype GPU-parallel unrolling in `tpt-lexicon-gpu`

## Phase 4 — Verification & GPU (spec §7, weeks 15–20)

- [x] `tpt-lexicon-verify`: define structural invariant set (balanced delimiters, valid type references, borrow-rule validity, etc.)
- [x] `tpt-lexicon-verify`: formal proof contracts for IR edits (spec the proof system/approach — e.g. SMT-backed, type-driven, or custom checker)
- [x] `tpt-lexicon-verify`: rejection path wired into `tpt-lexicon-translate` before text rendering
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
- [x] Fractal IR too rigid for open-ended text → fractal compression restricted to structured data; NL uses semantic pooling (owned by Phase 2)
- [ ] `no_std` parser coverage gaps → Tree-sitter C-binding interop fallback where pure-Rust `no_std` isn't practical (owned by Phase 2/4)

## Review Findings & Follow-ups (platform review, 2026-07-28)

### A — Correctness bugs (highest priority)

- [x] Fix lossy decompression in `tpt-lexicon-ir/src/compress.rs` (`apply_compression_bytes`): now only matches full-payload patterns for lossless round-trips
- [x] Add regression test with a pattern that is a strict substring of the node payload to lock in the fix above
- [x] Replace `Box::leak` with a single-buffer leak per deserialization in `node.rs` (`IrForest::from_bytes`) and `compress.rs` (`expand_rule`)
- [x] Fix unscoped `text.find("\"type\"")` in `tpt-lexicon-translate/src/hf_loader.rs` — now scoped to model object
- [x] Implement real `rule_index`/arity validation in `tpt-lexicon-verify/src/verify.rs` — added `verify_ir_with_rules` function
- [x] Wire `tpt-lexicon-translate` to depend on and call `tpt-lexicon-verify` before rendering text
- [x] Add proper `Error` enum to `tpt-lexicon-gpu` and change `GpuTokenizer::new()` to return `Result<Self, Error>`

### B — Adoption tooling (examples/templates/quickstart)

- [x] Fix broken README code examples in all 5 crate READMEs (snippets now compile against actual public API)
- [x] Add `cargo test --doc --workspace` to CI so README examples are compiled and can't silently drift again
- [x] Add workspace-level `examples/` directory: `quickstart.rs`, `pipeline.rs`, `hf_import.rs`
- [x] Add minimal starter-project template (`Cargo.toml` + `main.rs`) showing recommended crate combination
- [x] Add "Quickstart" section at top of root `README.md` with copy-pastable example, linking to `examples/`
- [x] Add top-level integration test exercising ingest → IR → verify → translate together

### C — Automation / CI hardening

- [x] Add `proptest` dev-dependency + property tests for `tpt-lexicon-core` (done in previous session)
- [x] Add `benches/` directory with `criterion` benchmarks for the tokenizer, wired into CI as informational (non-blocking) job
- [x] Delete the stray duplicate `todo - 1260728.md`

### D — Innovative additions (discuss before committing)

- [x] Small CLI (`tpt-lexicon-cli` crate) wrapping the pipeline end-to-end for manual experimentation
- [x] Architecture diagram (mermaid, in root README) showing ingest → core → ir → verify/translate/gpu data flow
- [x] Consider an opt-in `serde` feature flag for IR/vocab serialization (JSON export) without compromising `no_std`/zero-dep core goals

## crates.io Release Readiness (all 6 crates released together as `v0.1.0`)

- [ ] Confirm all 6 crate names are available/reserved on crates.io
- [x] Real `repository` URL in place of `REPLACE_ME` across all manifests
- [x] Every crate: `description`, `keywords` (≤5), `categories` (valid crates.io taxonomy), `readme` all set and accurate
- [x] Every crate: `README.md` reflects actual (not stub) functionality
- [x] Root `CHANGELOG.md` + per-crate changelog entries (Keep a Changelog format) for `0.1.0`
- [x] MSRV documented and verified in CI (`rust-version` in `[workspace.package]`, all crates inherit it)
- [x] All crate versions aligned at `0.1.0`
- [x] `cargo doc --workspace --no-deps` builds with zero warnings
- [x] `cargo clippy --workspace --all-targets -- -D warnings` clean
- [x] `cargo fmt --all -- --check` clean
- [ ] `cargo publish --dry-run` clean for every crate, in dependency order: `core` → `ingest`/`ir`/`verify` → `translate` → `gpu`
- [x] Crate-level `#![deny(missing_docs)]` (or equivalent) satisfied — no undocumented public API surface
- [ ] Tag `v0.1.0` release in git, publish all 6 crates in dependency order, verify install (`cargo add <crate>`) from a clean project

(End of file - total 132 lines)

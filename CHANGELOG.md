# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-07-28

### Added

- Workspace scaffold with `resolver = "2"`, shared `[workspace.package]` / `[workspace.dependencies]`
- Six crates: `tpt-lexicon-core`, `tpt-lexicon-ingest`, `tpt-lexicon-ir`, `tpt-lexicon-verify`, `tpt-lexicon-translate`, `tpt-lexicon-gpu`
- `tpt-lexicon-core`: zero-copy byte-slicing token API, sequential BPE tokenizer, vocabulary training, binary vocab serialization
- `tpt-lexicon-ingest`: streaming syntax-aware parser with chunk iterator, Markdown/JSON/NL paragraph boundary detection
- `tpt-lexicon-ir`: typed IR node schema, fractal compression with repetition detection, binary serialization
- `tpt-lexicon-translate`: legacy BPE bridge (JIT unrolling to token IDs), HuggingFace `tokenizer.json` loader, cross-IR text exports
- `tpt-lexicon-verify`: structural invariant verification (balanced delimiters, valid references, acyclic checks)
- `tpt-lexicon-gpu`: feature-gated stubs for wgpu/CUDA/Metal backends
- Dual license: MIT + Apache-2.0
- CI workflow: fmt, clippy, build+test (Linux/macOS/Windows), `no_std` target check, doc build
- `cargo test --doc --workspace` in CI to keep README examples verified
- MSRV CI job (`rust-version = "1.75"`) via `dtolnay/rust-toolchain`
- `examples/` workspace crate: `quickstart`, `pipeline`, `hf_import` runnable examples
- Cross-crate integration test (ingest → IR → compress → verify → translate)
- Property tests (`proptest`) for BPE round-trip in `tpt-lexicon-core` and compress/decompress in `tpt-lexicon-ir`
- Criterion benchmark harness in `tpt-lexicon-core`
- Per-crate `CHANGELOG.md` for all six crates
- Root `README.md` Quickstart section and Mermaid architecture diagram

### Fixed

- `tpt-lexicon-ir`: `apply_compression_bytes` now requires full-payload match; partial substring matches no longer silently discard prefix/suffix bytes on decompression
- `tpt-lexicon-translate`: `extract_model_type` scopes `"type"` field lookup to the `"model"` JSON object, avoiding false matches in normalizer/pre_tokenizer sections
- `tpt-lexicon-verify`: `verify_with_rules` now validates `rule_index` bounds and arity for `Compressed` IR nodes
- `tpt-lexicon-translate`: `LegacyBridge` gains `unroll_forest` and `unroll_forest_verified` (calls `verify_ir` before unrolling)
- `tpt-lexicon-gpu`: `GpuTokenizer::new` returns `Result<Self, GpuError>` instead of panicking when a backend is unavailable
- Five crate README code examples updated to compile against the actual public API

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

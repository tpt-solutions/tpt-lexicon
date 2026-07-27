# TPT Lexicon

A neuro-symbolic preprocessing & translation suite for LLM inference pipelines: parallel/zero-copy tokenization, a symbolic IR with fractal compression, formal verification of IR edits, and a translation bridge to legacy BPE tokenizers — so existing pre-trained models can benefit without retraining.

See [`spec.txt`](spec.txt) for the full design document and [`todo.md`](todo.md) for implementation progress.

Status: **pre-alpha** — workspace scaffold stage. No crate is published yet.

## Crates

| Crate | Role | `no_std` |
|---|---|---|
| [`tpt-lexicon-core`](crates/tpt-lexicon-core) | Zero-copy tokenizer engine (parallel prefix-sum BPE, SRAM-native vocab mapping) | yes |
| [`tpt-lexicon-ingest`](crates/tpt-lexicon-ingest) | Streaming, syntax-aware parser (code, Markdown, JSON, NL) | yes |
| [`tpt-lexicon-ir`](crates/tpt-lexicon-ir) | Symbolic IR with fractal/recursive compression | yes |
| [`tpt-lexicon-verify`](crates/tpt-lexicon-verify) | Formal verification of IR edits and outputs | yes |
| [`tpt-lexicon-translate`](crates/tpt-lexicon-translate) | Legacy BPE bridge + cross-IR translation (Tree-sitter, LSP, LLVM IR) | yes |
| [`tpt-lexicon-gpu`](crates/tpt-lexicon-gpu) | Optional wgpu/CUDA/Metal acceleration | no (feature-gated, opt-in) |

Each crate is independently usable — `tpt-lexicon-core` has no dependency on the rest of the workspace, and `tpt-lexicon-gpu` is entirely optional.

## Design goals

| Goal | Outcome |
|---|---|
| 1000× tokenization speedup | Sub-millisecond tokenization of 100K-token inputs |
| Zero-copy, `no_std` core | Caller-supplied buffers only; no stdlib required |
| Logarithmic context compression | O(log N) context usage via fractal IR compression |
| Universal legacy interoperability | 100% compatibility with HuggingFace/BPE tokenizers via the Translation Bridge |
| 100% structural validity | Output IR passes formal verification before rendering to text |
| Composable crate surface | Each crate usable independently; no forced coupling |

## Building

```sh
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.

# tpt-lexicon-gpu

Optional GPU acceleration (wgpu / CUDA / Metal) for SRAM-native parallel tokenization.

Part of [TPT Lexicon](https://github.com/tpt-solutions/tpt-lexicon).

## Features

- **Feature-gated backends** — enable `wgpu`, `cuda`, or `metal` features for GPU-accelerated tokenization
- **`GpuBackend`** — trait abstraction over available GPU backends
- **`GpuTokenizer`** — thin wrapper that delegates to the selected backend
- **Graceful fallback** — returns `Error::Unavailable` when no GPU backend is compiled in
- Fully optional — the rest of the TPT Lexicon workspace builds and runs without this crate

## Usage

```rust
use tpt_lexicon_gpu::GpuTokenizer;

// Without any feature flag, this returns Unavailable.
// Enable `wgpu` feature for actual GPU acceleration.
let result = GpuTokenizer::new();
assert!(result.is_err());
```

## Status

**pre-alpha** — stub implementation; GPU kernels not yet written. See the workspace
[`todo.md`](https://github.com/tpt-solutions/tpt-lexicon/blob/master/todo.md) for progress.

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE)
or [MIT license](../../LICENSE-MIT) at your option.

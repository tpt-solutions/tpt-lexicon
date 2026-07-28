# tpt-lexicon-gpu

Optional GPU acceleration (wgpu / CUDA / Metal) for SRAM-native parallel tokenization.

Part of [TPT Lexicon](https://github.com/tpt-solutions/tpt-lexicon).

## Features

- **Feature-gated backends** — enable `wgpu`, `cuda`, or `metal` features for GPU-accelerated tokenization
- **`GpuBackend`** — enum of available GPU backends (`Wgpu`, `Cuda`, `Metal`)
- **`GpuTokenizer`** — thin wrapper that delegates to the selected backend
- **`GpuError`** — typed error returned when no GPU backend is compiled in
- Fully optional — the rest of the TPT Lexicon workspace builds and runs without this crate

## Usage

```rust
use tpt_lexicon_gpu::{GpuBackend, GpuError, GpuTokenizer};

// Without any feature flag, GpuTokenizer::new returns GpuError::BackendUnavailable.
// Enable the `wgpu` feature for actual GPU acceleration.
let result = GpuTokenizer::new(GpuBackend::Wgpu);
assert!(matches!(result, Err(GpuError::BackendUnavailable { .. })));
```

## Status

**pre-alpha** — stub implementation; GPU kernels not yet written. See the workspace
[`todo.md`](https://github.com/tpt-solutions/tpt-lexicon/blob/master/todo.md) for progress.

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE)
or [MIT license](../../LICENSE-MIT) at your option.

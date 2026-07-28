//! Optional GPU acceleration (wgpu / CUDA / Metal) for SRAM-native parallel
//! tokenization.
//!
//! This crate provides feature-gated stubs for GPU backends. The core
//! tokenization stack runs on CPU without this crate.
//!
//! # Features
//!
//! | Feature | Description |
//! |---|---|
//! | `wgpu` | Portable wgpu compute shader backend |
//! | `cuda` | Raw CUDA backend |
//! | `metal` | Raw Metal backend |
//!
//! None of these features are enabled by default.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

extern crate alloc;

mod backend;

pub use crate::backend::{GpuBackend, GpuTokenizer};

/// Crate version, exposed for runtime feature negotiation with other
/// TPT Lexicon crates.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

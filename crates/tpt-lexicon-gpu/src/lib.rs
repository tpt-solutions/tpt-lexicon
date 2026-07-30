//! Optional GPU acceleration (wgpu / CUDA / Metal) for SRAM-native parallel
//! tokenization.
//!
//! This crate provides feature-gated backends for GPU acceleration. The core
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
//!
//! # MSRV note
//!
//! The `wgpu` feature requires a recent Rust toolchain (wgpu ≥0.19). Enable it
//! only if your Rust version is recent enough. The base crate (without features)
//! retains the workspace MSRV of 1.75.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

extern crate alloc;

mod backend;

#[cfg(feature = "wgpu")]
pub mod wgpu_backend;

pub use crate::backend::{GpuBackend, GpuError, GpuTokenizer};

/// Crate version, exposed for runtime feature negotiation with other
/// TPT Lexicon crates.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

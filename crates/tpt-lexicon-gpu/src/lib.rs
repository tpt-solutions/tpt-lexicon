//! Hardware acceleration. wgpu (portable) and optional raw CUDA/Metal backends. Entirely feature-gated; the core stack runs on CPU without it.
#![forbid(unsafe_code)]

/// Crate version, exposed for runtime feature negotiation with other
/// TPT Lexicon crates.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

//! Foundational tokenizer engine for TPT Lexicon.
//!
//! `tpt-lexicon-core` is the `no_std`, zero-copy tokenization layer: parallel
//! prefix-sum BPE and SRAM-native vocabulary mapping over caller-supplied
//! buffers. See the workspace README for the full architecture.
#![no_std]
#![forbid(unsafe_code)]

/// Crate version, exposed for runtime feature negotiation with other
/// TPT Lexicon crates.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

//! Streaming, syntax-aware parser. Yields semantic chunks (code via Tree-sitter interop, Markdown, JSON, natural-language paragraphs) without allocating full strings in memory.
#![no_std]
#![forbid(unsafe_code)]

/// Crate version, exposed for runtime feature negotiation with other
/// TPT Lexicon crates.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

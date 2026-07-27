//! The Translation Bridge. JIT-unrolls compressed IR into the exact token IDs expected by a standard pre-trained LLM, translates IR to external formats (Tree-sitter AST, LSP, LLVM IR), and loads external tokenizer.json vocabularies.
#![no_std]
#![forbid(unsafe_code)]

/// Crate version, exposed for runtime feature negotiation with other
/// TPT Lexicon crates.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

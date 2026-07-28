//! The Translation Bridge. JIT-unrolls compressed IR into the exact token IDs
//! expected by a standard pre-trained LLM, translates IR to external formats
//! (Tree-sitter AST, LSP, LLVM IR), and loads external tokenizer.json
//! vocabularies.
//!
//! # Overview
//!
//! - [`LegacyBridge`]: unrolls IR into discrete legacy token-ID sequences
//! - [`HfTokenizer`]: loads a HuggingFace `tokenizer.json` vocabulary
//! - Cross-IR export: [`to_tree_sitter_ast`], [`to_lsp_document`],
//!   [`to_llvm_ir`]
#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

extern crate alloc;

mod bridge;
mod cross_ir;
mod error;
mod hf_loader;

pub use crate::bridge::{LegacyBridge, LegacyVocab};
pub use crate::cross_ir::{to_llvm_ir, to_lsp_document, to_tree_sitter_ast};
pub use crate::error::{Error, Result};
pub use crate::hf_loader::HfTokenizer;

/// Crate version, exposed for runtime feature negotiation with other
/// TPT Lexicon crates.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

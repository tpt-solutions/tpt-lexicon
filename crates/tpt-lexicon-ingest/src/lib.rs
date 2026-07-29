//! Streaming, syntax-aware parser yielding semantic chunks without
//! full-string allocation.
//!
//! The [`Parser`] struct accepts a byte slice and produces an iterator of
//! [`Chunk`] values — borrowed sub-slices tagged with their content type
//! (text, code, Markdown, JSON, or natural-language paragraph).
//!
//! # Boundary detection
//!
//! | Format | Heuristic |
//! |---|---|
//! | Markdown | Fenced code blocks (` ``` `), ATX headers (`# `) |
//! | JSON | Balanced `{}`/`[]` at the top level |
//! | Paragraph | Two or more consecutive blank lines |
//!
//! # Feature flags
//!
//! - `tree-sitter`: Enable syntax-aware chunking via Tree-sitter grammars.
//! - `tree-sitter-rust`: Support Rust grammar in the Tree-sitter chunker.
//! - `tree-sitter-typescript`: Support TypeScript/TSX grammar in the
//!   Tree-sitter chunker.
//!
//! # Examples
//!
//! ```
//! use tpt_lexicon_ingest::{Parser, ChunkKind};
//!
//! let input = b"# Hello\n\nSome text.\n\n```rust\nlet x = 1;\n```\n";
//! let chunks: Vec<_> = Parser::new(input).collect();
//! assert!(chunks.len() >= 2);
//! assert_eq!(chunks[0].kind, ChunkKind::Markdown);
//! ```
#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod error;
mod parser;

#[cfg(feature = "tree-sitter")]
pub mod treesitter;

pub use crate::error::{Error, ParseError, Result};
pub use crate::parser::{Chunk, ChunkKind, Parser};

/// Crate version, exposed for runtime feature negotiation with other
/// TPT Lexicon crates.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

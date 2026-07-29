//! Feature-gated Tree-sitter integration for syntax-aware chunking.
//!
//! This module wraps the [`tree_sitter`] crate to parse source code into
//! structured [`Chunk`] sequences based on the syntax tree. It is only
//! available when the `tree-sitter` feature is enabled.
//!
//! # Feature flags
//!
//! | Feature | Enables |
//! |---|---|
//! | `tree-sitter` | Core Tree-sitter chunker using a generic [`Language`] |
//! | `tree-sitter-rust` | Convenience constructor for Rust grammar |
//! | `tree-sitter-typescript` | Convenience constructor for TypeScript/TSX grammar |

#![cfg(feature = "tree-sitter")]

use crate::{Chunk, ChunkKind};

/// A syntax-aware chunker backed by a Tree-sitter grammar.
///
/// Parses source code into a CST and decomposes it into a sequence of
/// [`Chunk`] values — each chunk corresponds to a top-level syntactic
/// node (function definition, class, struct, statement, etc.).
///
/// # Examples
///
/// ```ignore
/// use tpt_lexicon_ingest::treesitter::TreeSitterChunker;
///
/// let chunker = TreeSitterChunker::new(tree_sitter_rust::language());
/// let chunks = chunker.chunk(b"fn main() {}").unwrap();
/// assert!(!chunks.is_empty());
/// ```
#[derive(Debug)]
pub struct TreeSitterChunker {
    language: tree_sitter::Language,
}

impl TreeSitterChunker {
    /// Create a new chunker for the given language grammar.
    #[inline]
    pub fn new(language: tree_sitter::Language) -> Self {
        Self { language }
    }

    /// Parse `source` and decompose it into a sequence of chunks.
    ///
    /// Each returned chunk corresponds to a top-level named node in the
    /// parsed syntax tree. Chunks borrow directly from `source`.
    ///
    /// Returns `None` if parsing fails (e.g., the source is not valid UTF-8,
    /// the language grammar is not compatible, or a parse error occurs).
    pub fn chunk<'a>(&self, source: &'a [u8]) -> Option<alloc::vec::Vec<Chunk<'a>>> {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&self.language).ok()?;

        let source_str = core::str::from_utf8(source).ok()?;
        let tree = parser.parse(source_str, None)?;
        let root = tree.root_node();

        let mut chunks = alloc::vec::Vec::new();
        let mut cursor = root.walk();

        for child in root.children(&mut cursor) {
            if child.is_named() && child.child_count() >= 1 {
                let start = child.start_byte();
                let end = child.end_byte();
                if start < end && end <= source.len() {
                    chunks.push(Chunk {
                        kind: ChunkKind::Code,
                        bytes: &source[start..end],
                        offset: start,
                    });
                }
            }
        }

        Some(chunks)
    }
}

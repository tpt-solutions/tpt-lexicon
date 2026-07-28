//! Symbolic Intermediate Representation. Fractal compression identifies
//! repeating structural sub-graphs and replaces them with recursive grammar
//! rules for logarithmic context scaling.
//!
//! # Overview
//!
//! The IR layer provides:
//!
//! - **[`IrNode`]**: a tagged node in the intermediate representation tree
//! - **[`IrForest`]**: a top-level collection of IR nodes
//! - **Fractal compression**: detect repeating structural patterns and replace
//!   them with compact grammar rules via [`compress`] and [`decompress`]
//! - **Binary serialization**: versioned, compact binary format via
//!   [`IrForest::to_bytes`] / [`IrForest::from_bytes`]
//!
//! # Examples
//!
//! ```
//! use tpt_lexicon_ir::{IrForest, IrNode, compress, decompress};
//!
//! let forest = IrForest::from_nodes(vec![
//!     IrNode::code(b"function foo() { return 1; }"),
//!     IrNode::code(b"function bar() { return 2; }"),
//!     IrNode::code(b"function baz() { return 3; }"),
//! ]);
//!
//! let (compressed, rules) = compress(&forest, 1);
//! assert!(!rules.is_empty());
//!
//! let decompressed = decompress(&compressed, &rules);
//! assert_eq!(decompressed.node_count(), forest.node_count());
//! ```
#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

extern crate alloc;

mod compress;
mod error;
mod node;

pub use crate::compress::{compress, decompress};
pub use crate::error::{Error, Result};
pub use crate::node::{CompressRule, IrForest, IrNode};

/// Crate version, exposed for runtime feature negotiation with other
/// TPT Lexicon crates.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

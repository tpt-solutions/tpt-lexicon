//! Formal verification of TPT Lexicon IR edits and outputs against structural
//! invariants.
//!
//! # Invariants checked
//!
//! - **Balanced delimiters**: no unclosed brackets or braces in text payloads
//! - **Valid references**: all node indices in Reference and List nodes are
//!   within bounds
//! - **Acyclic references**: reference chains do not form cycles
//! - **Valid compression rules**: Compressed nodes reference existing rules
//!   with correct arity
#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

extern crate alloc;

mod error;
mod verify;

pub use crate::error::{Error, Result, VerifyError};
pub use crate::verify::{verify_ir, verify_with_rules, VerificationReport, Verify};

/// Crate version, exposed for runtime feature negotiation with other
/// TPT Lexicon crates.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

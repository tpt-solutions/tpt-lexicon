//! Formal verification of IR edits and outputs. Rejects outputs violating structural invariants (mismatched braces, invalid type references, broken borrow rules) before translation back to text.
#![no_std]
#![forbid(unsafe_code)]

/// Crate version, exposed for runtime feature negotiation with other
/// TPT Lexicon crates.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

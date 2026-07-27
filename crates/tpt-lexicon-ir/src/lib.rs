//! Symbolic Intermediate Representation. Fractal compression identifies repeating structural sub-graphs and replaces them with recursive grammar rules for logarithmic context scaling.
#![no_std]
#![forbid(unsafe_code)]

/// Crate version, exposed for runtime feature negotiation with other
/// TPT Lexicon crates.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

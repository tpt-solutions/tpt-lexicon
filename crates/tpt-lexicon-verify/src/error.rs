//! Error types for the verify crate.

use core::fmt;

/// Specific verification failure.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VerifyError {
    /// An unbalanced delimiter was found in a text payload.
    UnbalancedDelimiter {
        /// Byte offset within the payload where the imbalance was found.
        offset: usize,
        /// The unmatched delimiter character.
        delimiter: u8,
    },
    /// A node reference index is out of bounds.
    InvalidReference {
        /// Index of the node containing the bad reference.
        node_index: usize,
        /// The invalid reference target.
        target_index: usize,
        /// Number of available nodes.
        bound: usize,
    },
    /// A cycle was detected in the reference graph.
    CycleDetected {
        /// The node index where the cycle was detected.
        node_index: usize,
    },
    /// A Compressed node references an invalid rule index.
    InvalidRuleReference {
        /// Index of the node containing the bad rule reference.
        node_index: usize,
        /// The invalid rule index.
        rule_index: usize,
    },
    /// A Compressed node has wrong arity for its rule.
    ArityMismatch {
        /// Index of the node.
        node_index: usize,
        /// Expected arity.
        expected: usize,
        /// Actual number of arguments.
        actual: usize,
    },
}

impl fmt::Display for VerifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnbalancedDelimiter { offset, delimiter } => {
                write!(f, "unbalanced '{delimiter}' at offset {offset}",)
            }
            Self::InvalidReference {
                node_index,
                target_index,
                bound,
            } => {
                write!(
                    f,
                    "node {node_index} references index {target_index} (bound: {bound})"
                )
            }
            Self::CycleDetected { node_index } => {
                write!(f, "reference cycle detected at node {node_index}")
            }
            Self::InvalidRuleReference {
                node_index,
                rule_index,
            } => {
                write!(f, "node {node_index} references invalid rule {rule_index}")
            }
            Self::ArityMismatch {
                node_index,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "node {node_index}: expected arity {expected}, got {actual}"
                )
            }
        }
    }
}

/// A specialized `Result` type for verify operations.
pub type Result<T> = core::result::Result<T, Error>;

/// Top-level error type for the verify crate.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Error {
    /// Verification failed with one or more errors.
    VerificationFailed(VerifyError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::VerificationFailed(e) => write!(f, "{e}"),
        }
    }
}

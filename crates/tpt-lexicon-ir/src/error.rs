//! Error types for the IR crate.

use core::fmt;

/// Errors that can occur during IR operations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Error {
    /// IR binary data is truncated or corrupted.
    CorruptedData,
    /// IR binary format version is not supported.
    UnsupportedVersion {
        /// Version found in the binary data.
        found: u32,
        /// Maximum version this crate supports.
        expected_max: u32,
    },
    /// A node reference index is out of bounds.
    InvalidReference {
        /// The invalid index.
        index: usize,
        /// Number of available nodes.
        bound: usize,
    },
    /// A compression rule references a non-existent pattern.
    InvalidRule {
        /// Index of the invalid rule.
        index: usize,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CorruptedData => write!(f, "IR data is truncated or corrupted"),
            Self::UnsupportedVersion {
                found,
                expected_max,
            } => {
                write!(
                    f,
                    "unsupported IR version {found} (max supported: {expected_max})"
                )
            }
            Self::InvalidReference { index, bound } => {
                write!(f, "node reference {index} out of bounds (0..{bound})")
            }
            Self::InvalidRule { index } => {
                write!(
                    f,
                    "compression rule {index} references a non-existent pattern"
                )
            }
        }
    }
}

/// A specialized `Result` type for IR operations.
pub type Result<T> = core::result::Result<T, Error>;

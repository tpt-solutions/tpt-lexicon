//! Error types for the ingest crate.

use core::fmt;

/// Errors that can occur during parsing.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParseError {
    /// Input is not valid UTF-8 at the given byte offset.
    InvalidUtf8 {
        /// Byte offset of the invalid sequence.
        offset: usize,
    },
    /// A JSON structure could not be parsed (unbalanced delimiters).
    UnbalancedJson {
        /// Byte offset where the imbalance was detected.
        offset: usize,
    },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUtf8 { offset } => {
                write!(f, "invalid UTF-8 at byte offset {offset}")
            }
            Self::UnbalancedJson { offset } => {
                write!(f, "unbalanced JSON delimiters at byte offset {offset}")
            }
        }
    }
}

/// A specialized `Result` type for ingest operations.
pub type Result<T> = core::result::Result<T, Error>;

/// Error type for the ingest crate.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Error {
    /// Parse error.
    Parse(ParseError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(e) => write!(f, "{e}"),
        }
    }
}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self {
        Self::Parse(e)
    }
}

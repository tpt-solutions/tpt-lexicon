//! Error types for the translate crate.

use core::fmt;

/// Errors that can occur during translation operations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Error {
    /// The legacy vocabulary has no mapping for a required token.
    UnmappedToken {
        /// Description of the unmapped token.
        description: alloc::vec::Vec<u8>,
    },
    /// HuggingFace tokenizer.json parsing error.
    HfParseError {
        /// Description of the parse error.
        message: alloc::vec::Vec<u8>,
    },
    /// The HuggingFace tokenizer uses an unsupported model type.
    UnsupportedHfModel {
        /// The model type string found.
        model_type: alloc::vec::Vec<u8>,
    },
    /// An IR node could not be translated.
    IrTranslationError {
        /// Index of the problematic node.
        node_index: usize,
        /// Description of the error.
        message: alloc::vec::Vec<u8>,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnmappedToken { description } => {
                write!(
                    f,
                    "unmapped token: {}",
                    core::str::from_utf8(description).unwrap_or("<invalid utf-8>")
                )
            }
            Self::HfParseError { message } => {
                write!(
                    f,
                    "HF tokenizer parse error: {}",
                    core::str::from_utf8(message).unwrap_or("<invalid utf-8>")
                )
            }
            Self::UnsupportedHfModel { model_type } => {
                write!(
                    f,
                    "unsupported HF model type: {}",
                    core::str::from_utf8(model_type).unwrap_or("<invalid utf-8>")
                )
            }
            Self::IrTranslationError {
                node_index,
                message,
            } => {
                write!(
                    f,
                    "IR translation error at node {node_index}: {}",
                    core::str::from_utf8(message).unwrap_or("<invalid utf-8>")
                )
            }
        }
    }
}

/// A specialized `Result` type for translate operations.
pub type Result<T> = core::result::Result<T, Error>;

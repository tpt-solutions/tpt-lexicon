//! Error types for TPT Lexicon core operations.

use core::fmt;

/// Errors that can occur during core tokenization operations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Error {
    /// The vocabulary is empty — no merge rules are defined.
    EmptyVocabulary,
    /// Input contains an invalid UTF-8 sequence at the given byte offset.
    InvalidUtf8 {
        /// Byte offset of the invalid sequence.
        offset: usize,
    },
    /// A vocabulary entry is malformed (e.g., merge pair with zero bytes).
    MalformedVocabEntry {
        /// Index of the malformed entry.
        index: usize,
    },
    /// Vocabulary binary format version is not supported.
    UnsupportedVocabVersion {
        /// Version found in the binary data.
        found: u32,
        /// Maximum version this crate supports.
        expected_max: u32,
    },
    /// Vocabulary binary data is truncated or corrupted.
    CorruptedVocabData,
    /// Training failed because the corpus is too small to produce any merges.
    CorpusTooSmall,
    /// Training failed because the requested number of merges exceeds the
    /// available unique byte pairs.
    InsufficientPairs {
        /// Number of unique pairs found.
        available: usize,
        /// Number of merges requested.
        requested: usize,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyVocabulary => write!(f, "vocabulary is empty"),
            Self::InvalidUtf8 { offset } => {
                write!(f, "invalid UTF-8 sequence at byte offset {offset}")
            }
            Self::MalformedVocabEntry { index } => {
                write!(f, "malformed vocabulary entry at index {index}")
            }
            Self::UnsupportedVocabVersion { found, expected_max } => {
                write!(
                    f,
                    "unsupported vocabulary version {found} (max supported: {expected_max})"
                )
            }
            Self::CorruptedVocabData => write!(f, "vocabulary data is truncated or corrupted"),
            Self::CorpusTooSmall => write!(f, "corpus is too small to produce merges"),
            Self::InsufficientPairs { available, requested } => write!(
                f,
                "requested {requested} merges but only {available} unique pairs available"
            ),
        }
    }
}

/// A specialized `Result` type for core operations.
pub type Result<T> = core::result::Result<T, Error>;

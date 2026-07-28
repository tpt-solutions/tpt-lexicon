//! BPE vocabulary: merge rules, special tokens, and training.
//!
//! A [`Vocab`] stores the learned byte-pair merge rules produced by
//! [`Vocab::train`] and is consumed by [`crate::BpeTokenizer`].
//!
//! # Binary format
//!
//! Vocabulary binary layout (all multi-byte integers are little-endian):
//!
//! ```text
//! Offset  Size  Field
//! 0       4     Magic: b"TPLX"
//! 4       4     Version: u32 LE (currently 1)
//! 8       4     merge_count: u32 LE
//! 12      4     special_count: u32 LE
//! 16      ...   Merge entries: [left_len:u16 LE, left_bytes, right_len:u16 LE, right_bytes, rank:u32 LE]
//! ...     ...   Special tokens: [name_len:u16 LE, name_bytes, token_len:u16 LE, token_bytes]
//! ```

use core::fmt;

use crate::error::{Error, Result};

/// Magic bytes identifying TPT Lexicon vocabulary files.
pub const MAGIC: &[u8; 4] = b"TPLX";

/// Current vocabulary binary format version.
pub const VERSION: u32 = 1;

/// Maximum vocabulary binary format version this crate supports.
pub const MAX_SUPPORTED_VERSION: u32 = 1;

/// A byte-pair merge rule: two adjacent byte sequences that were merged during
/// training, assigned a rank (lower = merged first = higher priority).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MergeEntry {
    /// Left side of the merge pair.
    pub left: alloc::vec::Vec<u8>,
    /// Right side of the merge pair.
    pub right: alloc::vec::Vec<u8>,
    /// Merge priority rank. Lower values indicate earlier (higher-priority)
    /// merges.
    pub rank: u32,
}

impl MergeEntry {
    /// Returns the combined byte length of left + right.
    #[inline]
    pub fn total_len(&self) -> usize {
        self.left.len() + self.right.len()
    }
}

/// A named special token (e.g., `<|bos|>`, `<|eos|>`, `<|pad|>`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpecialToken {
    /// Human-readable name (e.g., `"bos"`).
    pub name: alloc::vec::Vec<u8>,
    /// Raw bytes of the special token as it appears in the input.
    pub token: alloc::vec::Vec<u8>,
}

/// BPE vocabulary: merge rules and special tokens.
///
/// Construct via [`Vocab::train`] or [`Vocab::from_bytes`].
///
/// # Examples
///
/// ```
/// use tpt_lexicon_core::Vocab;
///
/// // Train from a corpus
/// let corpus = b"hello world hello world hello universe";
/// let vocab = Vocab::train(corpus, 10).unwrap();
/// assert!(vocab.merge_count() > 0);
/// ```
#[derive(Debug, Clone)]
pub struct Vocab {
    /// Merge rules, sorted by rank (ascending = highest priority first).
    pub(crate) merges: alloc::vec::Vec<MergeEntry>,
    /// Named special tokens.
    pub(crate) specials: alloc::vec::Vec<SpecialToken>,
}

impl Vocab {
    /// Create an empty vocabulary.
    #[inline]
    pub fn new() -> Self {
        Self {
            merges: alloc::vec::Vec::new(),
            specials: alloc::vec::Vec::new(),
        }
    }

    /// Returns the number of merge rules.
    #[inline]
    pub fn merge_count(&self) -> usize {
        self.merges.len()
    }

    /// Returns the number of special tokens.
    #[inline]
    pub fn special_count(&self) -> usize {
        self.specials.len()
    }

    /// Returns a reference to the merge rules, sorted by ascending rank.
    #[inline]
    pub fn merges(&self) -> &[MergeEntry] {
        &self.merges
    }

    /// Returns a reference to the special tokens.
    #[inline]
    pub fn specials(&self) -> &[SpecialToken] {
        &self.specials
    }

    /// Find the highest-priority (lowest-rank) adjacent byte-pair in `tokens`.
    ///
    /// Returns `(pair_index, rank)` where `pair_index` is the index of the
    /// left token in the pair. Returns `None` if no mergeable pair exists.
    pub fn find_best_pair(&self, tokens: &[&[u8]]) -> Option<(usize, u32)> {
        if tokens.len() < 2 {
            return None;
        }

        let mut best_index = None;
        let mut best_rank = u32::MAX;

        for i in 0..tokens.len() - 1 {
            let left = tokens[i];
            let right = tokens[i + 1];

            for merge in &self.merges {
                if merge.left == left && merge.right == right && merge.rank < best_rank {
                    best_rank = merge.rank;
                    best_index = Some(i);
                    break;
                }
            }
        }

        best_index.map(|idx| (idx, best_rank))
    }

    /// Train a new vocabulary from a raw byte corpus.
    ///
    /// `num_merges` specifies the maximum number of merge rules to learn.
    /// The actual number may be lower if the corpus has fewer unique pairs.
    ///
    /// # Errors
    ///
    /// Returns [`Error::CorpusTooSmall`] if the corpus has fewer than 2 bytes.
    /// Returns [`Error::InsufficientPairs`] if the corpus has fewer unique
    /// pairs than `num_merges`.
    pub fn train(corpus: &[u8], num_merges: usize) -> Result<Self> {
        if corpus.len() < 2 {
            return Err(Error::CorpusTooSmall);
        }

        // Initialize: each byte is a separate token.
        let mut tokens: alloc::vec::Vec<alloc::vec::Vec<u8>> =
            corpus.iter().map(|&b| alloc::vec![b]).collect();

        let mut merges = alloc::vec::Vec::with_capacity(num_merges);

        for merge_rank in 0..num_merges as u32 {
            // Count frequency of all adjacent pairs.
            let mut pair_counts: alloc::collections::BTreeMap<
                (alloc::vec::Vec<u8>, alloc::vec::Vec<u8>),
                usize,
            > = alloc::collections::BTreeMap::new();

            for window in tokens.windows(2) {
                let key = (window[0].clone(), window[1].clone());
                *pair_counts.entry(key).or_insert(0) += 1;
            }

            if pair_counts.is_empty() {
                break;
            }

            // Find the most frequent pair.
            let ((left, right), &count) = pair_counts.iter().max_by_key(|&(_, &c)| c).unwrap();

            if count == 0 {
                break;
            }

            let left = left.clone();
            let right = right.clone();

            // Record the merge.
            merges.push(MergeEntry {
                left: left.clone(),
                right: right.clone(),
                rank: merge_rank,
            });

            // Apply the merge: replace all occurrences of (left, right) with
            // the concatenated result.
            let merged: alloc::vec::Vec<u8> = {
                let mut m = left.clone();
                m.extend_from_slice(&right);
                m
            };

            let mut new_tokens = alloc::vec::Vec::with_capacity(tokens.len());
            let mut i = 0;

            while i < tokens.len() {
                if i + 1 < tokens.len() && tokens[i] == left && tokens[i + 1] == right {
                    new_tokens.push(merged.clone());
                    i += 2;
                } else {
                    new_tokens.push(tokens[i].clone());
                    i += 1;
                }
            }

            tokens = new_tokens;
        }

        if merges.is_empty() {
            return Err(Error::InsufficientPairs {
                available: 0,
                requested: num_merges,
            });
        }

        Ok(Self {
            merges,
            specials: alloc::vec::Vec::new(),
        })
    }

    /// Add a named special token.
    pub fn add_special(&mut self, name: &[u8], token: &[u8]) {
        self.specials.push(SpecialToken {
            name: name.to_vec(),
            token: token.to_vec(),
        });
    }

    /// Serialize the vocabulary to a binary format.
    ///
    /// See the module-level documentation for the binary layout.
    pub fn to_bytes(&self) -> alloc::vec::Vec<u8> {
        let mut buf = alloc::vec::Vec::new();

        // Magic
        buf.extend_from_slice(MAGIC);

        // Version
        buf.extend_from_slice(&VERSION.to_le_bytes());

        // Merge count
        buf.extend_from_slice(&(self.merges.len() as u32).to_le_bytes());

        // Special token count
        buf.extend_from_slice(&(self.specials.len() as u32).to_le_bytes());

        // Merge entries
        for merge in &self.merges {
            buf.extend_from_slice(&(merge.left.len() as u16).to_le_bytes());
            buf.extend_from_slice(&merge.left);
            buf.extend_from_slice(&(merge.right.len() as u16).to_le_bytes());
            buf.extend_from_slice(&merge.right);
            buf.extend_from_slice(&merge.rank.to_le_bytes());
        }

        // Special tokens
        for special in &self.specials {
            buf.extend_from_slice(&(special.name.len() as u16).to_le_bytes());
            buf.extend_from_slice(&special.name);
            buf.extend_from_slice(&(special.token.len() as u16).to_le_bytes());
            buf.extend_from_slice(&special.token);
        }

        buf
    }

    /// Deserialize a vocabulary from binary data.
    ///
    /// # Errors
    ///
    /// Returns an error if the data is truncated, corrupted, or uses an
    /// unsupported version.
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        let mut pos = 0;

        // Magic
        if data.len() < 16 {
            return Err(Error::CorruptedVocabData);
        }
        if &data[0..4] != MAGIC {
            return Err(Error::CorruptedVocabData);
        }
        pos += 4;

        // Version
        let version = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap());
        pos += 4;
        if version > MAX_SUPPORTED_VERSION {
            return Err(Error::UnsupportedVocabVersion {
                found: version,
                expected_max: MAX_SUPPORTED_VERSION,
            });
        }

        // Merge count
        let merge_count = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;

        // Special count
        let special_count = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;

        // Merge entries
        let mut merges = alloc::vec::Vec::with_capacity(merge_count);
        for i in 0..merge_count {
            if pos + 2 > data.len() {
                return Err(Error::CorruptedVocabData);
            }
            let left_len = u16::from_le_bytes(data[pos..pos + 2].try_into().unwrap()) as usize;
            pos += 2;

            if pos + left_len > data.len() {
                return Err(Error::CorruptedVocabData);
            }
            let left = data[pos..pos + left_len].to_vec();
            pos += left_len;

            if pos + 2 > data.len() {
                return Err(Error::CorruptedVocabData);
            }
            let right_len = u16::from_le_bytes(data[pos..pos + 2].try_into().unwrap()) as usize;
            pos += 2;

            if pos + right_len > data.len() {
                return Err(Error::CorruptedVocabData);
            }
            let right = data[pos..pos + right_len].to_vec();
            pos += right_len;

            if pos + 4 > data.len() {
                return Err(Error::CorruptedVocabData);
            }
            let rank = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap());
            pos += 4;

            if left.is_empty() || right.is_empty() {
                return Err(Error::MalformedVocabEntry { index: i });
            }

            merges.push(MergeEntry { left, right, rank });
        }

        // Special tokens
        let mut specials = alloc::vec::Vec::with_capacity(special_count);
        for _ in 0..special_count {
            if pos + 2 > data.len() {
                return Err(Error::CorruptedVocabData);
            }
            let name_len = u16::from_le_bytes(data[pos..pos + 2].try_into().unwrap()) as usize;
            pos += 2;

            if pos + name_len > data.len() {
                return Err(Error::CorruptedVocabData);
            }
            let name = data[pos..pos + name_len].to_vec();
            pos += name_len;

            if pos + 2 > data.len() {
                return Err(Error::CorruptedVocabData);
            }
            let token_len = u16::from_le_bytes(data[pos..pos + 2].try_into().unwrap()) as usize;
            pos += 2;

            if pos + token_len > data.len() {
                return Err(Error::CorruptedVocabData);
            }
            let token = data[pos..pos + token_len].to_vec();
            pos += token_len;

            specials.push(SpecialToken { name, token });
        }

        Ok(Self { merges, specials })
    }
}

impl Default for Vocab {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Vocab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Vocab {{ merges: {}, specials: {} }}",
            self.merges.len(),
            self.specials.len()
        )
    }
}

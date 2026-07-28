//! Foundational tokenizer engine for TPT Lexicon.
//!
//! `tpt-lexicon-core` is the `no_std`, zero-copy tokenization layer providing
//! sequential BPE tokenization over caller-supplied buffers.
//!
//! # Overview
//!
//! This crate provides:
//!
//! - **Zero-copy tokens**: [`Token`] borrows directly from the caller's input
//!   buffer — no allocation, no copying.
//! - **BPE tokenization**: [`BpeTokenizer`] applies learned byte-pair merge
//!   rules to encode input bytes into a sequence of tokens.
//! - **Vocabulary training**: [`Vocab::train`] learns merge rules from a raw
//!   byte corpus.
//! - **Binary vocabulary I/O**: [`Vocab::to_bytes`] / [`Vocab::from_bytes`]
//!   for persistent storage with versioned format.
//!
//! The `alloc` feature (enabled by default) provides [`alloc::vec::Vec`]-based
//! convenience methods. All core types remain `no_std`-compatible.
//!
//! # Examples
//!
//! ```
//! use tpt_lexicon_core::{BpeTokenizer, Vocab};
//!
//! // Train a vocabulary
//! let corpus = b"hello world hello world hello universe";
//! let vocab = Vocab::train(corpus, 10).unwrap();
//!
//! // Tokenize
//! let tokenizer = BpeTokenizer::new(&vocab);
//! let tokens = tokenizer.encode(b"hello world");
//! assert_eq!(tokens.len(), 2);
//!
//! // Decode back (zero-copy)
//! let decoded = tokens.as_bytes();
//! assert_eq!(decoded, b"hello world");
//! ```
#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

extern crate alloc;

mod error;
mod sram;
mod token;
mod vocab;

pub use crate::error::{Error, Result};
pub use crate::sram::{find_best_pair_indexed, MergeIndex};
pub use crate::token::{Token, TokenSet};
pub use crate::vocab::{MergeEntry, SpecialToken, Vocab};

/// Crate version, exposed for runtime feature negotiation with other
/// TPT Lexicon crates.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The BPE tokenizer.
///
/// Wraps a [`Vocab`] and provides [`encode`] / decode methods.
///
/// [`encode`]: BpeTokenizer::encode
///
/// # Examples
///
/// ```
/// use tpt_lexicon_core::{BpeTokenizer, Vocab};
///
/// let vocab = Vocab::train(b"abc abc def", 5).unwrap();
/// let tok = BpeTokenizer::new(&vocab);
/// let tokens = tok.encode(b"abc");
/// let decoded = tokens.as_bytes();
/// assert_eq!(decoded, b"abc");
/// ```
#[derive(Debug)]
pub struct BpeTokenizer<'a> {
    vocab: &'a Vocab,
}

impl<'a> BpeTokenizer<'a> {
    /// Create a new tokenizer backed by the given vocabulary.
    #[inline]
    pub fn new(vocab: &'a Vocab) -> Self {
        Self { vocab }
    }

    /// Returns a reference to the underlying vocabulary.
    #[inline]
    pub fn vocab(&self) -> &'a Vocab {
        self.vocab
    }

    /// Encode `input` bytes into a token set using BPE merge rules.
    ///
    /// The returned [`TokenSet`] borrows directly from `input` — tokens are
    /// zero-copy sub-slices of the original data.
    ///
    /// # Algorithm
    ///
    /// 1. Initialize each byte as a separate token.
    /// 2. Find the highest-priority (lowest-rank) adjacent pair.
    /// 3. Merge that pair into a single token.
    /// 4. Repeat until no more merges are possible.
    pub fn encode<'b>(&self, input: &'b [u8]) -> TokenSet<'b> {
        if input.is_empty() {
            return TokenSet::new(input, &[]);
        }

        if self.vocab.merge_count() == 0 {
            // No merge rules — each byte is its own token.
            let tokens: alloc::vec::Vec<Token<'_>> = input.chunks(1).map(Token::new).collect();
            return TokenSet::new(input, &tokens);
        }

        // Phase 1: split into individual byte references.
        let mut pieces: alloc::vec::Vec<&[u8]> = input.chunks(1).collect();

        // Phase 2: iteratively merge the best pair.
        while let Some((pair_index, _rank)) = self.vocab.find_best_pair(&pieces) {
            let left = pieces[pair_index];
            let right = pieces[pair_index + 1];

            // Build merged slice: since left and right are contiguous
            // sub-slices of `input`, compute the merged range.
            let left_start = (left.as_ptr() as usize) - (input.as_ptr() as usize);
            let right_end = (right.as_ptr() as usize) - (input.as_ptr() as usize) + right.len();
            let merged = &input[left_start..right_end];

            pieces[pair_index] = merged;
            pieces.remove(pair_index + 1);
        }

        // Convert pieces to tokens.
        let tokens: alloc::vec::Vec<Token<'_>> =
            pieces.iter().map(|&piece| Token::new(piece)).collect();

        TokenSet::new(input, &tokens)
    }

    /// Encode input, collecting tokens into a `Vec`.
    ///
    /// Requires the `alloc` feature.
    #[cfg(feature = "alloc")]
    pub fn encode_to_vec<'b>(&self, input: &'b [u8]) -> alloc::vec::Vec<Token<'b>> {
        self.encode(input).iter().copied().collect()
    }

    /// Encode `input` using a pre-built [`MergeIndex`] for O(1) pair lookups.
    ///
    /// This is significantly faster than [`encode`](Self::encode) for large
    /// vocabularies because it replaces the linear scan over merge rules with
    /// hash-indexed lookups.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_lexicon_core::{BpeTokenizer, Vocab};
    ///
    /// let vocab = Vocab::train(b"abc abc def", 5).unwrap();
    /// let index = vocab.merge_index();
    /// let tok = BpeTokenizer::new(&vocab);
    /// let tokens = tok.encode_indexed(b"abc", &index);
    /// assert_eq!(tokens.as_bytes(), b"abc");
    /// ```
    pub fn encode_indexed<'b>(&self, input: &'b [u8], index: &MergeIndex) -> TokenSet<'b> {
        if input.is_empty() {
            return TokenSet::new(input, &[]);
        }

        if self.vocab.merge_count() == 0 || index.is_empty() {
            let tokens: alloc::vec::Vec<Token<'_>> = input.chunks(1).map(Token::new).collect();
            return TokenSet::new(input, &tokens);
        }

        let mut pieces: alloc::vec::Vec<&[u8]> = input.chunks(1).collect();

        while let Some((pair_index, _rank)) = find_best_pair_indexed(&pieces, index) {
            let left = pieces[pair_index];
            let right = pieces[pair_index + 1];

            let left_start = (left.as_ptr() as usize) - (input.as_ptr() as usize);
            let right_end =
                (right.as_ptr() as usize) - (input.as_ptr() as usize) + right.len();
            let merged = &input[left_start..right_end];

            pieces[pair_index] = merged;
            pieces.remove(pair_index + 1);
        }

        let tokens: alloc::vec::Vec<Token<'_>> =
            pieces.iter().map(|&piece| Token::new(piece)).collect();

        TokenSet::new(input, &tokens)
    }
}

/// Train a BPE vocabulary from a raw byte corpus.
///
/// Convenience function wrapping [`Vocab::train`].
///
/// # Arguments
///
/// * `corpus` — Raw byte input to learn from.
/// * `num_merges` — Maximum number of merge rules to learn.
///
/// # Errors
///
/// Returns an error if the corpus is too small or has insufficient pairs.
///
/// # Examples
///
/// ```
/// use tpt_lexicon_core::train_vocab;
///
/// let vocab = train_vocab(b"the cat sat on the mat", 100).unwrap();
/// assert!(vocab.merge_count() > 0);
/// ```
pub fn train_vocab(corpus: &[u8], num_merges: usize) -> Result<Vocab> {
    Vocab::train(corpus, num_merges)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_empty_input() {
        let vocab = Vocab::new();
        let tok = BpeTokenizer::new(&vocab);
        let tokens = tok.encode(b"");
        assert!(tokens.is_empty());
    }

    #[test]
    fn encode_single_byte_no_merges() {
        let vocab = Vocab::new();
        let tok = BpeTokenizer::new(&vocab);
        let tokens = tok.encode(b"A");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens.get(0).as_bytes(), b"A");
    }

    #[test]
    fn encode_multiple_bytes_no_merges() {
        let vocab = Vocab::new();
        let tok = BpeTokenizer::new(&vocab);
        let tokens = tok.encode(b"ABC");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens.get(0).as_bytes(), b"A");
        assert_eq!(tokens.get(1).as_bytes(), b"B");
        assert_eq!(tokens.get(2).as_bytes(), b"C");
    }

    #[test]
    fn decode_roundtrip_no_merges() {
        let vocab = Vocab::new();
        let tok = BpeTokenizer::new(&vocab);
        let input = b"hello";
        let tokens = tok.encode(input);
        let decoded = tokens.as_bytes();
        assert_eq!(decoded, input);
    }

    #[test]
    fn encode_with_merges() {
        let corpus = b"ab ab ab ab";
        let vocab = Vocab::train(corpus, 5).unwrap();
        let tok = BpeTokenizer::new(&vocab);

        let tokens = tok.encode(b"ab");
        assert!(
            tokens.len() <= 2,
            "expected merged tokens, got {}",
            tokens.len()
        );

        let decoded = tokens.as_bytes();
        assert_eq!(decoded, b"ab");
    }

    #[test]
    fn train_vocab_basic() {
        let corpus = b"hello world hello world hello universe";
        let vocab = Vocab::train(corpus, 10).unwrap();
        assert!(vocab.merge_count() > 0);
        assert!(vocab.merge_count() <= 10);
    }

    #[test]
    fn train_vocab_empty_corpus() {
        let result = Vocab::train(b"", 10);
        assert!(result.is_err());
    }

    #[test]
    fn train_vocab_single_byte() {
        let result = Vocab::train(b"A", 10);
        assert!(result.is_err());
    }

    #[test]
    fn vocab_roundtrip_bytes() {
        let corpus = b"abc abc def def";
        let original = Vocab::train(corpus, 5).unwrap();
        let serialized = original.to_bytes();
        let deserialized = Vocab::from_bytes(&serialized).unwrap();
        assert_eq!(original.merge_count(), deserialized.merge_count());
        assert_eq!(original.merges(), deserialized.merges());
    }

    #[test]
    fn vocab_from_bytes_magic_check() {
        let result = Vocab::from_bytes(b"XXXX");
        assert!(result.is_err());
    }

    #[test]
    fn vocab_from_bytes_truncated() {
        let result = Vocab::from_bytes(b"TPLX");
        assert!(result.is_err());
    }

    #[test]
    fn vocab_add_special() {
        let mut vocab = Vocab::new();
        vocab.add_special(b"bos", b"<|bos|>");
        vocab.add_special(b"eos", b"<|eos|>");
        assert_eq!(vocab.special_count(), 2);
        assert_eq!(vocab.specials()[0].name, b"bos");
        assert_eq!(vocab.specials()[1].token, b"<|eos|>");
    }

    #[test]
    fn token_basics() {
        let input = b"hello";
        let token = Token::new(&input[0..5]);
        assert_eq!(token.len(), 5);
        assert!(!token.is_empty());
        assert_eq!(token.as_bytes(), b"hello");
        assert_eq!(token.as_str(), Some("hello"));
    }

    #[test]
    fn token_empty() {
        let token = Token::new(b"");
        assert!(token.is_empty());
        assert_eq!(token.len(), 0);
    }

    #[test]
    fn token_set_indexing() {
        let input = b"hello world";
        let t1 = Token::new(&input[0..5]);
        let t2 = Token::new(&input[6..11]);
        let tokens = alloc::vec![t1, t2];
        let set = TokenSet::new(input, &tokens);

        assert_eq!(set.len(), 2);
        assert_eq!(set[0].as_bytes(), b"hello");
        assert_eq!(set[1].as_bytes(), b"world");
        assert_eq!(set.total_bytes(), 10);
        assert_eq!(set.as_bytes(), b"hello world");
    }

    #[test]
    fn decode_multi_token_roundtrip() {
        let corpus = b"aabb aabb aabb";
        let vocab = Vocab::train(corpus, 5).unwrap();
        let tok = BpeTokenizer::new(&vocab);

        let input = b"aabb";
        let tokens = tok.encode(input);
        assert_eq!(tokens.as_bytes(), input);
    }

    #[test]
    fn encode_repeated_pattern() {
        let corpus = b"the the the the the";
        let vocab = Vocab::train(corpus, 10).unwrap();
        let tok = BpeTokenizer::new(&vocab);

        let input = b"the the the";
        let tokens = tok.encode(input);
        assert_eq!(tokens.as_bytes(), input);
    }

    #[test]
    fn merge_entry_total_len() {
        let entry = MergeEntry {
            left: b"ab".to_vec(),
            right: b"c".to_vec(),
            rank: 0,
        };
        assert_eq!(entry.total_len(), 3);
    }

    use alloc::format;

    #[test]
    fn vocab_display() {
        let mut vocab = Vocab::new();
        vocab.add_special(b"bos", b"<|bos|>");
        assert_eq!(format!("{vocab}"), "Vocab { merges: 0, specials: 1 }");
    }

    #[test]
    fn token_as_str_invalid_utf8() {
        let token = Token::new(&[0xFF, 0xFE]);
        assert!(token.as_str().is_none());
    }

    #[test]
    fn find_best_pair_returns_lowest_rank() {
        let mut vocab = Vocab::new();
        vocab.merges.push(MergeEntry {
            left: b"h".to_vec(),
            right: b"e".to_vec(),
            rank: 10,
        });
        vocab.merges.push(MergeEntry {
            left: b"l".to_vec(),
            right: b"l".to_vec(),
            rank: 5,
        });

        let tokens: &[&[u8]] = &[b"h", b"e", b"l", b"l", b"o"];
        let result = vocab.find_best_pair(tokens);
        assert!(result.is_some());
        let (idx, rank) = result.unwrap();
        assert_eq!(idx, 2);
        assert_eq!(rank, 5);
    }

    #[test]
    fn find_best_pair_no_match() {
        let mut vocab = Vocab::new();
        vocab.merges.push(MergeEntry {
            left: b"x".to_vec(),
            right: b"y".to_vec(),
            rank: 0,
        });

        let tokens: &[&[u8]] = &[b"a", b"b", b"c"];
        assert!(vocab.find_best_pair(tokens).is_none());
    }

    #[test]
    fn find_best_pair_single_token() {
        let mut vocab = Vocab::new();
        vocab.merges.push(MergeEntry {
            left: b"a".to_vec(),
            right: b"b".to_vec(),
            rank: 0,
        });

        let tokens: &[&[u8]] = &[b"a"];
        assert!(vocab.find_best_pair(tokens).is_none());
    }

    #[test]
    fn train_and_tokenize_larger_corpus() {
        let corpus = b"the quick brown fox jumps over the lazy dog \
                       the quick brown fox jumps over the lazy dog \
                       the quick brown fox jumps over the lazy dog";
        let vocab = Vocab::train(corpus, 50).unwrap();
        let tok = BpeTokenizer::new(&vocab);

        let input = b"the quick brown fox";
        let tokens = tok.encode(input);
        assert_eq!(tokens.as_bytes(), input);
        assert!(tokens.len() <= 19); // at most 19 bytes = 19 tokens
    }

    #[test]
    fn encode_multibyte_utf8() {
        let input = "héllo wörld".as_bytes();
        let vocab = Vocab::new();
        let tok = BpeTokenizer::new(&vocab);
        let tokens = tok.encode(input);
        assert_eq!(tokens.as_bytes(), input);
        assert_eq!(tokens.len(), input.len());
    }

    #[test]
    fn vocab_serialization_preserves_specials() {
        let mut vocab = Vocab::train(b"abc abc def", 3).unwrap();
        vocab.add_special(b"bos", b"<|bos|>");
        vocab.add_special(b"eos", b"<|eos|>");

        let serialized = vocab.to_bytes();
        let deserialized = Vocab::from_bytes(&serialized).unwrap();

        assert_eq!(deserialized.special_count(), 2);
        assert_eq!(deserialized.specials()[0].name, b"bos");
        assert_eq!(deserialized.specials()[1].token, b"<|eos|>");
    }

    #[test]
    fn token_set_to_vec() {
        let input = b"hello";
        let tokens = alloc::vec![Token::new(&input[0..5])];
        let set = TokenSet::new(input, &tokens);
        assert_eq!(set.to_vec(), b"hello");
    }

    #[test]
    fn encode_to_vec_feature() {
        let vocab = Vocab::new();
        let tok = BpeTokenizer::new(&vocab);
        let tokens = tok.encode_to_vec(b"abc");
        assert_eq!(tokens.len(), 3);
    }
}

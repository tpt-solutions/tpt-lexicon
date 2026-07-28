//! Legacy BPE bridge: unrolls IR into token ID sequences.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

/// A mapping from byte sequences to legacy token IDs.
///
/// This mirrors the vocabulary used by a pre-trained model (e.g., Llama,
/// Mistral). The bridge uses it to translate IR content into the exact token
/// IDs the model expects.
#[derive(Debug, Clone)]
pub struct LegacyVocab {
    /// Mapping from byte sequences to token IDs.
    entries: BTreeMap<Vec<u8>, u32>,
    /// Reverse mapping from token IDs to byte sequences (for decode).
    reverse: BTreeMap<u32, Vec<u8>>,
    /// The ID assigned to unknown tokens.
    unk_id: u32,
}

impl LegacyVocab {
    /// Create a new legacy vocabulary with the given unknown-token ID.
    #[inline]
    pub fn new(unk_id: u32) -> Self {
        Self {
            entries: BTreeMap::new(),
            reverse: BTreeMap::new(),
            unk_id,
        }
    }

    /// Add a mapping from bytes to token ID.
    pub fn insert(&mut self, bytes: &[u8], id: u32) {
        self.entries.insert(bytes.to_vec(), id);
        self.reverse.insert(id, bytes.to_vec());
    }

    /// Look up the token ID for a byte sequence.
    #[inline]
    pub fn lookup(&self, bytes: &[u8]) -> Option<u32> {
        self.entries.get(bytes).copied()
    }

    /// Look up the bytes for a token ID.
    #[inline]
    pub fn reverse_lookup(&self, id: u32) -> Option<&[u8]> {
        self.reverse.get(&id).map(|v| v.as_slice())
    }

    /// Returns the unknown token ID.
    #[inline]
    pub fn unk_id(&self) -> u32 {
        self.unk_id
    }

    /// Returns the number of entries in the vocabulary.
    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if the vocabulary is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// The translation bridge: unrolls IR content into legacy token-ID sequences.
///
/// # Examples
///
/// ```
/// use tpt_lexicon_translate::{LegacyBridge, LegacyVocab};
///
/// let mut vocab = LegacyVocab::new(0);
/// vocab.insert(b"hello", 1);
/// vocab.insert(b" ", 2);
/// vocab.insert(b"world", 3);
///
/// let bridge = LegacyBridge::new(&vocab);
/// let ids = bridge.unroll_bytes(b"hello world");
/// assert_eq!(ids, vec![1, 2, 3]);
/// ```
#[derive(Debug)]
pub struct LegacyBridge<'a> {
    vocab: &'a LegacyVocab,
}

impl<'a> LegacyBridge<'a> {
    /// Create a new bridge with the given vocabulary.
    #[inline]
    pub fn new(vocab: &'a LegacyVocab) -> Self {
        Self { vocab }
    }

    /// Returns a reference to the underlying vocabulary.
    #[inline]
    pub fn vocab(&self) -> &'a LegacyVocab {
        self.vocab
    }

    /// Unroll raw bytes into legacy token IDs using a greedy longest-match
    /// algorithm.
    ///
    /// Starting from the beginning of `bytes`, finds the longest vocabulary
    /// entry that matches, emits its token ID, and advances past it. Unknown
    /// bytes are emitted as the vocabulary's `unk_id`.
    pub fn unroll_bytes(&self, bytes: &[u8]) -> Vec<u32> {
        let mut ids = Vec::new();
        let mut pos = 0;

        while pos < bytes.len() {
            let mut best_len = 0;
            let mut best_id = self.vocab.unk_id();

            // Try all possible lengths from longest to shortest.
            let max_len = (bytes.len() - pos).min(64);
            for len in (1..=max_len).rev() {
                if let Some(&id) = self.vocab.entries.get(&bytes[pos..pos + len]) {
                    best_len = len;
                    best_id = id;
                    break;
                }
            }

            ids.push(best_id);
            if best_len == 0 {
                // Unknown byte — advance by 1.
                pos += 1;
            } else {
                pos += best_len;
            }
        }

        ids
    }

    /// Decode a sequence of token IDs back to bytes using the vocabulary's
    /// reverse mapping.
    ///
    /// Unknown IDs are skipped.
    pub fn decode_ids(&self, ids: &[u32]) -> Vec<u8> {
        let mut bytes = Vec::new();
        for &id in ids {
            if let Some(entry) = self.vocab.reverse.get(&id) {
                bytes.extend_from_slice(entry);
            }
        }
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VERSION;
    use alloc::vec;

    fn make_vocab() -> LegacyVocab {
        let mut v = LegacyVocab::new(0);
        v.insert(b"hel", 1);
        v.insert(b"lo", 2);
        v.insert(b" ", 3);
        v.insert(b"world", 4);
        v.insert(b"h", 5);
        v.insert(b"e", 6);
        v.insert(b"l", 7);
        v.insert(b"o", 8);
        v
    }

    #[test]
    fn unroll_simple() {
        let v = make_vocab();
        let bridge = LegacyBridge::new(&v);
        let ids = bridge.unroll_bytes(b"hello world");
        // "hel"(1) + "lo"(2) + " "(3) + "world"(4)
        assert_eq!(ids, vec![1, 2, 3, 4]);
    }

    #[test]
    fn unroll_longest_match() {
        let v = make_vocab();
        let bridge = LegacyBridge::new(&v);
        let ids = bridge.unroll_bytes(b"hel");
        assert_eq!(ids, vec![1]);
    }

    #[test]
    fn unroll_unknown_bytes() {
        let v = make_vocab();
        let bridge = LegacyBridge::new(&v);
        let ids = bridge.unroll_bytes(b"xyz");
        assert_eq!(ids, vec![0, 0, 0]); // all unknown
    }

    #[test]
    fn decode_roundtrip() {
        let v = make_vocab();
        let bridge = LegacyBridge::new(&v);
        let ids = bridge.unroll_bytes(b"hello world");
        let decoded = bridge.decode_ids(&ids);
        assert_eq!(decoded, b"hello world");
    }

    #[test]
    fn vocab_basics() {
        let v = make_vocab();
        assert_eq!(v.len(), 8);
        assert!(!v.is_empty());
        assert_eq!(v.lookup(b"hel"), Some(1));
        assert_eq!(v.lookup(b"xyz"), None);
        assert_eq!(v.unk_id(), 0);
        assert_eq!(v.reverse_lookup(4), Some(b"world".as_slice()));
    }

    #[test]
    fn empty_input() {
        let v = make_vocab();
        let bridge = LegacyBridge::new(&v);
        let ids = bridge.unroll_bytes(b"");
        assert!(ids.is_empty());
    }

    #[test]
    fn greedy_finds_longest() {
        let mut v = LegacyVocab::new(0);
        v.insert(b"a", 1);
        v.insert(b"ab", 2);
        v.insert(b"abc", 3);
        let bridge = LegacyBridge::new(&v);
        let ids = bridge.unroll_bytes(b"abc");
        assert_eq!(ids, vec![3]);
    }

    #[test]
    fn version_is_nonempty() {
        assert!(!VERSION.is_empty());
    }
}

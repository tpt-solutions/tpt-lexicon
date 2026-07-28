//! Zero-copy token types for TPT Lexicon.
//!
//! A [`Token`] is an immutable borrow of a byte slice from the caller's input
//! buffer. Tokens carry no ownership and perform no allocation.
//!
//! [`TokenSet`] owns a vector of tokens and retains a reference to the
//! original input, enabling safe zero-copy decoding.

/// A single token — an immutable borrow of a contiguous byte range within the
/// caller's input buffer.
///
/// Tokens are produced by [`crate::BpeTokenizer::encode`] and never outlive the
/// input they were derived from.
///
/// # Examples
///
/// ```
/// use tpt_lexicon_core::Token;
///
/// let input = b"hello world";
/// let token = Token::new(&input[0..5]);
/// assert_eq!(token.as_bytes(), b"hello");
/// assert_eq!(token.len(), 5);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Token<'a> {
    bytes: &'a [u8],
}

impl<'a> Token<'a> {
    /// Create a new token from a byte slice.
    #[inline]
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    /// Returns the token as a byte slice.
    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.bytes
    }

    /// Returns the length of the token in bytes.
    #[inline]
    pub const fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Returns `true` if the token is empty (zero bytes).
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Interpret the token bytes as a UTF-8 string slice.
    ///
    /// Returns `None` if the bytes are not valid UTF-8.
    #[inline]
    pub fn as_str(&self) -> Option<&'a str> {
        core::str::from_utf8(self.bytes).ok()
    }
}

/// An ordered sequence of tokens with a reference to the original input buffer.
///
/// `TokenSet` is produced by [`crate::BpeTokenizer::encode`] and provides
/// iteration, indexing, and zero-copy decode via [`TokenSet::as_bytes`].
///
/// # Examples
///
/// ```
/// use tpt_lexicon_core::{Token, TokenSet};
///
/// let input = b"hello world";
/// let tokens = vec![Token::new(&input[0..5]), Token::new(&input[6..11])];
/// let set = TokenSet::new(input, &tokens);
/// assert_eq!(set.len(), 2);
/// assert_eq!(set.as_bytes(), b"hello world");
/// ```
#[derive(Debug, Clone)]
pub struct TokenSet<'a> {
    input: &'a [u8],
    tokens: alloc::vec::Vec<Token<'a>>,
}

impl<'a> TokenSet<'a> {
    /// Create a token set from an input buffer and a list of tokens.
    ///
    /// All tokens must borrow from the same `input` buffer.
    #[inline]
    pub fn new(input: &'a [u8], tokens: &[Token<'a>]) -> Self {
        Self {
            input,
            tokens: tokens.to_vec(),
        }
    }

    /// Returns the number of tokens in the set.
    #[inline]
    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    /// Returns `true` if the token set contains no tokens.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    /// Returns the total number of bytes across all tokens.
    #[inline]
    pub fn total_bytes(&self) -> usize {
        self.tokens.iter().map(|t| t.len()).sum()
    }

    /// Returns a reference to the token at the given index.
    ///
    /// # Panics
    ///
    /// Panics if `index >= self.len()`.
    #[inline]
    pub fn get(&self, index: usize) -> &Token<'a> {
        &self.tokens[index]
    }

    /// Returns an iterator over the tokens.
    #[inline]
    pub fn iter(&self) -> core::slice::Iter<'_, Token<'a>> {
        self.tokens.iter()
    }

    /// Decode the tokens back into the original contiguous byte range.
    ///
    /// Since all tokens are sub-slices of the same input buffer and appear in
    /// order, this returns a single contiguous slice spanning from the first
    /// token's start to the last token's end — zero-copy.
    #[inline]
    pub fn as_bytes(&self) -> &'a [u8] {
        if self.tokens.is_empty() {
            return &[];
        }

        let first = &self.tokens[0];
        let last = &self.tokens[self.tokens.len() - 1];

        let input_start = self.input.as_ptr() as usize;
        let first_start = first.as_bytes().as_ptr() as usize;
        let last_end = last.as_bytes().as_ptr() as usize + last.len();

        let start = first_start - input_start;
        let end = last_end - input_start;

        &self.input[start..end]
    }

    /// Collect all token bytes into a new `Vec<u8>`.
    ///
    /// Requires the `alloc` feature.
    #[inline]
    pub fn to_vec(&self) -> alloc::vec::Vec<u8> {
        let mut buf = alloc::vec::Vec::with_capacity(self.total_bytes());
        for token in &self.tokens {
            buf.extend_from_slice(token.as_bytes());
        }
        buf
    }
}

impl<'a> core::ops::Index<usize> for TokenSet<'a> {
    type Output = Token<'a>;

    #[inline]
    fn index(&self, index: usize) -> &Token<'a> {
        &self.tokens[index]
    }
}

impl<'a, 'b> IntoIterator for &'b TokenSet<'a> {
    type Item = &'b Token<'a>;
    type IntoIter = core::slice::Iter<'b, Token<'a>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.tokens.iter()
    }
}

impl<'a> AsRef<[Token<'a>]> for TokenSet<'a> {
    #[inline]
    fn as_ref(&self) -> &[Token<'a>] {
        &self.tokens
    }
}

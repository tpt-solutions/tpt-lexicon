//! Streaming parser with boundary detection.

use core::fmt;

/// The kind of content a chunk represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChunkKind {
    /// Plain text (default for unclassified content).
    Text,
    /// Source code (inside a fenced code block).
    Code,
    /// Markdown structural element (header, list, etc.).
    Markdown,
    /// JSON data (balanced `{}` or `[]` structure).
    Json,
    /// Natural-language paragraph (separated by blank lines).
    Paragraph,
}

impl fmt::Display for ChunkKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text => write!(f, "text"),
            Self::Code => write!(f, "code"),
            Self::Markdown => write!(f, "markdown"),
            Self::Json => write!(f, "json"),
            Self::Paragraph => write!(f, "paragraph"),
        }
    }
}

/// A semantic chunk of input — a borrowed sub-slice tagged with its content
/// type.
///
/// Chunks never allocate; they borrow directly from the parser's input buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chunk<'a> {
    /// The content type of this chunk.
    pub kind: ChunkKind,
    /// The raw bytes of this chunk.
    pub bytes: &'a [u8],
    /// Byte offset of this chunk within the original input.
    pub offset: usize,
}

impl<'a> Chunk<'a> {
    /// Returns the chunk as a UTF-8 string, if valid.
    #[inline]
    pub fn as_str(&self) -> Option<&'a str> {
        core::str::from_utf8(self.bytes).ok()
    }

    /// Returns the length of the chunk in bytes.
    #[inline]
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Returns `true` if the chunk is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

/// A streaming parser that yields [`Chunk`]s from a byte buffer.
///
/// The parser performs zero-copy boundary detection: each chunk borrows
/// directly from the input.
///
/// # Examples
///
/// ```
/// use tpt_lexicon_ingest::{Parser, ChunkKind};
///
/// let input = b"Hello world.\n\nSecond paragraph.";
/// let chunks: Vec<_> = Parser::new(input).collect();
/// assert_eq!(chunks.len(), 2);
/// assert_eq!(chunks[0].kind, ChunkKind::Paragraph);
/// assert_eq!(chunks[1].kind, ChunkKind::Paragraph);
/// ```
#[derive(Debug)]
pub struct Parser<'a> {
    input: &'a [u8],
    pos: usize,
    state: ParserState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParserState {
    /// Scanning for the next chunk boundary.
    Scanning,
    /// Inside a fenced code block.
    InCodeBlock {
        /// The fence character (typically ` ` `).
        fence_char: u8,
        /// How many consecutive fence characters opened the block.
        fence_count: usize,
    },
    /// Done — no more chunks.
    Done,
}

impl<'a> Parser<'a> {
    /// Create a new parser over the given input.
    #[inline]
    pub fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            pos: 0,
            state: ParserState::Scanning,
        }
    }

    /// Returns the remaining unparsed input.
    #[inline]
    pub fn remaining(&self) -> &'a [u8] {
        &self.input[self.pos..]
    }

    /// Returns the current byte position within the input.
    #[inline]
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Scan forward from `start` to find the next line that starts with
    /// `count` consecutive `fence_char` bytes followed by a space, newline,
    /// or end-of-input. Returns the byte offset of the start of that line.
    fn find_code_fence_end(&self, fence_char: u8, fence_count: usize) -> Option<usize> {
        let mut i = self.pos;
        while i < self.input.len() {
            // Find the next newline.
            let line_start = i;
            let line_end = self.input[i..]
                .iter()
                .position(|&b| b == b'\n')
                .map(|p| i + p + 1)
                .unwrap_or(self.input.len());

            let line = &self.input[line_start..line_end];

            // Check if this line starts with at least `fence_count` of the
            // fence character.
            if line.len() >= fence_count
                && line[..fence_count].iter().all(|&b| b == fence_char)
            {
                // Must be followed by space, newline, or end-of-input.
                let after_fence = line.get(fence_count).copied();
                match after_fence {
                    Some(b' ') | Some(b'\n') | Some(b'\r') | None => {
                        return Some(line_start);
                    }
                    _ => {}
                }
            }

            i = line_end;
        }
        None
    }

    /// Try to detect JSON starting at the current position.
    ///
    /// Returns `Some(end)` if a balanced JSON structure is found, where `end`
    /// is the byte after the closing delimiter.
    fn try_parse_json(&self) -> Option<usize> {
        let start = self.pos;
        if start >= self.input.len() {
            return None;
        }

        match self.input[start] {
            b'{' => self.scan_balanced(start, b'{', b'}'),
            b'[' => self.scan_balanced(start, b'[', b']'),
            _ => None,
        }
    }

    /// Scan a balanced structure (matching open/close delimiters), respecting
    /// string literals and nesting.
    fn scan_balanced(&self, start: usize, open: u8, close: u8) -> Option<usize> {
        let mut depth: i32 = 1;
        let mut i = start + 1;
        let mut in_string = false;
        let mut escape = false;

        while i < self.input.len() && depth > 0 {
            let b = self.input[i];

            if escape {
                escape = false;
            } else if b == b'\\' && in_string {
                escape = true;
            } else if b == b'"' {
                in_string = !in_string;
            } else if !in_string {
                match b {
                    b if b == open => depth += 1,
                    b if b == close => depth -= 1,
                    _ => {}
                }
            }

            i += 1;
        }

        if depth == 0 {
            Some(i)
        } else {
            None
        }
    }

    /// Scan for the end of the line starting at `pos` (handles `\n` and `\r\n`).
    fn line_end(&self, pos: usize) -> usize {
        if pos >= self.input.len() {
            return self.input.len();
        }
        let mut i = pos;
        while i < self.input.len() {
            if self.input[i] == b'\r' {
                return if i + 1 < self.input.len() && self.input[i + 1] == b'\n' {
                    i + 2
                } else {
                    i + 1
                };
            }
            if self.input[i] == b'\n' {
                return i + 1;
            }
            i += 1;
        }
        self.input.len()
    }

    /// Check if a line is blank (contains only whitespace).
    fn is_blank_line(&self, pos: usize) -> bool {
        let mut i = pos;
        while i < self.input.len() {
            match self.input[i] {
                b'\n' | b'\r' => return true,
                b' ' | b'\t' | b'\x0C' => i += 1,
                _ => return false,
            }
        }
        true
    }

    /// Find the end of a paragraph (terminated by a blank line or end of
    /// input).
    fn find_paragraph_end(&self) -> usize {
        let mut i = self.pos;

        while i < self.input.len() {
            if self.is_blank_line(i) {
                return i;
            }
            i = self.line_end(i);
        }

        self.input.len()
    }

    /// Find the end of a markdown structural line (header, etc.).
    fn find_markdown_line_end(&self) -> usize {
        self.line_end(self.pos)
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Chunk<'a>;

    fn next(&mut self) -> Option<Chunk<'a>> {
        loop {
            match self.state {
                ParserState::Done => return None,
                ParserState::Scanning => {
                    if self.pos >= self.input.len() {
                        self.state = ParserState::Done;
                        return None;
                    }

                    let line = &self.input[self.pos..];

                    // Check for fenced code block.
                    if line.starts_with(b"```") {
                        let fence_char = b'`';
                        let fence_count = line
                            .iter()
                            .take_while(|&&b| b == fence_char)
                            .count();
                        self.state = ParserState::InCodeBlock {
                            fence_char,
                            fence_count,
                        };

                        // Skip the opening fence line.
                        let opening_end = self.line_end(self.pos);
                        self.pos = opening_end;

                        // Find the closing fence.
                        if let Some(end) = self.find_code_fence_end(fence_char, fence_count) {
                            let chunk = Chunk {
                                kind: ChunkKind::Code,
                                bytes: &self.input[opening_end..end],
                                offset: opening_end,
                            };
                            self.pos = end;
                            // Skip the closing fence line.
                            self.pos = self.line_end(self.pos);
                            self.state = ParserState::Scanning;
                            return Some(chunk);
                        } else {
                            // No closing fence — treat rest as code.
                            let chunk = Chunk {
                                kind: ChunkKind::Code,
                                bytes: &self.input[opening_end..],
                                offset: opening_end,
                            };
                            self.pos = self.input.len();
                            self.state = ParserState::Done;
                            return Some(chunk);
                        }
                    }

                    // Check for Markdown header or structural element.
                    if line.starts_with(b"# ")
                        || line.starts_with(b"## ")
                        || line.starts_with(b"### ")
                        || line.starts_with(b"#### ")
                        || line.starts_with(b"##### ")
                        || line.starts_with(b"###### ")
                        || line.starts_with(b"- ")
                        || line.starts_with(b"* ")
                        || line.starts_with(b"> ")
                    {
                        let end = self.find_markdown_line_end();
                        let chunk = Chunk {
                            kind: ChunkKind::Markdown,
                            bytes: &self.input[self.pos..end],
                            offset: self.pos,
                        };
                        self.pos = end;
                        return Some(chunk);
                    }

                    // Check for JSON.
                    if let Some(end) = self.try_parse_json() {
                        let chunk = Chunk {
                            kind: ChunkKind::Json,
                            bytes: &self.input[self.pos..end],
                            offset: self.pos,
                        };
                        self.pos = end;
                        return Some(chunk);
                    }

                    // Default: paragraph / text.
                    let end = self.find_paragraph_end();
                    let chunk = Chunk {
                        kind: ChunkKind::Paragraph,
                        bytes: &self.input[self.pos..end],
                        offset: self.pos,
                    };
                    self.pos = end;
                    // Skip past any blank line separators.
                    while self.pos < self.input.len() && self.is_blank_line(self.pos) {
                        self.pos = self.line_end(self.pos);
                    }
                    return Some(chunk);
                }
                ParserState::InCodeBlock {
                    fence_char,
                    fence_count,
                } => {
                    // This state is transient — we resolve it in the
                    // Scanning branch. If we somehow end up here, skip.
                    if let Some(end) = self.find_code_fence_end(fence_char, fence_count) {
                        self.pos = self.line_end(end);
                    } else {
                        self.pos = self.input.len();
                    }
                    self.state = ParserState::Scanning;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;
    use crate::VERSION;

    #[test]
    fn empty_input() {
        let chunks: Vec<_> = Parser::new(b"").collect();
        assert!(chunks.is_empty());
    }

    #[test]
    fn single_paragraph() {
        let chunks: Vec<_> = Parser::new(b"Hello world").collect();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].kind, ChunkKind::Paragraph);
        assert_eq!(chunks[0].bytes, b"Hello world");
    }

    #[test]
    fn two_paragraphs() {
        let input = b"First paragraph.\n\nSecond paragraph.";
        let chunks: Vec<_> = Parser::new(input).collect();
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].bytes, b"First paragraph.\n");
        assert_eq!(chunks[1].bytes, b"Second paragraph.");
    }

    #[test]
    fn markdown_header() {
        let input = b"# Title\nSome text.";
        let chunks: Vec<_> = Parser::new(input).collect();
        assert!(chunks.len() >= 2);
        assert_eq!(chunks[0].kind, ChunkKind::Markdown);
        assert_eq!(chunks[0].bytes, b"# Title\n");
    }

    #[test]
    fn code_block() {
        let input = b"```rust\nlet x = 1;\n```\n";
        let chunks: Vec<_> = Parser::new(input).collect();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].kind, ChunkKind::Code);
        assert_eq!(chunks[0].bytes, b"let x = 1;\n");
    }

    #[test]
    fn code_block_no_closing_fence() {
        let input = b"```rust\nlet x = 1;\nlet y = 2;";
        let chunks: Vec<_> = Parser::new(input).collect();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].kind, ChunkKind::Code);
    }

    #[test]
    fn json_object() {
        let input = b"{\"key\": \"value\"}";
        let chunks: Vec<_> = Parser::new(input).collect();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].kind, ChunkKind::Json);
    }

    #[test]
    fn json_array() {
        let input = b"[1, 2, 3]";
        let chunks: Vec<_> = Parser::new(input).collect();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].kind, ChunkKind::Json);
    }

    #[test]
    fn json_nested() {
        let input = b"{\"arr\": [1, 2], \"obj\": {\"nested\": true}}";
        let chunks: Vec<_> = Parser::new(input).collect();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].kind, ChunkKind::Json);
        assert_eq!(chunks[0].bytes, input);
    }

    #[test]
    fn mixed_content() {
        let input = b"# Title\n\nSome text.\n\n```rust\ncode\n```\n\n{\"k\": 1}\n";
        let chunks: Vec<_> = Parser::new(input).collect();
        assert!(chunks.len() >= 4);

        let kinds: Vec<_> = chunks.iter().map(|c| c.kind).collect();
        assert!(kinds.contains(&ChunkKind::Markdown));
        assert!(kinds.contains(&ChunkKind::Paragraph));
        assert!(kinds.contains(&ChunkKind::Code));
        assert!(kinds.contains(&ChunkKind::Json));
    }

    #[test]
    fn bullet_list_as_markdown() {
        let input = b"- Item one\n- Item two\n";
        let chunks: Vec<_> = Parser::new(input).collect();
        assert!(chunks.iter().any(|c| c.kind == ChunkKind::Markdown));
    }

    #[test]
    fn blockquote_as_markdown() {
        let input = b"> A quote\n";
        let chunks: Vec<_> = Parser::new(input).collect();
        assert!(chunks.iter().any(|c| c.kind == ChunkKind::Markdown));
    }

    #[test]
    fn chunk_as_str() {
        let input = b"hello";
        let chunks: Vec<_> = Parser::new(input).collect();
        assert_eq!(chunks[0].as_str(), Some("hello"));
    }

    #[test]
    fn chunk_len_and_empty() {
        let input = b"abc";
        let chunks: Vec<_> = Parser::new(input).collect();
        assert_eq!(chunks[0].len(), 3);
        assert!(!chunks[0].is_empty());
    }

    #[test]
    fn parser_position_and_remaining() {
        let mut parser = Parser::new(b"abc");
        assert_eq!(parser.position(), 0);
        assert_eq!(parser.remaining(), b"abc");
        parser.next();
        assert!(parser.position() > 0);
    }

    #[test]
    fn unclosed_json_falls_through_to_paragraph() {
        let input = b"{\"incomplete\": ";
        let chunks: Vec<_> = Parser::new(input).collect();
        assert_eq!(chunks.len(), 1);
        // Unclosed JSON should be treated as paragraph/text
        assert!(chunks[0].kind == ChunkKind::Paragraph || chunks[0].kind == ChunkKind::Text);
    }

    #[test]
    fn multiple_code_blocks() {
        let input = b"Text\n\n```\ncode1\n```\n\nMore text\n\n```\ncode2\n```\n";
        let chunks: Vec<_> = Parser::new(input).collect();
        let code_chunks: Vec<_> = chunks
            .iter()
            .filter(|c| c.kind == ChunkKind::Code)
            .collect();
        assert_eq!(code_chunks.len(), 2);
    }

    #[test]
    fn version_is_nonempty() {
        assert!(!VERSION.is_empty());
    }
}

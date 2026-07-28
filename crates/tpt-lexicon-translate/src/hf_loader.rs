//! HuggingFace `tokenizer.json` loader.
//!
//! Parses a minimal subset of the HuggingFace `tokenizer.json` format to
//! extract vocabulary mappings and merge rules.

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::error::{Error, Result};

/// A loaded HuggingFace tokenizer configuration.
///
/// Contains the vocabulary map (token string -> ID), merge rules, and model
/// type information.
#[derive(Debug, Clone)]
pub struct HfTokenizer {
    /// The model type (e.g., `"BPE"`, `"WordPiece"`).
    pub model_type: String,
    /// Vocabulary mapping: token string -> token ID.
    pub vocab: BTreeMap<String, u32>,
    /// BPE merge rules as ordered pairs of token strings.
    pub merges: Vec<(String, String)>,
    /// Special token mappings: name -> token string.
    pub special_tokens: BTreeMap<String, String>,
}

impl HfTokenizer {
    /// Parse a HuggingFace `tokenizer.json` from raw bytes.
    ///
    /// This is a minimal parser that extracts the `model.vocab`,
    /// `model.merges`, and `added_tokens` fields. It does not require serde.
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON is malformed or uses an unsupported model
    /// type.
    pub fn from_json(data: &[u8]) -> Result<Self> {
        let text = core::str::from_utf8(data).map_err(|_| Error::HfParseError {
            message: b"input is not valid UTF-8".to_vec(),
        })?;

        let model_type = extract_string_field(text, "type").unwrap_or_else(|| "BPE".to_string());

        if model_type != "BPE" && model_type != "WordPiece" {
            return Err(Error::UnsupportedHfModel {
                model_type: model_type.into_bytes(),
            });
        }

        let vocab = extract_vocab_map(text);
        let merges = extract_merges(text);
        let special_tokens = extract_special_tokens(text);

        Ok(Self {
            model_type,
            vocab,
            merges,
            special_tokens,
        })
    }

    /// Returns the number of vocabulary entries (including special tokens).
    #[inline]
    pub fn vocab_size(&self) -> usize {
        self.vocab.len() + self.special_tokens.len()
    }

    /// Look up the token ID for a string.
    #[inline]
    pub fn token_id(&self, token: &str) -> Option<u32> {
        self.vocab.get(token).copied()
    }

    /// Look up the token string for an ID.
    pub fn id_to_token(&self, id: u32) -> Option<&str> {
        self.vocab
            .iter()
            .find(|(_, &v)| v == id)
            .map(|(k, _)| k.as_str())
    }
}

/// Extract a simple string field value from JSON text.
///
/// Searches for `"field_name": "value"` pattern.
fn extract_string_field(text: &str, field: &str) -> Option<String> {
    let pattern = alloc::format!("\"{field}\"");
    let field_pos = text.find(&pattern)?;

    // Skip past the field name and colon.
    let after_field = text.get(field_pos + pattern.len()..)?;
    let colon_pos = after_field.find(':')?;
    let after_colon = after_field.get(colon_pos + 1..)?.trim_start();

    // Find the opening quote.
    let quote_start = after_colon.find('"')?;
    let value_start = quote_start + 1;
    let value_end = after_colon[value_start..].find('"')?;

    Some(after_colon[value_start..value_start + value_end].to_string())
}

/// Extract the vocabulary map from JSON text.
///
/// Parses `"vocab": { "token": id, ... }` or `"model": { "vocab": { ... } }`.
fn extract_vocab_map(text: &str) -> BTreeMap<String, u32> {
    let mut map = BTreeMap::new();

    // Find "vocab" object.
    let vocab_start = match text.find("\"vocab\"") {
        Some(p) => p,
        None => return map,
    };

    // Find the opening brace of the vocab object.
    let after_vocab = &text[vocab_start..];
    let brace_offset = match after_vocab.find('{') {
        Some(p) => p,
        None => return map,
    };

    let json_obj = &after_vocab[brace_offset..];
    if let Some(content) = extract_balanced_content(json_obj, '{', '}') {
        parse_string_int_map(content, &mut map);
    }

    map
}

/// Extract merge rules from JSON text.
fn extract_merges(text: &str) -> Vec<(String, String)> {
    let mut merges = Vec::new();

    let merges_start = match text.find("\"merges\"") {
        Some(p) => p,
        None => return merges,
    };

    let after_merges = &text[merges_start..];
    let bracket_offset = match after_merges.find('[') {
        Some(p) => p,
        None => return merges,
    };

    let json_arr = &after_merges[bracket_offset..];
    if let Some(content) = extract_balanced_content(json_arr, '[', ']') {
        // Each entry is a string like "token1 token2".
        let mut i = 0;
        while i < content.len() {
            // Find next quoted string.
            if let Some(q) = content[i..].find('"') {
                let start = i + q + 1;
                if let Some(end) = content[start..].find('"') {
                    let entry = &content[start..start + end];
                    if let Some(space_pos) = entry.find(' ') {
                        let left = entry[..space_pos].to_string();
                        let right = entry[space_pos + 1..].to_string();
                        merges.push((left, right));
                    }
                    i = start + end + 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    merges
}

/// Extract special token mappings from JSON text.
fn extract_special_tokens(text: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();

    // Look for "added_tokens" array.
    let added_start = match text.find("\"added_tokens\"") {
        Some(p) => p,
        None => return map,
    };

    let after_added = &text[added_start..];
    let bracket_offset = match after_added.find('[') {
        Some(p) => p,
        None => return map,
    };

    let json_arr = &after_added[bracket_offset..];
    if let Some(content) = extract_balanced_content(json_arr, '[', ']') {
        // Each entry is an object with "content" and optionally "id".
        let mut i = 0;
        while i < content.len() {
            if let Some(obj_start) = content[i..].find('{') {
                let abs_start = i + obj_start;
                if let Some(obj_content) = extract_balanced_content(&content[abs_start..], '{', '}')
                {
                    let content_val = extract_string_field(obj_content, "content");
                    if let Some(token) = content_val {
                        let name = alloc::format!("special_{}", map.len());
                        map.insert(name, token);
                    }
                    i = abs_start + obj_content.len() + 2; // skip past }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    map
}

/// Extract the content between matching delimiters.
fn extract_balanced_content(text: &str, open: char, close: char) -> Option<&str> {
    let start = text.find(open)?;
    let mut depth = 1;
    let mut i = start + 1;

    while i < text.len() && depth > 0 {
        let ch = text.as_bytes()[i] as char;
        if ch == open {
            depth += 1;
        } else if ch == close {
            depth -= 1;
        }
        i += 1;
    }

    if depth == 0 {
        Some(&text[start + 1..i - 1])
    } else {
        None
    }
}

/// Parse a JSON object of string->integer pairs.
fn parse_string_int_map(content: &str, map: &mut BTreeMap<String, u32>) {
    let mut i = 0;
    let bytes = content.as_bytes();

    while i < bytes.len() {
        // Find opening quote.
        while i < bytes.len() && bytes[i] != b'"' {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        i += 1; // skip opening quote

        // Find closing quote.
        let key_start = i;
        while i < bytes.len() && bytes[i] != b'"' {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        let key = &content[key_start..i];
        i += 1; // skip closing quote

        // Skip colon and whitespace.
        while i < bytes.len() && (bytes[i] == b':' || bytes[i] == b' ' || bytes[i] == b'\n') {
            i += 1;
        }

        // Parse integer value.
        let val_start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i > val_start {
            if let Ok(val) = content[val_start..i].parse::<u32>() {
                map.insert(key.to_string(), val);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VERSION;
    use alloc::vec::Vec;

    #[test]
    fn parse_minimal_tokenizer_json() {
        let json = r#"{
            "model": {
                "type": "BPE",
                "vocab": {
                    "hello": 0,
                    "world": 1,
                    " ": 2
                },
                "merges": ["h e", "hel lo"]
            },
            "added_tokens": [
                {"content": "<|bos|>", "id": 3},
                {"content": "<|eos|>", "id": 4}
            ]
        }"#;

        let tok = HfTokenizer::from_json(json.as_bytes()).unwrap();
        assert_eq!(tok.model_type, "BPE");
        assert_eq!(tok.token_id("hello"), Some(0));
        assert_eq!(tok.token_id("world"), Some(1));
        assert_eq!(tok.merges.len(), 2);
        assert_eq!(tok.merges[0], ("h".to_string(), "e".to_string()));
        assert_eq!(tok.merges[1], ("hel".to_string(), "lo".to_string()));
    }

    #[test]
    fn parse_unsupported_model() {
        let json = r#"{"model": {"type": "Unigram"}}"#;
        let result = HfTokenizer::from_json(json.as_bytes());
        assert!(result.is_err());
    }

    #[test]
    fn parse_invalid_utf8() {
        let result = HfTokenizer::from_json(&[0xFF, 0xFE]);
        assert!(result.is_err());
    }

    #[test]
    fn parse_empty_json() {
        let json = r#"{}"#;
        let tok = HfTokenizer::from_json(json.as_bytes()).unwrap();
        assert_eq!(tok.vocab_size(), 0);
    }

    #[test]
    fn id_to_token_lookup() {
        let mut tok = HfTokenizer {
            model_type: "BPE".to_string(),
            vocab: BTreeMap::new(),
            merges: Vec::new(),
            special_tokens: BTreeMap::new(),
        };
        tok.vocab.insert("hello".to_string(), 42);
        assert_eq!(tok.id_to_token(42), Some("hello"));
        assert_eq!(tok.id_to_token(99), None);
    }

    #[test]
    fn version_is_nonempty() {
        assert!(!VERSION.is_empty());
    }
}

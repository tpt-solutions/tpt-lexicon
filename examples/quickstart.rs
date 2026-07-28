//! Quickstart: train a vocabulary → tokenize → decode (zero-copy).
//!
//! Run with:
//!   cargo run --example quickstart
use tpt_lexicon_core::{BpeTokenizer, Vocab};

fn main() {
    // 1. Train a vocabulary from a byte corpus.
    let corpus = b"hello world hello world hello universe";
    let vocab = Vocab::train(corpus, 20).expect("training failed");
    println!("Merge rules learned: {}", vocab.merge_count());

    // 2. Encode input bytes into tokens (zero-copy borrow from input).
    let tokenizer = BpeTokenizer::new(&vocab);
    let input = b"hello world";
    let tokens = tokenizer.encode(input);
    println!("Token count: {}", tokens.len());
    for (i, tok) in tokens.iter().enumerate() {
        let s = std::str::from_utf8(tok.as_bytes()).unwrap_or("<binary>");
        println!("  [{i}] {s:?}");
    }

    // 3. Decode back to bytes (zero-copy: no allocation, just re-joining sub-slices).
    let decoded = tokens.as_bytes();
    assert_eq!(decoded, input, "round-trip must be lossless");
    println!("Decoded: {:?}", std::str::from_utf8(decoded).unwrap());
}

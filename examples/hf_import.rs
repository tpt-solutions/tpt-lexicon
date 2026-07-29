//! Load a HuggingFace `tokenizer.json` and translate into core vocabulary format.
//!
//! Run with:
//!   cargo run --example hf_import
use tpt_lexicon_translate::{HfTokenizer, LegacyBridge, LegacyVocab};

fn main() {
    // 1. Load the fixture tokenizer.json bundled with the examples.
    let json_path = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/tokenizer.json");
    let data = std::fs::read(json_path).expect("could not read fixture");

    let hf = HfTokenizer::from_json(&data).expect("parse failed");
    println!("Model type : {}", hf.model_type);
    println!("Vocab size : {}", hf.vocab.len());
    println!("Merge rules: {}", hf.merges.len());
    println!("Special toks: {}", hf.special_tokens.len());

    // 2. Build a LegacyVocab from the HF vocab map.
    let mut vocab = LegacyVocab::new(*hf.vocab.get("<unk>").unwrap_or(&u32::MAX));
    for (token, &id) in &hf.vocab {
        vocab.insert(token.as_bytes(), id);
    }
    println!("\nLegacyVocab entries: {}", vocab.len());

    // 3. Test a round-trip lookup.
    let bridge = LegacyBridge::new(&vocab);
    let ids = bridge.unroll_bytes(b"hello world");
    println!("Token IDs for b\"hello world\": {ids:?}");

    // 4. Decode back.
    let decoded = bridge.decode_ids(&ids);
    println!(
        "Decoded: {:?}",
        std::str::from_utf8(&decoded).unwrap_or("<binary>")
    );
}

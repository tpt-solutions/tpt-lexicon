//! Full pipeline: ingest → IR → compress → verify → translate.
//!
//! Run with:
//!   cargo run --example pipeline
use tpt_lexicon_ingest::Parser;
use tpt_lexicon_ir::{compress, decompress, IrForest, IrNode};
use tpt_lexicon_translate::{LegacyBridge, LegacyVocab};
use tpt_lexicon_verify::verify_with_rules;

fn main() {
    // 1. Ingest: parse a mixed document into semantic chunks.
    let source = b"# Introduction\n\nHello world.\n\n```rust\nfn main() {}\n```\n";
    let chunks: Vec<_> = Parser::new(source).collect();
    println!("Parsed {} chunks:", chunks.len());
    for chunk in &chunks {
        let text = std::str::from_utf8(chunk.bytes).unwrap_or("<binary>");
        println!("  [{:?}] {text:?}", chunk.kind);
    }

    // 2. Build an IR forest from the chunks.
    let nodes: Vec<IrNode<'_>> = chunks.iter().map(|c| IrNode::text(c.bytes)).collect();
    let forest = IrForest::from_nodes(nodes);
    println!("\nIR forest: {} nodes", forest.node_count());

    // 3. Compress repeated patterns.
    let (compressed_forest, rules) = compress(&forest, 10);
    println!(
        "After compression: {} nodes, {} rules",
        compressed_forest.node_count(),
        rules.len()
    );

    // 4. Verify structural integrity (including rule-index bounds and arity).
    match verify_with_rules(&compressed_forest, &rules) {
        Ok(report) => println!("Verification: passed ({} checks)", {
            let _ = report;
            "all"
        }),
        Err(e) => {
            eprintln!("Verification failed: {e}");
            std::process::exit(1);
        }
    }

    // 5. Decompress and translate to legacy token IDs.
    let decompressed = decompress(&compressed_forest, &rules);
    let mut vocab = LegacyVocab::new(0);
    vocab.insert(b"Hello world.", 1);
    vocab.insert(b"fn main() {}", 2);
    let bridge = LegacyBridge::new(&vocab);
    let ids = bridge.unroll_forest(&decompressed);
    println!("Legacy token IDs: {ids:?}");
}

// TPT Lexicon starter template.
// Includes the recommended crate combination: ingest → ir → verify → translate.

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // 1. Ingest: stream-parse input into semantic chunks
    let input = br#"# Hello

Some text.

```rust
fn hello() -> i32 { 42 }
```

{"key": "value"}
"#;

    let chunks: Vec<_> = tpt_lexicon_ingest::Parser::new(input).collect();
    println!("Parsed {} chunks:", chunks.len());
    for chunk in &chunks {
        println!("  [{:?}] {} bytes at offset {}",
            chunk.kind, chunk.len(), chunk.offset);
    }

    // 2. IR: build an IR forest from the chunks
    let ir_nodes: Vec<_> = chunks.iter().map(|c| {
        match c.kind {
            tpt_lexicon_ingest::ChunkKind::Code => {
                tpt_lexicon_ir::IrNode::code(c.bytes)
            }
            tpt_lexicon_ingest::ChunkKind::Json => {
                tpt_lexicon_ir::IrNode::structured(c.bytes)
            }
            _ => tpt_lexicon_ir::IrNode::text(c.bytes),
        }
    }).collect();
    let forest = tpt_lexicon_ir::IrForest::from_nodes(ir_nodes);
    println!("\nIR forest: {} nodes", forest.node_count());

    // 3. Verify: check structural invariants
    tpt_lexicon_verify::verify_ir(&forest)?;
    println!("IR verification: passed");

    // 4. Translate: use the legacy bridge to convert text to token IDs
    let mut vocab = tpt_lexicon_translate::LegacyVocab::new(0);
    vocab.insert(b"hello", 1);
    vocab.insert(b"world", 2);
    let bridge = tpt_lexicon_translate::LegacyBridge::new(&vocab);
    let ids = bridge.unroll_bytes(b"hello world");
    println!("Token IDs: {:?}", ids);

    // 5. Serialize/deserialize the IR
    let bytes = forest.to_bytes();
    let restored = tpt_lexicon_ir::IrForest::from_bytes(&bytes)?;
    tpt_lexicon_verify::verify_ir(&restored)?;
    println!("IR round-trip verified ({} bytes)", bytes.len());

    Ok(())
}

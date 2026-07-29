//! Cross-crate integration test: ingest → IR → compress → verify → translate.
//!
//! Exercises the full pipeline in a single test to catch cross-crate breakage.

use tpt_lexicon_ingest::{ChunkKind, Parser};
use tpt_lexicon_ir::{compress, decompress, IrForest, IrNode};
use tpt_lexicon_translate::{LegacyBridge, LegacyVocab};
use tpt_lexicon_verify::verify_ir;

/// Build a minimal `LegacyVocab` from a byte slice (one token per byte).
fn byte_vocab(corpus: &[u8]) -> LegacyVocab {
    let mut vocab = LegacyVocab::new(0);
    for (i, &b) in corpus.iter().enumerate() {
        vocab.insert(&[b], i as u32 + 1);
    }
    vocab
}

#[test]
fn ingest_then_ir_verify_translate() {
    // Use fenced code blocks so the ingest parser recognises Code chunks.
    let source = b"```\nreturn 42;\n```\n```\nreturn 42;\n```\n";

    // 1. Ingest: parse source into chunks.
    let parser = Parser::new(source);
    let chunks: Vec<_> = parser.collect();
    assert!(!chunks.is_empty(), "parser must yield at least one chunk");
    assert!(
        chunks.iter().any(|c| c.kind == ChunkKind::Code),
        "source must contain at least one Code chunk"
    );

    // 2. Build IR forest from chunk bytes.
    let nodes: Vec<IrNode<'_>> = chunks
        .iter()
        .map(|c| match c.kind {
            ChunkKind::Code => IrNode::code(c.bytes),
            ChunkKind::Markdown => IrNode::text(c.bytes),
            _ => IrNode::text(c.bytes),
        })
        .collect();
    let forest = IrForest::from_nodes(nodes);
    assert!(!forest.is_empty());

    // 3. Compress: find repeated patterns.
    let (compressed, rules) = compress(&forest, 20);

    // 4. Verify the compressed forest is structurally valid.
    let report = verify_ir(&compressed).expect("compressed IR must be valid");
    assert!(report.passed, "verification must pass on well-formed IR");

    // 5. Decompress and check round-trip fidelity.
    let decompressed = decompress(&compressed, &rules);
    let original_bytes: Vec<&[u8]> = forest.nodes().iter().filter_map(|n| n.as_bytes()).collect();
    let roundtrip_bytes: Vec<&[u8]> = decompressed
        .nodes()
        .iter()
        .filter_map(|n| n.as_bytes())
        .collect();
    assert_eq!(
        original_bytes, roundtrip_bytes,
        "compress + decompress must be lossless"
    );

    // 6. Translate: unroll bytes to legacy token IDs and decode back.
    let unique_bytes: Vec<u8> = {
        let mut seen = std::collections::HashSet::new();
        b"return 42;\n "
            .iter()
            .copied()
            .filter(|b| seen.insert(*b))
            .collect()
    };
    let vocab = byte_vocab(&unique_bytes);
    let bridge = LegacyBridge::new(&vocab);

    for node in decompressed.nodes() {
        if let Some(bytes) = node.as_bytes() {
            let ids = bridge.unroll_bytes(bytes);
            let decoded = bridge.decode_ids(&ids);
            assert_eq!(
                decoded, bytes,
                "unroll → decode must be lossless for byte-level vocab"
            );
        }
    }
}

#[test]
fn empty_forest_passes_verify() {
    let forest = IrForest::new();
    let report = verify_ir(&forest).expect("empty forest must pass verify");
    assert!(report.passed);
}

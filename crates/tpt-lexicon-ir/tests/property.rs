use proptest::prelude::*;
use tpt_lexicon_ir::{compress, decompress, IrForest, IrNode};

/// Collect all byte payloads from byte-carrying nodes in a forest.
fn collect_bytes(forest: &IrForest<'_>) -> Vec<Vec<u8>> {
    forest
        .nodes()
        .iter()
        .filter_map(|n| n.as_bytes().map(|b| b.to_vec()))
        .collect()
}

proptest! {
    /// compress → decompress round-trip must be lossless for any byte payload.
    ///
    /// Creates a forest with two identical Code nodes containing the generated
    /// bytes so the pattern appears ≥2 times (required for compression to create
    /// a rule). Checks that the decompressed bytes equal the original bytes.
    #[test]
    fn compress_decompress_roundtrip(
        payload in proptest::collection::vec(any::<u8>(), 4..=8)
    ) {
        // Two identical nodes so the pattern has frequency ≥ 2 and gets a rule.
        let forest = IrForest::from_nodes(vec![
            IrNode::code(&payload),
            IrNode::code(&payload),
        ]);

        let original_bytes = collect_bytes(&forest);

        let (compressed, rules) = compress(&forest, 10);
        let decompressed = decompress(&compressed, &rules);

        let decompressed_bytes = collect_bytes(&decompressed);

        prop_assert_eq!(
            original_bytes, decompressed_bytes,
            "compress → decompress must be lossless"
        );
    }

    /// Forests with only non-compressible nodes (no repetition) pass through unchanged.
    #[test]
    fn compress_non_repeated_is_identity(
        a in proptest::collection::vec(any::<u8>(), 4..=8),
        b in proptest::collection::vec(any::<u8>(), 4..=8),
    ) {
        // Use different suffixes to ensure the two payloads differ.
        let mut payload_a = a;
        let mut payload_b = b;
        payload_a.push(0xAA);
        payload_b.push(0xBB);

        let forest = IrForest::from_nodes(vec![
            IrNode::code(&payload_a),
            IrNode::code(&payload_b),
        ]);

        let original_bytes = collect_bytes(&forest);

        let (compressed, rules) = compress(&forest, 10);
        let decompressed = decompress(&compressed, &rules);
        let decompressed_bytes = collect_bytes(&decompressed);

        prop_assert_eq!(
            original_bytes, decompressed_bytes,
            "non-repeated patterns must survive compress+decompress unchanged"
        );
    }
}

#[test]
fn compress_decompress_exact_window_boundary() {
    // 4-byte payload hits the window-size lower bound exactly.
    let payload = b"abcd";
    let forest = IrForest::from_nodes(vec![
        IrNode::code(payload),
        IrNode::code(payload),
        IrNode::code(b"efgh"),
    ]);

    let original_bytes = collect_bytes(&forest);
    let (compressed, rules) = compress(&forest, 10);
    let decompressed = decompress(&compressed, &rules);
    assert_eq!(collect_bytes(&decompressed), original_bytes);
    // The repeated pattern should have produced at least one rule.
    assert!(!rules.is_empty(), "repeated 4-byte pattern must produce a rule");
}

#[test]
fn exposes_version() {
    assert!(!tpt_lexicon_ingest::VERSION.is_empty());
}

#[test]
fn parse_basic() {
    let input = b"# Title\n\nSome text.\n\n```rust\ncode\n```\n";
    let chunks: Vec<_> = tpt_lexicon_ingest::Parser::new(input).collect();
    assert!(!chunks.is_empty());
}

#[test]
fn exposes_version() {
    assert!(!tpt_lexicon_core::VERSION.is_empty());
}

#[test]
fn core_types_are_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<tpt_lexicon_core::Vocab>();
}

#[test]
fn train_and_tokenize_roundtrip() {
    let corpus = b"the cat sat on the mat the cat";
    let vocab = tpt_lexicon_core::Vocab::train(corpus, 20).unwrap();
    let tok = tpt_lexicon_core::BpeTokenizer::new(&vocab);

    let input = b"the cat";
    let tokens = tok.encode(input);
    assert_eq!(tokens.as_bytes(), input);
}

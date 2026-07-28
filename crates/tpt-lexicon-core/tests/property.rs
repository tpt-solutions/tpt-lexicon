use proptest::prelude::*;
use tpt_lexicon_core::{BpeTokenizer, Vocab};

// A fixed training corpus large enough that Vocab::train always succeeds.
const CORPUS: &[u8] = b"hello world hello universe world hello foo bar baz qux";

proptest! {
    #[test]
    fn encode_roundtrip_arbitrary_bytes(input in proptest::collection::vec(any::<u8>(), 1..256)) {
        let vocab = Vocab::train(CORPUS, 10).expect("train must not fail");
        let tokenizer = BpeTokenizer::new(&vocab);
        let tokens = tokenizer.encode(&input);
        // Zero-copy round-trip: decoded bytes must equal original input.
        assert_eq!(tokens.as_bytes(), input.as_slice());
    }

    #[test]
    fn encode_total_bytes_equals_input_len(
        input in proptest::collection::vec(any::<u8>(), 1..128)
    ) {
        let vocab = Vocab::train(CORPUS, 8).expect("train must not fail");
        let tokenizer = BpeTokenizer::new(&vocab);
        let tokens = tokenizer.encode(&input);
        assert_eq!(tokens.total_bytes(), input.len());
    }
}

#[test]
fn encode_empty_input_roundtrip() {
    let vocab = Vocab::train(CORPUS, 5).expect("train must not fail");
    let tokenizer = BpeTokenizer::new(&vocab);
    let tokens = tokenizer.encode(b"");
    assert_eq!(tokens.as_bytes(), b"");
}

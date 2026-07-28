//! Property-based tests for tpt-lexicon-core.
//!
//! These tests verify critical invariants across a wide range of byte inputs
//! using a deterministic pseudo-random generator (no external test framework
//! required).

#[cfg(test)]
mod properties {
    use std::vec::Vec;
    use tpt_lexicon_core::{BpeTokenizer, Token, Vocab};

    /// Simple xorshift PRNG for deterministic, reproducible test inputs.
    struct Rng {
        state: u64,
    }

    impl Rng {
        fn new(seed: u64) -> Self {
            Self {
                state: seed.wrapping_add(1),
            }
        }

        fn next_u64(&mut self) -> u64 {
            self.state ^= self.state << 13;
            self.state ^= self.state >> 7;
            self.state ^= self.state << 17;
            self.state
        }

        fn next_bytes(&mut self, len: usize) -> Vec<u8> {
            (0..len).map(|_| self.next_u64() as u8).collect()
        }
    }

    // ─── Property 1: encode never panics on any byte input ───

    #[test]
    fn prop_encode_never_panics_empty_vocab() {
        let vocab = Vocab::new();
        let tok = BpeTokenizer::new(&vocab);
        let mut rng = Rng::new(42);

        for _ in 0..200 {
            let len = (rng.next_u64() % 256) as usize;
            let input = rng.next_bytes(len);
            let tokens = tok.encode(&input);
            assert_eq!(tokens.as_bytes(), input.as_slice());
        }
    }

    #[test]
    fn prop_encode_never_panics_trained_vocab() {
        let corpus = b"the quick brown fox jumps over the lazy dog ";
        let vocab = Vocab::train(corpus, 20).unwrap();
        let tok = BpeTokenizer::new(&vocab);
        let mut rng = Rng::new(123);

        for _ in 0..200 {
            let len = (rng.next_u64() % 512) as usize;
            let input = rng.next_bytes(len);
            let tokens = tok.encode(&input);
            assert_eq!(tokens.as_bytes(), input.as_slice());
        }
    }

    // ─── Property 2: encode_indexed produces identical results to encode ───

    #[test]
    fn prop_encode_indexed_matches_encode() {
        let corpus = b"hello world hello world foo bar baz hello";
        let vocab = Vocab::train(corpus, 15).unwrap();
        let index = vocab.merge_index();
        let tok = BpeTokenizer::new(&vocab);
        let mut rng = Rng::new(456);

        for _ in 0..100 {
            let len = (rng.next_u64() % 200) as usize;
            let input = rng.next_bytes(len);
            let tokens_linear = tok.encode(&input);
            let tokens_indexed = tok.encode_indexed(&input, &index);
            assert_eq!(
                tokens_linear.len(),
                tokens_indexed.len(),
                "length mismatch for input of {} bytes",
                len
            );
            assert_eq!(
                tokens_linear.as_bytes(),
                tokens_indexed.as_bytes(),
                "content mismatch for input of {} bytes",
                len
            );
        }
    }

    // ─── Property 3: token count is always ≤ input length ───

    #[test]
    fn prop_token_count_bounded() {
        let corpus = b"abcdefghij abcdefghij abcdefghij";
        let vocab = Vocab::train(corpus, 10).unwrap();
        let tok = BpeTokenizer::new(&vocab);
        let mut rng = Rng::new(789);

        for _ in 0..100 {
            let len = (rng.next_u64() % 300) as usize;
            let input = rng.next_bytes(len);
            let tokens = tok.encode(&input);
            assert!(
                tokens.len() <= input.len(),
                "token count {} exceeds input length {}",
                tokens.len(),
                input.len()
            );
        }
    }

    // ─── Property 4: every token is a non-empty sub-slice of input ───

    #[test]
    fn prop_tokens_are_valid_sub_slices() {
        let corpus = b"test data test data test data";
        let vocab = Vocab::train(corpus, 8).unwrap();
        let tok = BpeTokenizer::new(&vocab);
        let mut rng = Rng::new(321);

        for _ in 0..100 {
            let len = (rng.next_u64() % 100 + 1) as usize;
            let input = rng.next_bytes(len);
            let tokens = tok.encode(&input);

            for token in tokens.iter() {
                assert!(!token.is_empty(), "token must not be empty");
                assert_eq!(token.len(), token.as_bytes().len());
                let ptr = token.as_bytes().as_ptr() as usize;
                let input_start = input.as_ptr() as usize;
                let input_end = input_start + input.len();
                assert!(
                    ptr >= input_start && ptr + token.len() <= input_end,
                    "token not within input bounds"
                );
            }
        }
    }

    // ─── Property 5: total_bytes of tokens == input length ───

    #[test]
    fn prop_total_bytes_equals_input_length() {
        let corpus = b"abcabc abcabc";
        let vocab = Vocab::train(corpus, 5).unwrap();
        let tok = BpeTokenizer::new(&vocab);
        let mut rng = Rng::new(654);

        for _ in 0..100 {
            let len = (rng.next_u64() % 200) as usize;
            let input = rng.next_bytes(len);
            let tokens = tok.encode(&input);
            assert_eq!(
                tokens.total_bytes(),
                input.len(),
                "total_bytes mismatch for {} bytes",
                len
            );
        }
    }

    // ─── Property 6: vocab serialization round-trip preserves merges ───

    #[test]
    fn prop_vocab_roundtrip_preserves_merges() {
        let mut rng = Rng::new(999);

        for seed in 0..20 {
            let corpus_len = (rng.next_u64() % 100 + 10) as usize;
            let corpus = rng.next_bytes(corpus_len);
            let num_merges = (rng.next_u64() % 5 + 1) as usize;

            if let Ok(vocab) = Vocab::train(&corpus, num_merges) {
                let serialized = vocab.to_bytes();
                let deserialized = Vocab::from_bytes(&serialized).unwrap();
                assert_eq!(
                    vocab.merge_count(),
                    deserialized.merge_count(),
                    "merge count mismatch at seed {seed}"
                );
                let tok1 = BpeTokenizer::new(&vocab);
                let tok2 = BpeTokenizer::new(&deserialized);
                let input = rng.next_bytes(50);
                let t1 = tok1.encode(&input);
                let t2 = tok2.encode(&input);
                assert_eq!(
                    t1.len(),
                    t2.len(),
                    "token count mismatch at seed {seed}"
                );
            }
        }
    }

    // ─── Property 7: MergeIndex lookups match linear scan ───

    #[test]
    fn prop_merge_index_matches_linear() {
        let mut rng = Rng::new(111);

        for seed in 0..20 {
            let corpus_len = (rng.next_u64() % 80 + 10) as usize;
            let corpus = rng.next_bytes(corpus_len);
            let num_merges = (rng.next_u64() % 8 + 1) as usize;

            if let Ok(vocab) = Vocab::train(&corpus, num_merges) {
                let index = vocab.merge_index();

                for _ in 0..20 {
                    let a = rng.next_bytes(3);
                    let b = rng.next_bytes(3);
                    let linear_result = vocab.find_best_pair(&[&a, &b]);
                    let indexed_result =
                        tpt_lexicon_core::find_best_pair_indexed(&[&a, &b], &index);

                    match (linear_result, indexed_result) {
                        (Some((_, r1)), Some((_, r2))) => {
                            assert_eq!(r1, r2, "rank mismatch at seed {seed}")
                        }
                        (None, None) => {}
                        _ => panic!(
                            "result mismatch at seed {seed}: linear={linear_result:?}, indexed={indexed_result:?}"
                        ),
                    }
                }
            }
        }
    }

    // ─── Property 8: empty input always produces zero tokens ───

    #[test]
    fn prop_empty_input_always_zero_tokens() {
        let corpus = b"hello world";
        let vocab = Vocab::train(corpus, 5).unwrap();
        let tok = BpeTokenizer::new(&vocab);
        let tokens = tok.encode(b"");
        assert_eq!(tokens.len(), 0);
        assert_eq!(tokens.total_bytes(), 0);
        assert!(tokens.is_empty());
    }

    // ─── Property 9: single-byte input always produces one token ───

    #[test]
    fn prop_single_byte_input_one_token() {
        let corpus = b"abcdef abcdef";
        let vocab = Vocab::train(corpus, 5).unwrap();
        let tok = BpeTokenizer::new(&vocab);
        let mut rng = Rng::new(222);

        for _ in 0..50 {
            let byte = rng.next_u64() as u8;
            let input = [byte];
            let tokens = tok.encode(&input);
            assert_eq!(tokens.len(), 1, "single byte should produce one token");
            assert_eq!(tokens.get(0).as_bytes(), &[byte]);
        }
    }

    // ─── Property 10: Token::as_str is None iff bytes are not valid UTF-8 ───

    #[test]
    fn prop_as_str_consistency() {
        let mut rng = Rng::new(333);

        for _ in 0..200 {
            let len = (rng.next_u64() % 20 + 1) as usize;
            let bytes = rng.next_bytes(len);
            let token = Token::new(&bytes);

            match core::str::from_utf8(&bytes) {
                Ok(s) => assert_eq!(token.as_str(), Some(s)),
                Err(_) => assert!(token.as_str().is_none()),
            }
        }
    }

    // ─── Property 11: encode on all-zeros input never panics ───

    #[test]
    fn prop_all_zeros_never_panics() {
        let vocab = Vocab::train(b"ab ab ab", 3).unwrap();
        let tok = BpeTokenizer::new(&vocab);
        let input = vec![0u8; 512];
        let tokens = tok.encode(&input);
        assert_eq!(tokens.total_bytes(), 512);
    }

    // ─── Property 12: encode on all-0xFF input never panics ───

    #[test]
    fn prop_all_0xff_never_panics() {
        let vocab = Vocab::train(b"ab ab ab", 3).unwrap();
        let tok = BpeTokenizer::new(&vocab);
        let input = vec![0xFFu8; 256];
        let tokens = tok.encode(&input);
        assert_eq!(tokens.total_bytes(), 256);
    }
}

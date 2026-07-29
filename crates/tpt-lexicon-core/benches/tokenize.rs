use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tpt_lexicon_core::{BpeTokenizer, Vocab};

fn large_corpus() -> Vec<u8> {
    let words = [
        "the ", "quick ", "brown ", "fox ", "jumps ", "over ", "lazy ", "dog ", "hello ", "world ",
        "foo ", "bar ", "baz ", "token ", "izer ",
    ];
    let mut corpus = Vec::with_capacity(100_000);
    for i in 0..10_000 {
        corpus.extend_from_slice(words[i % words.len()].as_bytes());
    }
    corpus
}

fn large_input() -> Vec<u8> {
    let words = [
        "the quick brown fox jumps over the lazy dog ",
        "hello world foo bar baz tokenizer ",
        "the fox jumps over the quick brown dog ",
        "tokenizer hello world baz bar foo ",
    ];
    let mut input = Vec::with_capacity(10_000);
    for i in 0..500 {
        input.extend_from_slice(words[i % words.len()].as_bytes());
    }
    input
}

fn bench_encode_small(c: &mut Criterion) {
    let corpus = large_corpus();
    let vocab = Vocab::train(&corpus, 100).unwrap();
    let tok = BpeTokenizer::new(&vocab);
    let input = b"hello world foo bar baz";

    c.bench_function("encode/small", |b| b.iter(|| tok.encode(black_box(input))));
}

fn bench_encode_large(c: &mut Criterion) {
    let corpus = large_corpus();
    let vocab = Vocab::train(&corpus, 100).unwrap();
    let tok = BpeTokenizer::new(&vocab);
    let input = large_input();

    c.bench_function("encode/large", |b| b.iter(|| tok.encode(black_box(&input))));
}

fn bench_encode_indexed_small(c: &mut Criterion) {
    let corpus = large_corpus();
    let vocab = Vocab::train(&corpus, 100).unwrap();
    let index = vocab.merge_index();
    let tok = BpeTokenizer::new(&vocab);
    let input = b"hello world foo bar baz";

    c.bench_function("encode_indexed/small", |b| {
        b.iter(|| tok.encode_indexed(black_box(input), &index))
    });
}

fn bench_encode_indexed_large(c: &mut Criterion) {
    let corpus = large_corpus();
    let vocab = Vocab::train(&corpus, 100).unwrap();
    let index = vocab.merge_index();
    let tok = BpeTokenizer::new(&vocab);
    let input = large_input();

    c.bench_function("encode_indexed/large", |b| {
        b.iter(|| tok.encode_indexed(black_box(&input), &index))
    });
}

fn bench_vocab_train_small(c: &mut Criterion) {
    let corpus = large_corpus();

    c.bench_function("vocab_train/100_merges", |b| {
        b.iter(|| Vocab::train(black_box(&corpus), black_box(100)).unwrap())
    });
}

fn bench_vocab_train_medium(c: &mut Criterion) {
    let corpus = large_corpus();

    c.bench_function("vocab_train/500_merges", |b| {
        b.iter(|| Vocab::train(black_box(&corpus), black_box(500)).unwrap())
    });
}

fn bench_vocab_serialize_roundtrip(c: &mut Criterion) {
    let corpus = large_corpus();
    let vocab = Vocab::train(&corpus, 100).unwrap();
    let bytes = vocab.to_bytes();

    c.bench_function("vocab_serialize/to_bytes", |b| b.iter(|| vocab.to_bytes()));

    c.bench_function("vocab_serialize/from_bytes", |b| {
        b.iter(|| Vocab::from_bytes(black_box(&bytes)).unwrap())
    });
}

fn bench_merge_index_build(c: &mut Criterion) {
    let corpus = large_corpus();
    let vocab = Vocab::train(&corpus, 500).unwrap();

    c.bench_function("merge_index/build", |b| b.iter(|| vocab.merge_index()));
}

criterion_group!(
    benches,
    bench_encode_small,
    bench_encode_large,
    bench_encode_indexed_small,
    bench_encode_indexed_large,
    bench_vocab_train_small,
    bench_vocab_train_medium,
    bench_vocab_serialize_roundtrip,
    bench_merge_index_build,
);
criterion_main!(benches);

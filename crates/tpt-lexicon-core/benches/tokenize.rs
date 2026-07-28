use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tpt_lexicon_core::{BpeTokenizer, Vocab};

fn bench_encode(c: &mut Criterion) {
    let corpus = b"hello world hello world foo bar baz foo bar baz hello";
    let vocab = Vocab::train(corpus, 20).unwrap();
    let tok = BpeTokenizer::new(&vocab);
    let input = b"hello world foo bar baz";

    c.bench_function("encode_small", |b| {
        b.iter(|| tok.encode(black_box(input)))
    });
}

fn bench_train(c: &mut Criterion) {
    let corpus = b"hello world hello world foo bar baz foo bar baz hello universe";

    c.bench_function("vocab_train_small", |b| {
        b.iter(|| Vocab::train(black_box(corpus), black_box(20)).unwrap())
    });
}

criterion_group!(benches, bench_encode, bench_train);
criterion_main!(benches);

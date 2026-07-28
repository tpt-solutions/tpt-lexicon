# tpt-lexicon-core

Zero-copy BPE tokenizer engine for [TPT Lexicon](https://github.com/tpt-solutions/tpt-lexicon).

`no_std`, zero-copy UTF-8 slicing, sequential BPE tokenization, and vocabulary mapping. This crate has no dependency on the rest of the workspace and is safe to use standalone.

## Features

- **Zero-copy tokenization** — `Token<'a>` borrows directly from the input slice; no allocation for the hot path
- **`TokenSet<'a>`** — stores both the input buffer and the token list, enabling safe `as_bytes()` decode without re-joining
- **Vocabulary training** — `Vocab::train()` performs greedy pair-counting merge to build a vocabulary from raw corpus bytes
- **Binary serialization** — `Vocab` round-trips through a compact binary format (`to_bytes()` / `from_bytes()`)
- **`no_std`** — only needs `alloc`; safe for embedded and kernel contexts

## Usage

```rust
use tpt_lexicon_core::{BpeTokenizer, Vocab};

let corpus = b"hello world hello world hello";
let vocab = Vocab::train(corpus, 20).unwrap();

let tok = BpeTokenizer::new(&vocab);
let tokens = tok.encode(b"hello world");
let decoded = tokens.as_bytes(); // zero-copy decode
assert_eq!(decoded, b"hello world");
```

## Status

**pre-alpha** — see the workspace [`todo.md`](https://github.com/tpt-solutions/tpt-lexicon/blob/master/todo.md)
for implementation progress.

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE)
or [MIT license](../../LICENSE-MIT) at your option.

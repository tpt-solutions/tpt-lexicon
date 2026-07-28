# tpt-lexicon-translate

Translation bridge: JIT-unrolls IR into legacy BPE token IDs and cross-translates to external IRs.

Part of [TPT Lexicon](https://github.com/tpt-solutions/tpt-lexicon).

## Features

- **`LegacyBridge`** — greedy longest-match unrolling of `IrForest` nodes into legacy token-ID sequences
- **`LegacyVocab`** — maps token IDs to byte slices and back, supporting any BPE vocabulary
- **`HfTokenizer`** — minimal `tokenizer.json` parser for loading HuggingFace BPE/WordPiece vocabularies (no `serde` required)
- **Cross-IR exports** — `to_tree_sitter_ast()`, `to_lsp_document()`, `to_llvm_ir()` for interoperability with external toolchains
- **`no_std`** — only needs `alloc`

## Usage

```rust
use tpt_lexicon_translate::{LegacyBridge, LegacyVocab};
use tpt_lexicon_ir::{IrForest, IrNode};

let mut vocab = LegacyVocab::new(0); // 0 = unknown-token ID
vocab.insert(b"hello", 1);
vocab.insert(b" ", 2);
vocab.insert(b"world", 3);

let bridge = LegacyBridge::new(&vocab);

// Unroll raw bytes into token IDs.
let ids = bridge.unroll_bytes(b"hello world");
assert_eq!(ids, vec![1, 2, 3]);

// Or unroll an entire IR forest (Text/Code/Structured nodes only).
let forest = IrForest::from_nodes(vec![IrNode::text(b"hello world")]);
let ids = bridge.unroll_forest(&forest);
assert_eq!(ids, vec![1, 2, 3]);
```

## Status

**pre-alpha** — see the workspace [`todo.md`](https://github.com/tpt-solutions/tpt-lexicon/blob/master/todo.md)
for implementation progress.

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE)
or [MIT license](../../LICENSE-MIT) at your option.

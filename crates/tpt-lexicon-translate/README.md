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
use tpt_lexicon_ir::{IrNode, IrForest};

let mut vocab = LegacyVocab::new(0);
vocab.insert(0, b"hello");
vocab.insert(1, b" ");

let bridge = LegacyBridge::new(&vocab);
let forest = IrForest::from_nodes(vec![IrNode::text(b"hello world")]);
let ids = bridge.unroll(&forest);
```

## Status

**pre-alpha** — see the workspace [`todo.md`](https://github.com/tpt-solutions/tpt-lexicon/blob/master/todo.md)
for implementation progress.

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE)
or [MIT license](../../LICENSE-MIT) at your option.

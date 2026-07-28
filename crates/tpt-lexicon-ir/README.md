# tpt-lexicon-ir

Symbolic intermediate representation with fractal compression for structured data.

Part of [TPT Lexicon](https://github.com/tpt-solutions/tpt-lexicon).

## Features

- **`IrNode` enum** — `Text`, `Code`, `Structured`, `List`, `Reference`, `Compressed` node types for heterogeneous IR
- **`IrForest`** — ordered forest with `from_nodes()`, `node_count()`, `get()`, and iteration
- **Binary serialization** — `to_bytes()` / `from_bytes()` with magic header and version byte
- **Fractal compression** — `compress()` detects repeated byte patterns and collapses them into `Compressed` nodes; `decompress()` expands them back losslessly
- **`CompressRule`** — user-configurable pattern→macro mapping for compression
- **`no_std`** — only needs `alloc`

## Usage

```rust
use tpt_lexicon_ir::{IrNode, IrForest, compress, decompress};

let forest = IrForest::from_nodes(vec![
    IrNode::code(b"function foo() { return 1; }"),
    IrNode::code(b"function foo() { return 1; }"),
    IrNode::code(b"function foo() { return 1; }"),
]);

// Compress detects the repeated pattern and generates rules automatically.
let (compressed, rules) = compress(&forest, 5);
assert!(!rules.is_empty());

// Decompress expands rules back to original byte content.
let decompressed = decompress(&compressed, &rules);
assert_eq!(decompressed.node_count(), forest.node_count());
```

## Status

**pre-alpha** — see the workspace [`todo.md`](https://github.com/tpt-solutions/tpt-lexicon/blob/master/todo.md)
for implementation progress.

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE)
or [MIT license](../../LICENSE-MIT) at your option.

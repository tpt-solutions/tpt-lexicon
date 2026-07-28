# tpt-lexicon-verify

Formal verification of TPT Lexicon IR edits and outputs against structural invariants.

Part of [TPT Lexicon](https://github.com/tpt-solutions/tpt-lexicon).

## Features

- **`Verify` trait** — implemented on `IrForest`; call `.verify()` to get a `VerificationReport`
- **Balanced delimiters** — checks that `()`, `[]`, `{}`, and `<>` are properly nested and closed in text nodes
- **Valid references** — confirms that `Reference` and `Compressed` nodes point to existing indices
- **Cycle detection** — traverses the reference graph and rejects forests with cycles
- **Standalone `verify_ir()`** — function-based verification for external callers
- **`no_std`** — no allocator requirements beyond what `IrForest` itself needs

## Usage

```rust
use tpt_lexicon_verify::Verify;
use tpt_lexicon_ir::{IrNode, IrForest};

let forest = IrForest::from_nodes(vec![
    IrNode::text(b"hello"),
    IrNode::Reference(0),
]);
let report = forest.verify();
assert!(report.passed);
```

## Status

**pre-alpha** — see the workspace [`todo.md`](https://github.com/tpt-solutions/tpt-lexicon/blob/master/todo.md)
for implementation progress.

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE)
or [MIT license](../../LICENSE-MIT) at your option.

# tpt-lexicon-ingest

Streaming, syntax-aware parser yielding semantic chunks without full-string allocation.

Part of [TPT Lexicon](https://github.com/tpt-solutions/tpt-lexicon).

## Features

- **Streaming chunk iterator** — processes input in caller-supplied chunks; never buffers the full document
- **Markdown detection** — headers, fenced code blocks, blockquotes, bullet lists, paragraph breaks
- **Code fence tracking** — balanced open/close fence detection with configurable fence characters
- **JSON boundary detection** — recognizes `{}`, `[]`, and nested JSON structures
- **Paragraph boundary detection** — splits on blank lines for natural-language text
- **Zero-copy** — each `Chunk` borrows directly from the input buffer
- **`no_std`** — only needs `alloc`

## Usage

```rust
use tpt_lexicon_ingest::Parser;

let input = "# Hello\n\nSome text\n\n```rust\nlet x = 1;\n```";
let parser = Parser::new(input.as_bytes());
for chunk in parser {
    println!("{:?}: {:?}", chunk.kind(), chunk.as_str());
}
```

## Status

**pre-alpha** — see the workspace [`todo.md`](https://github.com/tpt-solutions/tpt-lexicon/blob/master/todo.md)
for implementation progress.

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE)
or [MIT license](../../LICENSE-MIT) at your option.

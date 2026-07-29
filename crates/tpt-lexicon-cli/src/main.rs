//! TPT Lexicon CLI — pipeline experimentation tool.
//!
//! Reads input from stdin or a file, runs it through the full pipeline
//! (ingest → IR → verify → translate), and prints results.

use std::error::Error;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::process;

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let input = if args.len() > 1 {
        let path = PathBuf::from(&args[1]);
        fs::read(&path)?
    } else {
        let mut buf = Vec::new();
        std::io::stdin().lock().read_to_end(&mut buf)?;
        buf
    };

    // Phase 1: Ingest
    let chunks: Vec<_> = tpt_lexicon_ingest::Parser::new(&input).collect();
    eprintln!("[ingest] {} chunks parsed", chunks.len());

    // Phase 2: IR
    let ir_nodes: Vec<_> = chunks
        .iter()
        .map(|c| match c.kind {
            tpt_lexicon_ingest::ChunkKind::Code => tpt_lexicon_ir::IrNode::code(c.bytes),
            tpt_lexicon_ingest::ChunkKind::Json => tpt_lexicon_ir::IrNode::structured(c.bytes),
            _ => tpt_lexicon_ir::IrNode::text(c.bytes),
        })
        .collect();
    let forest = tpt_lexicon_ir::IrForest::from_nodes(ir_nodes);
    eprintln!("[ir] {} nodes in forest", forest.node_count());

    // Phase 3: Verify
    tpt_lexicon_verify::verify_ir(&forest)?;
    eprintln!("[verify] structural invariants: PASS");

    // Phase 4: Serialize (for debugging)
    let serialized = forest.to_bytes();
    eprintln!("[ir] serialized: {} bytes", serialized.len());

    // Print chunks to stdout for piping
    for chunk in chunks {
        let label = chunk.kind.to_string();
        println!(
            "[{label}] {}",
            core::str::from_utf8(chunk.bytes).unwrap_or("<binary>")
        );
    }

    Ok(())
}

# TPT Lexicon Starter Template

```bash
# Copy the template
cp -r templates/starter my-project
cd my-project

# Build and run
cargo run
```

This template includes the recommended crate combination for a TPT Lexicon
pipeline: `core` + `ingest` + `ir` + `verify` + `translate`.

For GPU acceleration, uncomment `tpt-lexicon-gpu` in `Cargo.toml`.

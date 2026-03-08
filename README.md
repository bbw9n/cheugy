# Cheugy

Code Archaeology Engine — browse code by meaning.

## First Working Scaffold

This repository contains a working local pipeline:

1. Perl extractors emit JSONL evidence.
2. Rust core normalizes evidence into observations/entities/relations/clusters.
3. Rust CLI drives `init`, `scan`, `build`, `inspect`, `query`, `explore`.
4. Ratatui-based TUI provides a base explorer screen.

## Run

```bash
cargo run -p cheugy -- init
cargo run -p cheugy -- scan .
cargo run -p cheugy -- build .
cargo run -p cheugy -- inspect service api
cargo run -p cheugy -- query "monetization"
cargo run -p cheugy -- explore
```

## Extending Extractors (Perl -> Rust binding)

1. Create `extractors/<name>.pl` that prints evidence JSONL records.
2. Add the script to `extractors/manifest.json`.
3. Add or update an `ObservationAdapter` in `core/src/patterns.rs`.
4. Optionally add inference rules in `core/src/relation_engine.rs`.

The pipeline auto-loads enabled extractors from manifest, so new software system/design pattern probes are plug-in additions.

# Cheugy Architecture

## Flow

Evidence -> Observations -> Entities -> Relations -> Semantic Clusters

## Runtime

1. `cheugy scan <path>`
2. Rust loads `extractors/manifest.json`
3. Rust launches each Perl extractor and parses JSONL evidence
4. `cheugy build` normalizes evidence into typed architecture records
5. `cheugy explore` renders cluster/entity/code panes in TUI

## Extensibility

### Add new software system pattern

1. Add a Perl extractor under `extractors/<name>.pl`
2. Register it in `extractors/manifest.json`
3. Add a Rust `ObservationAdapter` in `core/src/patterns.rs`
4. Optional: add relation inference rules in `core/src/relation_engine.rs`

This keeps extractor evolution independent from the core command pipeline.

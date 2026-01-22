# Claude Code Instructions

See [AGENTS.md](./AGENTS.md) for the wires workflow.

## Project Overview

`wires` is a Rust CLI tool. Structure:

- `src/main.rs` - CLI entry point and argument parsing (clap)
- `src/lib.rs` - Public API exports
- `src/models.rs` - Core types: Wire, WireId, Status, WireError
- `src/db.rs` - SQLite operations
- `src/format.rs` - Output formatting (JSON/table)
- `src/commands/` - Individual command implementations

## Development

```bash
cargo build              # debug build
cargo build --release    # release build
cargo test               # run tests
cargo clippy             # lints
```

## Conventions

- Use newtypes for domain identifiers (WireId, not String)
- Prefer enums over stringly-typed values
- Domain errors go in WireError enum
- Commands return Result<(), WireError>

## Type Safety Philosophy

This project prioritizes compile-time guarantees. If the type system can prevent a bug, it should. See the "Why Rust?" section in README.md.

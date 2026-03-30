---
description: >
  Scaffold a new Rust CLI with Ferrite defaults when starting a project from scratch
  or when replacing a weak initial shell with production-grade output patterns.
user-invocable: true
---

Use this command for new or nearly empty Rust CLI projects. Do not use it for mature crates that already have a stable architecture unless the user explicitly wants a larger rewrite.

If `.ferrite.md` exists, read it first.

## Deliverables

Generate all of the following:

1. `src/output.rs` — full design token and output helper module
2. `Cargo.toml` additions — `clap`, `indicatif`, `dialoguer`, `console`, `anyhow`, `thiserror`, `ctrlc` with appropriate features
3. `src/main.rs` — skeleton with `clap` parsing, error handling, Ctrl+C support, and clean success/error output

## `Cargo.toml` additions

Use these dependencies unless the project already chose compatible equivalents:

```toml
[dependencies]
anyhow = "1"
clap = { version = "4", features = ["derive"] }
console = "0.15"
ctrlc = "3"
dialoguer = "0.11"
indicatif = "0.17"
thiserror = "1"
```

## `src/output.rs` skeleton

Use the normalized helper module from Ferrite's `normalize` command, including `sym_ok()`, `sym_warn()`, `sym_err()`, `sym_info()`, semantic `Tone`, `paint()`, `section()`, `action()`, `detail()`, and `key_value()`.

## `src/main.rs` skeleton

The generated main should include:

- `#[derive(Parser)]` root CLI using `clap`
- a `run(args: Cli) -> anyhow::Result<()>` function
- `CliError` using `thiserror`
- `print_error_with_debug()` and non-zero exit codes
- `ctrlc::set_handler` for interactive commands
- one sample subcommand that demonstrates stdout/stderr separation and a clean summary

## Constraints

- No `todo!()` or placeholder branches
- No bare `unwrap()` in runtime paths
- No decorative banners
- No fake progress metrics

---
description: >
  Harden a Rust CLI's error handling when failures feel improvised, debug output leaks
  to users, or exit behavior is inconsistent across commands.
user-invocable: true
---

Use this command when errors need structural improvement, not just copy edits.

If `.ferrite.md` exists, read it first.

## Goals

Produce a hardened error module and wire it through the CLI.

## Checks

### Error Handling
- [ ] Does `thiserror` model expected user-facing failure modes?
- [ ] Is `anyhow` used for propagation and context layering, not as the primary user message format?
- [ ] Do user-facing errors go to stderr?
- [ ] Do exit codes map to failure intent?
- [ ] Do raw chains only appear behind `--debug`?
- [ ] Does SIGINT exit 130 and restore terminal state?
- [ ] Are unexpected errors (non-user errors) accompanied by a bug report URL?
- [ ] Are repeated errors of the same type grouped under one header?
- [ ] Is the most important error information printed last (bottom of output)?

### Robustness (new — from CLIG)
- [ ] Is all user input validated before any work begins?
- [ ] Are network/external operations protected by configurable timeouts?
- [ ] Is the tool idempotent — safe to re-run after partial completion?
- [ ] Does `--force` exist for overwriting existing state?
- [ ] Does the tool use atomic writes (write to .tmp, then rename)?
- [ ] Is the tool tested when piped: `mytool | cat`?
- [ ] Is the tool tested with `NO_COLOR=1` and `CI=true`?

## Working method

1. Read `Cargo.toml`, `src/main.rs`, and any `error.rs`, `output.rs`, or command modules.
2. Inventory current `Result` types, `?` propagation, `panic!`, `unwrap`, `expect`, and `std::process::exit` usage.
3. Create or replace a hardened error module with:
   - `CliError` enum using `thiserror`
   - `exit_codes` module
   - `error_hint()`
   - `print_error_with_debug()`
4. Route command execution through a single `run()` function and a single `main()` exit path.
5. Install or improve a Ctrl+C handler if the tool is interactive.

## Hardened module shape

Use a structure like this:

```rust
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("could not read configuration file")]
    ConfigRead { path: std::path::PathBuf, source: std::io::Error },
    #[error("invalid command-line argument")]
    InvalidArgument { name: &'static str, value: String },
    #[error("network connection failed")]
    Network { source: std::io::Error },
    #[error("permission denied")]
    Permission { path: std::path::PathBuf, source: std::io::Error },
}

pub mod exit_codes {
    pub const SUCCESS: i32 = 0;
    pub const ERROR: i32 = 1;
    pub const MISUSE: i32 = 2;
    pub const CONFIG: i32 = 3;
    pub const NETWORK: i32 = 4;
    pub const PERMISSION: i32 = 5;
    pub const INTERRUPTED: i32 = 130;
}
```

## Constraints

- Do not leave mixed error styles in different subcommands.
- Do not surface raw `Debug` output to end users.
- Do not exit `0` on failure.
- Do not keep multiple ad-hoc error printers once the hardened path exists.

## Final check

After edits, scan for `println!`, `panic!`, `unwrap`, `expect`, raw `anyhow!`, and direct `exit(...)` calls to ensure the hardened path is actually used.

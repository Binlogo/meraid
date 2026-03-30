# Error Design

Errors are the most important output your CLI produces. They are read under stress.

## Structure every error as WHAT → WHY → HOW

- **WHAT** happened in plain language.
- **WHY** it likely happened, when the cause is knowable.
- **HOW** the user can recover next.

Bad:

```text
Error: invalid input
```

Better:

```text
Error: could not read configuration file
Why: /Users/alex/.config/ferrite/config.toml does not exist
How: run `ferrite init` or pass `--config <path>`
```

## Use `thiserror` for structured CLI failures

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("could not read configuration file")]
    ConfigRead { path: std::path::PathBuf, source: std::io::Error },

    #[error("invalid package name")]
    InvalidPackageName { input: String },

    #[error("network connection failed")]
    Network { url: String, source: std::io::Error },

    #[error("permission denied")]
    Permission { path: std::path::PathBuf, source: std::io::Error },

    #[error("interactive confirmation required")]
    NonInteractive { flag: &'static str },
}
```

`#[error(...)]` should read like the WHAT line. Keep it calm and specific.

## Add context with `anyhow`

```rust
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

fn load_config(path: &Path) -> Result<String> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("while reading {}", path.display()))?;
    Ok(contents)
}
```

Use `with_context()` for operator-facing or debug detail, not as the primary user message.

## Hints belong in code, not in hope

```rust
fn error_hint(err: &CliError) -> Option<String> {
    match err {
        CliError::ConfigRead { path, .. } => Some(format!(
            "Create {} with `ferrite init`, or pass `--config <path>`." ,
            path.display()
        )),
        CliError::InvalidPackageName { .. } => {
            Some("Use lowercase letters, numbers, `_`, or `-` only.".to_string())
        }
        CliError::Network { .. } => Some(
            "Check your network connection, proxy settings, or retry with `--debug` for more detail."
                .to_string(),
        ),
        CliError::Permission { path, .. } => Some(format!(
            "Verify you can write to {} or choose another output directory.",
            path.display()
        )),
        CliError::NonInteractive { flag } => Some(format!(
            "Re-run with {} to confirm explicitly in CI or redirected output.",
            flag
        )),
    }
}
```

## Exit codes should communicate intent

```rust
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

## Print errors to stderr with structure

```rust
use anyhow::Error;
use console::style;

pub fn print_error(err: &Error) {
    eprintln!("{} {}", style("Error:").red().bold(), err);

    let mut current = err.source();
    let mut first = true;
    while let Some(source) = current {
        if first {
            eprintln!("{} {}", style("Why:").yellow().bold(), source);
            first = false;
        } else {
            eprintln!("  caused by: {}", source);
        }
        current = source.source();
    }
}
```

This is a reasonable default formatter for `anyhow::Error`. It gives structure without dumping an unbounded backtrace.

## Gate raw chains behind `--debug`

```rust
use anyhow::Error;
use console::style;

pub fn print_error_with_debug(err: &Error, debug: bool) {
    eprintln!("{} {}", style("Error:").red().bold(), err);

    if debug {
        for (index, cause) in err.chain().skip(1).enumerate() {
            if index == 0 {
                eprintln!("{} {}", style("Why:").yellow().bold(), cause);
            } else {
                eprintln!("  caused by: {}", cause);
            }
        }
    }
}
```

Without `--debug`, keep the chain short. With `--debug`, show the full propagation path.

## Main function pattern

```rust
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("could not read configuration file")]
    ConfigRead { path: PathBuf, source: std::io::Error },
    #[error("invalid package name")]
    InvalidPackageName { input: String },
    #[error("interactive confirmation required")]
    NonInteractive { flag: &'static str },
    #[error("permission denied")]
    Permission { path: PathBuf, source: std::io::Error },
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

fn error_hint(err: &CliError) -> Option<String> {
    match err {
        CliError::ConfigRead { path, .. } => Some(format!(
            "Create {} with `ferrite init`, or pass `--config <path>`.",
            path.display()
        )),
        CliError::InvalidPackageName { .. } => {
            Some("Use lowercase letters, numbers, `_`, or `-` only.".to_string())
        }
        CliError::NonInteractive { flag } => Some(format!(
            "Re-run with {} to confirm explicitly.",
            flag
        )),
        CliError::Permission { path, .. } => Some(format!(
            "Verify you can write to {}.",
            path.display()
        )),
    }
}

fn print_error(err: &anyhow::Error) {
    eprintln!("Error: {}", err);
}

fn print_error_with_debug(err: &anyhow::Error, debug: bool) {
    print_error(err);
    if debug {
        for cause in err.chain().skip(1) {
            eprintln!("  caused by: {}", cause);
        }
    }
}

fn exit_code(err: &(dyn std::error::Error + 'static)) -> i32 {
    if let Some(cli) = err.downcast_ref::<CliError>() {
        match cli {
            CliError::ConfigRead { .. } => exit_codes::CONFIG,
            CliError::InvalidPackageName { .. } => exit_codes::MISUSE,
            CliError::NonInteractive { .. } => exit_codes::MISUSE,
            CliError::Permission { .. } => exit_codes::PERMISSION,
        }
    } else {
        exit_codes::ERROR
    }
}

fn run() -> Result<()> {
    let path = PathBuf::from("missing-config.toml");
    let _ = fs::read_to_string(&path)
        .map_err(|source| CliError::ConfigRead {
            path: path.clone(),
            source,
        })
        .with_context(|| format!("startup failed while opening {}", path.display()))?;
    Ok(())
}

fn main() {
    let debug = std::env::args().any(|arg| arg == "--debug");

    match run() {
        Ok(()) => std::process::exit(exit_codes::SUCCESS),
        Err(err) => {
            if let Some(cli) = err.downcast_ref::<CliError>() {
                eprintln!("Error: {}", cli);
                if let Some(hint) = error_hint(cli) {
                    eprintln!("How: {}", hint);
                }
                if debug {
                    for cause in err.chain().skip(1) {
                        eprintln!("  caused by: {}", cause);
                    }
                }
                std::process::exit(exit_code(cli));
            }

            print_error_with_debug(&err, debug);
            std::process::exit(exit_codes::ERROR);
        }
    }
}
```

Cross-reference:
- Read `ux-writing-cli.md` for message wording.
- Read `interaction-patterns.md` when failures must explain `--yes` or non-interactive behavior.

## Error Placement

> "Put the most important information at the end of the output." — CLIG

In terminal output, the eye is drawn to the *bottom*. For multi-step
operations that end in failure, the error should be the last thing printed:

```rust
// ✗ Bad — error buried under subsequent output
eprintln!("✗ Failed to compile src/lib.rs");
println!("  Checking src/main.rs…");
println!("  Checking src/utils.rs…");

// ✓ Good — error is the final line the user sees
println!("  Checking src/main.rs… ok");
println!("  Checking src/utils.rs… ok");
eprintln!("\n{} Failed to compile src/lib.rs", console::style("✗").red());
eprintln!("  {}", console::style("error[E0308]: mismatched types").dim());
```

## Error Grouping

When multiple errors of the same type occur, group them:

```rust
// ✗ Bad — N identical-looking lines
eprintln!("✗ Missing field: name");
eprintln!("✗ Missing field: email");
eprintln!("✗ Missing field: age");

// ✓ Good — grouped under one header
eprintln!("{} Validation failed — 3 required fields missing:",
    console::style("✗").red());
for field in &missing_fields {
    eprintln!("  {} {}", console::style("→").dim(), field);
}
```

## Bug Report URL

For unexpected errors (not user errors), provide a pre-filled bug report URL:

```rust
fn is_user_error(err: &anyhow::Error) -> bool {
    err.downcast_ref::<CliError>().map_or(false, |e| {
        matches!(e,
            CliError::InvalidInput { .. } |
            CliError::ConfigNotFound { .. } |
            CliError::PortInUse { .. }
        )
    })
}

fn print_error(err: &anyhow::Error, debug: bool) {
    // ... existing WHAT/WHY/HOW structure ...

    if !is_user_error(err) {
        eprintln!(
            "  {} This looks like a bug. Report at: {}",
            console::style("→").cyan(),
            console::style("https://github.com/your-handle/mytool/issues").dim()
        );
    }
}
```

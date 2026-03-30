---
description: >
  Normalize a Rust CLI onto shared output tokens and helpers when formatting,
  symbols, and message styles have drifted across files or subcommands.
user-invocable: true
---

Use this command when a crate has many ad-hoc `println!`, styling calls, or duplicated symbol constants. Do not use it for tiny one-command examples that do not need a shared output module.

If `.ferrite.md` exists, read it first.

## Step 1 — Inventory

- Read `Cargo.toml`, `src/main.rs`, and all CLI entry points.
- Find existing output helpers, symbol constants, and direct `println!` / `eprintln!` calls.
- List the current style vocabulary: symbols, colors, indentation, headings, summaries, warnings, and errors.
- Inventory whether the CLI already exposes `--json` or `--plain` output modes, and where machine-readable output is currently formatted.
- Decide whether stdout and stderr responsibilities are currently mixed.

## Step 2 — Create `src/output.rs`

Create or replace `src/output.rs` with a single design-token module like this. The module should own both human-readable tokens and machine-readable escape hatches so every command uses one output vocabulary:

```rust
use anyhow::Result;
use console::{style, StyledObject, Term};

pub struct Symbols {
    pub ok: &'static str,
    pub warn: &'static str,
    pub err: &'static str,
    pub info: &'static str,
    pub arrow: &'static str,
}

const UNICODE: Symbols = Symbols {
    ok: "✓",
    warn: "▲",
    err: "✗",
    info: "•",
    arrow: "→",
};

const ASCII: Symbols = Symbols {
    ok: "OK",
    warn: "!",
    err: "X",
    info: "i",
    arrow: "->",
};

#[derive(Clone, Copy)]
pub enum Tone {
    Success,
    Warning,
    Error,
    Info,
    Muted,
}

pub fn symbols() -> &'static Symbols {
    if std::env::var_os("FERRITE_ASCII").is_some() {
        &ASCII
    } else {
        &UNICODE
    }
}

pub fn sym_ok() -> &'static str {
    symbols().ok
}

pub fn sym_warn() -> &'static str {
    symbols().warn
}

pub fn sym_err() -> &'static str {
    symbols().err
}

pub fn sym_info() -> &'static str {
    symbols().info
}

pub fn sym_arrow() -> &'static str {
    symbols().arrow
}

pub fn paint(text: impl Into<String>, tone: Tone) -> String {
    let text = text.into();
    if !console::colors_enabled() {
        return text;
    }

    let styled: StyledObject<String> = match tone {
        Tone::Success => style(text).green(),
        Tone::Warning => style(text).yellow(),
        Tone::Error => style(text).red(),
        Tone::Info => style(text).cyan(),
        Tone::Muted => style(text).dim(),
    };

    styled.to_string()
}

pub fn section(term: &Term, title: &str) -> Result<()> {
    term.write_line("")?;
    term.write_line(&style(title).bold().to_string())?;
    Ok(())
}

pub fn action(term: &Term, symbol: &str, tone: Tone, message: &str) -> Result<()> {
    term.write_line(&format!("{} {}", paint(symbol, tone), message))?;
    Ok(())
}

pub fn detail(term: &Term, message: &str) -> Result<()> {
    term.write_line(&format!("  {}", paint(message, Tone::Muted)))?;
    Ok(())
}

pub fn key_value(term: &Term, rows: &[(&str, String)]) -> Result<()> {
    let width = rows.iter().map(|(label, _)| label.len()).max().unwrap_or(0);
    for (label, value) in rows {
        term.write_line(&format!(
            "  {}  {}",
            paint(format!("{:width$}", label, width = width), Tone::Muted),
            value
        ))?;
    }
    Ok(())
}

pub fn success(term: &Term, message: &str) -> Result<()> {
    action(term, sym_ok(), Tone::Success, message)
}

pub fn warning(term: &Term, message: &str) -> Result<()> {
    action(term, sym_warn(), Tone::Warning, message)
}

pub fn error(term: &Term, message: &str) -> Result<()> {
    action(term, sym_err(), Tone::Error, message)
}

pub fn info(term: &Term, message: &str) -> Result<()> {
    action(term, sym_info(), Tone::Info, message)
}

/// Print machine-readable JSON output
pub fn print_json<T: serde::Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

/// Print plain tab-separated output for grep/awk pipelines
pub fn print_plain<S: AsRef<str>>(fields: &[S]) {
    let line = fields
        .iter()
        .map(|field| field.as_ref())
        .collect::<Vec<_>>()
        .join("\t");
    println!("{}", line);
}
```

Use `print_json()` for `--json` output and `print_plain()` for `--plain` output. Neither function should emit color, symbols, or extra prose.


## Step 3 — Systematic replacement

- Replace inline symbol literals with `sym_*()` helpers.
- Replace repeated color calls with `paint()`.
- Replace repeated summary layouts with `section()`, `action()`, `detail()`, and `key_value()`.
- Route `--json` output through `print_json()` and `--plain` output through `print_plain()` instead of ad-hoc formatting.
- Preserve intentional stderr usage for errors and status output.
- Remove dead style helpers after migration.

## Step 4 — Verify

Run targeted searches to prove normalization happened:

```bash
grep -R "println!(\|eprintln!(" src
grep -R "style(" src
grep -R "✓\|▲\|✗\|→" src
grep -R "Color::\|Stylize" src
grep -R "serde_json::to_string_pretty\|join(\"\\t\")" src
```

Expected outcome: direct styling and raw symbol usage should be rare or eliminated outside `src/output.rs`, and machine-readable output should flow through shared helpers instead of bespoke formatting.

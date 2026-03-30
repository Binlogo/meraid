# Color & Symbols

Ferrite treats color as semantic signal. If color does not encode stable meaning, it is noise.

## Semantic color vocabulary

| Meaning | Default color | Typical use |
| --- | --- | --- |
| Success | Green | Completed steps, created resources, pass states |
| Warning | Yellow | Recoverable risk, skipped work, degraded mode |
| Error | Red | Failures, destructive actions, invalid states |
| Info | Cyan | Neutral status, progress labels, helpful hints |
| Muted | Dim | Secondary detail, paths, counts, timestamps |

Use these meanings consistently across the CLI. Do not make success cyan in one command and green in another.

## Symbol sets

Use Unicode by default, but provide an ASCII fallback.

```rust
pub struct Symbols {
    pub ok: &'static str,
    pub warn: &'static str,
    pub err: &'static str,
    pub info: &'static str,
    pub arrow: &'static str,
    pub bullet: &'static str,
}

pub const UNICODE: Symbols = Symbols {
    ok: "✓",
    warn: "▲",
    err: "✗",
    info: "•",
    arrow: "→",
    bullet: "•",
};

pub const ASCII: Symbols = Symbols {
    ok: "OK",
    warn: "!",
    err: "X",
    info: "i",
    arrow: "->",
    bullet: "*",
};
```

## Platform and capability detection

Do not hardcode Unicode everywhere. Use a helper that can fall back when needed.

```rust
use std::env;

pub fn symbols() -> &'static Symbols {
    let ascii_requested = env::var_os("FERRITE_ASCII").is_some()
        || env::var_os("TERM_PROGRAM").as_deref() == Some(std::ffi::OsStr::new("Apple_Terminal"))
            && env::var_os("LC_CTYPE").is_none();

    if ascii_requested {
        &ASCII
    } else {
        &UNICODE
    }
}
```

If the project has better environment knowledge, prefer that. The important part is having one place where fallback policy lives.

## Respect `NO_COLOR`

`console::colors_enabled()` already tracks whether colors should be emitted. Build your styling around it.

```rust
use console::{style, StyledObject};

#[derive(Clone, Copy)]
pub enum Tone {
    Success,
    Warning,
    Error,
    Info,
    Muted,
}

pub fn styled(text: impl Into<String>, tone: Tone) -> String {
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
```

This wrapper keeps styling decisions centralized and automatically respects `NO_COLOR`.

## Recommended output helpers

```rust
use anyhow::Result;
use console::Term;

pub fn print_success(term: &Term, message: &str) -> Result<()> {
    let sym = symbols();
    term.write_line(&format!("{} {}", styled(sym.ok, Tone::Success), message))?;
    Ok(())
}

pub fn print_warning(term: &Term, message: &str) -> Result<()> {
    let sym = symbols();
    term.write_line(&format!("{} {}", styled(sym.warn, Tone::Warning), message))?;
    Ok(())
}

pub fn print_error(term: &Term, message: &str) -> Result<()> {
    let sym = symbols();
    term.write_line(&format!("{} {}", styled(sym.err, Tone::Error), message))?;
    Ok(())
}
```

## What not to color

Do not add semantic color to content that users often copy, compare, or scan literally:

- Paths
- Timestamps
- Shell commands
- JSON or machine-readable output
- Every line of a report

Bad:

```text
/path/to/project/src/main.rs   2026-03-30T10:22:11Z   built successfully
```

If all of it is green, none of it stands out.

Better:

```text
✓ Built project
  /path/to/project/src/main.rs
  2026-03-30T10:22:11Z
```

## Full working example

```rust
use anyhow::Result;
use console::{style, StyledObject, Term};
use std::env;

pub struct Symbols {
    pub ok: &'static str,
    pub warn: &'static str,
    pub err: &'static str,
    pub info: &'static str,
}

pub const UNICODE: Symbols = Symbols {
    ok: "✓",
    warn: "▲",
    err: "✗",
    info: "•",
};

pub const ASCII: Symbols = Symbols {
    ok: "OK",
    warn: "!",
    err: "X",
    info: "i",
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
    if env::var_os("FERRITE_ASCII").is_some() {
        &ASCII
    } else {
        &UNICODE
    }
}

pub fn styled(text: impl Into<String>, tone: Tone) -> String {
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

fn main() -> Result<()> {
    let term = Term::stdout();
    let sym = symbols();

    term.write_line(&format!("{} Project created", styled(sym.ok, Tone::Success)))?;
    term.write_line(&format!("{} Dry run only", styled(sym.warn, Tone::Warning)))?;
    term.write_line(&format!("{} Missing Cargo.toml", styled(sym.err, Tone::Error)))?;
    term.write_line(&format!("{} Try --help for examples", styled(sym.info, Tone::Info)))?;
    term.write_line(&format!("  {}", styled("target/debug/ferrite", Tone::Muted)))?;
    Ok(())
}
```

Cross-reference:
- Read `output-hierarchy.md` for where these styles should appear.
- Read `progress-and-feedback.md` for spinner and progress-bar color usage.

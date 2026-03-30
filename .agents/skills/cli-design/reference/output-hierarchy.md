# Output Hierarchy & Typography

Ferrite uses four output levels. If a line does not fit one of these levels, it probably should not be printed.

| Level | Purpose | Typical styling |
| --- | --- | --- |
| Section header | Marks a new phase or report block | Bold, high-contrast, separated by a blank line |
| Primary action | States what just happened or what is happening | Symbol + short sentence |
| Secondary detail | Adds context, counts, paths, durations, or explanations | 2-space indentation, muted style |
| Debug | Hidden unless `--debug` or verbose logging is enabled | Dim, clearly labeled |

## Rules

- Indent nested detail with exactly 2 spaces. Never use tabs.
- Use blank lines as punctuation between sections.
- Align key-value output when users scan vertically.
- Render tables with `─` (U+2500), not repeated ASCII `-`, when Unicode is available.
- Treat debug output as opt-in. Normal users should never fight through it.

## Pattern: Section headers and primary actions

```rust
use anyhow::Result;
use console::{style, Term};

fn main() -> Result<()> {
    let term = Term::stdout();

    term.write_line("")?;
    term.write_line(&style("Build summary").bold().to_string())?;
    term.write_line(&format!("{} Compiled 12 crates", style("✓").green().bold()))?;
    term.write_line(&format!("  {}", style("target/release/ferrite").dim()))?;

    Ok(())
}
```

Why this works:
- The blank line gives the section room to breathe.
- The header is visually distinct without turning into a banner.
- The path is detail, so it is indented and muted.

## Pattern: Key-value summaries with alignment

Use fixed-width labels when output is summary-like and meant for quick scanning.

```rust
use anyhow::Result;
use console::{style, Term};

fn print_kv(term: &Term, rows: &[(&str, String)]) -> Result<()> {
    let width = rows.iter().map(|(label, _)| label.len()).max().unwrap_or(0);

    for (label, value) in rows {
        term.write_line(&format!(
            "  {}  {}",
            style(format!("{:width$}", label, width = width)).dim(),
            value
        ))?;
    }

    Ok(())
}

fn main() -> Result<()> {
    let term = Term::stdout();
    let rows = [
        ("crate", "ferrite-cli-design".to_string()),
        ("artifacts", "4 providers".to_string()),
        ("elapsed", "1.42s".to_string()),
    ];

    term.write_line(&style("Publish ready").bold().to_string())?;
    print_kv(&term, &rows)?;
    Ok(())
}
```

This uses `format!("{:width$}", ...)` so values form a clean column without manual spacing.

## Pattern: Visual rhythm with blank lines

Bad rhythm packs everything together:

```text
Build summary
✓ Compiled 12 crates
  target/release/ferrite
✓ Wrote 4 provider skill packs
Warnings: 1
```

Better rhythm groups by meaning:

```text
Build summary
✓ Compiled 12 crates
  target/release/ferrite

✓ Wrote 4 provider skill packs
  .claude/, .cursor/, .gemini/, .codex/

Warnings
  1 deprecated example updated
```

Blank lines are punctuation. Use them to separate thoughts, not to add decoration.

## Pattern: Unicode tables

Prefer a light divider instead of noisy box art.

```rust
use anyhow::Result;
use console::{style, Term};

fn print_table(term: &Term, headers: &[&str], rows: &[Vec<&str>]) -> Result<()> {
    let mut widths = headers.iter().map(|header| header.len()).collect::<Vec<_>>();

    for row in rows {
        for (index, cell) in row.iter().enumerate() {
            widths[index] = widths[index].max(cell.len());
        }
    }

    let header = headers
        .iter()
        .enumerate()
        .map(|(index, value)| format!("{:width$}", value, width = widths[index]))
        .collect::<Vec<_>>()
        .join("  ");

    let divider = widths
        .iter()
        .map(|width| "─".repeat(*width))
        .collect::<Vec<_>>()
        .join("  ");

    term.write_line(&style(header).bold().to_string())?;
    term.write_line(&style(divider).dim().to_string())?;

    for row in rows {
        let line = row
            .iter()
            .enumerate()
            .map(|(index, value)| format!("{:width$}", value, width = widths[index]))
            .collect::<Vec<_>>()
            .join("  ");
        term.write_line(&line)?;
    }

    Ok(())
}

fn main() -> Result<()> {
    let term = Term::stdout();
    let rows = vec![
        vec!["claude", "ok", "SKILL.md + 12 commands"],
        vec!["cursor", "ok", "SKILL.md + 12 commands"],
        vec!["gemini", "ok", "SKILL.md + 12 commands"],
        vec!["codex", "ok", "SKILL.md + 12 commands"],
    ];

    print_table(&term, &["provider", "status", "artifacts"], &rows)?;
    Ok(())
}
```

## Full working example

This example demonstrates all four levels together.

```rust
use anyhow::Result;
use console::{style, Term};

fn print_debug(term: &Term, debug: bool, message: &str) -> Result<()> {
    if debug {
        term.write_line(&format!("  {} {}", style("debug").dim(), style(message).dim()))?;
    }
    Ok(())
}

fn print_summary(term: &Term, rows: &[(&str, String)]) -> Result<()> {
    let width = rows.iter().map(|(label, _)| label.len()).max().unwrap_or(0);
    for (label, value) in rows {
        term.write_line(&format!(
            "  {}  {}",
            style(format!("{:width$}", label, width = width)).dim(),
            value
        ))?;
    }
    Ok(())
}

fn main() -> Result<()> {
    let term = Term::stdout();
    let debug = true;

    term.write_line("")?;
    term.write_line(&style("Release build").bold().to_string())?;
    term.write_line(&format!("{} Compiled workspace", style("✓").green().bold()))?;
    term.write_line(&format!("  {}", style("target/release/ferrite").dim()))?;
    print_debug(&term, debug, "cargo build --release exited 0")?;

    term.write_line("")?;
    term.write_line(&style("Summary").bold().to_string())?;
    print_summary(
        &term,
        &[
            ("crates", "12".into()),
            ("warnings", "0".into()),
            ("elapsed", "1.42s".into()),
        ],
    )?;

    Ok(())
}
```

Cross-reference:
- Read `color-and-symbols.md` for semantic styling and symbol fallback.
- Read `ux-writing-cli.md` when changing the wording carried by these output layers.

## Machine-readable Output

Human-readable formatting (colors, multi-line cells, decorative symbols)
breaks scripting. Provide escape hatches.

### `--plain` Flag

Disables all decorative formatting. Output becomes one record per line,
suitable for `grep`, `awk`, and shell scripts:

```rust
fn print_item(name: &str, version: &str, status: &str, plain: bool) {
    if plain {
        // Tab-separated, no color, no symbols — pipe-safe
        println!("{}\t{}\t{}", name, version, status);
    } else {
        // Human-readable with color and symbols
        let sym = if status == "ok" {
            console::style("✓").green().to_string()
        } else {
            console::style("✗").red().to_string()
        };
        println!("{} {:<20} {}", sym, name, console::style(version).dim());
    }
}
```

### `--json` Flag

Structured output for programmatic consumption. Use `serde_json`:

```rust
use serde::Serialize;

#[derive(Serialize)]
struct ItemOutput {
    name: String,
    version: String,
    status: String,
}

fn print_results(items: &[Item], json: bool) -> anyhow::Result<()> {
    if json {
        let output: Vec<ItemOutput> = items.iter()
            .map(|i| ItemOutput {
                name: i.name.clone(),
                version: i.version.clone(),
                status: i.status.to_string(),
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        for item in items {
            print_item_human(item);
        }
    }
    Ok(())
}
```

### Pager for Long Output

When output exceeds one screen, pipe through a pager — but only when stdout
is a TTY:

```rust
fn with_pager(content: &str) -> anyhow::Result<()> {
    if !console::Term::stdout().is_term() {
        // Piped — print directly, no pager
        print!("{}", content);
        return Ok(());
    }

    // Use less with sensible flags:
    // -F  quit if content fits on one screen
    // -I  case-insensitive search
    // -R  pass ANSI color codes through
    // -X  don't clear screen on exit
    let mut pager = std::process::Command::new("less")
        .args(["-FIRX"])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .unwrap_or_else(|_| {
            // less not available — fall back to direct print
            print!("{}", content);
            std::process::exit(0);
        });

    if let Some(stdin) = pager.stdin.take() {
        use std::io::Write;
        let mut stdin = stdin;
        stdin.write_all(content.as_bytes())?;
    }
    pager.wait()?;
    Ok(())
}
```

### State Change Confirmation

When a command modifies system state, always tell the user what changed:

```rust
// ✗ Bad — silent success
fs::remove_file(&path)?;

// ✓ Good — confirm the outcome
fs::remove_file(&path)?;
println!(
    "{} Deleted {}",
    console::style("✓").green(),
    console::style(path.display()).dim()
);

// ✓ Good — for bulk operations, summarize
println!(
    "{} Deleted {} files from {}",
    console::style("✓").green(),
    count,
    console::style(dir.display()).dim()
);
```

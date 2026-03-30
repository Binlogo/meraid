# Progress & Feedback

Users do not want motion. They want confidence.

## When to Show Progress

> "Responsive is more important than fast." — CLIG
> Print *something* within 100ms. Always. Even if the operation is fast.

| Operation duration | Appropriate indicator |
|--------------------|-----------------------|
| Any duration | Print the action name *before* starting work (< 100ms) |
| < 300ms | Action name only — no spinner needed |
| 300ms – 2s | Spinner, no label update needed |
| 2s – 10s | Spinner with descriptive current-step label |
| > 10s | Spinner + elapsed time counter |
| Known total | Progress bar with real percentage |
| Parallel tasks | `MultiProgress` or sequential with clear grouping |

The 100ms rule is absolute. A user who sees nothing for 100ms will assume
the tool is broken. Print intent before work:

```rust
// ✓ Always announce before doing
println!("{} Connecting to {}…",
    console::style("◆").cyan(),
    console::style(&host).dim()
);
let conn = connect(&host).await?; // This might take 50ms or 5s
```

## Spinner helper

Use braille ticks, not ASCII pinwheels.

```rust
use indicatif::{ProgressBar, ProgressStyle};

pub fn make_spinner(msg: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .expect("valid spinner template")
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    spinner.set_message(msg.to_string());
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));
    spinner
}
```

## CI-safe fallback

When terminals are not interactive, skip the spinner and print one plain status line instead.

```rust
use indicatif::ProgressBar;

pub fn make_spinner_or_plain(msg: &str) -> Option<ProgressBar> {
    if std::env::var_os("CI").is_some() || !console::Term::stderr().is_term() {
        eprintln!("{}", msg);
        None
    } else {
        Some(make_spinner(msg))
    }
}
```

## Finish cleanly

Never leave the user with an orphaned spinner frame.

```rust
use indicatif::ProgressBar;

pub fn finish_step(progress: Option<ProgressBar>, message: &str) {
    if let Some(progress) = progress {
        progress.finish_and_clear();
        eprintln!("{}", message);
    } else {
        eprintln!("{}", message);
    }
}
```

## Real progress bars

Only use a progress bar when both position and total are real.

```rust
use indicatif::{ProgressBar, ProgressStyle};

pub fn make_progress_bar(len: u64, prefix: &str) -> ProgressBar {
    let progress = ProgressBar::new(len);
    progress.set_style(
        ProgressStyle::with_template("{prefix:.bold}  [{bar:20}]  {pos}/{len}  {msg}")
            .expect("valid progress template")
            .progress_chars("█░ "),
    );
    progress.set_prefix(prefix.to_string());
    progress
}
```

## Parallel work with `MultiProgress`

```rust
use anyhow::Result;
use indicatif::{MultiProgress, ProgressStyle};

fn main() -> Result<()> {
    let multi = MultiProgress::new();
    let style = ProgressStyle::with_template("{spinner:.cyan} {msg}")?
        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]);

    let fetch = multi.add(indicatif::ProgressBar::new_spinner());
    fetch.set_style(style.clone());
    fetch.set_message("Fetching crates.io index");
    fetch.enable_steady_tick(std::time::Duration::from_millis(80));

    let build = multi.add(indicatif::ProgressBar::new_spinner());
    build.set_style(style);
    build.set_message("Compiling workspace");
    build.enable_steady_tick(std::time::Duration::from_millis(80));

    std::thread::sleep(std::time::Duration::from_millis(250));
    fetch.finish_and_clear();
    build.finish_and_clear();
    Ok(())
}
```

`MultiProgress` is for genuinely parallel tasks. Do not use it to dramatize sequential work.

## Outcome-first summaries

A good summary starts with what happened, then gives timing and warnings.

```rust
use anyhow::Result;
use console::{style, Term};

pub fn print_summary(term: &Term, created: usize, elapsed: std::time::Duration, warnings: &[String]) -> Result<()> {
    term.write_line(&format!(
        "{} Generated {} files in {:.2?}",
        style("✓").green().bold(),
        created,
        elapsed
    ))?;

    if !warnings.is_empty() {
        term.write_line("")?;
        term.write_line(&style("Warnings").yellow().bold().to_string())?;
        for warning in warnings {
            term.write_line(&format!("  {}", warning))?;
        }
    }

    Ok(())
}
```

## Full working example

```rust
use anyhow::Result;
use console::Term;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::time::{Duration, Instant};

fn make_spinner(msg: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .expect("valid spinner template")
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    spinner.set_message(msg.to_string());
    spinner.enable_steady_tick(Duration::from_millis(80));
    spinner
}

fn make_spinner_or_plain(msg: &str) -> Option<ProgressBar> {
    if std::env::var_os("CI").is_some() || !Term::stderr().is_term() {
        eprintln!("{}", msg);
        None
    } else {
        Some(make_spinner(msg))
    }
}

fn finish_step(progress: Option<ProgressBar>, message: &str) {
    if let Some(progress) = progress {
        progress.finish_and_clear();
    }
    eprintln!("{}", message);
}

fn make_progress_bar(len: u64, prefix: &str) -> ProgressBar {
    let progress = ProgressBar::new(len);
    progress.set_style(
        ProgressStyle::with_template("{prefix:.bold}  [{bar:20}]  {pos}/{len}  {msg}")
            .expect("valid progress template")
            .progress_chars("█░ "),
    );
    progress.set_prefix(prefix.to_string());
    progress
}

fn print_summary(term: &Term, elapsed: Duration, warnings: &[String]) -> Result<()> {
    term.write_line(&format!("✓ Built release artifacts in {:.2?}", elapsed))?;
    if !warnings.is_empty() {
        term.write_line("")?;
        term.write_line("Warnings")?;
        for warning in warnings {
            term.write_line(&format!("  {}", warning))?;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let started = Instant::now();
    let prep = make_spinner_or_plain("Resolving dependencies");
    std::thread::sleep(Duration::from_millis(600));
    finish_step(prep, "Resolved dependencies");

    let progress = make_progress_bar(3, "build");
    for crate_name in ["core", "ui", "cli"] {
        progress.inc(1);
        progress.set_message(format!("compiled {}", crate_name));
        std::thread::sleep(Duration::from_millis(150));
    }
    progress.finish_and_clear();

    let multi = MultiProgress::new();
    let upload = multi.add(make_spinner("Uploading release artifacts"));
    std::thread::sleep(Duration::from_millis(250));
    upload.finish_and_clear();

    print_summary(&Term::stdout(), started.elapsed(), &["Built without debug symbols".to_string()])?;
    Ok(())
}
```

Cross-reference:
- Read `interaction-patterns.md` before mixing progress UI with prompts.
- Read `error-design.md` for how failures should interrupt and summarize work.

## Bug Report on Unexpected Errors

When an operation fails in an unexpected way (not a user error), provide a
path to report it:

```rust
fn print_unexpected_error(err: &anyhow::Error) {
    eprintln!(
        "\n{} Unexpected error — this looks like a bug.\n",
        console::style("✗").red()
    );
    eprintln!("  {}", err);
    eprintln!(
        "\n  {} Please report this at: {}",
        console::style("→").cyan(),
        console::style("https://github.com/your-handle/mytool/issues").underlined()
    );
    eprintln!(
        "  {} Run with {} for full details.",
        console::style("→").dim(),
        console::style("--debug").bold()
    );
}
```

# Interaction Patterns

Interactive CLI flows should feel deliberate, fast, and safe. Prompts are not an excuse to stop designing.

## Core rules

- Always use `dialoguer` with `ColorfulTheme`.
- Validate inline with `.validate_with(...)` so users fail early, not after a full prompt sequence.
- Destructive confirmations must default to `false`.
- Add `--yes` or `-y` when a prompt can block automation.
- In CI or redirected terminals, fail fast with an actionable message instead of waiting on stdin.

## Detect whether interaction is allowed

```rust
use console::Term;

pub fn is_interactive() -> bool {
    std::env::var_os("CI").is_none() && Term::stdout().is_term()
}
```

This is intentionally strict: CI and non-terminal stdout both disable interactive behavior.

## Require interaction or an explicit bypass flag

```rust
use anyhow::{bail, Result};

pub fn require_interactive_or_flag(flag_name: &str, allowed: bool) -> Result<()> {
    if allowed || is_interactive() {
        return Ok(());
    }

    bail!(
        "Cannot prompt in a non-interactive environment. Re-run with {} to confirm explicitly.",
        flag_name
    )
}
```

Use this before `Confirm`, `Input`, `Select`, or `MultiSelect`.

## Text input with inline validation

```rust
use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Input};

fn prompt_package_name() -> Result<String> {
    let theme = ColorfulTheme::default();
    let value = Input::with_theme(&theme)
        .with_prompt("Package name")
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.trim().is_empty() {
                return Err("package name cannot be empty");
            }
            if input.contains(' ') {
                return Err("package name must not contain spaces");
            }
            Ok(())
        })
        .interact_text()?;

    Ok(value)
}
```

## Confirmation prompt with safe default

```rust
use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Confirm};

fn confirm_delete(path: &str, yes: bool) -> Result<bool> {
    Ok(yes
        || Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Delete {}?", path))
            .default(false)
            .interact()?)
}
```

Destructive default `true` is hostile. Make users consciously opt in.

## Selection lists

```rust
use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, MultiSelect, Select};

fn choose_profile() -> Result<usize> {
    let profiles = ["debug", "release", "dist"];
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Build profile")
        .items(&profiles)
        .default(0)
        .interact()?;
    Ok(selection)
}

fn choose_targets() -> Result<Vec<usize>> {
    let targets = ["linux-x86_64", "darwin-aarch64", "windows-x86_64"];
    let picks = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Target platforms")
        .items(&targets)
        .defaults(&[true, true, false])
        .interact()?;
    Ok(picks)
}
```

`.default(0)` and `defaults(&[...])` matter. Good defaults reduce decision load.

## Ctrl+C handling

If the app shows prompts, progress, or a hidden cursor, Ctrl+C must restore the terminal before exiting.

```rust
use anyhow::{Context, Result};
use console::Term;
use std::process;

pub fn install_ctrlc_handler() -> Result<()> {
    ctrlc::set_handler(move || {
        let term = Term::stderr();
        let _ = term.show_cursor();
        let _ = term.write_line("Aborted.");
        process::exit(130);
    })
    .context("failed to install Ctrl+C handler")?;

    Ok(())
}
```

## Full working example

```rust
use anyhow::{bail, Context, Result};
use console::Term;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};
use std::process;

fn is_interactive() -> bool {
    std::env::var_os("CI").is_none() && Term::stdout().is_term()
}

fn require_interactive_or_flag(flag_name: &str, allowed: bool) -> Result<()> {
    if allowed || is_interactive() {
        return Ok(());
    }

    bail!(
        "Cannot prompt in a non-interactive environment. Re-run with {} to confirm explicitly.",
        flag_name
    )
}

fn install_ctrlc_handler() -> Result<()> {
    ctrlc::set_handler(move || {
        let term = Term::stderr();
        let _ = term.show_cursor();
        let _ = term.write_line("Aborted.");
        process::exit(130);
    })
    .context("failed to install Ctrl+C handler")?;

    Ok(())
}

fn main() -> Result<()> {
    install_ctrlc_handler()?;

    let yes = false;
    require_interactive_or_flag("--yes", yes)?;

    let theme = ColorfulTheme::default();

    let package = Input::with_theme(&theme)
        .with_prompt("Package name")
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.trim().is_empty() {
                Err("package name cannot be empty")
            } else {
                Ok(())
            }
        })
        .interact_text()?;

    let profiles = ["debug", "release", "dist"];
    let profile = Select::with_theme(&theme)
        .with_prompt("Build profile")
        .items(&profiles)
        .default(0)
        .interact()?;

    let targets = ["linux-x86_64", "darwin-aarch64", "windows-x86_64"];
    let picks = MultiSelect::with_theme(&theme)
        .with_prompt("Target platforms")
        .items(&targets)
        .defaults(&[true, true, false])
        .interact()?;

    let confirmed = yes
        || Confirm::with_theme(&theme)
            .with_prompt(format!(
                "Create package {} with {} profile for {} target(s)?",
                package,
                profiles[profile],
                picks.len()
            ))
            .default(false)
            .interact()?;

    if !confirmed {
        let term = Term::stderr();
        term.write_line("Aborted.")?;
        process::exit(130);
    }

    Term::stdout().write_line("Project scaffolded.")?;
    Ok(())
}
```

Cross-reference:
- Read `error-design.md` for how non-interactive failures should be reported.
- Read `progress-and-feedback.md` before mixing prompts with spinners.

## `--no-input` — Full Non-interactive Mode

`--no-input` is distinct from `--yes`. They serve different purposes:

| Flag | Meaning |
|------|---------|
| `--yes` / `-y` | Skip *confirmation* prompts for destructive operations |
| `--no-input` | Disable *all* interactive prompts — fail if required input is missing |

Both must exist on any CLI with interactive elements.

```rust
#[derive(Parser)]
struct Args {
    /// Skip confirmation prompts (for scripting)
    #[arg(short = 'y', long)]
    yes: bool,

    /// Disable all interactive prompts; fail if required input is missing
    #[arg(long, global = true)]
    no_input: bool,
}

fn prompt_or_fail<T>(
    args: &Args,
    flag_value: Option<T>,
    flag_name: &str,
    prompt_fn: impl FnOnce() -> anyhow::Result<T>,
) -> anyhow::Result<T> {
    if let Some(v) = flag_value {
        return Ok(v);
    }
    if args.no_input || !console::Term::stdout().is_term() {
        anyhow::bail!(
            "Missing required value. Pass --{} <value> for non-interactive use.",
            flag_name
        );
    }
    prompt_fn()
}
```

## Password Input

Never echo secrets as the user types. Use `dialoguer::Password`:

```rust
use dialoguer::{theme::ColorfulTheme, Password};

let password = Password::with_theme(&ColorfulTheme::default())
    .with_prompt("Database password")
    .with_confirmation("Confirm password", "Passwords do not match")
    .interact()?;
```

For `--no-input` mode, accept via `--password-file` or stdin pipe:

```rust
fn read_secret(args: &Args) -> anyhow::Result<String> {
    if let Some(ref path) = args.password_file {
        return Ok(std::fs::read_to_string(path)?.trim().to_string());
    }
    if !console::Term::stdout().is_term() {
        let mut s = String::new();
        std::io::stdin().read_line(&mut s)?;
        return Ok(s.trim().to_string());
    }
    if args.no_input {
        anyhow::bail!(
            "Password required. Pass --password-file <path> \
             or pipe via stdin for non-interactive use."
        );
    }
    Ok(Password::with_theme(&ColorfulTheme::default())
        .with_prompt("Password")
        .interact()?)
}
```

## Multi-step Flow Escape

In any multi-step interactive flow, always show how to abort on the first prompt:

```rust
fn run_wizard(args: &Args) -> anyhow::Result<()> {
    println!(
        "{} Starting setup wizard {}",
        console::style("◆").cyan(),
        console::style("(Ctrl+C to cancel)").dim()
    );
    // ... rest of wizard
}
```

## Double Ctrl+C for Long Cleanup

When cleanup after interruption may take time, support a second Ctrl+C to
force-quit immediately:

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

let interrupted = Arc::new(AtomicBool::new(false));
let flag = interrupted.clone();

ctrlc::set_handler(move || {
    if flag.load(Ordering::SeqCst) {
        // Second Ctrl+C — skip cleanup, force exit
        eprintln!("\n{}", console::style("Force quit.").dim());
        std::process::exit(130);
    }
    flag.store(true, Ordering::SeqCst);
    eprintln!(
        "\n{} Cleaning up… {}",
        console::style("◆").yellow(),
        console::style("(Ctrl+C again to force quit)").dim()
    );
})?;

// In your cleanup loop, check the flag:
while has_cleanup_work() {
    if interrupted.load(Ordering::SeqCst) {
        break;
    }
    do_cleanup_step()?;
}
```

# UX Writing for CLIs

CLI writing should be fast to scan, easy to trust, and hard to misread under pressure.

## `clap` derive example

This example uses effect-based help text and four subcommands.

```rust
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "ferrite",
    version,
    about = "Design and audit Rust CLI output so commands read clearly in terminals and CI logs."
)]
pub struct Cli {
    #[arg(long, global = true, help = "Show underlying error chains and internal context.")]
    pub debug: bool,

    #[arg(short, long, global = true, help = "Print extra status lines for diagnosis.")]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Audit(AuditArgs),
    Init(InitArgs),
    Build(BuildArgs),
    Clean(CleanArgs),
}

#[derive(Debug, Args)]
#[command(about = "Review a CLI crate for output hierarchy, error quality, and interaction safety.")]
pub struct AuditArgs {
    #[arg(help = "Path to the crate or workspace to inspect.")]
    pub path: PathBuf,
}

#[derive(Debug, Args)]
#[command(about = "Create Ferrite configuration files in the current project.")]
pub struct InitArgs {
    #[arg(long, help = "Overwrite existing Ferrite files if they already exist.")]
    pub force: bool,
}

#[derive(Debug, Args)]
#[command(about = "Compile the project and print a release-style summary of build results.")]
pub struct BuildArgs {
    #[arg(long, help = "Skip dependency updates and reuse the existing lockfile.")]
    pub locked: bool,

    #[arg(short = 'j', long, help = "Set the number of parallel compilation jobs.")]
    pub jobs: Option<usize>,
}

#[derive(Debug, Args)]
#[command(about = "Remove generated output files without touching source code.")]
pub struct CleanArgs {
    #[arg(short, long, help = "Confirm deletion without prompting interactively.")]
    pub yes: bool,
}
```

## Good vs bad `about` strings

| Bad | Why it fails | Better |
| --- | --- | --- |
| `Build the project` | Restates the command name | `Compile the workspace and summarize created artifacts.` |
| `Run audit` | Says nothing about outcome | `Inspect CLI output, help text, and errors for design issues.` |
| `Manage config` | Too broad | `Read or update Ferrite settings for the current repository.` |

Effect-based help text answers: what changes when I run this?

## Tone by message type

| Message type | Tone | Example |
| --- | --- | --- |
| Success | Crisp, specific, calm | `Created 4 release artifacts.` |
| Warning | Cautious, actionable | `Skipped color output because NO_COLOR is set.` |
| Error | Direct, non-blaming | `Could not open Cargo.toml.` |
| Progress | Present tense, concrete | `Compiling workspace` |
| Prompt | Clear decision language | `Delete generated cache files?` |
| Completion | Outcome-first, then detail | `Published crate in 3.1s.` |

## Words to avoid

| Avoid | Prefer |
| --- | --- |
| process | build, delete, write, upload, parse |
| handle | read, validate, render, confirm |
| operation | build, scan, sync, cleanup |
| invalid input | explain the exact invalid field |
| something went wrong | the actual failure |
| successfully | omit it unless contrast matters |

## Consistent terminology rule

Pick one word per concept and keep it everywhere:

- If the help text says `workspace`, do not switch to `project` in errors.
- If a flag says `profile`, do not call it `mode` in prompts.
- If the command produces `artifacts`, do not summarize them as `outputs` unless they are truly generic.

Terminology drift makes CLIs feel improvised.

## Good message patterns

Bad:

```text
Error occurred.
```

Better:

```text
Error: could not parse --jobs value
How: pass a positive integer such as `--jobs 8`
```

Bad:

```text
Do you want to continue?
```

Better:

```text
Delete 18 generated cache files?
```

Bad:

```text
Running build process...
```

Better:

```text
Compiling workspace
```

## Checklist

- Does each `about` string describe the effect of the command?
- Does each flag description answer “what happens if I pass this?”
- Are warnings specific about what was skipped or degraded?
- Are prompt verbs concrete and reversible when possible?
- Could a tired user understand the error without reading source code?

Cross-reference:
- Read `error-design.md` for WHAT/WHY/HOW structure.
- Read `output-hierarchy.md` to place these messages at the right visual level.

## Arguments & Flags — CLIG Conventions

→ *These rules govern how flags are named and structured. Violating them
creates muscle-memory conflicts for users who know other CLI tools.*

**DO**: Prefer `--flags` over positional arguments — flags are self-documenting
and forward-compatible:

```rust
// ✗ Positional — ambiguous, hard to extend
// mytool deploy my-app production

// ✓ Flags — clear intent, easy to add new options later
// mytool deploy --app my-app --env production

#[derive(Parser)]
struct DeployArgs {
    #[arg(long)]
    app: String,
    #[arg(long, default_value = "production")]
    env: String,
}
```

**DO**: Provide both short and long forms for common flags:

```rust
/// Print each step as it runs
#[arg(short = 'v', long)]
verbose: bool,

/// Suppress all non-essential output
#[arg(short = 'q', long)]
quiet: bool,

/// Skip confirmation prompts
#[arg(short = 'f', long)]
force: bool,
```

**DO**: Use these standard flag names — don't invent alternatives:

| Flag | Short | Meaning |
|------|-------|---------|
| `--all` | `-a` | Include all items (e.g. hidden files) |
| `--debug` | `-d` | Show debug output |
| `--dry-run` | `-n` | Simulate without making changes |
| `--force` | `-f` | Skip safety checks / overwrite |
| `--help` | `-h` | Show help (handled by clap) |
| `--json` | | Machine-readable JSON output |
| `--no-input` | | Disable all interactive prompts |
| `--output` | `-o` | Output file or directory |
| `--plain` | | Plain text output for piping/grep |
| `--port` | `-p` | Port number |
| `--quiet` | `-q` | Suppress non-essential output |
| `--timeout` | | Operation timeout in seconds |
| `--verbose` | `-v` | Detailed output |
| `--version` | | Show version (handled by clap) |
| `--yes` | `-y` | Skip confirmation for destructive ops |

**DO**: Support `--dry-run` / `-n` for any command that modifies state:

```rust
if args.dry_run {
    println!(
        "{} Would delete {} (--dry-run, no changes made)",
        console::style("◆").yellow(),
        path.display()
    );
    return Ok(());
}
```

**DO**: Accept secrets via `--password-file` or stdin, never `--password`:

```
✗  mytool login --password secret123   (visible in ps, shell history)
✓  mytool login --password-file ~/.mytool-pass
✓  echo "secret" | mytool login
```

**DON'T**: Use more than one positional argument unless it's a well-known
pattern like `cp <src> <dst>`  
**DON'T**: Reserve single-letter flags for rarely-used options — save them
for the most common flags  
**DON'T**: Use `-h` for anything other than help  

## Subcommand Anti-Patterns

These three patterns permanently damage a CLI's extensibility. Avoid them.

**DON'T**: Create a catch-all subcommand that handles unknown input:

```rust
// ✗ Bad — you can never add a subcommand named "echo" or "run" again
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if known_subcommand(&args[1]) {
        dispatch(&args[1]);
    } else {
        // Treat unknown input as implicit "run" subcommand
        run_command(&args[1..]);
    }
}

// ✓ Good — explicit subcommands only, unknown input = error + help
#[derive(Subcommand)]
enum Commands {
    Run { command: Vec<String> },
    Build,
    Deploy,
}
```

**DON'T**: Allow abbreviated subcommand names — use explicit aliases instead:

```rust
// ✗ Bad — accepting "ins" as "install" locks your namespace forever
// (clap does NOT do this by default — don't enable it)

// ✓ Good — explicit aliases that you commit to supporting
#[command(alias = "rm")]
Remove { name: String },
```

**DON'T**: Have similarly-named commands that differ subtly:

```
✗  update / upgrade    (users never remember which does what)
✗  delete / remove     (pick one)
✗  init / setup        (pick one)

✓  Pick one word per concept. Document aliases if needed for migration.
```

## Deprecation & Future-proofing

**DO**: Add new flags rather than changing existing flag behavior  
**DO**: Warn at runtime before removing a deprecated flag:

```rust
// In your arg handler, before the flag is removed:
if args.old_flag.is_some() {
    eprintln!(
        "{} --old-flag is deprecated and will be removed in v2.0.\n  \
         Use --new-flag instead: mytool build --new-flag <value>",
        console::style("⚠").yellow()
    );
}
```

**DO**: Encourage `--json` or `--plain` in scripts — human-readable output
format can change freely, machine-readable output must stay stable  
**DON'T**: Change the meaning of an existing flag in a patch or minor release  
**DON'T**: Remove a flag without a deprecation warning period  

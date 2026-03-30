# Robustness Reference

→ *Consult this reference when using `/harden` or when a CLI needs to handle
real-world misuse: bad input, network failures, concurrent invocations,
interrupted operations.*

Robustness is both objective and subjective. Software should *be* robust, and
it should *feel* robust — immediate, solid, like a mechanical machine rather
than a flimsy soft switch.

## Input Validation

Validate early. Fail before doing any work. Never let bad input propagate.

**DO**: Use `clap`'s built-in value parsers for type-level validation:

```rust
#[derive(Parser)]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value_t = 8080,
          value_parser = clap::value_parser!(u16).range(1..))]
    port: u16,

    /// Output directory (must exist)
    #[arg(short, long, value_parser = existing_dir)]
    output: PathBuf,
}

fn existing_dir(s: &str) -> Result<PathBuf, String> {
    let p = PathBuf::from(s);
    if p.is_dir() { Ok(p) } else { Err(format!("{:?} is not a directory", s)) }
}
```

**DO**: For cross-field validation, check after parsing and fail with context:

```rust
fn validate(args: &Args) -> anyhow::Result<()> {
    if args.start > args.end {
        anyhow::bail!(
            "--start ({}) must be less than --end ({})",
            args.start, args.end
        );
    }
    Ok(())
}
```

**DO**: Validate before starting any long-running operation — not halfway through  
**DON'T**: Use `.unwrap()` or `.expect()` on user-provided input  
**DON'T**: Let invalid input reach business logic and produce confusing downstream errors  

## Responsiveness

> "Responsive is more important than fast." — CLIG

**DO**: Print something within 100ms — even just the action name before work begins:

```rust
// Print intent before starting work — never let the terminal go silent
println!("{} Connecting to {}…",
    console::style("◆").cyan(),
    console::style(&args.host).dim()
);
// Now do the actual work
let conn = connect(&args.host).await?;
```

**DO**: Show estimated time remaining for operations > 10s  
**DO**: For network operations, print what you're about to do *before* the request  
**DON'T**: Start a long operation in silence — users will assume it's broken  

## Timeouts

**DO**: Set timeouts on all network and external process operations:

```rust
use std::time::Duration;

#[derive(Parser)]
struct Args {
    /// Request timeout in seconds
    #[arg(long, default_value_t = 30)]
    timeout: u64,
}

let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(args.timeout))
    .connect_timeout(Duration::from_secs(10))
    .build()?;
```

**DO**: Make timeout duration configurable via flag  
**DO**: Document the default timeout in the flag's `about` string  
**DON'T**: Use no timeout — a hung network call will hang the entire CLI forever  

## Idempotency

**DO**: Design commands to be safely re-runnable — check state before acting:

```rust
// Check before acting, report current state clearly
if output_dir.exists() && !args.force {
    println!(
        "{} {} already exists — skipping (use --force to overwrite)",
        console::style("◆").dim(),
        output_dir.display()
    );
    return Ok(());
}
```

**DO**: Use `--force` to override idempotent guards, not `--yes`  
**DON'T**: Fail with an error when re-running a successfully completed operation  
**DON'T**: Silently overwrite without informing the user  

## Crash-only Design

> "If you can avoid cleanup, or defer it to next run, your program can exit
> immediately on failure or interruption." — CLIG

**DO**: Prefer exiting immediately over complex cleanup routines  
**DO**: Write to temp files, then atomically rename on success:

```rust
use std::fs;

let tmp = output_path.with_extension("tmp");
fs::write(&tmp, &content)
    .with_context(|| format!("Failed to write to {:?}", tmp))?;
fs::rename(&tmp, &output_path)
    .with_context(|| format!("Failed to finalize {:?}", output_path))?;
```

**DO**: Design so that an interrupted operation can be safely re-run from scratch  
**DON'T**: Hold exclusive locks or leave temp files that block re-running after a crash  

## Handling Misuse

People will wrap your tool in scripts, run it on bad connections, invoke it
concurrently, and use it in environments you never tested.

**DO**: Test your tool piped: `mytool | cat`, `mytool 2>/dev/null`  
**DO**: Test with `NO_COLOR=1`, `CI=true`, and on a slow network  
**DO**: Handle concurrent invocations gracefully — use file locks if needed:

```rust
use fs2::FileExt; // fs2 crate

let lock_file = fs::File::create(lock_path)?;
lock_file.try_lock_exclusive().map_err(|_| {
    anyhow::anyhow!(
        "Another instance of {} is already running",
        env!("CARGO_PKG_NAME")
    )
})?;
```

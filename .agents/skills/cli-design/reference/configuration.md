# Configuration Reference

→ *Consult this reference when a CLI needs persistent settings, environment
variable support, or project-level configuration files.*

## Precedence Order

Apply configuration from highest to lowest priority. Never let a lower-priority
source silently override a higher one. Document this order in `--help`.

```
1. Command-line flags          (highest — explicit user intent)
2. Environment variables       (context-specific overrides)
3. Project config file         (mytool.toml in working directory)
4. User config file            (XDG config dir)
5. Built-in defaults           (lowest)
```

Implement explicitly:

```rust
#[derive(Parser)]
struct Args {
    /// API endpoint (overrides MYTOOL_API_URL and config file)
    #[arg(long, env = "MYTOOL_API_URL")]
    api_url: Option<String>,
}

fn resolve_api_url(args: &Args, config: &Config) -> String {
    // clap's `env` attribute handles flag > env var automatically.
    // We only need to handle the config file fallback:
    args.api_url
        .clone()
        .unwrap_or_else(|| config.api_url.clone())
        .unwrap_or_else(|| "https://api.example.com".to_string())
}
```

## File Locations — Follow XDG

**DO**: Use the XDG Base Directory specification. Never create `~/.mytool`:

```rust
// Use the `dirs` crate
fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from(".config"))
        .join(env!("CARGO_PKG_NAME"))
}

fn config_file() -> PathBuf {
    config_dir().join("config.toml")
}

// XDG locations:
// Config:  ~/.config/mytool/config.toml
// Data:    ~/.local/share/mytool/
// Cache:   ~/.cache/mytool/
```

**DO**: Support `--config <path>` to override the default location  
**DON'T**: Create `~/.mytool`, `~/.mytoolrc`, or `~/.mytool.conf`  
**DON'T**: Scatter files across the home directory  

## Config File Format

**DO**: Use TOML — consistent with the Cargo ecosystem:

```toml
# ~/.config/mytool/config.toml
api_url = "https://api.example.com"
timeout = 30
default_output = "./dist"
```

```rust
use serde::Deserialize;

#[derive(Deserialize, Default)]
struct Config {
    api_url: Option<String>,
    timeout: Option<u64>,
    default_output: Option<PathBuf>,
}

fn load_config(path: &Path) -> anyhow::Result<Config> {
    if !path.exists() {
        return Ok(Config::default());
    }
    let content = fs::read_to_string(path)
        .with_context(|| format!("Cannot read config at {}", path.display()))?;
    toml::from_str(&content)
        .with_context(|| format!("Invalid TOML in {}", path.display()))
}
```

## Environment Variables

**DO**: Use SCREAMING_SNAKE_CASE, prefixed with your tool name:

```
MYTOOL_API_URL
MYTOOL_LOG_LEVEL
MYTOOL_TIMEOUT
```

**DO**: Respect standard environment variables where they apply:

| Variable | When to check |
|----------|--------------|
| `NO_COLOR` | Before any colored output |
| `TERM` | Terminal capability detection |
| `EDITOR` | When opening files for editing |
| `PAGER` | When piping long output |
| `COLUMNS` | Terminal width for layout |
| `CI` | Disable interactive elements |
| `HOME` | User home directory fallback |

**DO**: Document all supported env vars in `--help` output  
**DO**: Use `clap`'s `env` attribute to wire env vars to flags automatically:

```rust
#[arg(long, env = "MYTOOL_API_URL")]
api_url: Option<String>,
```

**DON'T**: Use env var names without a tool-specific prefix (except standard ones)  
**DON'T**: Read secrets from env vars — use files or stdin:

```rust
// ✗ Bad — leaks to ps output, logs, child processes
#[arg(long, env = "MYTOOL_PASSWORD")]
password: Option<String>,

// ✓ Good — read from file or stdin
#[arg(long)]
password_file: Option<PathBuf>,

fn read_password(args: &Args) -> anyhow::Result<String> {
    match &args.password_file {
        Some(path) => Ok(fs::read_to_string(path)?.trim().to_string()),
        None => {
            // Read from stdin if piped, else prompt
            if !console::Term::stdout().is_term() {
                let mut s = String::new();
                std::io::stdin().read_line(&mut s)?;
                Ok(s.trim().to_string())
            } else {
                Ok(dialoguer::Password::with_theme(
                    &dialoguer::theme::ColorfulTheme::default()
                )
                .with_prompt("Password")
                .interact()?)
            }
        }
    }
}
```

**DON'T**: Use `.env` files as a substitute for a proper config file  
**DON'T**: Read multi-line values from env vars  

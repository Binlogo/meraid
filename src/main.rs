//! Mermaid CLI - Render Mermaid diagrams in your terminal
//! AI-friendly: designed for AI coding agents to use

use clap::{Parser, ValueEnum, ValueHint};
use meraid::{ColorMode, Diagram, DiagramType, Theme, ThemeType};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, IsTerminal, Read};
use std::path::PathBuf;
use unicode_width::UnicodeWidthStr;

#[derive(Parser)]
#[command(name = "meraid")]
#[command(version)]
#[command(about = "Render Mermaid diagrams in your terminal")]
struct Cli {
    /// Input file, or - to read from stdin
    #[arg(value_hint = ValueHint::FilePath)]
    input: Option<PathBuf>,

    /// Color theme
    #[arg(long, default_value = "default", value_enum)]
    theme: CliTheme,

    /// ASCII-only box-drawing (no Unicode)
    #[arg(long, short = 'a')]
    ascii: bool,

    /// Horizontal padding inside boxes
    #[arg(long, default_value = "4")]
    padding_x: usize,

    /// Vertical padding inside boxes
    #[arg(long, default_value = "2")]
    padding_y: usize,

    /// Output format
    #[arg(long, default_value = "text", value_enum)]
    format: OutputFormat,

    /// When to emit ANSI color: auto (TTY only), always, or never
    #[arg(long, default_value = "auto", value_enum)]
    color: ColorChoice,
}

#[derive(Clone, Copy, ValueEnum)]
enum ColorChoice {
    Auto,
    Always,
    Never,
}

/// The environment inputs that gate color, separated from the decision so the
/// policy can be unit-tested without touching real env/TTY.
struct ColorEnv {
    stdout_is_tty: bool,
    no_color: bool,
    term_dumb: bool,
    truecolor: bool,
}

impl ColorEnv {
    /// Gather the real environment.
    fn detect() -> Self {
        Self {
            stdout_is_tty: io::stdout().is_terminal(),
            no_color: std::env::var_os("NO_COLOR").is_some(),
            term_dumb: std::env::var("TERM").map(|t| t == "dumb").unwrap_or(false),
            truecolor: matches!(
                std::env::var("COLORTERM").as_deref(),
                Ok("truecolor") | Ok("24bit")
            ),
        }
    }
}

/// Resolve the effective color mode. JSON output is always uncolored. `Always`
/// forces color (overriding NO_COLOR); `Auto` requires a capable TTY. Depth is
/// truecolor when `COLORTERM` advertises it, otherwise ANSI-256.
fn decide_color_mode(choice: ColorChoice, json_mode: bool, env: &ColorEnv) -> ColorMode {
    if json_mode {
        return ColorMode::None;
    }
    let want_color = match choice {
        ColorChoice::Never => false,
        ColorChoice::Always => true,
        ColorChoice::Auto => env.stdout_is_tty && !env.no_color && !env.term_dumb,
    };
    if !want_color {
        return ColorMode::None;
    }
    if env.truecolor {
        ColorMode::TrueColor
    } else {
        ColorMode::Ansi256
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum CliTheme {
    Default,
    Terra,
    Neon,
    Mono,
    Amber,
    Phosphor,
}

impl CliTheme {
    fn as_str(self) -> &'static str {
        match self {
            CliTheme::Default => "default",
            CliTheme::Terra => "terra",
            CliTheme::Neon => "neon",
            CliTheme::Mono => "mono",
            CliTheme::Amber => "amber",
            CliTheme::Phosphor => "phosphor",
        }
    }
}

impl From<CliTheme> for ThemeType {
    fn from(t: CliTheme) -> Self {
        match t {
            CliTheme::Default => ThemeType::Default,
            CliTheme::Terra => ThemeType::Terra,
            CliTheme::Neon => ThemeType::Neon,
            CliTheme::Mono => ThemeType::Mono,
            CliTheme::Amber => ThemeType::Amber,
            CliTheme::Phosphor => ThemeType::Phosphor,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

/// JSON output structure for AI-friendly parsing
#[derive(Serialize, Deserialize)]
struct JsonOutput {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    diagram: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonError>,
    metadata: JsonMetadata,
}

#[derive(Serialize, Deserialize)]
struct JsonError {
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    column: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    suggestion: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct JsonMetadata {
    diagram_type: String,
    theme: String,
    width: usize,
    height: usize,
    nodes: usize,
    edges: usize,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let json_mode = matches!(cli.format, OutputFormat::Json);

    // Read input
    let source = read_input(&cli.input, json_mode, cli.theme.as_str())?;

    // Build renderer
    let theme: ThemeType = cli.theme.into();
    let theme = Theme::get(theme);
    let color_mode = decide_color_mode(cli.color, json_mode, &ColorEnv::detect());
    let renderer = meraid::Renderer::new(theme.clone())
        .ascii_only(cli.ascii)
        .padding(cli.padding_x, cli.padding_y)
        .color_mode(color_mode);

    // Render
    match meraid::parse_mermaid(&source).map(|diagram| {
        let layout = meraid::Layout::new(&diagram).layout();
        let output = renderer.render(&diagram, &layout);
        (diagram, layout, output)
    }) {
        Ok((diagram, _layout, output)) => {
            if json_mode {
                let (width, height) = output_dimensions(&output);
                let nodes = node_count(&diagram);
                let edges = edge_count(&diagram);
                let json_output = JsonOutput {
                    success: true,
                    diagram: Some(output),
                    error: None,
                    metadata: JsonMetadata {
                        diagram_type: format!("{:?}", diagram.diagram_type).to_lowercase(),
                        theme: cli.theme.as_str().to_string(),
                        width,
                        height,
                        nodes,
                        edges,
                    },
                };
                println!("{}", serde_json::to_string_pretty(&json_output)?);
            } else {
                print!("{}", output);
            }
        }
        Err(e) => {
            let error_msg = e.to_string();

            if json_mode {
                let (line, column) = parse_error_position(&error_msg);
                let json_output = JsonOutput {
                    success: false,
                    diagram: None,
                    error: Some(JsonError {
                        message: error_msg.clone(),
                        line,
                        column,
                        suggestion: generate_suggestion(&error_msg),
                    }),
                    metadata: JsonMetadata {
                        diagram_type: "unknown".to_string(),
                        theme: cli.theme.as_str().to_string(),
                        width: 0,
                        height: 0,
                        nodes: 0,
                        edges: 0,
                    },
                };
                println!("{}", serde_json::to_string_pretty(&json_output)?);
            } else {
                eprintln!("error: {}", error_msg);
                if let Some(suggestion) = generate_suggestion(&error_msg) {
                    eprintln!("hint: {}", suggestion);
                }
            }
            std::process::exit(1);
        }
    }

    Ok(())
}

fn read_input(path: &Option<PathBuf>, json_mode: bool, theme: &str) -> anyhow::Result<String> {
    match path {
        Some(p) if p.as_os_str() == "-" => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            Ok(buf)
        }
        Some(p) => fs::read_to_string(p)
            .map_err(|e| anyhow::anyhow!("cannot read '{}': {}", p.display(), e)),
        None => {
            if io::stdin().is_terminal() {
                let msg = "no input provided — pipe Mermaid source via stdin or pass a file path";
                if json_mode {
                    let output = JsonOutput {
                        success: false,
                        diagram: None,
                        error: Some(JsonError {
                            message: msg.to_string(),
                            line: None,
                            column: None,
                            suggestion: Some("echo 'graph LR\\nA --> B' | meraid".to_string()),
                        }),
                        metadata: JsonMetadata {
                            diagram_type: "unknown".to_string(),
                            theme: theme.to_string(),
                            width: 0,
                            height: 0,
                            nodes: 0,
                            edges: 0,
                        },
                    };
                    println!("{}", serde_json::to_string_pretty(&output)?);
                    std::process::exit(1);
                }
                anyhow::bail!("{}", msg);
            }
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            Ok(buf)
        }
    }
}

/// Measure the rendered output's visible dimensions (display columns × rows),
/// ignoring trailing blank lines and trailing whitespace.
fn output_dimensions(output: &str) -> (usize, usize) {
    let lines: Vec<&str> = output.lines().map(|l| l.trim_end()).collect();
    let height = lines
        .iter()
        .rposition(|l| !l.is_empty())
        .map_or(0, |i| i + 1);
    let width = lines[..height]
        .iter()
        .map(|l| UnicodeWidthStr::width(*l))
        .max()
        .unwrap_or(0);
    (width, height)
}

/// Count the "nodes" of a diagram in the sense most meaningful for its type.
fn node_count(d: &Diagram) -> usize {
    match d.diagram_type {
        DiagramType::ER => d.entities.len(),
        DiagramType::Sequence => d.participants.len(),
        _ => d.nodes.len(),
    }
}

/// Count the "edges" (connections/relationships) of a diagram.
fn edge_count(d: &Diagram) -> usize {
    match d.diagram_type {
        DiagramType::Class | DiagramType::ER => d.edges.len() + d.relationships.len(),
        _ => d.edges.len(),
    }
}

/// Parse error message to extract line and column numbers
fn parse_error_position(error: &str) -> (Option<usize>, Option<usize>) {
    let line_patterns = ["line ", "at line ", "line: "];
    let col_patterns = ["column ", "col ", "position "];

    let mut line = None;
    let mut col = None;

    for pattern in &line_patterns {
        if let Some(idx) = error.find(pattern) {
            let rest = &error[idx + pattern.len()..];
            if let Some(end) = rest.find(|c: char| !c.is_ascii_digit()) {
                if let Ok(n) = rest[..end].parse::<usize>() {
                    line = Some(n);
                    break;
                }
            }
        }
    }

    for pattern in &col_patterns {
        if let Some(idx) = error.find(pattern) {
            let rest = &error[idx + pattern.len()..];
            if let Some(end) = rest.find(|c: char| !c.is_ascii_digit()) {
                if let Ok(n) = rest[..end].parse::<usize>() {
                    col = Some(n);
                    break;
                }
            }
        }
    }

    (line, col)
}

/// Generate helpful suggestions based on error type
fn generate_suggestion(error: &str) -> Option<String> {
    let error_lower = error.to_lowercase();

    if error_lower.contains("unknown diagram type") || error_lower.contains("parse error") {
        Some("check the diagram type keyword: graph, sequenceDiagram, classDiagram, stateDiagram-v2, erDiagram, pie".to_string())
    } else if error_lower.contains("syntax") {
        Some("verify Mermaid syntax at https://mermaid.js.org/intro/".to_string())
    } else if error_lower.contains("unexpected token") {
        Some("check for typos or invalid characters in the diagram source".to_string())
    } else if error_lower.contains("empty") {
        Some("provide non-empty Mermaid diagram source".to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use meraid::ColorMode;

    fn env(is_tty: bool, no_color: bool, term_dumb: bool, truecolor: bool) -> ColorEnv {
        ColorEnv {
            stdout_is_tty: is_tty,
            no_color,
            term_dumb,
            truecolor,
        }
    }

    #[test]
    fn json_mode_is_always_uncolored() {
        // Even --color=always with a truecolor TTY: JSON must stay clean.
        let e = env(true, false, false, true);
        assert_eq!(
            decide_color_mode(ColorChoice::Always, true, &e),
            ColorMode::None
        );
    }

    #[test]
    fn never_disables_color() {
        let e = env(true, false, false, true);
        assert_eq!(
            decide_color_mode(ColorChoice::Never, false, &e),
            ColorMode::None
        );
    }

    #[test]
    fn always_overrides_no_color_and_picks_depth() {
        // Explicit --color=always beats NO_COLOR.
        let truecolor = env(false, true, true, true);
        assert_eq!(
            decide_color_mode(ColorChoice::Always, false, &truecolor),
            ColorMode::TrueColor
        );
        let only256 = env(false, true, true, false);
        assert_eq!(
            decide_color_mode(ColorChoice::Always, false, &only256),
            ColorMode::Ansi256
        );
    }

    #[test]
    fn auto_emits_color_only_on_a_capable_tty() {
        // Happy path: TTY, no NO_COLOR, not dumb, truecolor advertised.
        assert_eq!(
            decide_color_mode(ColorChoice::Auto, false, &env(true, false, false, true)),
            ColorMode::TrueColor
        );
        // TTY but no truecolor → 256.
        assert_eq!(
            decide_color_mode(ColorChoice::Auto, false, &env(true, false, false, false)),
            ColorMode::Ansi256
        );
    }

    #[test]
    fn auto_is_silent_when_piped_or_suppressed() {
        // Not a TTY (piped / redirected).
        assert_eq!(
            decide_color_mode(ColorChoice::Auto, false, &env(false, false, false, true)),
            ColorMode::None
        );
        // NO_COLOR set.
        assert_eq!(
            decide_color_mode(ColorChoice::Auto, false, &env(true, true, false, true)),
            ColorMode::None
        );
        // TERM=dumb.
        assert_eq!(
            decide_color_mode(ColorChoice::Auto, false, &env(true, false, true, true)),
            ColorMode::None
        );
    }
}

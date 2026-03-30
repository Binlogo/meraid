//! Mermaid CLI - Render Mermaid diagrams in your terminal
//! AI-friendly: designed for AI coding agents to use

use clap::{Parser, ValueEnum, ValueHint};
use meraid::{Theme, ThemeType};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, IsTerminal, Read};
use std::path::PathBuf;

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
    let renderer = meraid::Renderer::new(theme.clone())
        .ascii_only(cli.ascii)
        .padding(cli.padding_x, cli.padding_y);

    // Render
    match meraid::parse_mermaid(&source).and_then(|diagram| {
        let layout = meraid::Layout::new(&diagram).layout();
        let output = renderer.render(&diagram, &layout);
        Ok((diagram, layout, output))
    }) {
        Ok((diagram, layout, output)) => {
            if json_mode {
                let json_output = JsonOutput {
                    success: true,
                    diagram: Some(output),
                    error: None,
                    metadata: JsonMetadata {
                        diagram_type: format!("{:?}", diagram.diagram_type).to_lowercase(),
                        theme: cli.theme.as_str().to_string(),
                        width: layout.width,
                        height: layout.height,
                        nodes: diagram.nodes.len(),
                        edges: diagram.edges.len(),
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

//! Mermaid CLI - Render Mermaid diagrams in your terminal
//! AI-friendly: designed for AI coding agents to use

use clap::{Parser, ValueHint};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use meraid::{render_with_theme, Theme, ThemeType};

#[derive(Parser)]
#[command(name = "meraid")]
#[command(about = "Render Mermaid diagrams in your terminal", long_about = None)]
struct Cli {
    /// Input file (or - for stdin)
    #[arg(value_hint = ValueHint::FilePath)]
    input: Option<PathBuf>,
    
    /// Output theme
    #[arg(long, default_value = "default")]
    theme: String,
    
    /// ASCII-only output (no Unicode box-drawing)
    #[arg(long, short = 'a')]
    ascii: bool,
    
    /// Horizontal padding inside boxes
    #[arg(long, default_value = "4")]
    padding_x: usize,
    
    /// Vertical padding inside boxes
    #[arg(long, default_value = "2")]
    padding_y: usize,
    
    /// Sharp corners on edge turns
    #[arg(long, short = 's')]
    sharp_edges: bool,
    
    /// Output format: text or json
    #[arg(long, default_value = "text")]
    format: String,
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

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    // Check if JSON output is requested
    let json_mode = cli.format.to_lowercase() == "json";
    
    // Read input
    let source = if let Some(path) = &cli.input {
        if path.as_os_str() == "-" {
            // Read from stdin
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            buffer
        } else {
            // Read from file
            fs::read_to_string(path)?
        }
    } else {
        // Read from stdin if no file provided
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        if buffer.trim().is_empty() {
            let error_msg = "Error: No input provided. Use - for stdin or provide a file.";
            if json_mode {
                let output = JsonOutput {
                    success: false,
                    diagram: None,
                    error: Some(JsonError {
                        message: error_msg.to_string(),
                        line: None,
                        column: None,
                        suggestion: Some("Pipe Mermaid code via stdin: echo 'graph LR; A --> B' | meraid".to_string()),
                    }),
                    metadata: JsonMetadata {
                        diagram_type: "unknown".to_string(),
                        theme: cli.theme.clone(),
                        width: 0,
                        height: 0,
                        nodes: 0,
                        edges: 0,
                    },
                };
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                eprintln!("{}", error_msg);
            }
            std::process::exit(1);
        }
        buffer
    };
    
    // Parse theme
    let theme_type: ThemeType = cli.theme.parse().unwrap_or_default();
    let theme = Theme::get(theme_type);
    
    // Try to render
    match render_with_theme(&source, theme) {
        Ok(output) => {
            if json_mode {
                // Get diagram info for metadata
                let diagram = meraid::parse_mermaid(&source)?;
                let layout = meraid::Layout::new(&diagram).layout();
                
                let json_output = JsonOutput {
                    success: true,
                    diagram: Some(output),
                    error: None,
                    metadata: JsonMetadata {
                        diagram_type: format!("{:?}", diagram.diagram_type),
                        theme: cli.theme,
                        width: layout.width,
                        height: layout.height,
                        nodes: diagram.nodes.len(),
                        edges: diagram.edges.len(),
                    },
                };
                println!("{}", serde_json::to_string_pretty(&json_output)?);
            } else {
                // Plain text output
                println!("{}", output);
            }
        }
        Err(e) => {
            let error_msg = e.to_string();
            
            if json_mode {
                // Try to extract line/column from error message
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
                        theme: cli.theme,
                        width: 0,
                        height: 0,
                        nodes: 0,
                        edges: 0,
                    },
                };
                println!("{}", serde_json::to_string_pretty(&json_output)?);
            } else {
                // Plain text error
                eprintln!("Error: {}", error_msg);
                if let Some(suggestion) = generate_suggestion(&error_msg) {
                    eprintln!("Suggestion: {}", suggestion);
                }
            }
            std::process::exit(1);
        }
    }
    
    Ok(())
}

/// Parse error message to extract line and column numbers
fn parse_error_position(error: &str) -> (Option<usize>, Option<usize>) {
    // Common patterns: "at line X", "line X", "position X"
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
        Some("Check Mermaid syntax. Common issues: missing diagram type keyword (graph, sequenceDiagram, etc.)".to_string())
    } else if error_lower.contains("syntax") {
        Some("Verify Mermaid syntax is correct. See https://mermaid.js.org/intro/".to_string())
    } else if error_lower.contains("unexpected token") {
        Some("Check for typos or invalid characters in your Mermaid diagram".to_string())
    } else if error_lower.contains("empty") {
        Some("Provide non-empty Mermaid diagram code".to_string())
    } else {
        None
    }
}

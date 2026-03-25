//! Mermaid CLI - Render Mermaid diagrams in your terminal

use clap::{Parser, ValueHint};
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
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
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
            eprintln!("Error: No input provided. Use - for stdin or provide a file.");
            std::process::exit(1);
        }
        buffer
    };
    
    // Parse theme
    let theme_type: ThemeType = cli.theme.parse().unwrap_or_default();
    let theme = Theme::get(theme_type);
    
    // Render
    let output = render_with_theme(&source, theme)?;
    
    // Print to terminal
    println!("{}", output);
    
    Ok(())
}

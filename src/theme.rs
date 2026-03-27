//! Terminal themes for colored output

use serde::{Deserialize, Serialize};

/// Color codes for terminals
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
    
    /// Convert to ANSI 256 color
    pub fn to_ansi256(&self) -> u8 {
        if self.r == self.g && self.g == self.b {
            let gray = self.r;
            if gray < 8 { return 16; }
            if gray > 248 { return 231; }
            return ((gray - 8) / 247) * 24 + 232;
        }
        
        let r = (self.r as f32 / 255.0 * 5.0) as u8;
        let g = (self.g as f32 / 255.0 * 5.0) as u8;
        let b = (self.b as f32 / 255.0 * 5.0) as u8;
        
        16 + r * 36 + g * 6 + b
    }
    
    /// Convert to ANSI escape sequence
    pub fn to_escape(&self) -> String {
        format!("\x1b[38;5;{}m", self.to_ansi256())
    }
}

/// Theme for diagram rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub node_bg: Color,
    pub node_fg: Color,
    pub edge: Color,
    pub edge_label: Color,
    pub start_end: Color,
}

impl Theme {
    /// Get theme by type
    pub fn get(theme_type: ThemeType) -> Self {
        match theme_type {
            ThemeType::Default => default_theme(),
            ThemeType::Terra => terra_theme(),
            ThemeType::Neon => neon_theme(),
            ThemeType::Mono => mono_theme(),
            ThemeType::Amber => amber_theme(),
            ThemeType::Phosphor => phosphor_theme(),
        }
    }
}

/// Theme types available
#[derive(Debug, Clone, Copy, Default)]
pub enum ThemeType {
    #[default]
    Default,
    Terra,
    Neon,
    Mono,
    Amber,
    Phosphor,
}

impl std::str::FromStr for ThemeType {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "default" => Ok(ThemeType::Default),
            "terra" => Ok(ThemeType::Terra),
            "neon" => Ok(ThemeType::Neon),
            "mono" => Ok(ThemeType::Mono),
            "amber" => Ok(ThemeType::Amber),
            "phosphor" => Ok(ThemeType::Phosphor),
            _ => Ok(ThemeType::Default),
        }
    }
}

// Theme definitions
fn default_theme() -> Theme {
    Theme {
        name: "default".to_string(),
        node_bg: Color::new(0, 0, 0),
        node_fg: Color::new(255, 255, 255),
        edge: Color::new(255, 255, 0),
        edge_label: Color::new(255, 255, 0),
        start_end: Color::new(128, 128, 128),
    }
}

fn terra_theme() -> Theme {
    Theme {
        name: "terra".to_string(),
        node_bg: Color::new(45, 35, 25),
        node_fg: Color::new(255, 220, 180),
        edge: Color::new(255, 180, 100),
        edge_label: Color::new(255, 180, 100),
        start_end: Color::new(100, 80, 60),
    }
}

fn neon_theme() -> Theme {
    Theme {
        name: "neon".to_string(),
        node_bg: Color::new(20, 0, 30),
        node_fg: Color::new(255, 0, 255),
        edge: Color::new(0, 255, 127),
        edge_label: Color::new(0, 255, 255),
        start_end: Color::new(128, 0, 128),
    }
}

fn mono_theme() -> Theme {
    Theme {
        name: "mono".to_string(),
        node_bg: Color::new(0, 0, 0),
        node_fg: Color::new(255, 255, 255),
        edge: Color::new(192, 192, 192),
        edge_label: Color::new(192, 192, 192),
        start_end: Color::new(128, 128, 128),
    }
}

fn amber_theme() -> Theme {
    Theme {
        name: "amber".to_string(),
        node_bg: Color::new(30, 20, 0),
        node_fg: Color::new(255, 192, 0),
        edge: Color::new(255, 128, 0),
        edge_label: Color::new(255, 192, 0),
        start_end: Color::new(128, 96, 0),
    }
}

fn phosphor_theme() -> Theme {
    Theme {
        name: "phosphor".to_string(),
        node_bg: Color::new(0, 10, 0),
        node_fg: Color::new(0, 255, 0),
        edge: Color::new(0, 200, 0),
        edge_label: Color::new(0, 255, 0),
        start_end: Color::new(0, 128, 0),
    }
}

/// ANSI escape code helpers
pub mod ansi {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const ITALIC: &str = "\x1b[3m";
    
    /// Move cursor
    pub fn cursor_up(n: u16) -> String { format!("\x1b[{}A", n) }
    pub fn cursor_down(n: u16) -> String { format!("\x1b[{}B", n) }
    pub fn cursor_forward(n: u16) -> String { format!("\x1b[{}C", n) }
    pub fn cursor_back(n: u16) -> String { format!("\x1b[{}D", n) }
    pub fn cursor_position(row: u16, col: u16) -> String { format!("\x1b[{};{}H", row, col) }
    
    /// Clear screen
    pub fn clear_screen() -> String { "\x1b[2J".to_string() }
    pub fn clear_line() -> String { "\x1b[2K".to_string() }
}

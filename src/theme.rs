//! Terminal themes for colored output

use serde::{Deserialize, Serialize};

/// How much color a render should emit. Resolved by the CLI from `--color`,
/// stdout TTY detection, `NO_COLOR`, and `COLORTERM`, then handed to the
/// renderer. `None` produces byte-for-byte monochrome output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorMode {
    #[default]
    None,
    Ansi256,
    TrueColor,
}

/// Color codes for terminals
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
            if gray < 8 {
                return 16;
            }
            if gray > 248 {
                return 231;
            }
            // Map gray 8..=248 onto the 24-step grayscale ramp (indices
            // 232..=255). Multiply before dividing so the step isn't truncated
            // to zero.
            return 232 + ((gray as u16 - 8) * 24 / 247) as u8;
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

    /// Foreground SGR escape for the given color mode. Returns an empty string
    /// for `ColorMode::None` so callers can prepend it unconditionally.
    pub fn fg(&self, mode: ColorMode) -> String {
        match mode {
            ColorMode::None => String::new(),
            ColorMode::Ansi256 => format!("\x1b[38;5;{}m", self.to_ansi256()),
            ColorMode::TrueColor => format!("\x1b[38;2;{};{};{}m", self.r, self.g, self.b),
        }
    }
}

/// Theme for diagram rendering.
///
/// Each role is `Option<Color>`: `Some(c)` paints that role with `c`, `None`
/// inherits the terminal's own color. The `default` theme leaves every role
/// `None`, so selecting it is a no-op even when color is enabled.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    /// Reserved: box-interior fill. Not painted yet (foreground-only).
    pub node_bg: Option<Color>,
    pub node_fg: Option<Color>,
    pub edge: Option<Color>,
    pub edge_label: Option<Color>,
    pub start_end: Option<Color>,
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
    // Every role inherits the terminal's own colors — selecting `default` is a
    // no-op even when color is enabled.
    Theme {
        name: "default".to_string(),
        node_bg: None,
        node_fg: None,
        edge: None,
        edge_label: None,
        start_end: None,
    }
}

fn terra_theme() -> Theme {
    Theme {
        name: "terra".to_string(),
        node_bg: Some(Color::new(45, 35, 25)),
        node_fg: Some(Color::new(255, 220, 180)),
        edge: Some(Color::new(255, 180, 100)),
        edge_label: Some(Color::new(255, 180, 100)),
        start_end: Some(Color::new(100, 80, 60)),
    }
}

fn neon_theme() -> Theme {
    Theme {
        name: "neon".to_string(),
        node_bg: Some(Color::new(20, 0, 30)),
        node_fg: Some(Color::new(255, 0, 255)),
        edge: Some(Color::new(0, 255, 127)),
        edge_label: Some(Color::new(0, 255, 255)),
        start_end: Some(Color::new(128, 0, 128)),
    }
}

fn mono_theme() -> Theme {
    Theme {
        name: "mono".to_string(),
        node_bg: Some(Color::new(0, 0, 0)),
        node_fg: Some(Color::new(255, 255, 255)),
        edge: Some(Color::new(192, 192, 192)),
        edge_label: Some(Color::new(192, 192, 192)),
        start_end: Some(Color::new(128, 128, 128)),
    }
}

fn amber_theme() -> Theme {
    Theme {
        name: "amber".to_string(),
        node_bg: Some(Color::new(30, 20, 0)),
        node_fg: Some(Color::new(255, 192, 0)),
        edge: Some(Color::new(255, 128, 0)),
        edge_label: Some(Color::new(255, 192, 0)),
        start_end: Some(Color::new(128, 96, 0)),
    }
}

fn phosphor_theme() -> Theme {
    Theme {
        name: "phosphor".to_string(),
        node_bg: Some(Color::new(0, 10, 0)),
        node_fg: Some(Color::new(0, 255, 0)),
        edge: Some(Color::new(0, 200, 0)),
        edge_label: Some(Color::new(0, 255, 0)),
        start_end: Some(Color::new(0, 128, 0)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fg_escape_depends_on_color_mode() {
        let c = Color::new(255, 0, 255);
        // None emits nothing — output stays byte-for-byte monochrome.
        assert_eq!(c.fg(ColorMode::None), "");
        // TrueColor emits the exact RGB triple.
        assert_eq!(c.fg(ColorMode::TrueColor), "\x1b[38;2;255;0;255m");
        // Ansi256 emits the quantized index.
        assert_eq!(
            c.fg(ColorMode::Ansi256),
            format!("\x1b[38;5;{}m", c.to_ansi256())
        );
    }

    #[test]
    fn to_ansi256_does_not_collapse_grays_to_near_black() {
        // Regression: the grayscale branch used integer division by 247, which
        // truncated to 0 for every gray <= 248, mapping them all to index 232
        // (near-black). A light gray must land high in the 232..=255 ramp and
        // be brighter than a dark gray.
        let dark = Color::new(40, 40, 40).to_ansi256();
        let light = Color::new(192, 192, 192).to_ansi256();
        assert!(
            light > dark,
            "light gray ({light}) should map brighter than dark gray ({dark})"
        );
        assert!(
            light > 240,
            "192-gray should sit high in the grayscale ramp, got {light}"
        );
    }
}

/// ANSI escape code helpers
pub mod ansi {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const ITALIC: &str = "\x1b[3m";

    /// Move cursor
    pub fn cursor_up(n: u16) -> String {
        format!("\x1b[{}A", n)
    }
    pub fn cursor_down(n: u16) -> String {
        format!("\x1b[{}B", n)
    }
    pub fn cursor_forward(n: u16) -> String {
        format!("\x1b[{}C", n)
    }
    pub fn cursor_back(n: u16) -> String {
        format!("\x1b[{}D", n)
    }
    pub fn cursor_position(row: u16, col: u16) -> String {
        format!("\x1b[{};{}H", row, col)
    }

    /// Clear screen
    pub fn clear_screen() -> String {
        "\x1b[2J".to_string()
    }
    pub fn clear_line() -> String {
        "\x1b[2K".to_string()
    }
}

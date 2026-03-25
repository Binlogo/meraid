//! Terminal renderer for diagrams

use crate::diagram::{Diagram, DiagramType, EdgeStyle, Node};
use crate::layout::{LayoutResult, Position};
use crate::theme::Theme;

/// Renderer for terminal output
pub struct Renderer {
    theme: Theme,
    ascii_only: bool,
    padding_x: usize,
    padding_y: usize,
}

impl Renderer {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            ascii_only: false,
            padding_x: 4,
            padding_y: 1,
        }
    }
    
    pub fn ascii_only(mut self, ascii: bool) -> Self {
        self.ascii_only = ascii;
        self
    }
    
    pub fn padding(mut self, x: usize, y: usize) -> Self {
        self.padding_x = x;
        self.padding_y = y;
        self
    }
    
    /// Render diagram to string
    pub fn render(&self, diagram: &Diagram, layout: &LayoutResult) -> String {
        match diagram.diagram_type {
            DiagramType::Flowchart => self.render_flowchart(diagram, layout),
            DiagramType::Sequence => self.render_sequence(diagram, layout),
            DiagramType::Class => self.render_class(diagram, layout),
            DiagramType::State => self.render_state(diagram, layout),
            DiagramType::Pie => self.render_pie(diagram, layout),
            _ => self.render_flowchart(diagram, layout),
        }
    }
    
    fn render_flowchart(&self, diagram: &Diagram, layout: &LayoutResult) -> String {
        let mut output = String::new();
        
        // Create canvas
        let canvas_width = layout.width.max(80);
        let canvas_height = layout.height.max(20);
        let mut canvas: Vec<Vec<char>> = vec![
            vec![' '; canvas_width]
        ];
        
        // Ensure we have enough rows
        for _ in canvas.len()..canvas_height {
            canvas.push(vec![' '; layout.width.max(80)]);
        }
        
        // Draw nodes
        for node in &diagram.nodes {
            if let Some(pos) = layout.positions.get(&node.id) {
                self.draw_node(&mut canvas, pos, node.get_label());
            }
        }
        
        // Draw edges
        for edge in &diagram.edges {
            if let (Some(from_pos), Some(to_pos)) = (
                layout.positions.get(&edge.from),
                layout.positions.get(&edge.to)
            ) {
                self.draw_edge(&mut canvas, from_pos, to_pos, edge.label.as_deref());
            }
        }
        
        // Convert to string
        for row in &canvas {
            for ch in row {
                output.push(*ch);
            }
            output.push('\n');
        }
        
        output
    }
    
    fn render_sequence(&self, diagram: &Diagram, _layout: &LayoutResult) -> String {
        let mut output = String::new();
        
        // Draw participants
        let participant_width = 12;
        
        // Header
        for (i, participant) in diagram.participants.iter().enumerate() {
            let _x = i * (participant_width + 6);
            let padded = self.pad_string(participant, participant_width);
            output.push_str(&format!("{:^width$}", padded, width = participant_width));
            if i < diagram.participants.len() - 1 {
                output.push_str("  ");
            }
        }
        output.push_str("\n\n");
        
        // Draw edges/messages
        for edge in &diagram.edges {
            let from_idx = diagram.participants.iter().position(|p| p == &edge.from).unwrap_or(0);
            let to_idx = diagram.participants.iter().position(|p| p == &edge.to).unwrap_or(0);
            
            let arrow_str = if edge.style == EdgeStyle::Solid { "──▶" } else { "──▶" };
            
            if from_idx < to_idx {
                // Left to right
                output.push_str(&format!("{:>width$}", format!("──{}", arrow_str), width = from_idx * (participant_width + 2) + participant_width / 2));
            } else {
                // Right to left
                let reverse_arrow = if edge.style == EdgeStyle::Solid { "◀──" } else { "◀──" };
                output.push_str(&format!("{:width$}", format!("{}{}", reverse_arrow, arrow_str), width = to_idx * (participant_width + 2) + participant_width / 2));
            }
            
            if let Some(label) = &edge.label {
                output.push_str(&format!(" {}\n", label));
            } else {
                output.push('\n');
            }
        }
        
        output
    }
    
    fn render_class(&self, diagram: &Diagram, layout: &LayoutResult) -> String {
        let mut output = String::new();
        
        for node in &diagram.nodes {
            if let Some(_pos) = layout.positions.get(&node.id) {
                // Draw class box
                output.push_str(&"┌".to_string());
                output.push_str(&"─".repeat(16));
                output.push_str(&"┐".to_string());
                output.push('\n');
                
                // Class name
                output.push_str(&"│");
                output.push_str(&format!("{:^16}", node.get_label()));
                output.push_str(&"│");
                output.push('\n');
                
                // Divider
                output.push_str(&"├".to_string());
                output.push_str(&"─".repeat(16));
                output.push_str(&"┤");
                output.push('\n');
                
                // Empty body (simplified)
                output.push_str(&"│".repeat(17));
                output.push('\n');
                output.push_str(&"└".to_string());
                output.push_str(&"─".repeat(16));
                output.push_str(&"┘".to_string());
                output.push('\n');
                output.push('\n');
            }
        }
        
        // Draw relationships
        for rel in &diagram.relationships {
            let arrow = match rel.rel_type.as_str() {
                "<|--" => "◄───",
                "*--" => "●───",
                "o--" => "○───",
                _ => "───",
            };
            output.push_str(&format!("{} {}\n", rel.from, arrow));
        }
        
        output
    }
    
    fn render_state(&self, diagram: &Diagram, _layout: &LayoutResult) -> String {
        let mut output = String::new();
        
        for edge in &diagram.edges {
            let from = if edge.from == "[*]" { "●".to_string() } else { edge.from.clone() };
            let to = if edge.to == "[*]" { "◉".to_string() } else { edge.to.clone() };
            
            output.push_str(&from);
            output.push_str(" ──▶ ");
            output.push_str(&to);
            
            if let Some(label) = &edge.label {
                output.push_str(&format!(" : {}", label));
            }
            output.push('\n');
        }
        
        output
    }
    
    fn render_pie(&self, diagram: &Diagram, _layout: &LayoutResult) -> String {
        let mut output = String::new();
        
        // Calculate total
        let total: f64 = diagram.nodes.iter()
            .map(|n| {
                let parts: Vec<&str> = n.label.split(':').collect();
                parts.get(1).and_then(|s| s.trim().parse::<f64>().ok()).unwrap_or(0.0)
            })
            .sum();
        
        if total == 0.0 {
            return "No data to display".to_string();
        }
        
        // Draw bar chart
        let max_bar_width = 40;
        
        for node in &diagram.nodes {
            let parts: Vec<&str> = node.label.split(':').collect();
            let label_str = parts.get(0).map(|s| *s).unwrap_or("");
            let value: f64 = parts.get(1).and_then(|s| s.trim().parse::<f64>().ok()).unwrap_or(0.0);
            
            let percentage = value / total;
            let bar_chars = (percentage * max_bar_width as f64) as usize;
            
            output.push_str(label_str);
            output.push_str("┃");
            output.push_str(&"█".repeat(bar_chars));
            output.push_str(&format!(" {:.1}%\n", percentage * 100.0));
        }
        
        output
    }
    
    fn draw_node(&self, canvas: &mut Vec<Vec<char>>, pos: &Position, label: &str) {
        let px = pos.x;
        let py = pos.y;
        let w = pos.width;
        let h = pos.height;
        
        // Ensure canvas is large enough
        let required_height = py + h + 2;
        if canvas.len() < required_height {
            canvas.resize(required_height, vec![' '; canvas[0].len()]);
        }
        
        let chars = if self.ascii_only {
            BoxChars::ascii()
        } else {
            BoxChars::unicode()
        };
        
        // Top border
        if px + w < canvas[0].len() {
            canvas[py][px] = chars.top_left;
            for x in (px + 1)..(px + w - 1) {
                canvas[py][x] = chars.horizontal;
            }
            canvas[py][px + w - 1] = chars.top_right;
        }
        
        // Bottom border
        if py + h < canvas.len() && px + w < canvas[0].len() {
            canvas[py + h][px] = chars.bottom_left;
            for x in (px + 1)..(px + w - 1) {
                canvas[py + h][x] = chars.horizontal;
            }
            canvas[py + h][px + w - 1] = chars.bottom_right;
        }
        
        // Vertical borders and content
        for y in (py + 1)..(py + h) {
            if y < canvas.len() && px < canvas[0].len() {
                canvas[y][px] = chars.vertical;
            }
            if y < canvas.len() && px + w - 1 < canvas[0].len() {
                canvas[y][px + w - 1] = chars.vertical;
            }
        }
        
        // Label (centered)
        let label_y = py + h / 2;
        if label_y < canvas.len() {
            let label_x = px + 1;
            let padded = self.pad_string(label, w - 2);
            for (i, ch) in padded.chars().enumerate() {
                if label_x + i < canvas[0].len() {
                    canvas[label_y][label_x + i] = ch;
                }
            }
        }
    }
    
    fn draw_edge(&self, canvas: &mut Vec<Vec<char>>, from: &Position, to: &Position, _label: Option<&str>) {
        // Simple horizontal/vertical edge routing
        let from_x = from.x + from.width;
        let from_y = from.y + from.height / 2;
        let to_x = to.x;
        let _to_y = to.y + to.height / 2;
        
        // Horizontal line
        let start_x = from_x.min(to_x);
        let end_x = from_x.max(to_x);
        
        for x in (start_x + 1)..end_x {
            if x < canvas[0].len() && from_y < canvas.len() {
                canvas[from_y][x] = if self.ascii_only { '-' } else { '─' };
            }
        }
        
        // Arrow
        if to_x > from_x && to_x - 1 < canvas[0].len() && from_y < canvas.len() {
            canvas[from_y][to_x - 1] = if self.ascii_only { '>' } else { '▶' };
        }
    }
    
    fn pad_string(&self, s: &str, width: usize) -> String {
        if s.len() >= width {
            s.chars().take(width).collect()
        } else {
            let left = (width - s.len()) / 2;
            let right = width - s.len() - left;
            format!("{}{}{}", " ".repeat(left), s, " ".repeat(right))
        }
    }
}

/// Box drawing characters
struct BoxChars {
    top_left: char,
    top_right: char,
    bottom_left: char,
    bottom_right: char,
    horizontal: char,
    vertical: char,
}

impl BoxChars {
    fn unicode() -> Self {
        Self {
            top_left: '┌',
            top_right: '┐',
            bottom_left: '└',
            bottom_right: '┘',
            horizontal: '─',
            vertical: '│',
        }
    }
    
    fn ascii() -> Self {
        Self {
            top_left: '+',
            top_right: '+',
            bottom_left: '+',
            bottom_right: '+',
            horizontal: '-',
            vertical: '|',
        }
    }
}

impl Node {
    fn get_label(&self) -> &str {
        &self.label
    }
}

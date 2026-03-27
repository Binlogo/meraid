//! Terminal renderer for diagrams

use crate::diagram::{Diagram, DiagramType, EdgeStyle, Entity, Node};
use crate::layout::{LayoutResult, Position};
use crate::theme::Theme;

/// Calculate the display width of a string in a terminal.
/// - ASCII characters: 1 cell width
/// - CJK (Chinese, Japanese, Korean): 2 cell widths
/// - Other wide characters: 2 cell widths
/// - Control characters: 0 cell width
fn str_width(s: &str) -> usize {
    s.chars().map(|c| {
        if c.is_ascii() {
            1
        } else if is_cjk(c) {
            2
        } else if c.is_whitespace() {
            1
        } else {
            // Default for other characters
            2
        }
    }).sum()
}

/// Check if a character is CJK (Chinese, Japanese, Korean)
fn is_cjk(c: char) -> bool {
    matches!(c,
        '\u{4E00}'..='\u{9FFF}' |  // CJK Unified Ideographs
        '\u{3400}'..='\u{4DBF}' |  // CJK Unified Ideographs Extension A
        '\u{F900}'..='\u{FAFF}' |  // CJK Compatibility Ideographs
        '\u{3000}'..='\u{303F}' |  // CJK Symbols and Punctuation
        '\u{FF00}'..='\u{FFEF}' |  // Fullwidth Forms
        '\u{3040}'..='\u{309F}' |  // Hiragana
        '\u{30A0}'..='\u{30FF}' |  // Katakana
        '\u{AC00}'..='\u{D7AF}'    // Hangul Syllables
    )
}

/// Renderer for terminal output
#[allow(dead_code)]
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
            DiagramType::ER => self.render_er(diagram, layout),
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
            
            let arrow_str = match edge.style {
                EdgeStyle::Solid => "──▶",
                EdgeStyle::Dotted => "┄┄▶",
                _ => "──▶",
            };
            
            if from_idx < to_idx {
                // Left to right
                output.push_str(&format!("{:>width$}", format!("──{}", arrow_str), width = from_idx * (participant_width + 2) + participant_width / 2));
            } else {
                // Right to left
                let reverse_arrow = match edge.style {
                    EdgeStyle::Solid => "◀──",
                    EdgeStyle::Dotted => "◀┄┄",
                    _ => "◀──",
                };
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
                // Calculate box width based on content (using display width for CJK support)
                let mut max_width = str_width(node.get_label());
                for member in &node.members {
                    let prefix = match member.visibility {
                        crate::diagram::Visibility::Public => "+",
                        crate::diagram::Visibility::Private => "-",
                        crate::diagram::Visibility::Protected => "#",
                        crate::diagram::Visibility::Package => "~",
                    };
                    let member_str = format!("{} {}", prefix, member.name);
                    max_width = max_width.max(str_width(&member_str));
                }
                let box_width = max_width.clamp(16, 40);

                // Draw class box top
                output.push('┌');
                output.push_str(&"─".repeat(box_width));
                output.push_str("┐\n");
                
                // Class name (centered)
                output.push('│');
                output.push_str(&format!("{:^width$}", node.get_label(), width = box_width));
                output.push_str("│\n");
                
                // Divider
                output.push('├');
                output.push_str(&"─".repeat(box_width));
                output.push_str("┤\n");
                
                // Members - each on separate line
                if node.members.is_empty() {
                    output.push('│');
                    output.push_str(&" ".repeat(box_width));
                    output.push_str("│\n");
                } else {
                    for member in &node.members {
                        let prefix = match member.visibility {
                            crate::diagram::Visibility::Public => "+",
                            crate::diagram::Visibility::Private => "-",
                            crate::diagram::Visibility::Protected => "#",
                            crate::diagram::Visibility::Package => "~",
                        };
                        let suffix = match member.member_type {
                            crate::diagram::MemberType::Field => "",
                            crate::diagram::MemberType::Method => "()",
                        };
                        let member_str = format!("{}{}{}", prefix, member.name, suffix);
                        output.push('│');
                        output.push_str(&format!("{:<width$}", member_str, width = box_width));
                        output.push_str("│\n");
                    }
                }
                
                // Bottom
                output.push('└');
                output.push_str(&"─".repeat(box_width));
                output.push_str("┘\n");
                output.push('\n');
            }
        }
        
        // Draw relationships
        for rel in &diagram.relationships {
            let arrow = match rel.rel_type.as_str() {
                "<|--" => "◄───",
                "*--" => "●───",
                "o--" => "○───",
                "--|>" => "───►",
                "..>" => "····▶",
                "..|>" => "····►",
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
            let label_str = parts.first().copied().unwrap_or("");
            let value: f64 = parts.get(1).and_then(|s| s.trim().parse::<f64>().ok()).unwrap_or(0.0);
            
            let percentage = value / total;
            let bar_chars = (percentage * max_bar_width as f64) as usize;
            
            output.push_str(label_str);
            output.push('┃');
            output.push_str(&"█".repeat(bar_chars));
            output.push_str(&format!(" {:.1}%\n", percentage * 100.0));
        }
        
        output
    }

    fn render_er(&self, diagram: &Diagram, _layout: &LayoutResult) -> String {
        let mut output = String::new();
        
        // Render entities
        for entity in &diagram.entities {
            let box_width = self.calculate_er_box_width(entity);
            
            // Top border
            output.push('┌');
            output.push_str(&"─".repeat(box_width));
            output.push_str("┐\n");
            
            // Entity name (centered)
            output.push('│');
            output.push_str(&format!("{:^width$}", entity.name, width = box_width));
            output.push_str("│\n");
            
            // Divider
            output.push('├');
            output.push_str(&"─".repeat(box_width));
            output.push_str("┤\n");
            
            // Attributes
            for attr in &entity.attributes {
                let pk_marker = if attr.is_primary_key { "PK" } else { "  " };
                let fk_marker = if attr.is_foreign_key { "FK" } else { "  " };
                let attr_line = format!("{} {} : {}", pk_marker, fk_marker, attr.name);
                output.push('│');
                output.push_str(&format!("{:<width$}", attr_line, width = box_width));
                output.push_str("│\n");
            }
            
            // Bottom border
            output.push('└');
            output.push_str(&"─".repeat(box_width));
            output.push_str("┘\n");
            output.push('\n');
        }
        
        // Render relationships
        for rel in &diagram.relationships {
            // Parse relationship type like "||--o{" into left and right cardinality
            let (left_card, right_card) = if let Some(dash_pos) = rel.rel_type.find("--") {
                let left = &rel.rel_type[..dash_pos];
                let right = &rel.rel_type[dash_pos + 2..];
                (left, right)
            } else {
                ("--", "--")
            };
            output.push_str(&format!("{} {}--{} {}\n", rel.from, left_card, right_card, rel.to));
        }
        
        output
    }
    
    fn calculate_er_box_width(&self, entity: &Entity) -> usize {
        let mut max_width = str_width(&entity.name);
        for attr in &entity.attributes {
            let attr_str = format!("{}  {} : {}", 
                if attr.is_primary_key { "PK" } else { "  " },
                if attr.is_foreign_key { "FK" } else { "  " },
                attr.name
            );
            max_width = max_width.max(str_width(&attr_str));
        }
        max_width.clamp(20, 50)
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
            for _x in (px + 1)..(px + w - 1) {
                canvas[py][_x] = chars.horizontal;
            }
            canvas[py][px + w - 1] = chars.top_right;
        }
        
        // Bottom border
        if py + h < canvas.len() && px + w < canvas[0].len() {
            canvas[py + h][px] = chars.bottom_left;
            for _x in (px + 1)..(px + w - 1) {
                canvas[py + h][_x] = chars.horizontal;
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
    
    fn draw_edge(&self, canvas: &mut [Vec<char>], from: &Position, to: &Position, label: Option<&str>) {
        // Manhattan-style edge routing (horizontal then vertical, or vice versa)
        let from_x = from.x + from.width;
        let from_y = from.y + from.height / 2;
        let to_x = to.x;
        let to_y = to.y + to.height / 2;
        
        // Determine routing direction based on relative positions
        let dy = to_y.abs_diff(from_y);
        let dx = to_x.abs_diff(from_x);
        let (mid_x, mid_y, vertical_first) = if dy > dx {
            // Vertical distance is greater, route vertically first
            (from_x, to_y, true)
        } else {
            // Route horizontally first (default)
            (to_x, from_y, false)
        };
        
        if vertical_first {
            // Vertical segment first, then horizontal
            // From -> mid point (vertical)
            let vert_start = from_y.min(mid_y);
            let vert_end = from_y.max(mid_y);
            for y in (vert_start + 1)..vert_end {
                if y < canvas.len() && from_x < canvas[0].len() {
                    canvas[y][from_x] = if self.ascii_only { '|' } else { '│' };
                }
            }
            
            // Horizontal segment
            let horiz_start = from_x.min(to_x);
            let horiz_end = from_x.max(to_x);
            for x in (horiz_start + 1)..horiz_end {
                if mid_y < canvas.len() && x < canvas[0].len() {
                    canvas[mid_y][x] = if self.ascii_only { '-' } else { '─' };
                }
            }
            
            // Vertical segment to target (mid_y -> to_y)
            let vert_start = mid_y.min(to_y);
            let vert_end = mid_y.max(to_y);
            for y in (vert_start + 1)..vert_end {
                if y < canvas.len() && to_x < canvas[0].len() {
                    canvas[y][to_x] = if self.ascii_only { '|' } else { '│' };
                }
            }
        } else {
            // Horizontal first, then vertical
            // From -> mid point (horizontal)
            let horiz_start = from_x.min(mid_x);
            let horiz_end = from_x.max(mid_x);
            for x in (horiz_start + 1)..horiz_end {
                if from_y < canvas.len() && x < canvas[0].len() {
                    canvas[from_y][x] = if self.ascii_only { '-' } else { '─' };
                }
            }
            
            // Vertical segment
            let vert_start = from_y.min(to_y);
            let vert_end = from_y.max(to_y);
            for y in (vert_start + 1)..vert_end {
                if y < canvas.len() && mid_x < canvas[0].len() {
                    canvas[y][mid_x] = if self.ascii_only { '|' } else { '│' };
                }
            }
            
            // Horizontal segment to target (mid_x -> to_x)
            let horiz_start = mid_x.min(to_x);
            let horiz_end = mid_x.max(to_x);
            for x in (horiz_start + 1)..horiz_end {
                if to_y < canvas.len() && x < canvas[0].len() {
                    canvas[to_y][x] = if self.ascii_only { '-' } else { '─' };
                }
            }
        }
        
        // Draw arrow head at destination
        if to_x > from_x {
            // Arrow pointing right
            if to_y < canvas.len() && to_x > 0 && to_x < canvas[0].len() {
                canvas[to_y][to_x - 1] = if self.ascii_only { '>' } else { '▶' };
            }
        } else if to_x < from_x {
            // Arrow pointing left
            if to_y < canvas.len() && to_x + 1 < canvas[0].len() {
                canvas[to_y][to_x + 1] = if self.ascii_only { '<' } else { '◀' };
            }
        }
        
        // Draw edge label if present
        if let Some(label_text) = label {
            let label_x = (from_x + to_x) / 2;
            let label_y = if vertical_first { mid_y } else { (from_y + to_y) / 2 };
            if label_y < canvas.len() {
                let padded = self.pad_string(label_text, 8);
                for (i, ch) in padded.chars().enumerate() {
                    if label_x + i < canvas[0].len() {
                        canvas[label_y][label_x + i] = ch;
                    }
                }
            }
        }
    }
    
    fn pad_string(&self, s: &str, width: usize) -> String {
        let s_width = str_width(s);
        if s_width >= width {
            // Truncate to display width
            let mut result = String::new();
            let mut current_width = 0;
            for c in s.chars() {
                let c_width = if c.is_ascii() { 1 } else if is_cjk(c) { 2 } else { 1 };
                if current_width + c_width > width {
                    break;
                }
                result.push(c);
                current_width += c_width;
            }
            result
        } else {
            let left = (width - s_width) / 2;
            let right = width - s_width - left;
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

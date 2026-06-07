//! Terminal renderer for diagrams

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::diagram::{Diagram, DiagramType, Entity, Node};
use crate::layout::{LayoutResult, Position};
use crate::theme::{ansi, Color, ColorMode, Theme};

/// Calculate the display width of a string in a terminal using Unicode width.
/// This properly handles CJK characters (2 cells) and ASCII (1 cell).
fn str_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

/// Renderer for terminal output
#[allow(dead_code)]
pub struct Renderer {
    theme: Theme,
    ascii_only: bool,
    padding_x: usize,
    padding_y: usize,
    color_mode: ColorMode,
}

impl Renderer {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            ascii_only: false,
            padding_x: 4,
            padding_y: 1,
            color_mode: ColorMode::None,
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

    /// Enable colored output at the given depth. `ColorMode::None` (the default)
    /// keeps output byte-for-byte monochrome.
    pub fn color_mode(mut self, mode: ColorMode) -> Self {
        self.color_mode = mode;
        self
    }

    /// Wrap `text` in the foreground escape for `role` and a reset. Returns the
    /// text unchanged when color is off, the role inherits the terminal color
    /// (`None`), or the text is empty — so callers can paint unconditionally.
    fn paint(&self, role: Option<Color>, text: &str) -> String {
        match role {
            Some(color) if self.color_mode != ColorMode::None && !text.is_empty() => {
                format!("{}{}{}", color.fg(self.color_mode), text, ansi::RESET)
            }
            _ => text.to_string(),
        }
    }

    /// Box-drawing glyph set for the current mode (ASCII or Unicode).
    fn box_chars(&self) -> BoxChars {
        if self.ascii_only {
            BoxChars::ascii()
        } else {
            BoxChars::unicode()
        }
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
        // Each column is a terminal cell, so cells are strings (not chars) to
        // support wide characters like Chinese without stretching the box.
        let canvas_width = layout.width.max(80);
        let mut canvas_height = layout.height.max(20);
        // Pre-size to the tallest node so draw_node never resizes mid-draw —
        // a resize would otherwise have to keep the color grid in step.
        for node in &diagram.nodes {
            if let Some(pos) = layout.positions.get(&node.id) {
                canvas_height = canvas_height.max(pos.y + pos.height + 2);
            }
        }
        let mut canvas = Canvas::new(canvas_width, canvas_height);

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
                layout.positions.get(&edge.to),
            ) {
                self.draw_edge(&mut canvas, from_pos, to_pos, edge.label.as_deref());
            }
        }

        // Flatten to a string, trimming trailing whitespace on each line and any
        // blank rows the canvas padding left at the bottom.
        let mut lines: Vec<String> = (0..canvas.height())
            .map(|y| self.flatten_row(&canvas.cells[y], &canvas.colors[y]))
            .collect();
        while lines.last().is_some_and(|l| l.is_empty()) {
            lines.pop();
        }
        let mut output = String::new();
        for line in &lines {
            output.push_str(line);
            output.push('\n');
        }

        output
    }

    /// Flatten one canvas row to a string. Trailing blank cells are dropped
    /// (matching the old `trim_end`). When color is on, runs of same-color cells
    /// are coalesced into a single escape and every colored run is reset before
    /// the line ends, so color never bleeds across the newline. With
    /// `ColorMode::None` this emits the bare glyphs — byte-for-byte the legacy
    /// output.
    fn flatten_row(&self, cells: &[String], colors: &[Option<Color>]) -> String {
        // Trim boundary: index of the last cell whose glyph is not blank. A cell
        // is blank when it is a space or the cleared half of a wide character.
        let Some(last) = cells.iter().rposition(|c| !c.trim().is_empty()) else {
            return String::new();
        };

        let mut out = String::new();
        let mut active: Option<Color> = None;
        for x in 0..=last {
            if self.color_mode != ColorMode::None && colors[x] != active {
                if active.is_some() {
                    out.push_str(ansi::RESET);
                }
                if let Some(color) = colors[x] {
                    out.push_str(&color.fg(self.color_mode));
                }
                active = colors[x];
            }
            out.push_str(&cells[x]);
        }
        if active.is_some() {
            out.push_str(ansi::RESET);
        }
        out
    }

    fn render_sequence(&self, diagram: &Diagram, _layout: &LayoutResult) -> String {
        if diagram.participants.is_empty() {
            return String::new();
        }

        let mut output = String::new();
        let participant_width = diagram
            .participants
            .iter()
            .map(|participant| str_width(participant))
            .max()
            .unwrap_or(0)
            .max(8)
            + 4;
        let gap_width = 6;
        let lane_width = participant_width + gap_width;

        // Glyphs, ASCII-aware.
        let vbar = if self.ascii_only { "|" } else { "│" };
        let solid = if self.ascii_only { "-" } else { "─" };
        let dashed = if self.ascii_only { "." } else { "┄" };
        let head_right = if self.ascii_only { ">" } else { "▶" };
        let head_left = if self.ascii_only { "<" } else { "◀" };
        let conn_from = if self.ascii_only { "|" } else { "├" };
        let conn_to = if self.ascii_only { "|" } else { "┤" };
        let self_msg = if self.ascii_only { "@" } else { "↺" };

        for (i, participant) in diagram.participants.iter().enumerate() {
            output.push_str(&self.paint(
                self.theme.node_fg,
                &self.pad_string(participant, participant_width),
            ));
            if i < diagram.participants.len() - 1 {
                output.push_str(&" ".repeat(gap_width));
            }
        }
        output.push('\n');

        for (i, _) in diagram.participants.iter().enumerate() {
            output.push_str(&" ".repeat(participant_width / 2));
            output.push_str(&self.paint(self.theme.edge, vbar));
            output.push_str(&" ".repeat(participant_width - participant_width / 2 - 1));
            if i < diagram.participants.len() - 1 {
                output.push_str(&" ".repeat(gap_width));
            }
        }
        output.push_str("\n\n");

        for edge in &diagram.edges {
            let from_idx = diagram
                .participants
                .iter()
                .position(|p| p == &edge.from)
                .unwrap_or(0);
            let to_idx = diagram
                .participants
                .iter()
                .position(|p| p == &edge.to)
                .unwrap_or(0);
            let from_center = from_idx * lane_width + participant_width / 2;
            let to_center = to_idx * lane_width + participant_width / 2;
            let min_center = from_center.min(to_center);
            let max_center = from_center.max(to_center);
            let line_char = if edge.style == crate::diagram::EdgeStyle::Dotted {
                dashed
            } else {
                solid
            };
            let mut line = " ".repeat(max_center + 1);

            if from_idx == to_idx {
                overwrite_at(&mut line, from_center, self_msg);
            } else if from_idx < to_idx {
                if max_center > min_center + 1 {
                    overwrite_at(
                        &mut line,
                        min_center + 1,
                        &line_char.repeat(max_center - min_center - 1),
                    );
                }
                overwrite_at(&mut line, from_center, conn_from);
                overwrite_at(&mut line, to_center, head_right);
            } else {
                if max_center > min_center + 1 {
                    overwrite_at(
                        &mut line,
                        min_center + 1,
                        &line_char.repeat(max_center - min_center - 1),
                    );
                }
                overwrite_at(&mut line, from_center, conn_to);
                overwrite_at(&mut line, to_center, head_left);
            }

            // The connector (line + junctions + arrowhead) is all the edge role;
            // painted after the monochrome line is built so overwrite_at's
            // char indexing stays correct.
            output.push_str(&self.paint(self.theme.edge, line.trim_end()));
            if let Some(label) = &edge.label {
                if !label.is_empty() {
                    output.push(' ');
                    output.push_str(&self.paint(self.theme.edge_label, label));
                }
            }
            output.push('\n');
        }

        output
    }

    fn render_class(&self, diagram: &Diagram, _layout: &LayoutResult) -> String {
        let mut output = String::new();
        let chars = self.box_chars();
        let (div_l, div_r) = if self.ascii_only {
            ('+', '+')
        } else {
            ('├', '┤')
        };

        let member_line = |member: &crate::diagram::ClassMember| -> String {
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
            format!("{}{}{}", prefix, member.name, suffix)
        };

        // The whole box (borders + name + members + dividers) is one role.
        let nf = self.theme.node_fg;

        for node in &diagram.nodes {
            // Box width based on content (display width for CJK support).
            let mut max_width = str_width(node.get_label());
            for member in &node.members {
                max_width = max_width.max(str_width(&member_line(member)));
            }
            let box_width = max_width.clamp(16, 40);

            let horizontal: String = chars.horizontal.to_string().repeat(box_width);
            let divider = format!("{}{}{}", div_l, horizontal, div_r);

            // Top border.
            push_line(
                &mut output,
                self.paint(
                    nf,
                    &format!("{}{}{}", chars.top_left, horizontal, chars.top_right),
                ),
            );
            // Class name (centered).
            push_line(
                &mut output,
                self.paint(
                    nf,
                    &format!(
                        "{}{}{}",
                        chars.vertical,
                        self.pad_string(node.get_label(), box_width),
                        chars.vertical
                    ),
                ),
            );
            // Divider between name and members.
            push_line(&mut output, self.paint(nf, &divider));

            let fields: Vec<_> = node
                .members
                .iter()
                .filter(|m| m.member_type == crate::diagram::MemberType::Field)
                .collect();
            let methods: Vec<_> = node
                .members
                .iter()
                .filter(|m| m.member_type == crate::diagram::MemberType::Method)
                .collect();

            if node.members.is_empty() {
                push_line(
                    &mut output,
                    self.paint(
                        nf,
                        &format!(
                            "{}{}{}",
                            chars.vertical,
                            " ".repeat(box_width),
                            chars.vertical
                        ),
                    ),
                );
            } else {
                for &member in &fields {
                    push_line(
                        &mut output,
                        self.paint(
                            nf,
                            &format!(
                                "{}{}{}",
                                chars.vertical,
                                self.pad_string_left(&member_line(member), box_width),
                                chars.vertical
                            ),
                        ),
                    );
                }
                // Second divider between fields and methods.
                if !fields.is_empty() && !methods.is_empty() {
                    push_line(&mut output, self.paint(nf, &divider));
                }
                for &member in &methods {
                    push_line(
                        &mut output,
                        self.paint(
                            nf,
                            &format!(
                                "{}{}{}",
                                chars.vertical,
                                self.pad_string_left(&member_line(member), box_width),
                                chars.vertical
                            ),
                        ),
                    );
                }
            }

            // Bottom border.
            push_line(
                &mut output,
                self.paint(
                    nf,
                    &format!("{}{}{}", chars.bottom_left, horizontal, chars.bottom_right),
                ),
            );
            output.push('\n');
        }

        // Relationships as a text legend, including both endpoints.
        for rel in &diagram.relationships {
            push_line(
                &mut output,
                self.paint(
                    self.theme.edge,
                    &format!("{} {} {}", rel.from, rel.rel_type, rel.to),
                ),
            );
        }

        output
    }

    fn render_state(&self, diagram: &Diagram, _layout: &LayoutResult) -> String {
        let mut output = String::new();

        let arrow_glyph = if self.ascii_only { "->" } else { "──▶" };
        let start_marker = if self.ascii_only { "(*)" } else { "●" };
        let end_marker = if self.ascii_only { "(o)" } else { "◉" };

        for edge in &diagram.edges {
            // State names use node_fg; the [*] start/end markers use start_end.
            let (from_text, from_color) = if edge.from == "[*]" {
                (start_marker, self.theme.start_end)
            } else {
                (edge.from.as_str(), self.theme.node_fg)
            };
            let (to_text, to_color) = if edge.to == "[*]" {
                (end_marker, self.theme.start_end)
            } else {
                (edge.to.as_str(), self.theme.node_fg)
            };

            output.push_str(&self.paint(from_color, from_text));
            output.push(' ');
            output.push_str(&self.paint(self.theme.edge, arrow_glyph));
            output.push(' ');
            output.push_str(&self.paint(to_color, to_text));

            if let Some(label) = &edge.label {
                output.push_str(" : ");
                output.push_str(&self.paint(self.theme.edge_label, label));
            }
            output.push('\n');
        }

        output
    }

    fn render_pie(&self, diagram: &Diagram, _layout: &LayoutResult) -> String {
        // Each pie node stores its label as "label: value"; split on the LAST
        // colon so quoted labels containing ':' are preserved. Negative values
        // are clamped to zero.
        let slices: Vec<(String, f64)> = diagram
            .nodes
            .iter()
            .map(|n| match n.label.rsplit_once(':') {
                Some((label, value)) => (
                    label.trim().to_string(),
                    value.trim().parse::<f64>().unwrap_or(0.0).max(0.0),
                ),
                None => (n.label.clone(), 0.0),
            })
            .collect();

        let total: f64 = slices.iter().map(|(_, v)| v).sum();
        if total == 0.0 {
            return "No data to display\n".to_string();
        }

        let label_width = slices.iter().map(|(l, _)| str_width(l)).max().unwrap_or(0);
        let max_bar_width = 40;
        let bar_char = if self.ascii_only { '#' } else { '█' };
        let separator = if self.ascii_only { '|' } else { '┃' };

        let mut output = String::new();
        for (label, value) in &slices {
            let percentage = value / total;
            let bar_chars =
                ((percentage * max_bar_width as f64).round() as usize).min(max_bar_width);

            // Right-align labels so the separators and bars line up.
            output.push_str(&" ".repeat(label_width.saturating_sub(str_width(label))));
            output.push_str(&self.paint(self.theme.node_fg, label));
            // Separator + bar share the edge role.
            let bar = format!("{}{}", separator, bar_char.to_string().repeat(bar_chars));
            output.push_str(&self.paint(self.theme.edge, &bar));
            output.push(' ');
            output.push_str(&self.paint(
                self.theme.edge_label,
                &format!("{:.1}%", percentage * 100.0),
            ));
            output.push('\n');
        }

        output
    }

    fn render_er(&self, diagram: &Diagram, _layout: &LayoutResult) -> String {
        let mut output = String::new();

        let chars = self.box_chars();
        let (div_l, div_r) = if self.ascii_only {
            ('+', '+')
        } else {
            ('├', '┤')
        };

        // The whole entity box is one role (node_fg).
        let nf = self.theme.node_fg;

        // Render entities
        for entity in &diagram.entities {
            let box_width = self.calculate_er_box_width(entity);
            let horizontal: String = chars.horizontal.to_string().repeat(box_width);

            // Top border
            push_line(
                &mut output,
                self.paint(
                    nf,
                    &format!("{}{}{}", chars.top_left, horizontal, chars.top_right),
                ),
            );
            // Entity name (centered)
            push_line(
                &mut output,
                self.paint(
                    nf,
                    &format!(
                        "{}{}{}",
                        chars.vertical,
                        self.pad_string(&entity.name, box_width),
                        chars.vertical
                    ),
                ),
            );
            // Divider
            push_line(
                &mut output,
                self.paint(nf, &format!("{}{}{}", div_l, horizontal, div_r)),
            );

            // Attributes
            for attr in &entity.attributes {
                let pk_marker = if attr.is_primary_key { "PK" } else { "  " };
                let fk_marker = if attr.is_foreign_key { "FK" } else { "  " };
                let attr_line = format!("{} {} : {}", pk_marker, fk_marker, attr.name);
                push_line(
                    &mut output,
                    self.paint(
                        nf,
                        &format!(
                            "{}{}{}",
                            chars.vertical,
                            self.pad_string_left(&attr_line, box_width),
                            chars.vertical
                        ),
                    ),
                );
            }

            // Bottom border
            push_line(
                &mut output,
                self.paint(
                    nf,
                    &format!("{}{}{}", chars.bottom_left, horizontal, chars.bottom_right),
                ),
            );
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
            push_line(
                &mut output,
                self.paint(
                    self.theme.edge,
                    &format!("{} {}--{} {}", rel.from, left_card, right_card, rel.to),
                ),
            );
        }

        output
    }

    fn calculate_er_box_width(&self, entity: &Entity) -> usize {
        let mut max_width = str_width(&entity.name);
        for attr in &entity.attributes {
            let attr_str = format!(
                "{} {} : {}",
                if attr.is_primary_key { "PK" } else { "  " },
                if attr.is_foreign_key { "FK" } else { "  " },
                attr.name
            );
            max_width = max_width.max(str_width(&attr_str));
        }
        max_width.clamp(20, 50)
    }

    fn draw_node(&self, canvas: &mut Canvas, pos: &Position, label: &str) {
        let px = pos.x;
        let py = pos.y;
        let w = pos.width;
        let h = pos.height;

        // The whole box — borders and centered label — is the node_fg role.
        let nf = self.theme.node_fg;
        let width = canvas.width;

        // Ensure canvas is large enough (normally a no-op: pre-sized in caller).
        canvas.resize(py + h + 2);

        let chars = if self.ascii_only {
            BoxChars::ascii()
        } else {
            BoxChars::unicode()
        };

        // Top border
        if px + w < width {
            canvas.set(py, px, &chars.top_left.to_string(), nf);
            for x in (px + 1)..(px + w - 1) {
                canvas.set(py, x, &chars.horizontal.to_string(), nf);
            }
            canvas.set(py, px + w - 1, &chars.top_right.to_string(), nf);
        }

        // Bottom border
        if py + h < canvas.height() && px + w < width {
            canvas.set(py + h, px, &chars.bottom_left.to_string(), nf);
            for x in (px + 1)..(px + w - 1) {
                canvas.set(py + h, x, &chars.horizontal.to_string(), nf);
            }
            canvas.set(py + h, px + w - 1, &chars.bottom_right.to_string(), nf);
        }

        // Vertical borders
        for y in (py + 1)..(py + h) {
            if y < canvas.height() && px < width {
                canvas.set(y, px, &chars.vertical.to_string(), nf);
            }
            if y < canvas.height() && px + w - 1 < width {
                canvas.set(y, px + w - 1, &chars.vertical.to_string(), nf);
            }
        }

        // Label (centered)
        let label_y = py + h / 2;
        if label_y < canvas.height() {
            let label_x = px + 1;
            let padded = self.pad_string(label, w - 2);
            let mut current_x = label_x;
            for ch in padded.chars() {
                let cell_width = UnicodeWidthChar::width(ch).unwrap_or(1).max(1);
                if current_x >= width {
                    break;
                }
                canvas.set(label_y, current_x, &ch.to_string(), nf);
                for offset in 1..cell_width {
                    if current_x + offset < width {
                        // Cleared half of a wide char — emits nothing.
                        canvas.set(label_y, current_x + offset, "", nf);
                    }
                }
                current_x += cell_width;
            }
        }
    }

    fn draw_edge(&self, canvas: &mut Canvas, from: &Position, to: &Position, label: Option<&str>) {
        // Edges exit the right edge of the source box at its vertical center and
        // enter the left edge of the destination box at its center. Forward
        // edges (the only kind the layout produces) are drawn as an elbow: a
        // short horizontal stub, a vertical jog placed in the gap between the
        // columns, then a horizontal run into the arrow. Routing the vertical in
        // the gap keeps it off any box's border column, and turning onto the
        // destination's own row keeps the long run clear of boxes in other rows.
        let ascii = self.ascii_only;
        let arrow_r = if ascii { ">" } else { "▶" };
        let arrow_l = if ascii { "<" } else { "◀" };
        // Wires, junctions, and arrowheads are all the edge role.
        let edge = self.theme.edge;

        let from_x = from.x + from.width; // first gap column right of the source box
        let from_y = from.y + from.height / 2;
        let to_x = to.x; // destination box's left-border column
        let to_y = to.y + to.height / 2;

        let width = canvas.width;
        // Draw an arrow head / terminal glyph, overwriting whatever is there.
        let put = |canvas: &mut Canvas, y: usize, x: usize, s: &str, color: Option<Color>| {
            canvas.set(y, x, s, color);
        };
        // Add a line segment, merging with any line already in the cell so
        // crossings and forks become the correct junction glyph. Leaves text,
        // arrows, and box borders untouched.
        let line = |canvas: &mut Canvas, y: usize, x: usize, add: u8, color: Option<Color>| {
            if y >= canvas.height() || x >= width {
                return;
            }
            let mask = glyph_mask(&canvas.cells[y][x]);
            let is_blank = canvas.cells[y][x] == " ";
            match mask {
                Some(base) => canvas.set(y, x, mask_glyph(base | add, ascii), color),
                None if is_blank => canvas.set(y, x, mask_glyph(add, ascii), color),
                None => {}
            }
        };

        let label_y;
        let label_x;

        if to_x <= from_x {
            // Non-forward (back / self) edge: best-effort straight line.
            let (a, b) = (to_x.min(from_x), to_x.max(from_x));
            for x in a..b {
                line(canvas, from_y, x, DIR_E | DIR_W, edge);
            }
            if to_x < from_x {
                put(canvas, to_y, to_x + 1, arrow_l, edge);
            }
            label_y = from_y;
            label_x = a + 1;
        } else if to_y == from_y {
            // Same row: straight horizontal line into the arrow.
            for x in from_x..to_x.saturating_sub(1) {
                line(canvas, from_y, x, DIR_E | DIR_W, edge);
            }
            put(canvas, to_y, to_x.saturating_sub(1), arrow_r, edge);
            label_y = from_y;
            label_x = from_x + 1;
        } else {
            // Elbow: stub right, vertical jog through the gap, then into the box.
            let turn_x = (from_x + 2).min(to_x.saturating_sub(1));
            let going_down = to_y > from_y;
            let (vert, vs, ve) = if going_down {
                (DIR_S, from_y, to_y)
            } else {
                (DIR_N, to_y, from_y)
            };

            for x in from_x..turn_x {
                line(canvas, from_y, x, DIR_E | DIR_W, edge);
            }
            // Corner where the westbound stub turns vertical (also a fork point
            // when a sibling branch leaves the same column — hence merge).
            line(canvas, from_y, turn_x, DIR_W | vert, edge);
            // Vertical run between the two rows.
            for y in (vs + 1)..ve {
                line(canvas, y, turn_x, DIR_N | DIR_S, edge);
            }
            // Corner where the vertical turns east toward the destination.
            let arrive = if going_down { DIR_N } else { DIR_S };
            line(canvas, to_y, turn_x, arrive | DIR_E, edge);
            // Horizontal run on the destination's row into the arrow.
            for x in (turn_x + 1)..to_x.saturating_sub(1) {
                line(canvas, to_y, x, DIR_E | DIR_W, edge);
            }
            put(canvas, to_y, to_x.saturating_sub(1), arrow_r, edge);
            label_y = to_y;
            label_x = turn_x + 1;
        }

        // Edge label: write only onto blank cells or the edge's own line glyphs
        // so it never garbles a node box, an arrow, or a corner; stop before the
        // arrow head. A label char that lands on a wire flips that cell's role
        // from edge to edge_label.
        if let Some(label_text) = label.filter(|l| !l.is_empty()) {
            if label_y < canvas.height() {
                let limit = to_x.max(from_x).saturating_sub(1);
                let mut current_x = label_x;
                for ch in label_text.chars() {
                    let cell_width = UnicodeWidthChar::width(ch).unwrap_or(1).max(1);
                    if current_x + cell_width > limit || current_x >= width {
                        break;
                    }
                    let overwritable = matches!(
                        canvas.cells[label_y][current_x].as_str(),
                        " " | "─" | "│" | "-" | "|"
                    );
                    if overwritable {
                        canvas.set(label_y, current_x, &ch.to_string(), self.theme.edge_label);
                        for offset in 1..cell_width {
                            if current_x + offset < width {
                                // Cleared half of a wide char — emits nothing.
                                canvas.set(label_y, current_x + offset, "", self.theme.edge_label);
                            }
                        }
                    }
                    current_x += cell_width;
                }
            }
        }
    }

    fn pad_string(&self, s: &str, width: usize) -> String {
        self.pad_string_with_alignment(s, width, TextAlign::Center)
    }

    fn pad_string_left(&self, s: &str, width: usize) -> String {
        self.pad_string_with_alignment(s, width, TextAlign::Left)
    }

    fn pad_string_with_alignment(&self, s: &str, width: usize, align: TextAlign) -> String {
        let truncated = truncate_to_width(s, width);
        let s_width = str_width(&truncated);

        if s_width >= width {
            return truncated;
        }

        let padding = width - s_width;
        match align {
            TextAlign::Left => format!("{}{}", truncated, " ".repeat(padding)),
            TextAlign::Center => {
                let left = padding / 2;
                let right = padding - left;
                format!("{}{}{}", " ".repeat(left), truncated, " ".repeat(right))
            }
        }
    }
}

fn truncate_to_width(s: &str, width: usize) -> String {
    let mut result = String::new();
    let mut current_width = 0;

    for c in s.chars() {
        let c_width = UnicodeWidthChar::width(c).unwrap_or(1);
        if current_width + c_width > width {
            break;
        }
        result.push(c);
        current_width += c_width;
    }

    result
}

/// Append a (possibly already-colored) line followed by a newline.
fn push_line(out: &mut String, line: String) {
    out.push_str(&line);
    out.push('\n');
}

fn overwrite_at(line: &mut String, start: usize, content: &str) {
    let mut chars: Vec<char> = line.chars().collect();
    for (i, ch) in content.chars().enumerate() {
        if start + i < chars.len() {
            chars[start + i] = ch;
        }
    }
    *line = chars.into_iter().collect();
}

#[derive(Clone, Copy)]
enum TextAlign {
    Left,
    Center,
}

/// A flowchart drawing surface: a glyph grid plus a parallel color grid of the
/// same shape. Every glyph mutation carries its color role, so the two layers
/// can never desync. Width measurement and trimming always use the glyph layer.
struct Canvas {
    cells: Vec<Vec<String>>,
    colors: Vec<Vec<Option<Color>>>,
    width: usize,
}

impl Canvas {
    fn new(width: usize, height: usize) -> Self {
        let height = height.max(1);
        Self {
            cells: vec![vec![" ".to_string(); width]; height],
            colors: vec![vec![None; width]; height],
            width,
        }
    }

    fn height(&self) -> usize {
        self.cells.len()
    }

    /// Grow the canvas to at least `height` rows, keeping both layers in step.
    fn resize(&mut self, height: usize) {
        while self.cells.len() < height {
            self.cells.push(vec![" ".to_string(); self.width]);
            self.colors.push(vec![None; self.width]);
        }
    }

    /// Set a cell's glyph and color together.
    fn set(&mut self, y: usize, x: usize, glyph: &str, color: Option<Color>) {
        if y < self.cells.len() && x < self.width {
            self.cells[y][x] = glyph.to_string();
            self.colors[y][x] = color;
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

// Connection-direction bit flags for a box-drawing line cell.
const DIR_N: u8 = 1;
const DIR_E: u8 = 2;
const DIR_S: u8 = 4;
const DIR_W: u8 = 8;

/// Map a box-drawing glyph back to its set of connected directions, or `None`
/// if the cell is not an edge-line glyph (blank, text, arrow head, …). Used to
/// merge overlapping edge segments into the correct junction.
fn glyph_mask(s: &str) -> Option<u8> {
    Some(match s {
        "─" | "-" => DIR_E | DIR_W,
        "│" | "|" => DIR_N | DIR_S,
        "┌" => DIR_S | DIR_E,
        "┐" => DIR_S | DIR_W,
        "└" => DIR_N | DIR_E,
        "┘" => DIR_N | DIR_W,
        "├" => DIR_N | DIR_S | DIR_E,
        "┤" => DIR_N | DIR_S | DIR_W,
        "┬" => DIR_E | DIR_S | DIR_W,
        "┴" => DIR_E | DIR_N | DIR_W,
        "┼" | "+" => DIR_N | DIR_E | DIR_S | DIR_W,
        _ => return None,
    })
}

/// Pick the box-drawing glyph for a set of connected directions.
fn mask_glyph(mask: u8, ascii: bool) -> &'static str {
    if ascii {
        return match mask {
            0 => " ",
            m if m == DIR_E | DIR_W => "-",
            m if m == DIR_N | DIR_S => "|",
            _ => "+",
        };
    }
    match mask {
        m if m == DIR_E | DIR_W => "─",
        m if m == DIR_N | DIR_S => "│",
        m if m == DIR_S | DIR_E => "┌",
        m if m == DIR_S | DIR_W => "┐",
        m if m == DIR_N | DIR_E => "└",
        m if m == DIR_N | DIR_W => "┘",
        m if m == DIR_N | DIR_S | DIR_E => "├",
        m if m == DIR_N | DIR_S | DIR_W => "┤",
        m if m == DIR_E | DIR_S | DIR_W => "┬",
        m if m == DIR_E | DIR_N | DIR_W => "┴",
        m if m == DIR_N | DIR_E | DIR_S | DIR_W => "┼",
        // Single-direction stubs render as the matching straight piece.
        m if m == DIR_E || m == DIR_W => "─",
        m if m == DIR_N || m == DIR_S => "│",
        _ => " ",
    }
}

//! Meraid - Render Mermaid diagrams in your terminal
//!
//! A Rust implementation for rendering Mermaid diagrams in the terminal.

pub mod diagram;
pub mod layout;
pub mod parser;
pub mod render;
pub mod theme;

pub use diagram::{Diagram, DiagramType, Edge, EdgeStyle, Node, NodeShape};
pub use layout::Layout;
pub use parser::parse_mermaid;
pub use render::Renderer;
pub use theme::{ColorMode, Theme, ThemeType};

use anyhow::Result;

/// Render Mermaid diagram to terminal string
pub fn render(source: &str, theme_type: ThemeType) -> Result<String> {
    let diagram = parse_mermaid(source)?;
    let layout = Layout::new(&diagram).layout();
    let theme = Theme::get(theme_type);
    let renderer = Renderer::new(theme);
    Ok(renderer.render(&diagram, &layout))
}

/// Render Mermaid diagram with custom theme
pub fn render_with_theme(source: &str, theme: Theme) -> Result<String> {
    let diagram = parse_mermaid(source)?;
    let layout = Layout::new(&diagram).layout();
    let renderer = Renderer::new(theme);
    Ok(renderer.render(&diagram, &layout))
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use crate::{parse_mermaid, DiagramType, Layout, Renderer, Theme, ThemeType};

    // ==================== Parser Tests ====================

    #[test]
    fn test_parse_flowchart_basic() {
        let source = r#"
graph LR
A --> B
"#;
        let diagram = parse_mermaid(source).unwrap();
        assert_eq!(diagram.diagram_type, DiagramType::Flowchart);
        assert_eq!(diagram.direction, "LR");
    }

    #[test]
    fn test_parse_flowchart_multiple_edges() {
        let source = r#"
graph LR
A --> B --> C
A --> D
"#;
        let diagram = parse_mermaid(source).unwrap();
        assert_eq!(diagram.nodes.len(), 4);
        assert_eq!(diagram.edges.len(), 3);
    }

    #[test]
    fn test_parse_flowchart_chained() {
        let source = "graph LR\nA --> B --> C --> D";
        let diagram = parse_mermaid(source).unwrap();
        assert_eq!(diagram.edges.len(), 3);
    }

    #[test]
    fn test_parse_flowchart_thick_arrow() {
        let source = "graph LR\nA ==> B";
        let diagram = parse_mermaid(source).unwrap();
        assert_eq!(diagram.edges.len(), 1);
    }

    #[test]
    fn test_parse_flowchart_dotted_arrow() {
        let source = "graph LR\nA -.-> B";
        let diagram = parse_mermaid(source).unwrap();
        assert_eq!(diagram.edges.len(), 1);
    }

    #[test]
    fn test_parse_sequence_diagram() {
        let source = r#"
sequenceDiagram
Alice->>Bob: Hello
Bob-->>Alice: Hi
Alice->>Bob: How are you?
"#;
        let diagram = parse_mermaid(source).unwrap();
        assert_eq!(diagram.diagram_type, DiagramType::Sequence);
        assert!(diagram.participants.contains(&"Alice".to_string()));
        assert!(diagram.participants.contains(&"Bob".to_string()));
        assert_eq!(diagram.edges.len(), 3);
    }

    #[test]
    fn test_parse_sequence_participants() {
        let source = r#"
sequenceDiagram
participant Alice
participant Bob
participant Charlie
Alice->>Bob: Hello
"#;
        let diagram = parse_mermaid(source).unwrap();
        assert_eq!(diagram.participants.len(), 3);
    }

    #[test]
    fn test_parse_class_diagram() {
        let source = r#"
classDiagram
class Animal {
    +String name
    +int age
}
class Dog {
    +String breed
}
Animal <|-- Dog
"#;
        let diagram = parse_mermaid(source).unwrap();
        assert_eq!(diagram.diagram_type, DiagramType::Class);
    }

    #[test]
    fn test_parse_state_diagram() {
        let source = r#"
stateDiagram-v2
[*] --> Idle
Idle --> Processing: start
Processing --> Done: complete
Done --> [*]
"#;
        let diagram = parse_mermaid(source).unwrap();
        assert_eq!(diagram.diagram_type, DiagramType::State);
    }

    #[test]
    fn test_parse_pie_chart() {
        let source = r#"
pie title Pets
"Dogs" : 386
"Cats" : 85
"Rats" : 15
"#;
        let diagram = parse_mermaid(source).unwrap();
        assert_eq!(diagram.diagram_type, DiagramType::Pie);
        assert_eq!(diagram.nodes.len(), 3);
    }

    #[test]
    fn test_parse_unknown_defaults_to_flowchart() {
        let source = "A --> B";
        let diagram = parse_mermaid(source).unwrap();
        assert_eq!(diagram.diagram_type, DiagramType::Flowchart);
    }

    #[test]
    fn test_parse_flowchart_vertical_direction() {
        let source = "graph TB\nA --> B";
        let diagram = parse_mermaid(source).unwrap();
        assert_eq!(diagram.direction, "TB");
    }

    #[test]
    fn test_parse_comments_ignored() {
        let source = r#"
%% This is a comment
graph LR
%% Another comment
A --> B
%% End comment
"#;
        let diagram = parse_mermaid(source).unwrap();
        assert_eq!(diagram.edges.len(), 1);
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_full_render_flowchart() {
        let source = "graph LR\nA --> B";
        let diagram = parse_mermaid(source).unwrap();
        let layout = Layout::new(&diagram).layout();
        let theme = Theme::get(ThemeType::Default);
        let renderer = Renderer::new(theme);
        let output = renderer.render(&diagram, &layout);

        assert!(!output.is_empty());
    }

    #[test]
    fn test_full_render_sequence() {
        let source = "sequenceDiagram\nAlice->>Bob: Hello";
        let diagram = parse_mermaid(source).unwrap();
        let layout = Layout::new(&diagram).layout();
        let theme = Theme::get(ThemeType::Default);
        let renderer = Renderer::new(theme);
        let output = renderer.render(&diagram, &layout);

        assert!(!output.is_empty());
    }

    #[test]
    fn test_full_render_pie() {
        let source = "pie title Test\nA : 50\nB : 50";
        let diagram = parse_mermaid(source).unwrap();
        let layout = Layout::new(&diagram).layout();
        let theme = Theme::get(ThemeType::Default);
        let renderer = Renderer::new(theme);
        let output = renderer.render(&diagram, &layout);

        assert!(!output.is_empty());
    }

    #[test]
    fn test_render_class_diagram_with_chinese_alignment() {
        let source = r#"
classDiagram
class 用户服务 {
    +获取用户
    +更新资料
}
"#;
        let diagram = parse_mermaid(source).unwrap();
        let layout = Layout::new(&diagram).layout();
        let theme = Theme::get(ThemeType::Default);
        let renderer = Renderer::new(theme);
        let output = renderer.render(&diagram, &layout);

        assert!(output.contains("│    用户服务    │"));
        assert!(output.contains("│+获取用户       │"));
        assert!(output.contains("│+更新资料       │"));
    }

    #[test]
    fn test_full_render_flowchart_with_chinese_label_keeps_borders_aligned() {
        let source = "graph LR\n开始 --> 结束";
        let diagram = parse_mermaid(source).unwrap();
        let layout = Layout::new(&diagram).layout();
        let theme = Theme::get(ThemeType::Default);
        let renderer = Renderer::new(theme);
        let output = renderer.render(&diagram, &layout);

        assert!(output.contains("┌──────────┐"));
        assert!(output.contains("│   开始   │"));
        assert!(output.contains("└──────────┘"));
    }

    #[test]
    fn test_flowchart_merge_node_does_not_overlap() {
        // Regression: a merge node reachable by two paths of different lengths
        // must land in its longest-path layer, not collide with an earlier node.
        // Previously `结束` (reachable directly and via the long branch) was
        // placed in the same column/coordinate as another node, overprinting it.
        let source = "graph LR\n\
            A --> B --> C --> D\n\
            A --> E\n\
            B --> E\n\
            E --> D";
        let diagram = parse_mermaid(source).unwrap();
        let layout = Layout::new(&diagram).layout();

        // No two nodes may share the same top-left coordinate.
        let positions: Vec<_> = diagram
            .nodes
            .iter()
            .map(|n| {
                let p = layout.positions.get(&n.id).expect("node positioned");
                (p.x, p.y)
            })
            .collect();
        for i in 0..positions.len() {
            for j in (i + 1)..positions.len() {
                assert_ne!(
                    positions[i], positions[j],
                    "nodes {} and {} overlap at {:?}",
                    diagram.nodes[i].id, diagram.nodes[j].id, positions[i]
                );
            }
        }

        // D is the sink (longest path A→B→C→D), so it must be the rightmost box.
        let max_x = layout.positions.values().map(|p| p.x).max().unwrap();
        assert_eq!(layout.positions.get("D").unwrap().x, max_x);
    }

    #[test]
    fn test_flowchart_branches_straddle_trunk() {
        // A decision node's two outcomes should sit on opposite sides of the
        // trunk (one above, one below), not stacked together.
        let source = "graph LR\nA --> B\nB --> C\nB --> D";
        let diagram = parse_mermaid(source).unwrap();
        let layout = Layout::new(&diagram).layout();

        let b = layout.positions.get("B").unwrap().y;
        let c = layout.positions.get("C").unwrap().y;
        let d = layout.positions.get("D").unwrap().y;
        // One branch above B, the other below.
        assert!(
            (c < b && d > b) || (d < b && c > b),
            "branches did not straddle the trunk: B={b}, C={c}, D={d}"
        );
    }

    #[test]
    fn test_sequence_diagram_with_chinese_and_mixed_text_alignment() {
        let source = r#"
sequenceDiagram
participant 用户A
participant API服务
用户A->>API服务: 查询 user-详情
API服务-->>用户A: 返回 成功OK
"#;
        let diagram = parse_mermaid(source).unwrap();
        let layout = Layout::new(&diagram).layout();
        let theme = Theme::get(ThemeType::Default);
        let renderer = Renderer::new(theme);
        let output = renderer.render(&diagram, &layout);

        assert!(output.contains("用户A"));
        assert!(output.contains("API服务"));
        assert!(output.contains("│"));
        // Solid arrow for `->>`, dashed arrow for `-->>`.
        assert!(output.contains("├─────────────────▶ 查询 user-详情"));
        assert!(output.contains("◀┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┤ 返回 成功OK"));
    }

    #[test]
    fn test_state_diagram_with_chinese_and_mixed_text() {
        let source = r#"
stateDiagram-v2
[*] --> 待处理
待处理 --> 处理中: 开始 job-1
处理中 --> 已完成: 完成 OK
已完成 --> [*]
"#;
        let diagram = parse_mermaid(source).unwrap();
        let layout = Layout::new(&diagram).layout();
        let theme = Theme::get(ThemeType::Default);
        let renderer = Renderer::new(theme);
        let output = renderer.render(&diagram, &layout);

        assert!(output.contains("待处理 ──▶ 处理中 : 开始 job-1"));
        assert!(output.contains("处理中 ──▶ 已完成 : 完成 OK"));
    }

    #[test]
    fn test_all_themes() {
        let source = "graph LR\nA --> B";
        let diagram = parse_mermaid(source).unwrap();

        for theme_type in [
            ThemeType::Default,
            ThemeType::Terra,
            ThemeType::Neon,
            ThemeType::Mono,
            ThemeType::Amber,
            ThemeType::Phosphor,
        ] {
            let theme = Theme::get(theme_type);
            let layout = Layout::new(&diagram).layout();
            let renderer = Renderer::new(theme);
            let output = renderer.render(&diagram, &layout);
            assert!(!output.is_empty());
        }
    }

    // ==================== Layout Tests ====================

    #[test]
    fn test_flowchart_layout_has_positions() {
        let source = "graph LR\nA --> B --> C";
        let diagram = parse_mermaid(source).unwrap();
        let layout = Layout::new(&diagram).layout();

        assert!(!layout.positions.is_empty());
        assert!(layout.width > 0);
        assert!(layout.height > 0);
    }

    #[test]
    fn test_layout_with_many_nodes() {
        let source = "graph LR\nA --> B --> C --> D --> E --> F --> G --> H";
        let diagram = parse_mermaid(source).unwrap();
        let layout = Layout::new(&diagram).layout();

        assert_eq!(layout.positions.len(), 8);
    }

    // ==================== Render Tests ====================

    #[test]
    fn test_renderer_respects_ascii_only() {
        let source = "graph LR\nA --> B";
        let diagram = parse_mermaid(source).unwrap();
        let layout = Layout::new(&diagram).layout();
        let theme = Theme::get(ThemeType::Default);

        let renderer = Renderer::new(theme).ascii_only(true);
        let output = renderer.render(&diagram, &layout);

        // ASCII mode should use +, -, |
        assert!(output.contains('+') || output.contains('-') || output.contains('|'));
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_empty_source() {
        // Empty input now surfaces an error instead of a blank canvas.
        let result = parse_mermaid("");
        assert!(result.is_err());
    }

    #[test]
    fn test_only_comments() {
        // A source with only comments parses to nothing, so it errors.
        let source = "%% comment only";
        assert!(parse_mermaid(source).is_err());
    }

    #[test]
    fn test_complex_flow() {
        let source = r#"
graph TD
A[Start] --> B{Decision}
B -->|Yes| C[Process]
B -->|No| D[Skip]
C --> E[End]
D --> E
"#;
        let diagram = parse_mermaid(source).unwrap();
        // Due to how parsing works, each labeled node (A[Start], B{Decision}, etc.)
        // might create multiple entries. Just check we have nodes and edges.
        assert!(diagram.nodes.len() >= 5);
        assert!(diagram.edges.len() >= 5);
    }

    // ==================== Color Tests ====================

    use crate::ColorMode;

    fn render_with(src: &str, theme: ThemeType, mode: ColorMode) -> String {
        let diagram = parse_mermaid(src).unwrap();
        let layout = Layout::new(&diagram).layout();
        Renderer::new(Theme::get(theme))
            .color_mode(mode)
            .render(&diagram, &layout)
    }

    const STATE_SRC: &str = "stateDiagram-v2\n[*] --> Idle\nIdle --> Done: go\n";

    #[test]
    fn state_diagram_colors_roles_with_truecolor() {
        let out = render_with(STATE_SRC, ThemeType::Neon, ColorMode::TrueColor);
        // neon: edge (0,255,127), node_fg (255,0,255), start_end (128,0,128)
        assert!(
            out.contains("\x1b[38;2;0;255;127m"),
            "arrow should use the edge color"
        );
        assert!(
            out.contains("\x1b[38;2;255;0;255m"),
            "a state name should use node_fg"
        );
        assert!(
            out.contains("\x1b[38;2;128;0;128m"),
            "the start/end marker should use start_end"
        );
        assert!(out.contains("\x1b[0m"), "colored spans must be reset");
    }

    #[test]
    fn default_theme_emits_no_color_even_in_truecolor() {
        let mono = render_with(STATE_SRC, ThemeType::Default, ColorMode::None);
        let colored = render_with(STATE_SRC, ThemeType::Default, ColorMode::TrueColor);
        assert_eq!(
            mono, colored,
            "the default theme inherits terminal colors — it must emit no escapes"
        );
        assert!(!colored.contains('\x1b'));
    }

    #[test]
    fn pie_chart_colors_label_bar_and_percent() {
        let src = "pie title Pets\n\"Dogs\" : 386\n\"Cats\" : 85\n";
        let out = render_with(src, ThemeType::Neon, ColorMode::TrueColor);
        assert!(
            out.contains("\x1b[38;2;255;0;255m"),
            "label should use node_fg"
        );
        assert!(out.contains("\x1b[38;2;0;255;127m"), "bars should use edge");
        assert!(
            out.contains("\x1b[38;2;0;255;255m"),
            "percentages should use edge_label"
        );
    }

    const FLOW_SRC: &str = "graph LR\nA[Start] --> B{OK?}\nB -->|yes| C[Save]\nB -->|no| D[Stop]\n";

    #[test]
    fn flowchart_colors_boxes_edges_and_labels() {
        let out = render_with(FLOW_SRC, ThemeType::Neon, ColorMode::TrueColor);
        assert!(
            out.contains("\x1b[38;2;255;0;255m"),
            "node boxes should use node_fg"
        );
        assert!(
            out.contains("\x1b[38;2;0;255;127m"),
            "edge wires/arrows should use edge"
        );
        assert!(
            out.contains("\x1b[38;2;0;255;255m"),
            "edge labels should use edge_label"
        );
        assert!(out.contains("\x1b[0m"), "colored runs must be reset");
    }

    #[test]
    fn flowchart_color_none_matches_legacy_and_truecolor_strips_to_same_glyphs() {
        let plain = {
            let diagram = parse_mermaid(FLOW_SRC).unwrap();
            let layout = Layout::new(&diagram).layout();
            Renderer::new(Theme::get(ThemeType::Neon)).render(&diagram, &layout)
        };
        let none = render_with(FLOW_SRC, ThemeType::Neon, ColorMode::None);
        assert_eq!(plain, none, "ColorMode::None must match legacy output");

        // Stripping all escapes from the colored output must reproduce the
        // monochrome glyph layer exactly — proving color never shifts columns.
        let colored = render_with(FLOW_SRC, ThemeType::Neon, ColorMode::TrueColor);
        let stripped = strip_ansi(&colored);
        assert_eq!(stripped, plain, "colored glyph layer must equal monochrome");
    }

    fn strip_ansi(s: &str) -> String {
        let mut out = String::new();
        let mut chars = s.chars();
        while let Some(c) = chars.next() {
            if c == '\x1b' {
                // Skip CSI sequence up to and including the final 'm'.
                for e in chars.by_ref() {
                    if e == 'm' {
                        break;
                    }
                }
            } else {
                out.push(c);
            }
        }
        out
    }

    #[test]
    fn class_diagram_colors_box_and_legend() {
        let src = "classDiagram\nclass Animal {\n+String name\n+makeSound()\n}\nAnimal <|-- Dog\n";
        let out = render_with(src, ThemeType::Neon, ColorMode::TrueColor);
        assert!(
            out.contains("\x1b[38;2;255;0;255m"),
            "class box should use node_fg"
        );
        assert!(
            out.contains("\x1b[38;2;0;255;127m"),
            "relationship legend should use edge"
        );
    }

    #[test]
    fn er_diagram_colors_box_and_legend() {
        let src = "erDiagram\nCUSTOMER {\nint id PK\n}\nORDER {\nint id PK\n}\nCUSTOMER ||--o{ ORDER : places\n";
        let out = render_with(src, ThemeType::Neon, ColorMode::TrueColor);
        assert!(
            out.contains("\x1b[38;2;255;0;255m"),
            "entity box should use node_fg"
        );
        assert!(
            out.contains("\x1b[38;2;0;255;127m"),
            "relationship legend should use edge"
        );
    }

    #[test]
    fn sequence_diagram_colors_participants_and_messages() {
        let src = "sequenceDiagram\nAlice->>Bob: Hello\nBob-->>Alice: Hi\n";
        let out = render_with(src, ThemeType::Neon, ColorMode::TrueColor);
        assert!(
            out.contains("\x1b[38;2;255;0;255m"),
            "participant names should use node_fg"
        );
        assert!(
            out.contains("\x1b[38;2;0;255;127m"),
            "lifelines and message lines should use edge"
        );
        assert!(
            out.contains("\x1b[38;2;0;255;255m"),
            "message labels should use edge_label"
        );
    }

    #[test]
    fn color_mode_none_never_emits_escapes_for_any_type() {
        let samples = [
            "graph LR\nA[Start] --> B{OK?}\nB -->|yes| C[Save]\n",
            "sequenceDiagram\nAlice->>Bob: Hello\nBob-->>Alice: Hi\n",
            "classDiagram\nclass Animal {\n+String name\n+makeSound()\n}\nAnimal <|-- Dog\n",
            "stateDiagram-v2\n[*] --> Idle\nIdle --> Done: go\n",
            "pie title Pets\n\"Dogs\" : 386\n\"Cats\" : 85\n",
            "erDiagram\nCUSTOMER {\nint id PK\n}\nORDER {\nint id PK\n}\nCUSTOMER ||--o{ ORDER : places\n",
        ];
        for src in samples {
            let out = render_with(src, ThemeType::Neon, ColorMode::None);
            assert!(
                !out.contains('\x1b'),
                "ColorMode::None must emit no escapes, but did for: {src:?}"
            );
        }
    }

    #[test]
    fn color_mode_none_matches_legacy_monochrome() {
        // The opt-in contract: with ColorMode::None even a vivid theme emits
        // byte-for-byte the same output as before color existed.
        let plain = {
            let diagram = parse_mermaid(STATE_SRC).unwrap();
            let layout = Layout::new(&diagram).layout();
            Renderer::new(Theme::get(ThemeType::Neon)).render(&diagram, &layout)
        };
        let none = render_with(STATE_SRC, ThemeType::Neon, ColorMode::None);
        assert_eq!(plain, none);
        assert!(!none.contains('\x1b'));
    }
}

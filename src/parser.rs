//! Mermaid syntax parser

use crate::diagram::{
    ClassMember, Diagram, DiagramType, Edge, EdgeStyle, Entity, EntityAttribute, MemberType, Node,
    NodeShape, Relationship, Visibility,
};
use std::collections::HashMap;

/// Parse Mermaid source code into a Diagram.
///
/// Returns an error when the diagram type is unsupported or when nothing
/// meaningful could be parsed, so callers (and the JSON output mode) can report
/// a failure instead of rendering a blank canvas.
pub fn parse_mermaid(source: &str) -> anyhow::Result<Diagram> {
    let source = source.trim();

    let diagram_type = detect_diagram_type(source);

    let diagram = match diagram_type {
        DiagramType::Flowchart => parse_flowchart(source)?,
        DiagramType::Sequence => parse_sequence(source)?,
        DiagramType::Class => parse_class(source)?,
        DiagramType::State => parse_state(source)?,
        DiagramType::ER => parse_er(source)?,
        DiagramType::Pie => parse_pie(source)?,
        DiagramType::GitGraph
        | DiagramType::Block
        | DiagramType::Treemap
        | DiagramType::Unknown => {
            anyhow::bail!(
                "unsupported diagram type: {:?} is not implemented yet — supported types are graph/flowchart, sequenceDiagram, classDiagram, stateDiagram-v2, erDiagram, pie",
                diagram_type
            );
        }
    };

    if diagram.is_empty() {
        anyhow::bail!(
            "empty diagram: no nodes, edges, or entities were parsed — check the diagram type keyword (graph, sequenceDiagram, classDiagram, stateDiagram-v2, erDiagram, pie) and the statement syntax"
        );
    }

    Ok(diagram)
}

fn detect_diagram_type(source: &str) -> DiagramType {
    let first_line = source.lines().next().unwrap_or("").trim();

    if first_line.starts_with("flowchart") || first_line.starts_with("graph") {
        DiagramType::Flowchart
    } else if first_line.starts_with("sequenceDiagram") {
        DiagramType::Sequence
    } else if first_line.starts_with("classDiagram") {
        DiagramType::Class
    } else if first_line.starts_with("stateDiagram") {
        DiagramType::State
    } else if first_line.starts_with("erDiagram") {
        DiagramType::ER
    } else if first_line.starts_with("pie") {
        DiagramType::Pie
    } else if first_line.starts_with("gitGraph") {
        DiagramType::GitGraph
    } else if first_line.starts_with("block-beta") {
        DiagramType::Block
    } else if first_line.starts_with("treemap-beta") {
        DiagramType::Treemap
    } else {
        DiagramType::Flowchart // Default
    }
}

// ==================== Flowchart ====================

const FLOWCHART_ARROWS: [&str; 5] = ["-.->", "==>", "-->", "--x", "--o"];

fn parse_flowchart(source: &str) -> anyhow::Result<Diagram> {
    let mut diagram = Diagram {
        diagram_type: DiagramType::Flowchart,
        ..Default::default()
    };

    // Preserve declaration order while de-duplicating by node id.
    let mut nodes: Vec<Node> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();
    let mut edges: Vec<Edge> = Vec::new();

    for raw_line in source.lines() {
        // Mermaid allows ';' as a statement separator on a single line.
        for stmt in raw_line.split(';') {
            let stmt = stmt.trim();
            if stmt.is_empty() || stmt.starts_with("%%") || stmt.starts_with('{') {
                continue;
            }

            // Direction header: `graph LR`, `flowchart TD`, ...
            if stmt.starts_with("flowchart") || stmt.starts_with("graph") {
                if let Some(dir) = stmt.split_whitespace().nth(1) {
                    if matches!(dir, "LR" | "RL" | "TB" | "TD" | "BT") {
                        diagram.direction = dir.to_string();
                    }
                }
                continue;
            }

            if FLOWCHART_ARROWS.iter().any(|a| stmt.contains(a)) {
                parse_flowchart_edge(stmt, &mut nodes, &mut index, &mut edges);
            } else if stmt.split_whitespace().count() == 1 || stmt.contains(['[', '(', '{']) {
                // Standalone node declaration, e.g. `A` or `A[Label]`. Free-form
                // prose (multiple words, no shape) is ignored so unrecognized
                // input falls through to the empty-diagram error.
                let (id, label, shape) = parse_node_token(stmt);
                if !id.is_empty() {
                    upsert_node(&mut nodes, &mut index, id, label, shape);
                }
            }
        }
    }

    diagram.nodes = nodes;
    diagram.edges = edges;
    Ok(diagram)
}

/// Insert a node, preserving first-seen order. If a node was previously created
/// only as a bare reference, upgrade it once we learn its real label/shape.
fn upsert_node(
    nodes: &mut Vec<Node>,
    index: &mut HashMap<String, usize>,
    id: String,
    label: String,
    shape: NodeShape,
) {
    if let Some(&i) = index.get(&id) {
        if nodes[i].label == nodes[i].id && label != id {
            nodes[i].label = label;
            nodes[i].shape = shape;
        }
    } else {
        index.insert(id.clone(), nodes.len());
        let mut node = Node::new(id, label);
        node.shape = shape;
        nodes.push(node);
    }
}

fn parse_flowchart_edge(
    line: &str,
    nodes: &mut Vec<Node>,
    index: &mut HashMap<String, usize>,
    edges: &mut Vec<Edge>,
) {
    // Split the statement into node segments and the arrow tokens between them.
    let mut segments: Vec<&str> = Vec::new();
    let mut arrows: Vec<&str> = Vec::new();
    let mut remaining = line;

    loop {
        // Find the leftmost arrow; on a tie prefer the longer token.
        let mut best: Option<(usize, &str)> = None;
        for arrow in FLOWCHART_ARROWS {
            if let Some(pos) = remaining.find(arrow) {
                let replace = match best {
                    None => true,
                    Some((bp, ba)) => pos < bp || (pos == bp && arrow.len() > ba.len()),
                };
                if replace {
                    best = Some((pos, arrow));
                }
            }
        }

        match best {
            Some((pos, arrow)) => {
                segments.push(&remaining[..pos]);
                arrows.push(arrow);
                remaining = &remaining[pos + arrow.len()..];
            }
            None => {
                segments.push(remaining);
                break;
            }
        }
    }

    if segments.len() < 2 {
        return;
    }

    let (mut prev_id, first_label, first_shape) = parse_node_token(segments[0]);
    if prev_id.is_empty() {
        return;
    }
    upsert_node(nodes, index, prev_id.clone(), first_label, first_shape);

    for i in 1..segments.len() {
        let (edge_label, node_str) = extract_edge_label(segments[i]);
        let (to_id, to_label, to_shape) = parse_node_token(node_str);
        if to_id.is_empty() {
            continue;
        }
        upsert_node(nodes, index, to_id.clone(), to_label, to_shape);
        edges.push(Edge {
            from: prev_id.clone(),
            to: to_id.clone(),
            label: edge_label,
            style: style_from_arrow(arrows[i - 1]),
        });
        prev_id = to_id;
    }
}

fn style_from_arrow(arrow: &str) -> EdgeStyle {
    match arrow {
        "==>" => EdgeStyle::Thick,
        "-.->" => EdgeStyle::Dotted,
        "--x" => EdgeStyle::CrossEnd,
        "--o" => EdgeStyle::CircleEnd,
        _ => EdgeStyle::Solid,
    }
}

/// Strip a leading `|label|` edge-label segment from a node segment, returning
/// the optional label and the remaining node token.
fn extract_edge_label(segment: &str) -> (Option<String>, &str) {
    let trimmed = segment.trim_start();
    if let Some(rest) = trimmed.strip_prefix('|') {
        if let Some(end) = rest.find('|') {
            let label = rest[..end].trim();
            let node = rest[end + 1..].trim();
            return (
                if label.is_empty() {
                    None
                } else {
                    Some(label.to_string())
                },
                node,
            );
        }
    }
    (None, segment.trim())
}

/// Parse a node token like `A[Start]`, `B{Is valid?}`, `C([Done])` into
/// `(id, label, shape)`. Plain tokens (`A`) keep `id == label` and a rectangle.
fn parse_node_token(token: &str) -> (String, String, NodeShape) {
    let token = token.trim();
    if token.is_empty() {
        return (String::new(), String::new(), NodeShape::Rectangle);
    }

    match token.find(['[', '(', '{', '>']) {
        None => (token.to_string(), token.to_string(), NodeShape::Rectangle),
        Some(pos) => {
            let id_part = token[..pos].trim();
            let (label, shape) = parse_shape(&token[pos..]);
            let id = if id_part.is_empty() {
                label.clone()
            } else {
                id_part.to_string()
            };
            (id, label, shape)
        }
    }
}

fn parse_shape(wrapper: &str) -> (String, NodeShape) {
    // Order matters: check the longest / most-specific delimiters first.
    const SHAPES: [(&str, &str, NodeShape); 11] = [
        ("([", "])", NodeShape::Stadium),
        ("[[", "]]", NodeShape::Subroutine),
        ("[(", ")]", NodeShape::Cylinder),
        ("((", "))", NodeShape::Circle),
        ("{{", "}}", NodeShape::Hexagon),
        ("[/", "\\]", NodeShape::Trapezoid),
        ("[/", "/]", NodeShape::Parallelogram),
        ("[", "]", NodeShape::Rectangle),
        ("(", ")", NodeShape::Rounded),
        ("{", "}", NodeShape::Diamond),
        (">", "]", NodeShape::Asymmetric),
    ];

    for (open, close, shape) in SHAPES {
        if wrapper.len() >= open.len() + close.len()
            && wrapper.starts_with(open)
            && wrapper.ends_with(close)
        {
            let inner = &wrapper[open.len()..wrapper.len() - close.len()];
            return (inner.trim().to_string(), shape);
        }
    }

    // Unbalanced / unknown wrapper: strip stray bracket chars as a best effort.
    let inner = wrapper.trim_matches(|c| matches!(c, '[' | ']' | '(' | ')' | '{' | '}' | '>'));
    (inner.trim().to_string(), NodeShape::Rectangle)
}

// ==================== Sequence ====================

fn parse_sequence(source: &str) -> anyhow::Result<Diagram> {
    let mut diagram = Diagram {
        diagram_type: DiagramType::Sequence,
        ..Default::default()
    };

    for line in source.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with("%%") {
            continue;
        }

        if line.starts_with("participant") || line.starts_with("actor") {
            // `participant Alice` or `participant A as Alice` (alias display
            // name is not yet rendered; the declared id is used).
            if let Some(participant) = line.split_whitespace().nth(1) {
                let participant = participant.trim();
                if !participant.is_empty()
                    && !diagram.participants.contains(&participant.to_string())
                {
                    diagram.participants.push(participant.to_string());
                }
            }
            continue;
        }

        // Parse messages. Check the longest arrow tokens first so a dashed
        // message ('A-->>B') is not mis-split on '->>' into sender 'A-'.
        let arrow = if line.contains("-->>") {
            Some("-->>")
        } else if line.contains("->>") {
            Some("->>")
        } else if line.contains("-->") {
            Some("-->")
        } else if line.contains("->") {
            Some("->")
        } else {
            None
        };

        if let Some(arrow) = arrow {
            let style = if arrow.starts_with("--") {
                EdgeStyle::Dotted
            } else {
                EdgeStyle::Solid
            };

            let parts: Vec<&str> = line.splitn(2, arrow).collect();
            if parts.len() == 2 {
                let from = parts[0].trim();
                let (to, msg) = match parts[1].split_once(':') {
                    Some((to, msg)) => (to.trim(), msg.trim()),
                    None => (parts[1].trim(), ""),
                };

                if !from.is_empty() && !diagram.participants.contains(&from.to_string()) {
                    diagram.participants.push(from.to_string());
                }
                if !to.is_empty() && !diagram.participants.contains(&to.to_string()) {
                    diagram.participants.push(to.to_string());
                }

                let mut edge = Edge::new(from, to);
                edge.label = Some(msg.to_string());
                edge.style = style;
                diagram.edges.push(edge);
            }
        }
    }

    Ok(diagram)
}

// ==================== Class ====================

/// Class relationship tokens, ordered longest / most-specific first so that
/// e.g. `<|--` is matched before `--` and `--|>` before `-->`.
const CLASS_REL_TOKENS: [&str; 8] = ["..|>", "..>", "<|--", "--|>", "*--", "o--", "-->", "--"];

fn parse_class(source: &str) -> anyhow::Result<Diagram> {
    let mut diagram = Diagram {
        diagram_type: DiagramType::Class,
        ..Default::default()
    };

    let mut nodes: Vec<Node> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();
    let mut relationships: Vec<Relationship> = Vec::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        if line.starts_with("%%") || line.is_empty() || line.starts_with("classDiagram") {
            i += 1;
            continue;
        }

        // Class definition with a body (single or multi-line).
        if line.starts_with("class") && line.contains('{') {
            let class_name = line
                .split_whitespace()
                .nth(1)
                .unwrap_or("")
                .trim()
                .to_string();

            if !class_name.is_empty() {
                let mut full_def = line.to_string();
                let mut brace_balance =
                    line.matches('{').count() as isize - line.matches('}').count() as isize;

                while brace_balance > 0 && i + 1 < lines.len() {
                    i += 1;
                    let next_line = lines[i].trim();
                    full_def.push('\n');
                    full_def.push_str(next_line);
                    brace_balance += next_line.matches('{').count() as isize;
                    brace_balance -= next_line.matches('}').count() as isize;
                }

                let members = parse_class_members(&full_def);
                ensure_class(&mut nodes, &mut index, &class_name);
                let idx = index[&class_name];
                nodes[idx].members = members;
            }

            i += 1;
            continue;
        }

        // Bare class declaration: `class Foo`.
        if line.starts_with("class") {
            if let Some(name) = line.split_whitespace().nth(1) {
                ensure_class(&mut nodes, &mut index, name);
            }
            i += 1;
            continue;
        }

        // Relationship line.
        if let Some(token) = CLASS_REL_TOKENS.iter().copied().find(|t| line.contains(t)) {
            let parts: Vec<&str> = line.splitn(2, token).collect();
            if parts.len() == 2 {
                // Skip quoted cardinality labels (e.g. `"1"`, `"0..*"`) so they
                // are not mistaken for class names. Drop an optional `: label`.
                let from = parts[0]
                    .split_whitespace()
                    .rfind(|t| !t.starts_with('"'))
                    .unwrap_or("");
                let to = parts[1]
                    .split(':')
                    .next()
                    .unwrap_or("")
                    .split_whitespace()
                    .find(|t| !t.starts_with('"'))
                    .unwrap_or("");

                if !from.is_empty() && !to.is_empty() {
                    ensure_class(&mut nodes, &mut index, from);
                    ensure_class(&mut nodes, &mut index, to);
                    relationships.push(Relationship {
                        from: from.to_string(),
                        to: to.to_string(),
                        rel_type: token.to_string(),
                    });
                }
            }
        }

        i += 1;
    }

    diagram.nodes = nodes;
    diagram.relationships = relationships;
    Ok(diagram)
}

fn ensure_class(nodes: &mut Vec<Node>, index: &mut HashMap<String, usize>, name: &str) {
    if !index.contains_key(name) {
        index.insert(name.to_string(), nodes.len());
        nodes.push(Node::new(name, name));
    }
}

/// Parse class members from a class definition body.
fn parse_class_members(class_def: &str) -> Vec<ClassMember> {
    let mut members = Vec::new();

    if let Some(start) = class_def.find('{') {
        let rest = &class_def[start + 1..];
        let mut depth = 1;
        let mut end = None;
        for (i, c) in rest.char_indices() {
            if c == '{' {
                depth += 1;
            }
            if c == '}' {
                depth -= 1;
            }
            if depth == 0 {
                end = Some(i);
                break;
            }
        }

        if let Some(end) = end {
            let body = &rest[..end];

            for line in body.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                let visibility = if line.starts_with('+') {
                    Visibility::Public
                } else if line.starts_with('-') {
                    Visibility::Private
                } else if line.starts_with('#') {
                    Visibility::Protected
                } else if line.starts_with('~') {
                    Visibility::Package
                } else {
                    Visibility::Public
                };

                let line = line.trim_start_matches(['+', '-', '#', '~']);

                let member_type = if line.contains('(') && line.contains(')') {
                    MemberType::Method
                } else {
                    MemberType::Field
                };

                let name = if member_type == MemberType::Method {
                    line.split('(').next().unwrap_or(line).trim().to_string()
                } else if line.contains(':') {
                    line.split(':').next().unwrap_or(line).trim().to_string()
                } else {
                    line.trim().to_string()
                };

                if !name.is_empty() {
                    members.push(ClassMember {
                        name,
                        member_type,
                        visibility,
                    });
                }
            }
        }
    }

    members
}

// ==================== State ====================

fn parse_state(source: &str) -> anyhow::Result<Diagram> {
    let mut diagram = Diagram {
        diagram_type: DiagramType::State,
        ..Default::default()
    };

    for line in source.lines() {
        let line = line.trim();

        if line.starts_with("%%") || line.is_empty() || line.starts_with("stateDiagram") {
            continue;
        }

        if line.contains("-->") {
            let parts: Vec<&str> = line.splitn(2, "-->").collect();
            if parts.len() == 2 {
                // Keep `[*]` intact so start/end markers can be rendered.
                let from = parts[0].trim();
                let rest = parts[1].trim();

                let (to, label) = match rest.split_once(':') {
                    Some((to, label)) => (to.trim(), Some(label.trim().to_string())),
                    None => (rest, None),
                };

                if !from.is_empty() {
                    diagram.nodes.push(Node::new(from, from));
                }
                if !to.is_empty() {
                    diagram.nodes.push(Node::new(to, to));
                }

                let mut edge = Edge::new(from, to);
                edge.label = label;
                diagram.edges.push(edge);
            }
        }
    }

    Ok(diagram)
}

// ==================== ER ====================

/// Valid right-hand cardinality symbols for an ER relationship. These mirror
/// the left-hand symbols (`||`, `}|`, `}o`, `o{`, `o|`).
const ER_RIGHT_CARDINALITY: [&str; 5] = ["||", "|{", "|o", "o|", "o{"];

fn parse_er(source: &str) -> anyhow::Result<Diagram> {
    let mut diagram = Diagram {
        diagram_type: DiagramType::ER,
        ..Default::default()
    };

    let mut entities: Vec<Entity> = Vec::new();
    let mut relationships: Vec<Relationship> = Vec::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        if line.is_empty() || line.starts_with("%%") || line.starts_with("erDiagram") {
            i += 1;
            continue;
        }

        // Entity block: `NAME { ... }`. A line containing `--` is a
        // relationship, never an entity block (this avoids the curly braces in
        // cardinality symbols like `o{` being mistaken for entity bodies).
        if let Some(brace) = line.find('{') {
            if !line.contains("--") {
                let name = line[..brace].trim();
                let mut body = String::new();
                let mut depth = 1;
                let mut segment = line[brace + 1..].to_string();

                loop {
                    if let Some(close) = segment.find('}') {
                        body.push_str(&segment[..close]);
                        depth -= 1;
                        break;
                    }
                    body.push_str(&segment);
                    body.push('\n');
                    i += 1;
                    if i >= lines.len() {
                        break;
                    }
                    segment = lines[i].trim().to_string();
                }
                let _ = depth;

                if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    entities.push(Entity {
                        name: name.to_string(),
                        attributes: parse_er_body(&body),
                    });
                }

                i += 1;
                continue;
            }
        }

        if let Some(rel) = parse_er_relationship(line) {
            relationships.push(rel);
        }

        i += 1;
    }

    diagram.entities = entities;
    diagram.relationships = relationships;
    Ok(diagram)
}

/// Parse a relationship line: `ENTITY1 <left>--<right> ENTITY2 : label`.
fn parse_er_relationship(line: &str) -> Option<Relationship> {
    let dash = line.find("--")?;
    let before = &line[..dash];
    let after = &line[dash + 2..];

    let mut before_parts = before.split_whitespace();
    let from = before_parts.next()?;
    let left_card = before_parts.next().unwrap_or("");

    let right_card = ER_RIGHT_CARDINALITY
        .iter()
        .copied()
        .find(|c| after.starts_with(c))?;

    let rest = after[right_card.len()..].trim_start();
    let to = rest.split_whitespace().next().unwrap_or("");

    if from.is_empty() || to.is_empty() || !from.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return None;
    }

    Some(Relationship {
        from: from.to_string(),
        to: to.to_string(),
        rel_type: format!("{}--{}", left_card, right_card),
    })
}

fn parse_er_body(entity_def: &str) -> Vec<EntityAttribute> {
    let mut attrs = Vec::new();

    for line in entity_def.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let is_pk = line.contains(" PK") || line.ends_with(" PK");
        let is_fk = line.contains(" FK") || line.ends_with(" FK");

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            attrs.push(EntityAttribute {
                name: parts[1].to_string(),
                attr_type: parts[0].to_string(),
                is_primary_key: is_pk,
                is_foreign_key: is_fk,
            });
        }
    }

    attrs
}

// ==================== Pie ====================

fn parse_pie(source: &str) -> anyhow::Result<Diagram> {
    let mut diagram = Diagram {
        diagram_type: DiagramType::Pie,
        ..Default::default()
    };

    for line in source.lines() {
        let line = line.trim();

        if line.starts_with("pie")
            || line.starts_with("title")
            || line.starts_with("%%")
            || line.is_empty()
        {
            continue;
        }

        // Split on the LAST colon so quoted labels containing ':' are preserved.
        if let Some((label_part, value_part)) = line.rsplit_once(':') {
            let label = label_part.trim().trim_matches('"').trim();
            let value = value_part.trim();

            if !label.is_empty() {
                diagram
                    .nodes
                    .push(Node::new(label, format!("{}: {}", label, value)));
            }
        }
    }

    Ok(diagram)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_flowchart() {
        let source = "graph LR\nA --> B\nB --> C";
        let diagram = parse_mermaid(source).unwrap();
        assert_eq!(diagram.diagram_type, DiagramType::Flowchart);
        assert_eq!(diagram.nodes.len(), 3);
        assert_eq!(diagram.edges.len(), 2);
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
        assert_eq!(diagram.edges[0].style, EdgeStyle::Thick);
    }

    #[test]
    fn test_parse_flowchart_dotted_arrow() {
        let source = "graph LR\nA -.-> B";
        let diagram = parse_mermaid(source).unwrap();
        assert_eq!(diagram.edges.len(), 1);
        assert_eq!(diagram.edges[0].style, EdgeStyle::Dotted);
    }

    #[test]
    fn test_flowchart_node_shapes_and_labels() {
        let source = "graph TD\nA[Start] --> B{Is valid?}";
        let diagram = parse_mermaid(source).unwrap();
        assert_eq!(diagram.nodes.len(), 2);
        let a = diagram.nodes.iter().find(|n| n.id == "A").unwrap();
        assert_eq!(a.label, "Start");
        assert_eq!(a.shape, NodeShape::Rectangle);
        let b = diagram.nodes.iter().find(|n| n.id == "B").unwrap();
        assert_eq!(b.label, "Is valid?");
        assert_eq!(b.shape, NodeShape::Diamond);
    }

    #[test]
    fn test_flowchart_all_shapes() {
        let source = "graph LR\nA([Stadium]) --> B[[Sub]]\nC((Circle)) --> D{{Hex}}";
        let diagram = parse_mermaid(source).unwrap();
        let shape = |id: &str| diagram.nodes.iter().find(|n| n.id == id).unwrap().shape;
        assert_eq!(shape("A"), NodeShape::Stadium);
        assert_eq!(shape("B"), NodeShape::Subroutine);
        assert_eq!(shape("C"), NodeShape::Circle);
        assert_eq!(shape("D"), NodeShape::Hexagon);
    }

    #[test]
    fn test_flowchart_edge_label() {
        let source = "graph TD\nB -->|Yes| C\nB -->|No| D";
        let diagram = parse_mermaid(source).unwrap();
        // No phantom nodes from the |label| text.
        assert_eq!(diagram.nodes.len(), 3);
        let labels: Vec<_> = diagram
            .edges
            .iter()
            .filter_map(|e| e.label.clone())
            .collect();
        assert!(labels.contains(&"Yes".to_string()));
        assert!(labels.contains(&"No".to_string()));
    }

    #[test]
    fn test_flowchart_semicolon_separator() {
        let source = "graph LR; A-->B-->C";
        let diagram = parse_mermaid(source).unwrap();
        assert_eq!(diagram.direction, "LR");
        assert_eq!(diagram.nodes.len(), 3);
        assert_eq!(diagram.edges.len(), 2);
    }

    #[test]
    fn test_parse_sequence() {
        let source = "sequenceDiagram\nAlice->>Bob: Hello\nBob-->>Alice: Hi";
        let diagram = parse_mermaid(source).unwrap();
        assert_eq!(diagram.diagram_type, DiagramType::Sequence);
        assert!(diagram.participants.contains(&"Alice".to_string()));
        assert!(diagram.participants.contains(&"Bob".to_string()));
    }

    #[test]
    fn test_sequence_dashed_no_phantom_participant() {
        // Regression: 'Bob-->>Alice' must not create a phantom 'Bob-' participant.
        let source = "sequenceDiagram\nAlice->>Bob: Hello\nBob-->>Alice: Hi";
        let diagram = parse_mermaid(source).unwrap();
        assert_eq!(diagram.participants, vec!["Alice", "Bob"]);
        assert_eq!(diagram.edges.len(), 2);
        assert_eq!(diagram.edges[1].from, "Bob");
        assert_eq!(diagram.edges[1].to, "Alice");
        assert_eq!(diagram.edges[1].style, EdgeStyle::Dotted);
    }

    #[test]
    fn test_class_relationship_keeps_target() {
        let source = "classDiagram\nclass Animal\nclass Dog\nAnimal <|-- Dog";
        let diagram = parse_mermaid(source).unwrap();
        assert_eq!(diagram.relationships.len(), 1);
        let rel = &diagram.relationships[0];
        assert_eq!(rel.from, "Animal");
        assert_eq!(rel.to, "Dog");
        assert_eq!(rel.rel_type, "<|--");
    }

    #[test]
    fn test_class_plain_association() {
        let source = "classDiagram\nA -- B";
        let diagram = parse_mermaid(source).unwrap();
        assert_eq!(diagram.relationships.len(), 1);
        assert_eq!(diagram.relationships[0].rel_type, "--");
        assert_eq!(diagram.nodes.len(), 2);
    }

    #[test]
    fn test_class_directed_association_with_cardinality() {
        // Quoted cardinality labels must not become classes.
        let source = "classDiagram\nclass A\nclass B\nA \"1\" --> \"0..*\" B";
        let diagram = parse_mermaid(source).unwrap();
        assert_eq!(diagram.nodes.len(), 2);
        assert_eq!(diagram.relationships.len(), 1);
        assert_eq!(diagram.relationships[0].from, "A");
        assert_eq!(diagram.relationships[0].to, "B");
        assert_eq!(diagram.relationships[0].rel_type, "-->");
    }

    #[test]
    fn test_class_realization_not_mis_split() {
        // Regression: 'Dog --|> Animal' must not produce a junk class '> Animal'.
        let source = "classDiagram\nDog --|> Animal";
        let diagram = parse_mermaid(source).unwrap();
        assert_eq!(diagram.relationships[0].rel_type, "--|>");
        assert_eq!(diagram.relationships[0].to, "Animal");
        assert!(diagram.nodes.iter().all(|n| n.id != "> Animal"));
    }

    #[test]
    fn test_state_preserves_start_end_markers() {
        let source = "stateDiagram-v2\n[*] --> Idle\nDone --> [*]";
        let diagram = parse_mermaid(source).unwrap();
        assert_eq!(diagram.edges[0].from, "[*]");
        assert_eq!(diagram.edges[1].to, "[*]");
    }

    #[test]
    fn test_er_many_to_many_cardinality() {
        // Regression: right cardinality '|{' must not be dropped.
        let source = "erDiagram\nA }|--|{ B : two";
        let diagram = parse_mermaid(source).unwrap();
        assert_eq!(diagram.relationships.len(), 1);
        assert_eq!(diagram.relationships[0].rel_type, "}|--|{");
    }

    #[test]
    fn test_er_no_phantom_entity() {
        // Regression: adjacent relationship lines must not fabricate an entity.
        let source = "erDiagram\nA ||--o{ B : one\nC }|--|{ D : two";
        let diagram = parse_mermaid(source).unwrap();
        assert_eq!(diagram.entities.len(), 0);
        assert_eq!(diagram.relationships.len(), 2);
    }

    #[test]
    fn test_er_entities_with_attributes() {
        let source =
            "erDiagram\nCUSTOMER {\nint id PK\nstring name\n}\nCUSTOMER ||--o{ ORDER : places";
        let diagram = parse_mermaid(source).unwrap();
        assert_eq!(diagram.entities.len(), 1);
        assert_eq!(diagram.entities[0].name, "CUSTOMER");
        assert_eq!(diagram.entities[0].attributes.len(), 2);
        assert!(diagram.entities[0].attributes[0].is_primary_key);
        assert_eq!(diagram.relationships.len(), 1);
    }

    #[test]
    fn test_parse_pie() {
        let source = "pie title Pets\n\"Dogs\" : 386\n\"Cats\" : 85";
        let diagram = parse_mermaid(source).unwrap();
        assert_eq!(diagram.diagram_type, DiagramType::Pie);
        assert_eq!(diagram.nodes.len(), 2);
    }

    #[test]
    fn test_pie_label_with_colon() {
        // Regression: a quoted label containing ':' must not blank the chart.
        let source = "pie title T\n\"Time: morning\" : 50\n\"Time: evening\" : 50";
        let diagram = parse_mermaid(source).unwrap();
        assert_eq!(diagram.nodes.len(), 2);
        assert_eq!(diagram.nodes[0].label, "Time: morning: 50");
    }

    #[test]
    fn test_empty_input_errors() {
        assert!(parse_mermaid("").is_err());
        assert!(parse_mermaid("%% only a comment").is_err());
    }

    #[test]
    fn test_unsupported_type_errors() {
        assert!(parse_mermaid("gitGraph\ncommit").is_err());
    }
}

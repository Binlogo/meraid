//! Mermaid syntax parser

use crate::diagram::{ClassMember, Diagram, DiagramType, Edge, EdgeStyle, Entity, EntityAttribute, MemberType, Node, Relationship, Visibility};

/// Parse Mermaid source code into a Diagram
pub fn parse_mermaid(source: &str) -> anyhow::Result<Diagram> {
    let source = source.trim();
    
    // Detect diagram type
    let diagram_type = detect_diagram_type(source);
    
    match diagram_type {
        DiagramType::Flowchart => parse_flowchart(source),
        DiagramType::Sequence => parse_sequence(source),
        DiagramType::Class => parse_class(source),
        DiagramType::State => parse_state(source),
        DiagramType::ER => parse_er(source),
        DiagramType::Pie => parse_pie(source),
        _ => parse_flowchart(source), // Default to flowchart
    }
}

fn detect_diagram_type(source: &str) -> DiagramType {
    let first_line = source.lines().next().unwrap_or("").trim();
    
    if first_line.starts_with("flowchart") || first_line.starts_with("graph") {
        DiagramType::Flowchart
    } else if first_line.starts_with("sequenceDiagram") {
        DiagramType::Sequence
    } else if first_line.starts_with("classDiagram") {
        DiagramType::Class
    } else if first_line.starts_with("stateDiagram") || first_line.starts_with("stateDiagram-v2") {
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

fn parse_flowchart(source: &str) -> anyhow::Result<Diagram> {
    let mut diagram = Diagram::default();
    diagram.diagram_type = DiagramType::Flowchart;
    
    // Parse direction
    for line in source.lines() {
        let line = line.trim();
        if line.starts_with("flowchart") || line.starts_with("graph") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() > 1 {
                diagram.direction = parts[1].to_string();
            }
        }
    }
    
    let mut nodes: std::collections::HashMap<String, Node> = std::collections::HashMap::new();
    let mut edges: Vec<Edge> = Vec::new();
    
    // Parse statements
    for line in source.lines() {
        let line = line.trim();
        
        // Skip comments and directives
        if line.is_empty() || line.starts_with("%%") || line.starts_with('{') {
            continue;
        }
        
        // Parse edges
        if line.contains("-->") || line.contains("==>") || line.contains("-.-") || 
           line.contains("--x") || line.contains("--o") {
            parse_flowchart_edge(line, &mut nodes, &mut edges)?;
        }
    }
    
    diagram.nodes = nodes.into_values().collect();
    diagram.edges = edges;
    
    Ok(diagram)
}

fn parse_flowchart_edge(line: &str, nodes: &mut std::collections::HashMap<String, Node>, edges: &mut Vec<Edge>) -> anyhow::Result<()> {
    // Handle multiple edges on one line: A --> B --> C --> D
    let arrow_patterns = ["-->", "==>", "-.->", "--x", "--o"];
    
    let mut edge_parts: Vec<&str> = Vec::new();
    let mut remaining = line;
    
    while !remaining.is_empty() {
        let mut earliest_pos = usize::MAX;
        let mut earliest_arrow = "";
        
        for pattern in &arrow_patterns {
            if let Some(pos) = remaining.find(pattern) {
                if pos < earliest_pos {
                    earliest_pos = pos;
                    earliest_arrow = pattern;
                }
            }
        }
        
        if earliest_pos == usize::MAX {
            let node_id = remaining.trim();
            if !node_id.is_empty() {
                edge_parts.push(node_id);
            }
            break;
        }
        
        let node_part = &remaining[..earliest_pos];
        if !node_part.trim().is_empty() {
            edge_parts.push(node_part.trim());
        }
        
        remaining = &remaining[earliest_pos + earliest_arrow.len()..];
    }
    
    // Create edges from consecutive node pairs
    for window in edge_parts.windows(2) {
        let from_id = window[0];
        let to_id = window[1];
        
        let style = if line.contains("==>") {
            EdgeStyle::Thick
        } else if line.contains("-.-") {
            EdgeStyle::Dotted
        } else if line.contains("--x") {
            EdgeStyle::CrossEnd
        } else if line.contains("--o") {
            EdgeStyle::CircleEnd
        } else {
            EdgeStyle::Solid
        };
        
        nodes.entry(from_id.to_string()).or_insert_with(|| Node::new(from_id, from_id));
        nodes.entry(to_id.to_string()).or_insert_with(|| Node::new(to_id, to_id));
        
        edges.push(Edge {
            from: from_id.to_string(),
            to: to_id.to_string(),
            label: None,
            style,
        });
    }
    
    Ok(())
}

fn parse_sequence(source: &str) -> anyhow::Result<Diagram> {
    let mut diagram = Diagram::default();
    diagram.diagram_type = DiagramType::Sequence;
    
    for line in source.lines() {
        let line = line.trim();
        
        // Skip empty lines and directives
        if line.is_empty() || line.starts_with("%%") {
            continue;
        }
        
        if line.starts_with("participant") || line.starts_with("actor") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() > 1 {
                let participant = parts[1].trim();
                if !participant.is_empty() && !diagram.participants.contains(&participant.to_string()) {
                    diagram.participants.push(participant.to_string());
                }
            }
        }
        // Parse messages
        else if line.contains("->>") || line.contains("-->>") || line.contains("->") {
            let arrow = if line.contains("->>") {
                "->>"
            } else if line.contains("-->") {
                "-->"
            } else if line.contains("-->>") {
                "-->>"
            } else {
                "->"
            };
            
            let parts: Vec<&str> = line.split(arrow).collect();
            if parts.len() >= 2 {
                let from = parts[0].trim();
                let rest = parts[1];
                let to_and_msg: Vec<&str> = rest.split(':').collect();
                let to = to_and_msg[0].trim();
                let msg = to_and_msg.get(1).map(|s| s.trim()).unwrap_or("");
                
                if !from.is_empty() && !diagram.participants.contains(&from.to_string()) {
                    diagram.participants.push(from.to_string());
                }
                if !to.is_empty() && !diagram.participants.contains(&to.to_string()) {
                    diagram.participants.push(to.to_string());
                }
                
                let mut edge = Edge::new(from, to);
                edge.label = Some(msg.to_string());
                diagram.edges.push(edge);
            }
        }
    }
    
    Ok(diagram)
}

fn parse_class(source: &str) -> anyhow::Result<Diagram> {
    let mut diagram = Diagram::default();
    diagram.diagram_type = DiagramType::Class;
    
    let mut nodes: std::collections::HashMap<String, Node> = std::collections::HashMap::new();
    let mut relationships: Vec<Relationship> = Vec::new();
    
    // Join all lines for multi-line class body parsing
    let source_joined = source.replace("\n", " ");
    
    for line in source.lines() {
        let line = line.trim();
        
        // Skip comments and empty lines
        if line.starts_with("%%") || line.is_empty() {
            continue;
        }
        
        // Parse class definition with members (single or multi-line)
        if line.starts_with("class") && line.contains("{") {
            let class_name = line
                .split_whitespace()
                .nth(1)
                .unwrap_or("")
                .trim();
            
            if !class_name.is_empty() {
                // Check if it's a multi-line class (body ends on next line)
                if line.matches('{').count() == line.matches('}').count() {
                    // Single line class
                    let members = parse_class_members(line);
                    let mut node = Node::new(class_name, class_name);
                    node.members = members;
                    nodes.insert(class_name.to_string(), node);
                } else {
                    // Multi-line: need to find the closing brace in subsequent lines
                    let rest_lines: Vec<&str> = source.lines()
                        .skip_while(|l| !l.contains(class_name))
                        .skip(1)
                        .collect();
                    let rest = rest_lines.join(" ");
                    
                    let full_def = format!("{} {}", line, rest);
                    let members = parse_class_members(&full_def);
                    let mut node = Node::new(class_name, class_name);
                    node.members = members;
                    nodes.insert(class_name.to_string(), node);
                }
            }
        }
        
        // Parse relationships
        if line.contains("<|--") || line.contains("*--") || line.contains("o--") || 
           line.contains("--|") || line.contains("..>") || line.contains("..|>") {
            let rel_type = if line.contains("<|--") {
                "<|--"
            } else if line.contains("*--") {
                "*--"
            } else if line.contains("o--") {
                "o--"
            } else if line.contains("--|") {
                "--|"
            } else if line.contains("..|>") {
                "..|>"
            } else {
                "..>"
            };
            
            let parts: Vec<&str> = line.split(rel_type).collect();
            if parts.len() == 2 {
                let from = parts[0].trim();
                let to = parts[1].trim();
                
                if !from.is_empty() {
                    nodes.entry(from.to_string()).or_insert_with(|| Node::new(from, from));
                }
                if !to.is_empty() {
                    nodes.entry(to.to_string()).or_insert_with(|| Node::new(to, to));
                }
                
                relationships.push(Relationship {
                    from: from.to_string(),
                    to: to.to_string(),
                    rel_type: rel_type.to_string(),
                });
            }
        }
    }
    
    diagram.nodes = nodes.into_values().collect();
    diagram.relationships = relationships;
    
    Ok(diagram)
}

/// Parse class members from class definition body
fn parse_class_members(class_def: &str) -> Vec<ClassMember> {
    let mut members = Vec::new();
    
    // Find the body between { and }
    if let Some(start) = class_def.find('{') {
        let rest = &class_def[start+1..];
        // Find the matching closing brace
        let mut depth = 1;
        let mut end = 0;
        for (i, c) in rest.chars().enumerate() {
            if c == '{' { depth += 1; }
            if c == '}' { depth -= 1; }
            if depth == 0 {
                end = i;
                break;
            }
        }
        
        if end > 0 {
            let body = &rest[..end];
            
            // Split by } or { to get individual members
            for line in body.split(&['}', '{'][..]) {
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
                
                let line = line.trim_start_matches(&['+', '-', '#', '~'][..]);
                
                // Determine if field or method
                let member_type = if line.contains('(') && line.contains(')') {
                    MemberType::Method
                } else {
                    MemberType::Field
                };
                
                // Extract name
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

fn parse_state(source: &str) -> anyhow::Result<Diagram> {
    let mut diagram = Diagram::default();
    diagram.diagram_type = DiagramType::State;
    
    for line in source.lines() {
        let line = line.trim();
        
        if line.starts_with("%%") || line.is_empty() {
            continue;
        }
        
        if line.contains("-->") {
            let parts: Vec<&str> = line.split("-->").collect();
            if parts.len() >= 2 {
                let from = parts[0].trim().trim_start_matches("[*]");
                let rest = parts[1].trim();
                
                let (to, label) = if rest.contains(':') {
                    let label_parts: Vec<&str> = rest.split(':').collect();
                    (label_parts[0].trim(), Some(label_parts.get(1).map(|s| s.trim()).unwrap_or("").to_string()))
                } else {
                    (rest.trim(), None)
                };
                
                let to = to.trim_end_matches("[*]");
                
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

fn parse_er(source: &str) -> anyhow::Result<Diagram> {
    let mut diagram = Diagram::default();
    diagram.diagram_type = DiagramType::ER;
    
    let mut relationships: Vec<Relationship> = Vec::new();
    let mut entities: Vec<Entity> = Vec::new();
    
    for line in source.lines() {
        let line = line.trim();
        
        // Skip comments and keywords
        if line.starts_with("%%") || line.starts_with("erDiagram") || line.is_empty() {
            continue;
        }
        
        // Parse entity definition: EntityName { ... }
        if line.contains("{") && line.contains("}") {
            let entity_name = line.split('{').next().unwrap_or("").trim();
            if !entity_name.is_empty() {
                let attributes = parse_er_attributes(line);
                entities.push(Entity {
                    name: entity_name.to_string(),
                    attributes,
                });
            }
        }
        
        // Parse relationships
        if line.contains("--") {
            let rel_types = ["||--", "}|--", "||--", "o{--", "}o--", "o|--"];
            for rel_type in rel_types {
                if line.contains(rel_type) {
                    let parts: Vec<&str> = line.split(rel_type).collect();
                    if parts.len() == 2 {
                        let from = parts[0].trim();
                        let to = parts[1].trim();
                        
                        // Extract relationship label if present
                        let rel_label = if line.contains(':') {
                            let label_parts: Vec<&str> = line.split(':').collect();
                            Some(label_parts.get(1).map(|s| s.trim()).unwrap_or("").to_string())
                        } else {
                            None
                        };
                        
                        relationships.push(Relationship {
                            from: from.to_string(),
                            to: to.to_string(),
                            rel_type: rel_type.to_string(),
                        });
                        
                        // Ensure entities exist
                        if !from.is_empty() {
                            diagram.nodes.push(Node::new(from, from));
                        }
                        if !to.is_empty() {
                            diagram.nodes.push(Node::new(to, to));
                        }
                    }
                }
            }
        }
    }
    
    diagram.entities = entities;
    diagram.relationships = relationships;
    Ok(diagram)
}

/// Parse ER entity attributes
fn parse_er_attributes(entity_def: &str) -> Vec<EntityAttribute> {
    let mut attrs = Vec::new();
    
    if let Some(start) = entity_def.find('{') {
        if let Some(end) = entity_def.find('}') {
            let body = &entity_def[start+1..end];
            
            for line in body.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                
                let is_pk = line.starts_with("*") || line.contains(" PK");
                let is_fk = line.starts_with("+") || line.contains(" FK");
                
                let line = line.trim_start_matches(&['*', '+'][..]);
                
                let parts: Vec<&str> = line.split_whitespace().collect();
                if !parts.is_empty() {
                    let name = parts[0].to_string();
                    let attr_type = parts.get(1).unwrap_or(&"string").to_string();
                    
                    attrs.push(EntityAttribute {
                        name,
                        attr_type,
                        is_primary_key: is_pk,
                        is_foreign_key: is_fk,
                    });
                }
            }
        }
    }
    
    attrs
}

fn parse_pie(source: &str) -> anyhow::Result<Diagram> {
    let mut diagram = Diagram::default();
    diagram.diagram_type = DiagramType::Pie;
    
    for line in source.lines() {
        let line = line.trim();
        
        if line.starts_with("pie") || line.starts_with("title") || line.starts_with("%%") || line.is_empty() {
            continue;
        }
        
        if line.contains(':') {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 2 {
                let label = parts[0].trim().trim_matches('"');
                let value = parts[1].trim();
                
                diagram.nodes.push(Node::new(label, format!("{}: {}", label, value)));
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
        let source = r#"graph LR
A --> B
B --> C"#;
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
    }
    
    #[test]
    fn test_parse_flowchart_dotted_arrow() {
        let source = "graph LR\nA -.-> B";
        let diagram = parse_mermaid(source).unwrap();
        assert_eq!(diagram.edges.len(), 1);
    }
    
    #[test]
    fn test_parse_sequence() {
        let source = r#"sequenceDiagram
Alice->>Bob: Hello
Bob-->>Alice: Hi"#;
        let diagram = parse_mermaid(source).unwrap();
        assert_eq!(diagram.diagram_type, DiagramType::Sequence);
        assert!(diagram.participants.contains(&"Alice".to_string()));
        assert!(diagram.participants.contains(&"Bob".to_string()));
    }
    
    #[test]
    fn test_parse_pie() {
        let source = r#"pie title Pets
"Dogs" : 386
"Cats" : 85"#;
        let diagram = parse_mermaid(source).unwrap();
        assert_eq!(diagram.diagram_type, DiagramType::Pie);
        assert_eq!(diagram.nodes.len(), 2);
    }
}

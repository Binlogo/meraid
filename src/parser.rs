//! Mermaid syntax parser

use regex::Regex;
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
    let mut diagram = Diagram {
        diagram_type: DiagramType::Flowchart,
        ..Default::default()
    };
    
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
    let mut diagram = Diagram {
        diagram_type: DiagramType::Sequence,
        ..Default::default()
    };
    
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
    let mut diagram = Diagram {
        diagram_type: DiagramType::Class,
        ..Default::default()
    };

    let mut nodes: std::collections::HashMap<String, Node> = std::collections::HashMap::new();
    let mut relationships: Vec<Relationship> = Vec::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Skip comments and empty lines
        if line.starts_with("%%") || line.is_empty() {
            i += 1;
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
                let mut full_def = line.to_string();
                let mut brace_balance = line.matches('{').count() as isize - line.matches('}').count() as isize;

                while brace_balance > 0 && i + 1 < lines.len() {
                    i += 1;
                    let next_line = lines[i].trim();
                    full_def.push('\n');
                    full_def.push_str(next_line);
                    brace_balance += next_line.matches('{').count() as isize;
                    brace_balance -= next_line.matches('}').count() as isize;
                }

                let members = parse_class_members(&full_def);
                let mut node = Node::new(class_name, class_name);
                node.members = members;
                nodes.insert(class_name.to_string(), node);
            }

            i += 1;
            continue;
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

        i += 1;
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
        let mut end = None;
        for (i, c) in rest.char_indices() {
            if c == '{' { depth += 1; }
            if c == '}' { depth -= 1; }
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
    let mut diagram = Diagram {
        diagram_type: DiagramType::State,
        ..Default::default()
    };
    
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
    let mut diagram = Diagram {
        diagram_type: DiagramType::ER,
        ..Default::default()
    };
    diagram.diagram_type = DiagramType::ER;
    
    let mut relationships: Vec<Relationship> = Vec::new();
    let mut entities: Vec<Entity> = Vec::new();
    
    // Extract entity definitions using regex: WORD { ... }
    let entity_regex = Regex::new(r"(\w+)\s*\{([^}]*)\}").unwrap();
    for cap in entity_regex.captures_iter(source) {
        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let body = cap.get(2).map(|m| m.as_str()).unwrap_or("");
        
        if !name.is_empty() {
            let attributes = parse_er_body(body);
            entities.push(Entity {
                name: name.to_string(),
                attributes,
            });
        }
    }
    
    // Extract relationships
    // Mermaid ER format: ENTITY1 cardinality1--cardinality2 ENTITY2 : label
    // Cardinality: || (exactly one), }| (one or more), o| (zero or one), o{ (zero or more)
    for line in source.lines() {
        let line = line.trim();
        
        if line.starts_with("%%") || line.starts_with("erDiagram") || line.is_empty() {
            continue;
        }
        
        // Skip entity definition blocks (lines with curly braces that are entity definitions)
        if line.contains('{') && line.contains('}') && !line.contains("--") {
            continue;
        }
        
        // Look for relationship patterns with -- in the middle
        if let Some(dash_pos) = line.find("--") {
            let before = &line[..dash_pos];
            let after = &line[dash_pos + 2..]; // Skip "--"
            
            // Parse left side: ENTITY + cardinality
            let before_parts: Vec<&str> = before.split_whitespace().collect();
            if before_parts.is_empty() {
                continue;
            }
            let from = before_parts[0];
            let left_card = before_parts.get(1).unwrap_or(&"");
            
            // Parse right side: cardinality + rest
            // Find where the cardinality ends and entity name begins
            // Cardinality symbols: ||, }|, }o, o{, o|
            let right_card_end = if after.starts_with("||") {
                2
            } else if after.starts_with("}|") || after.starts_with("}o") || after.starts_with("o{") || after.starts_with("o|") {
                2
            } else {
                0
            };
            
            if right_card_end == 0 {
                continue; // No valid cardinality found
            }
            
            let right_card = &after[..right_card_end];
            let rest = &after[right_card_end..].trim_start();
            
            // Extract entity name and optional label
            let (to, _label): (&str, Option<String>) = if let Some(space_pos) = rest.find(' ') {
                let entity = &rest[..space_pos];
                let remaining = &rest[space_pos..];
                let lbl = if let Some(colon_pos) = remaining.find(':') {
                    Some(remaining[colon_pos + 1..].trim().trim_matches('"').to_string())
                } else {
                    None
                };
                (entity, lbl)
            } else {
                (rest, None)
            };
            
            // Build relationship type
            let rel_type = format!("{}--{}", left_card, right_card);
            
            if !from.is_empty() && !to.is_empty() && from.chars().all(|c| c.is_alphanumeric() || c == '_') {
                relationships.push(Relationship {
                    from: from.to_string(),
                    to: to.to_string(),
                    rel_type,
                });
            }
        }
    }
    
    diagram.entities = entities;
    diagram.relationships = relationships;
    Ok(diagram)
}



fn parse_er_body(entity_def: &str) -> Vec<EntityAttribute> {
    let mut attrs = Vec::new();
    
    // The body is already extracted (between { and }), so we can process it directly
    for line in entity_def.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        
        let is_pk = line.contains(" PK") || line.ends_with(" PK");
        let is_fk = line.contains(" FK") || line.ends_with(" FK");
        
        // Parse format: "type name" or "type name PK" or "type name FK"
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let attr_type = parts[0].to_string();
            let name = parts[1].to_string();
            
            attrs.push(EntityAttribute {
                name,
                attr_type,
                is_primary_key: is_pk,
                is_foreign_key: is_fk,
            });
        }
    }
    
    attrs
}

fn parse_pie(source: &str) -> anyhow::Result<Diagram> {
    let mut diagram = Diagram {
        diagram_type: DiagramType::Pie,
        ..Default::default()
    };
    
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

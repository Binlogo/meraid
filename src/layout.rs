//! Layout engine for positioning diagram nodes

use crate::diagram::Diagram;

/// Layout result with positioned nodes
#[derive(Debug, Clone)]
pub struct LayoutResult {
    pub positions: std::collections::HashMap<String, Position>,
    pub width: usize,
    pub height: usize,
}

/// Position of a node in the layout
#[derive(Debug, Clone)]
pub struct Position {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

/// Layout engine using grid-based positioning
pub struct Layout<'a> {
    diagram: &'a Diagram,
}

impl<'a> Layout<'a> {
    pub fn new(diagram: &'a Diagram) -> Self {
        Self { diagram }
    }
    
    /// Perform layout calculation
    pub fn layout(&self) -> LayoutResult {
        match self.diagram.diagram_type {
            crate::diagram::DiagramType::Flowchart => self.layout_flowchart(),
            crate::diagram::DiagramType::Sequence => self.layout_sequence(),
            crate::diagram::DiagramType::Class => self.layout_class(),
            crate::diagram::DiagramType::State => self.layout_state(),
            crate::diagram::DiagramType::Pie => self.layout_pie(),
            _ => self.layout_flowchart(),
        }
    }
    
    fn layout_flowchart(&self) -> LayoutResult {
        let mut positions: std::collections::HashMap<String, Position> = std::collections::HashMap::new();
        
        // Simple left-to-right layout
        // Group nodes by their distance from start
        let mut distances: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        
        // Find starting nodes (nodes with no incoming edges)
        let all_targets: std::collections::HashSet<String> = self.diagram.edges.iter()
            .map(|e| e.to.clone())
            .collect();
        
        let mut starting_nodes: Vec<String> = self.diagram.nodes.iter()
            .filter(|n| !all_targets.contains(&n.id))
            .map(|n| n.id.clone())
            .collect();
        
        if starting_nodes.is_empty() && !self.diagram.nodes.is_empty() {
            starting_nodes.push(self.diagram.nodes[0].id.clone());
        }
        
        // BFS to find distances
        let mut queue: Vec<(String, usize)> = starting_nodes.iter().map(|n| (n.clone(), 0)).collect();
        while let Some((node_id, dist)) = queue.pop() {
            if distances.contains_key(&node_id) {
                continue;
            }
            distances.insert(node_id.clone(), dist);
            
            // Find outgoing edges
            for edge in &self.diagram.edges {
                if edge.from == node_id {
                    queue.push((edge.to.clone(), dist + 1));
                }
            }
        }
        
        // Assign positions based on distance (layer)
        let mut layer_nodes: std::collections::HashMap<usize, Vec<String>> = std::collections::HashMap::new();
        for (node_id, dist) in &distances {
            layer_nodes.entry(*dist).or_default().push(node_id.clone());
        }
        
        // Position nodes
        let box_width = 12;
        let box_height = 3;
        let horizontal_gap = 4;
        let vertical_gap = 2;
        
        let mut max_width = 0;
        let mut max_height = 0;
        
        for (layer, nodes) in &layer_nodes {
            let x = layer * (box_width + horizontal_gap);
            for (i, node_id) in nodes.iter().enumerate() {
                let y = i * (box_height + vertical_gap);
                positions.insert(node_id.clone(), Position {
                    x,
                    y,
                    width: box_width,
                    height: box_height,
                });
                max_height = max_height.max(y + box_height);
            }
            max_width = max_width.max(x + box_width);
        }
        
        LayoutResult {
            positions,
            width: max_width + 2,
            height: max_height + 2,
        }
    }
    
    fn layout_sequence(&self) -> LayoutResult {
        let mut positions: std::collections::HashMap<String, Position> = std::collections::HashMap::new();
        
        let box_width = 10;
        let box_height = 3;
        let horizontal_gap = 8;
        
        for (i, participant) in self.diagram.participants.iter().enumerate() {
            let x = i * (box_width + horizontal_gap);
            positions.insert(participant.clone(), Position {
                x,
                y: 0,
                width: box_width,
                height: box_height,
            });
        }
        
        LayoutResult {
            positions,
            width: self.diagram.participants.len() * (box_width + horizontal_gap) + 2,
            height: 20,
        }
    }
    
    fn layout_class(&self) -> LayoutResult {
        let mut positions: std::collections::HashMap<String, Position> = std::collections::HashMap::new();
        
        let box_width = 16;
        let box_height = 6;
        let horizontal_gap = 4;
        let vertical_gap = 2;
        
        for (i, node) in self.diagram.nodes.iter().enumerate() {
            let x = (i % 3) * (box_width + horizontal_gap);
            let y = (i / 3) * (box_height + vertical_gap);
            positions.insert(node.id.clone(), Position {
                x,
                y,
                width: box_width,
                height: box_height,
            });
        }
        
        LayoutResult {
            positions,
            width: 3 * (box_width + horizontal_gap) + 2,
            height: (self.diagram.nodes.len() / 3 + 1) * (box_height + vertical_gap) + 2,
        }
    }
    
    fn layout_state(&self) -> LayoutResult {
        // Similar to flowchart but vertical by default
        let mut positions: std::collections::HashMap<String, Position> = std::collections::HashMap::new();
        
        let box_width = 12;
        let box_height = 3;
        let horizontal_gap = 4;
        let vertical_gap = 3;
        
        // Find start states
        let mut processed: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut queue: Vec<String> = self.diagram.edges.iter()
            .filter(|e| e.from.is_empty() || e.from == "[*]")
            .map(|e| e.to.clone())
            .collect();
        
        if queue.is_empty() && !self.diagram.nodes.is_empty() {
            queue.push(self.diagram.nodes[0].id.clone());
        }
        
        let mut y = 0;
        while let Some(node_id) = queue.pop() {
            if processed.contains(&node_id) {
                continue;
            }
            processed.insert(node_id.clone());
            
            positions.insert(node_id.clone(), Position {
                x: 0,
                y,
                width: box_width,
                height: box_height,
            });
            
            y += box_height + vertical_gap;
            
            // Find next states
            for edge in &self.diagram.edges {
                if edge.from == node_id {
                    queue.push(edge.to.clone());
                }
            }
        }
        
        LayoutResult {
            positions,
            width: box_width + horizontal_gap + 2,
            height: y + 2,
        }
    }
    
    fn layout_pie(&self) -> LayoutResult {
        // Pie charts are rendered as horizontal bar charts
        let mut positions: std::collections::HashMap<String, Position> = std::collections::HashMap::new();
        
        let bar_height = 3;
        let horizontal_gap = 1;
        
        for (i, node) in self.diagram.nodes.iter().enumerate() {
            positions.insert(node.id.clone(), Position {
                x: 0,
                y: i * (bar_height + horizontal_gap),
                width: 50, // Will be scaled during rendering
                height: bar_height,
            });
        }
        
        LayoutResult {
            positions,
            width: 60,
            height: self.diagram.nodes.len() * (bar_height + horizontal_gap) + 2,
        }
    }
}

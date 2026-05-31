//! Layout engine for positioning diagram nodes

use crate::diagram::Diagram;
use unicode_width::UnicodeWidthStr;

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
        use std::collections::HashMap;

        let nodes = &self.diagram.nodes;
        let edges = &self.diagram.edges;

        // Map each node id to its parse-order index for stable ordering and
        // O(1) lookup while relaxing edges.
        let index_of: HashMap<&str, usize> = nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (n.id.as_str(), i))
            .collect();

        // Longest-path layering: a node's layer is the longest chain of edges
        // reaching it from any source. We relax every edge until the layers
        // stop changing (bounded by the node count, which also makes any cycle
        // terminate safely). This guarantees each edge points to a strictly
        // higher layer, so two edge-connected nodes never land in the same
        // column — the overlap that previously made boxes overprint.
        let mut layer: Vec<usize> = vec![0; nodes.len()];
        for _ in 0..nodes.len() {
            let mut changed = false;
            for e in edges {
                if let (Some(&u), Some(&v)) =
                    (index_of.get(e.from.as_str()), index_of.get(e.to.as_str()))
                {
                    if u != v && layer[v] < layer[u] + 1 {
                        layer[v] = layer[u] + 1;
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }

        // Group node indices by layer, preserving parse order within a layer so
        // the vertical stacking is deterministic between runs.
        let max_layer = layer.iter().copied().max().unwrap_or(0);
        let mut layers: Vec<Vec<usize>> = vec![Vec::new(); max_layer + 1];
        for (i, &l) in layer.iter().enumerate() {
            layers[l].push(i);
        }

        // Vertical coordinate assignment. Because every edge points to a
        // strictly higher layer, a node's predecessors are always in
        // already-placed layers — so a single left-to-right sweep suffices.
        let mut preds: Vec<Vec<usize>> = vec![Vec::new(); nodes.len()];
        for e in edges {
            if let (Some(&u), Some(&v)) =
                (index_of.get(e.from.as_str()), index_of.get(e.to.as_str()))
            {
                if u != v {
                    preds[v].push(u);
                }
            }
        }

        // Sweep the layers left to right, centring each node under the mean row
        // of its predecessors. Siblings that want the same row are pushed apart
        // to keep a minimum separation, then the whole group is recentred on
        // that shared target — so a decision node's branches straddle the trunk
        // symmetrically (one up, one down) while the trunk stays on one line.
        const MIN_SEP: f64 = 1.0;
        let mut rows: Vec<f64> = vec![0.0; nodes.len()];
        for layer_nodes in &layers {
            if layer_nodes.is_empty() {
                continue;
            }
            let mut targets: Vec<(usize, f64)> = layer_nodes
                .iter()
                .map(|&n| {
                    let t = if preds[n].is_empty() {
                        0.0
                    } else {
                        preds[n].iter().map(|&u| rows[u]).sum::<f64>() / preds[n].len() as f64
                    };
                    (n, t)
                })
                .collect();
            // Order by target row, parse order breaking ties.
            targets.sort_by(|a, b| {
                a.1.partial_cmp(&b.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(a.0.cmp(&b.0))
            });
            // Push apart to maintain the minimum separation.
            let mut placed: Vec<f64> = Vec::with_capacity(targets.len());
            let mut prev = f64::NEG_INFINITY;
            for &(_, t) in &targets {
                let y = t.max(prev + MIN_SEP);
                placed.push(y);
                prev = y;
            }
            // Recenter the group on the mean target so it straddles the trunk.
            let want = targets.iter().map(|&(_, t)| t).sum::<f64>() / targets.len() as f64;
            let have = placed.iter().sum::<f64>() / placed.len() as f64;
            let shift = want - have;
            for (&(n, _), &y) in targets.iter().zip(placed.iter()) {
                rows[n] = y + shift;
            }
        }

        // Size boxes to the widest label, and the inter-column gap to the widest
        // edge label, so branch labels like "通过"/"失败" render without clipping.
        let box_height = 3;
        let vertical_gap = 2;
        let row_pitch = (box_height + vertical_gap) as f64;
        let box_width = nodes
            .iter()
            .map(|n| UnicodeWidthStr::width(n.label.as_str()) + 2)
            .max()
            .unwrap_or(12)
            .max(12);
        let edge_label_w = edges
            .iter()
            .filter_map(|e| e.label.as_deref())
            .map(UnicodeWidthStr::width)
            .max()
            .unwrap_or(0);
        let horizontal_gap = 4.max(edge_label_w + 4);

        // Map fractional rows onto the integer canvas grid.
        let min_row = rows.iter().copied().fold(f64::INFINITY, f64::min);
        let min_row = if min_row.is_finite() { min_row } else { 0.0 };

        let mut positions: HashMap<String, Position> = HashMap::new();
        let mut max_width = 0;
        let mut max_height = 0;
        for (l, layer_nodes) in layers.iter().enumerate() {
            let x = l * (box_width + horizontal_gap);
            for &n in layer_nodes {
                let y = ((rows[n] - min_row) * row_pitch).round().max(0.0) as usize;
                positions.insert(
                    nodes[n].id.clone(),
                    Position {
                        x,
                        y,
                        width: box_width,
                        height: box_height,
                    },
                );
                max_height = max_height.max(y + box_height);
            }
            if !layer_nodes.is_empty() {
                max_width = max_width.max(x + box_width);
            }
        }

        LayoutResult {
            positions,
            width: max_width + 2,
            height: max_height + 2,
        }
    }

    fn layout_sequence(&self) -> LayoutResult {
        let mut positions: std::collections::HashMap<String, Position> =
            std::collections::HashMap::new();

        let box_width = 10;
        let box_height = 3;
        let horizontal_gap = 8;

        for (i, participant) in self.diagram.participants.iter().enumerate() {
            let x = i * (box_width + horizontal_gap);
            positions.insert(
                participant.clone(),
                Position {
                    x,
                    y: 0,
                    width: box_width,
                    height: box_height,
                },
            );
        }

        LayoutResult {
            positions,
            width: self.diagram.participants.len() * (box_width + horizontal_gap) + 2,
            height: 20,
        }
    }

    fn layout_class(&self) -> LayoutResult {
        let mut positions: std::collections::HashMap<String, Position> =
            std::collections::HashMap::new();

        let box_width = 16;
        let box_height = 6;
        let horizontal_gap = 4;
        let vertical_gap = 2;

        for (i, node) in self.diagram.nodes.iter().enumerate() {
            let x = (i % 3) * (box_width + horizontal_gap);
            let y = (i / 3) * (box_height + vertical_gap);
            positions.insert(
                node.id.clone(),
                Position {
                    x,
                    y,
                    width: box_width,
                    height: box_height,
                },
            );
        }

        LayoutResult {
            positions,
            width: 3 * (box_width + horizontal_gap) + 2,
            height: (self.diagram.nodes.len() / 3 + 1) * (box_height + vertical_gap) + 2,
        }
    }

    fn layout_state(&self) -> LayoutResult {
        // Similar to flowchart but vertical by default
        let mut positions: std::collections::HashMap<String, Position> =
            std::collections::HashMap::new();

        let box_width = 12;
        let box_height = 3;
        let horizontal_gap = 4;
        let vertical_gap = 3;

        // Find start states
        let mut processed: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut queue: Vec<String> = self
            .diagram
            .edges
            .iter()
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

            positions.insert(
                node_id.clone(),
                Position {
                    x: 0,
                    y,
                    width: box_width,
                    height: box_height,
                },
            );

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
        let mut positions: std::collections::HashMap<String, Position> =
            std::collections::HashMap::new();

        let bar_height = 3;
        let horizontal_gap = 1;

        for (i, node) in self.diagram.nodes.iter().enumerate() {
            positions.insert(
                node.id.clone(),
                Position {
                    x: 0,
                    y: i * (bar_height + horizontal_gap),
                    width: 50, // Will be scaled during rendering
                    height: bar_height,
                },
            );
        }

        LayoutResult {
            positions,
            width: 60,
            height: self.diagram.nodes.len() * (bar_height + horizontal_gap) + 2,
        }
    }
}

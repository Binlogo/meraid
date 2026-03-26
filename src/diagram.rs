//! Diagram types representing parsed Mermaid graphs

use serde::{Deserialize, Serialize};

/// Types of Mermaid diagrams
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagramType {
    Flowchart,
    Sequence,
    Class,
    State,
    ER,
    Pie,
    GitGraph,
    Block,
    Treemap,
    Unknown,
}

/// Node shape styles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeShape {
    Rectangle,      // [text]
    Rounded,        // (text)
    Diamond,        // {text}
    Stadium,        // ([text])
    Subroutine,     // [[text]]
    Circle,         // ((text))
    DoubleCircle,   // (((text)))
    Hexagon,        // {{text}}
    Cylinder,       // [(text)]
    Asymmetric,     // >text]
    Parallelogram,  // [/text/]
    Trapezoid,      // [/text\]
}

/// Edge/Arrow styles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeStyle {
    Solid,          // -->
    Dotted,        // -.->
    Thick,         // ==>
    Bidirectional,  // <-->
    CircleEnd,     // --o
    CrossEnd,      // --x
}

/// A node in the diagram
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub label: String,
    pub shape: NodeShape,
    pub style: Option<String>,
    pub class: Option<String>,
    pub members: Vec<ClassMember>,
}

/// Class member (field or method)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassMember {
    pub name: String,
    pub member_type: MemberType,
    pub visibility: Visibility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberType {
    Field,
    Method,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Visibility {
    Public,    // +
    Private,   // -
    Protected, // #
    Package,   // ~
}

/// An edge connecting nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
    pub style: EdgeStyle,
}

/// A complete parsed diagram
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagram {
    pub diagram_type: DiagramType,
    pub direction: String,  // LR, RL, TB, BT
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub relationships: Vec<Relationship>,
    pub participants: Vec<String>,
}

impl Default for Diagram {
    fn default() -> Self {
        Self {
            diagram_type: DiagramType::Flowchart,
            direction: "LR".to_string(),
            nodes: Vec::new(),
            edges: Vec::new(),
            relationships: Vec::new(),
            participants: Vec::new(),
        }
    }
}

/// Class diagram relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub from: String,
    pub to: String,
    pub rel_type: String,  // <|--, *--, o--, --, ..>, ..|>
}

impl Node {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            shape: NodeShape::Rectangle,
            style: None,
            class: None,
            members: Vec::new(),
        }
    }
}

impl Edge {
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            label: None,
            style: EdgeStyle::Solid,
        }
    }
}

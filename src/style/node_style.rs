use crate::graph::NodeType;

pub struct NodeStyle;

impl NodeStyle {
    pub fn get_shape(node: &NodeType) -> String {
        match node {
            NodeType::Start(_) => "oval".to_string(),
            NodeType::End(_) => "oval".to_string(),
            NodeType::BasicBlock(_) => "box".to_string(),
            NodeType::Condition(_) => "diamond".to_string(),
            NodeType::Loop(_) => "hexagon".to_string(),
        }
    }

    pub fn get_style(node: &NodeType) -> String {
        match node {
            NodeType::Start(_) | NodeType::End(_) => "filled".to_string(),
            NodeType::Condition(_) => "filled".to_string(),
            NodeType::Loop(_) => "filled".to_string(),
            _ => "filled".to_string(),
        }
    }

    pub fn get_fillcolor(node: &NodeType) -> String {
        match node {
            NodeType::Start(_) => "lightgreen".to_string(),
            NodeType::End(_) => "lightpink".to_string(),
            NodeType::BasicBlock(_) => "lightblue".to_string(),
            NodeType::Condition(_) => "lightyellow".to_string(),
            NodeType::Loop(_) => "lightgray".to_string(),
        }
    }

    pub fn get_label(node: &NodeType) -> String {
        match node {
            NodeType::Start(name) => format!("Start: {}", name),
            NodeType::End(name) => format!("End: {}", name),
            NodeType::Condition(cond) => format!("Condition: {}", cond),
            NodeType::Loop(label) => format!("Loop: {}", label),
            NodeType::BasicBlock(code) => code.clone(),
        }
    }
} 
use crate::graph::NodeType;

pub struct NodeStyle;

impl NodeStyle {
    pub fn get_shape(node: &NodeType) -> String {
        match node {
            NodeType::Start(_, _) => "oval".to_string(),
            NodeType::End(_, _) => "oval".to_string(),
            NodeType::BasicBlock(_) => "box".to_string(),
            NodeType::Condition(_) => "diamond".to_string(),
            NodeType::Loop(_) => "hexagon".to_string(),
        }
    }

    pub fn get_style(node: &NodeType) -> String {
        match node {
            NodeType::Start(_, _) | NodeType::End(_, _) => "filled".to_string(),
            NodeType::Condition(_) => "filled".to_string(),
            NodeType::Loop(_) => "filled".to_string(),
            NodeType::BasicBlock(_) => "filled".to_string(),
        }
    }

    pub fn get_fillcolor(node: &NodeType) -> String {
        match node {
            NodeType::Start(_, is_test) => {
                if *is_test {
                    "palegreen".to_string()
                } else {
                    "lightgreen".to_string()
                }
            },
            NodeType::End(_, is_test) => {
                if *is_test {
                    "mistyrose".to_string()
                } else {
                    "lightpink".to_string()
                }
            },
            NodeType::BasicBlock(_) => "lightblue".to_string(),
            NodeType::Condition(_) => "lightyellow".to_string(),
            NodeType::Loop(_) => "lightgray".to_string(),
        }
    }

    pub fn get_label(node: &NodeType) -> String {
        node.label()
    }
} 
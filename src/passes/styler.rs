use crate::graph::FlowGraph;
use crate::style::{NodeStyle, EdgeStyle};

pub struct StyledGraph {
    pub nodes: Vec<StyledNode>,
    pub edges: Vec<StyledEdge>,
}

pub struct StyledNode {
    pub id: petgraph::graph::NodeIndex,
    pub shape: String,
    pub style: String,
    pub fillcolor: String,
    pub label: String,
}

pub struct StyledEdge {
    pub from: petgraph::graph::NodeIndex,
    pub to: petgraph::graph::NodeIndex,
    pub color: String,
    pub style: String,
    pub label: String,
}

impl Default for StyledGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl StyledGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
}

pub struct StylerPass;

impl StylerPass {
    pub fn apply_style(graph: &FlowGraph) -> StyledGraph {
        let mut styled = StyledGraph::new();
        
        // 处理节点
        for (id, node) in graph.nodes() {
            let shape = NodeStyle::get_shape(node);
            let style = NodeStyle::get_style(node);
            let fillcolor = NodeStyle::get_fillcolor(node);
            let label = NodeStyle::get_label(node);
            
            styled.nodes.push(StyledNode {
                id,
                shape,
                style,
                fillcolor,
                label,
            });
        }
        
        // 处理边
        for (from, to, weight) in graph.edges() {
            let (color, style) = EdgeStyle::get_color_and_style(weight);
            styled.edges.push(StyledEdge {
                from,
                to,
                color,
                style,
                label: weight.clone(),
            });
        }
        
        styled
    }
} 
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::{IntoNodeReferences, EdgeRef};
use std::collections::HashMap;
use crate::graph::NodeType;
use crate::passes::{StylerPass, DotRendererPass};

pub struct FlowGraph {
    pub(crate) graph: DiGraph<NodeType, String>,
    #[allow(dead_code)]
    node_map: HashMap<String, NodeIndex>,
}

impl Default for FlowGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl FlowGraph {
    pub fn new() -> Self {
        FlowGraph {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node_type: NodeType) -> NodeIndex {
        let id = self.graph.add_node(node_type);
        id
    }

    pub fn add_edge(&mut self, from: NodeIndex, to: NodeIndex, label: String) {
        self.graph.add_edge(from, to, label);
    }

    pub fn to_dot(&self) -> String {
        let styled = StylerPass::apply_style(self);
        DotRendererPass::render(&styled)
    }

    pub fn nodes(&self) -> impl Iterator<Item = (NodeIndex, &NodeType)> {
        self.graph.node_references()
    }

    pub fn edges(&self) -> impl Iterator<Item = (NodeIndex, NodeIndex, &String)> {
        self.graph.edge_references()
            .map(|e| (e.source(), e.target(), e.weight()))
    }
} 
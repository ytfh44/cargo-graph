use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::{IntoNodeReferences, EdgeRef};
use std::collections::{HashMap, HashSet};
use crate::graph::NodeType;
use crate::passes::{StylerPass, DotRendererPass};

#[derive(Debug, Clone)]
pub struct GraphConfig {
    pub include_tests: bool,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self {
            include_tests: false,
        }
    }
}

pub struct FlowGraph {
    pub(crate) graph: DiGraph<NodeType, String>,
    #[allow(dead_code)]
    node_map: HashMap<String, NodeIndex>,
    config: GraphConfig,
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
            config: GraphConfig::default(),
        }
    }

    pub fn with_config(config: GraphConfig) -> Self {
        FlowGraph {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
            config,
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

    fn get_visible_nodes(&self) -> HashSet<NodeIndex> {
        self.graph.node_references()
            .filter(|(_, node)| self.config.include_tests || !node.is_test())
            .map(|(id, _)| id)
            .collect()
    }

    pub fn nodes(&self) -> impl Iterator<Item = (NodeIndex, &NodeType)> {
        let visible_nodes = self.get_visible_nodes();
        self.graph.node_references()
            .filter(move |(id, node)| {
                if !self.config.include_tests && node.is_test() {
                    return false;
                }
                visible_nodes.contains(id)
            })
    }

    pub fn edges(&self) -> impl Iterator<Item = (NodeIndex, NodeIndex, &String)> {
        let visible_nodes = self.get_visible_nodes();
        self.graph.edge_references()
            .filter(move |e| {
                let source_node = self.graph.node_weight(e.source()).unwrap();
                let target_node = self.graph.node_weight(e.target()).unwrap();
                
                if !self.config.include_tests && (source_node.is_test() || target_node.is_test()) {
                    return false;
                }

                visible_nodes.contains(&e.source()) && visible_nodes.contains(&e.target())
            })
            .map(|e| (e.source(), e.target(), e.weight()))
    }

    pub fn config(&self) -> &GraphConfig {
        &self.config
    }

    pub fn set_config(&mut self, config: GraphConfig) {
        self.config = config;
    }
} 
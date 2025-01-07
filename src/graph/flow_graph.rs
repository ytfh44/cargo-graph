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

    fn get_function_nodes(&self, start_node: NodeIndex) -> HashSet<NodeIndex> {
        let mut nodes = HashSet::new();
        let mut stack = vec![start_node];
        
        while let Some(node_id) = stack.pop() {
            if nodes.insert(node_id) {
                for edge in self.graph.edges(node_id) {
                    stack.push(edge.target());
                }
            }
        }
        nodes
    }

    fn is_function_start(&self, node_id: NodeIndex) -> bool {
        if let Some(NodeType::Start(_, _)) = self.graph.node_weight(node_id) {
            true
        } else {
            false
        }
    }

    fn get_visible_nodes(&self) -> HashSet<NodeIndex> {
        let mut visible_nodes = HashSet::new();
        let mut test_function_nodes = HashSet::new();

        for (id, node) in self.graph.node_references() {
            if let NodeType::Start(_, is_test) = node {
                if *is_test {
                    test_function_nodes.extend(self.get_function_nodes(id));
                }
            }
        }

        for (id, _) in self.graph.node_references() {
            if self.config.include_tests || !test_function_nodes.contains(&id) {
                visible_nodes.insert(id);
            }
        }

        visible_nodes
    }

    pub fn nodes(&self) -> impl Iterator<Item = (NodeIndex, &NodeType)> {
        let visible_nodes = self.get_visible_nodes();
        self.graph.node_references()
            .filter(move |(id, _)| visible_nodes.contains(id))
    }

    pub fn edges(&self) -> impl Iterator<Item = (NodeIndex, NodeIndex, &String)> {
        let visible_nodes = self.get_visible_nodes();
        self.graph.edge_references()
            .filter(move |e| {
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
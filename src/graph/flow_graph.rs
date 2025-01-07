use petgraph::graph::{DiGraph, NodeIndex, Graph};
use petgraph::visit::{IntoNodeReferences, EdgeRef, DfsPostOrder};
use petgraph::Direction;
use std::collections::{HashMap, HashSet, VecDeque};
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

#[derive(Clone)]
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
        let mut merged_graph = self.clone();
        merged_graph.merge_basic_blocks();
        let styled = StylerPass::apply_style(&merged_graph);
        DotRendererPass::render(&styled)
    }

    fn merge_basic_blocks(&mut self) {
        let mut merged: HashSet<NodeIndex> = HashSet::new();
        let mut to_merge: VecDeque<NodeIndex> = VecDeque::new();
        let mut function_starts: HashSet<NodeIndex> = HashSet::new();

        // 首先收集所有函数的开始节点
        for node_id in self.graph.node_indices() {
            if let Some(NodeType::Start(_, _)) = self.graph.node_weight(node_id) {
                function_starts.insert(node_id);
            }
        }

        // 收集可合并的基本块
        for node_id in self.graph.node_indices() {
            if !merged.contains(&node_id) && 
               !function_starts.contains(&node_id) && 
               self.is_mergeable_block(node_id) {
                to_merge.push_back(node_id);
            }
        }

        while let Some(start_node) = to_merge.pop_front() {
            if merged.contains(&start_node) {
                continue;
            }

            // 收集可合并的序列
            let sequence = self.collect_mergeable_sequence(start_node, &merged, &function_starts);
            
            if sequence.len() > 1 {
                // 验证序列的有效性
                if self.validate_merge_sequence(&sequence) {
                    self.merge_sequence(&sequence);
                    merged.extend(sequence.iter());
                }
            }
        }
    }

    fn validate_merge_sequence(&self, sequence: &[NodeIndex]) -> bool {
        // 检查序列中的所有节点是否都存在且是基本块
        for &node_id in sequence {
            if self.graph.node_weight(node_id).is_none() {
                return false;
            }
            if !matches!(self.graph.node_weight(node_id), Some(NodeType::BasicBlock(_))) {
                return false;
            }
        }

        // 检查序列中的节点是否正确连接
        for window in sequence.windows(2) {
            let current = window[0];
            let next = window[1];
            
            // 验证边的连接
            let has_edge = self.graph.edges_directed(current, Direction::Outgoing)
                .any(|e| e.target() == next);
            
            if !has_edge {
                return false;
            }
        }

        true
    }

    fn collect_mergeable_sequence(
        &self,
        start_node: NodeIndex,
        merged: &HashSet<NodeIndex>,
        function_starts: &HashSet<NodeIndex>
    ) -> Vec<NodeIndex> {
        let mut sequence = Vec::new();
        let mut current = start_node;

        sequence.push(current);

        while let Some(next) = self.get_single_successor(current) {
            if merged.contains(&next) || 
               function_starts.contains(&next) || 
               !self.is_mergeable_block(next) {
                break;
            }
            sequence.push(next);
            current = next;
        }

        sequence
    }

    fn is_mergeable_block(&self, node_id: NodeIndex) -> bool {
        if let Some(NodeType::BasicBlock(_)) = self.graph.node_weight(node_id) {
            let in_degree = self.graph.edges_directed(node_id, Direction::Incoming).count();
            let out_degree = self.graph.edges_directed(node_id, Direction::Outgoing).count();
            
            // 确保节点有且仅有一个前驱和一个后继
            if in_degree != 1 || out_degree != 1 {
                return false;
            }

            // 检查前驱和后继节点的类型
            let has_valid_neighbors = self.graph.edges_directed(node_id, Direction::Incoming)
                .all(|e| self.is_valid_neighbor(e.source())) &&
                self.graph.edges_directed(node_id, Direction::Outgoing)
                .all(|e| self.is_valid_neighbor(e.target()));

            has_valid_neighbors
        } else {
            false
        }
    }

    fn is_valid_neighbor(&self, node_id: NodeIndex) -> bool {
        if let Some(node_type) = self.graph.node_weight(node_id) {
            match node_type {
                NodeType::Start(_, _) | NodeType::End(_, _) => false,
                _ => true
            }
        } else {
            false
        }
    }

    fn get_single_successor(&self, node_id: NodeIndex) -> Option<NodeIndex> {
        let mut successors = self.graph.neighbors_directed(node_id, Direction::Outgoing);
        let next = successors.next();
        if successors.next().is_none() {
            next
        } else {
            None
        }
    }

    fn merge_sequence(&mut self, sequence: &[NodeIndex]) {
        if sequence.is_empty() {
            return;
        }

        // 保存所有需要的边信息
        let first = sequence[0];
        let last = *sequence.last().unwrap();
        
        // 收集入边（除了第一个节点的）
        let in_edges: Vec<_> = sequence.iter().skip(1)
            .flat_map(|&node_id| {
                self.graph.edges_directed(node_id, Direction::Incoming)
                    .map(|e| (e.source(), e.target(), e.weight().clone()))
                    .collect::<Vec<_>>()
            })
            .collect();

        // 收集出边（除了最后一个节点的）
        let out_edges: Vec<_> = sequence.iter().take(sequence.len() - 1)
            .flat_map(|&node_id| {
                self.graph.edges_directed(node_id, Direction::Outgoing)
                    .map(|e| (e.source(), e.target(), e.weight().clone()))
                    .collect::<Vec<_>>()
            })
            .collect();

        // 合并内容
        let mut merged_content = String::new();
        for &node_id in sequence {
            if let Some(NodeType::BasicBlock(content)) = self.graph.node_weight(node_id) {
                if !merged_content.is_empty() {
                    merged_content.push_str("\n");
                }
                merged_content.push_str(content);
            }
        }

        // 更新第一个节点的内容
        if let Some(node_weight) = self.graph.node_weight_mut(first) {
            *node_weight = NodeType::BasicBlock(merged_content);
        }

        // 删除其他节点
        for &node_id in &sequence[1..] {
            self.graph.remove_node(node_id);
        }

        // 重新连接需要保留的边
        for (source, _, weight) in in_edges {
            if source != first && self.graph.node_weight(source).is_some() {
                self.graph.add_edge(source, first, weight);
            }
        }

        for (_, target, weight) in out_edges {
            if target != last && self.graph.node_weight(target).is_some() {
                self.graph.add_edge(first, target, weight);
            }
        }
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
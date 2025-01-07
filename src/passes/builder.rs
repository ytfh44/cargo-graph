use crate::graph::{FlowGraph, GraphConfig};
use crate::passes::{ControlFlowAnalyzerPass, ParserPass};
use syn::ItemFn;

pub struct GraphBuilderPass {
    graph: FlowGraph,
}

impl Default for GraphBuilderPass {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphBuilderPass {
    pub fn new() -> Self {
        Self {
            graph: FlowGraph::new(),
        }
    }

    pub fn with_config(config: GraphConfig) -> Self {
        Self {
            graph: FlowGraph::with_config(config),
        }
    }
    
    pub fn build(functions: Vec<ItemFn>) -> FlowGraph {
        Self::build_with_config(functions, GraphConfig::default())
    }

    pub fn build_with_config(functions: Vec<ItemFn>, config: GraphConfig) -> FlowGraph {
        let mut builder = Self::with_config(config);
        let mut analyzer = ControlFlowAnalyzerPass::new(&mut builder.graph);
        
        for func in functions {
            analyzer.analyze_function(&func);
        }
        
        builder.graph
    }
} 
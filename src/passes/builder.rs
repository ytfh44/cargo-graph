use crate::graph::FlowGraph;
use crate::passes::ControlFlowAnalyzerPass;
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
    
    pub fn build(functions: Vec<ItemFn>) -> FlowGraph {
        let mut builder = Self::new();
        let mut analyzer = ControlFlowAnalyzerPass::new(&mut builder.graph);
        
        for func in functions {
            analyzer.analyze_function(&func);
        }
        
        builder.graph
    }
} 
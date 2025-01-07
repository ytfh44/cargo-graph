use anyhow::Result;
use std::fs;
use std::path::Path;

mod graph;
mod passes;
mod style;

pub use graph::{FlowGraph, NodeType};
pub use passes::*;

pub trait GraphRenderer {
    fn render(&self, graph: &FlowGraph) -> Result<String>;
    fn style(&self) -> &str;
    fn template(&self) -> &str;
}

pub struct DotRenderer {
    graph_type: String,
}

impl Default for DotRenderer {
    fn default() -> Self {
        Self {
            graph_type: "default".to_string(),
        }
    }
}

impl GraphRenderer for DotRenderer {
    fn render(&self, graph: &FlowGraph) -> Result<String> {
        let styled = StylerPass::apply_style(graph);
        Ok(DotRendererPass::render(&styled))
    }

    fn style(&self) -> &str {
        &self.graph_type
    }

    fn template(&self) -> &str {
        "default"
    }
}

pub struct CStyleFlowchartRenderer {
    template: String,
}

impl Default for CStyleFlowchartRenderer {
    fn default() -> Self {
        Self {
            template: "c-style".to_string(),
        }
    }
}

impl GraphRenderer for CStyleFlowchartRenderer {
    fn render(&self, graph: &FlowGraph) -> Result<String> {
        let styled = StylerPass::apply_style(graph);
        Ok(DotRendererPass::render(&styled))
    }

    fn style(&self) -> &str {
        "c-style"
    }

    fn template(&self) -> &str {
        &self.template
    }
}

pub fn analyze_file_with_renderer<R: GraphRenderer + ?Sized>(
    path: &Path,
    renderer: &R
) -> Result<String> {
    // 1. 读取源码
    let source = fs::read_to_string(path)?;
    
    // 2. 解析源码
    let ast = ParserPass::parse(&source)?;
    
    // 3. 收集函数
    let functions = FunctionCollectorPass::collect(&ast);
    
    // 4. 构建控制流图
    let flow_graph = GraphBuilderPass::build(functions);
    
    // 5. 渲染图
    renderer.render(&flow_graph)
} 
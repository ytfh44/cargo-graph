use cargo_graph::{CStyleFlowchartRenderer, FlowGraph, NodeType, GraphRenderer};
use anyhow::Result;

fn main() -> Result<()> {
    let renderer = CStyleFlowchartRenderer::default();
    let mut graph = FlowGraph::new();
    
    // 创建节点
    let start = graph.add_node(NodeType::Start("render".to_string()));
    let style = graph.add_node(NodeType::BasicBlock("获取样式".to_string()));
    let cond = graph.add_node(NodeType::Condition("是否为基本块?".to_string()));
    let basic = graph.add_node(NodeType::BasicBlock("应用圆角样式".to_string()));
    let other = graph.add_node(NodeType::BasicBlock("应用填充样式".to_string()));
    let end = graph.add_node(NodeType::End("render".to_string()));
    
    // 添加边
    graph.add_edge(start, style, "开始".to_string());
    graph.add_edge(style, cond, "获取完成".to_string());
    graph.add_edge(cond, basic, "是".to_string());
    graph.add_edge(cond, other, "否".to_string());
    graph.add_edge(basic, end, "完成".to_string());
    graph.add_edge(other, end, "完成".to_string());
    
    // 渲染并保存为SVG
    let dot = graph.render(&renderer)?;
    std::fs::write("render_flow.dot", dot)?;
    
    // 使用graphviz转换为SVG
    std::process::Command::new("dot")
        .args(["-Tsvg", "render_flow.dot", "-o", "render_flow.svg"])
        .status()?;
        
    println!("流程图已生成: render_flow.svg");
    Ok(())
}

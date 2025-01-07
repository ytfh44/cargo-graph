use anyhow::{Context, Result};
use petgraph::dot::Dot;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use syn::visit::{self, Visit};
use syn::{Block, Expr, ExprIf, ExprLoop, ExprMatch, ExprWhile, ItemFn, Stmt};

#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Start(String),           // 函数开始
    End(String),            // 函数结束
    BasicBlock(String),     // 基本代码块
    Condition(String),      // if/match条件
    Loop(String),          // 循环结构
}

impl NodeType {
    fn label(&self) -> String {
        match self {
            NodeType::Start(name) => format!("Start: {}", name),
            NodeType::End(name) => format!("End: {}", name),
            NodeType::BasicBlock(content) => content.clone(),
            NodeType::Condition(cond) => format!("Condition: {}", cond),
            NodeType::Loop(kind) => format!("Loop: {}", kind),
        }
    }
}

pub struct FlowGraph {
    graph: DiGraph<NodeType, String>,
    node_map: HashMap<String, NodeIndex>,
}

impl FlowGraph {
    pub fn new() -> Self {
        FlowGraph {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node_type: NodeType) -> NodeIndex {
        let idx = self.graph.add_node(node_type);
        idx
    }

    pub fn add_edge(&mut self, from: NodeIndex, to: NodeIndex, label: String) {
        self.graph.add_edge(from, to, label);
    }

    pub fn to_dot(&self) -> String {
        let dot = Dot::new(&self.graph);
        format!("{:?}", dot)
    }

    pub fn render<R: GraphRenderer>(&self, renderer: &R) -> Result<String> {
        renderer.render(self)
    }
}

struct ControlFlowVisitor<'a> {
    graph: &'a mut FlowGraph,
    current_node: Option<NodeIndex>,
    fn_start_node: Option<NodeIndex>,
    fn_end_node: Option<NodeIndex>,
}

impl<'a> ControlFlowVisitor<'a> {
    fn new(graph: &'a mut FlowGraph) -> Self {
        ControlFlowVisitor {
            graph,
            current_node: None,
            fn_start_node: None,
            fn_end_node: None,
        }
    }

    fn visit_block(&mut self, block: &Block, parent: Option<NodeIndex>) -> NodeIndex {
        let mut last_node = parent.unwrap_or_else(|| self.current_node.unwrap());
        
        for stmt in &block.stmts {
            match stmt {
                Stmt::Expr(expr, _) => {
                    match expr {
                        Expr::If(expr_if) => {
                            last_node = self.visit_if(expr_if, last_node);
                        }
                        Expr::While(expr_while) => {
                            last_node = self.visit_while(expr_while, last_node);
                        }
                        Expr::Loop(expr_loop) => {
                            last_node = self.visit_loop(expr_loop, last_node);
                        }
                        Expr::Match(expr_match) => {
                            last_node = self.visit_match(expr_match, last_node);
                        }
                        _ => {
                            // 创建基本块节点
                            let basic_block = self.graph.add_node(NodeType::BasicBlock(
                                format!("{}", quote::quote!(#expr))
                            ));
                            self.graph.add_edge(last_node, basic_block, "next".to_string());
                            last_node = basic_block;
                        }
                    }
                }
                _ => {
                    // 其他语句类型作为基本块处理
                    let basic_block = self.graph.add_node(NodeType::BasicBlock(
                        format!("{}", quote::quote!(#stmt))
                    ));
                    self.graph.add_edge(last_node, basic_block, "next".to_string());
                    last_node = basic_block;
                }
            }
        }
        
        last_node
    }

    fn visit_if(&mut self, expr_if: &ExprIf, parent: NodeIndex) -> NodeIndex {
        // 创建条件节点，使用更清晰的条件表达式
        let cond_text = format!("{}", quote::quote!(#expr_if.cond));
        let cond_node = self.graph.add_node(NodeType::Condition(cond_text));
        self.graph.add_edge(parent, cond_node, "进入判断".to_string());

        // 处理 then 分支
        let then_node = self.visit_block(&expr_if.then_branch, Some(cond_node));
        self.graph.add_edge(cond_node, then_node, "是".to_string());

        // 处理 else 分支
        let merge_node = self.graph.add_node(NodeType::BasicBlock("分支合并点".to_string()));
        if let Some((_, else_branch)) = &expr_if.else_branch {
            let else_node = match &**else_branch {
                Expr::Block(block) => self.visit_block(&block.block, Some(cond_node)),
                Expr::If(else_if) => self.visit_if(else_if, cond_node),
                _ => unreachable!(),
            };
            self.graph.add_edge(cond_node, else_node, "否".to_string());
            self.graph.add_edge(else_node, merge_node, "完成分支".to_string());
        } else {
            self.graph.add_edge(cond_node, merge_node, "否".to_string());
        }

        self.graph.add_edge(then_node, merge_node, "完成分支".to_string());
        merge_node
    }

    fn visit_while(&mut self, expr_while: &ExprWhile, parent: NodeIndex) -> NodeIndex {
        // 创建循环入口节点
        let loop_entry = self.graph.add_node(NodeType::BasicBlock("循环入口".to_string()));
        self.graph.add_edge(parent, loop_entry, "进入循环".to_string());

        // 创建条件节点
        let cond_text = format!("{}", quote::quote!(#expr_while.cond));
        let cond_node = self.graph.add_node(NodeType::Condition(cond_text));
        self.graph.add_edge(loop_entry, cond_node, "检查条件".to_string());

        // 处理循环体
        let body_node = self.visit_block(&expr_while.body, Some(cond_node));
        self.graph.add_edge(cond_node, body_node, "是".to_string());
        
        // 创建循环回边
        self.graph.add_edge(body_node, loop_entry, "继续循环".to_string());

        // 创建循环出口
        let exit_node = self.graph.add_node(NodeType::BasicBlock("循环结束".to_string()));
        self.graph.add_edge(cond_node, exit_node, "否".to_string());
        
        exit_node
    }

    fn visit_loop(&mut self, expr_loop: &ExprLoop, parent: NodeIndex) -> NodeIndex {
        // 创建循环入口节点
        let loop_entry = self.graph.add_node(NodeType::Loop("无条件循环".to_string()));
        self.graph.add_edge(parent, loop_entry, "进入循环".to_string());

        // 处理循环体
        let body_node = self.visit_block(&expr_loop.body, Some(loop_entry));
        
        // 创建循环回边
        self.graph.add_edge(body_node, loop_entry, "继续循环".to_string());

        // 创建循环出口（用于break语句）
        let exit_node = self.graph.add_node(NodeType::BasicBlock("循环结束".to_string()));
        self.graph.add_edge(loop_entry, exit_node, "跳出循环".to_string());
        
        exit_node
    }

    fn visit_match(&mut self, expr_match: &ExprMatch, parent: NodeIndex) -> NodeIndex {
        let match_node = self.graph.add_node(NodeType::Condition(
            format!("match {}", quote::quote!(#expr_match.expr))
        ));
        self.graph.add_edge(parent, match_node, "next".to_string());

        let merge_node = self.graph.add_node(NodeType::BasicBlock("after_match".to_string()));

        for arm in &expr_match.arms {
            let arm_node = self.graph.add_node(NodeType::BasicBlock(
                format!("case: {}", quote::quote!(#arm.pat))
            ));
            self.graph.add_edge(match_node, arm_node, "case".to_string());

            let body_node = match &*arm.body {
                Expr::Block(block) => self.visit_block(&block.block, Some(arm_node)),
                expr => {
                    let node = self.graph.add_node(NodeType::BasicBlock(
                        format!("{}", quote::quote!(#expr))
                    ));
                    self.graph.add_edge(arm_node, node, "next".to_string());
                    node
                }
            };
            self.graph.add_edge(body_node, merge_node, "next".to_string());
        }

        merge_node
    }
}

impl<'ast> Visit<'ast> for ControlFlowVisitor<'_> {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        let fn_name = node.sig.ident.to_string();
        
        // 创建函数开始和结束节点
        let start_node = self.graph.add_node(NodeType::Start(fn_name.clone()));
        let end_node = self.graph.add_node(NodeType::End(fn_name));
        
        self.fn_start_node = Some(start_node);
        self.fn_end_node = Some(end_node);
        self.current_node = Some(start_node);

        // 访问函数体
        let last_node = self.visit_block(&node.block, None);
        self.graph.add_edge(last_node, end_node, "return".to_string());
    }
}

pub trait GraphRenderer {
    fn render(&self, graph: &FlowGraph) -> Result<String>;
}

pub struct DotRenderer {
    node_shape: String,
    graph_type: String,
}

impl Default for DotRenderer {
    fn default() -> Self {
        Self {
            node_shape: "record".to_string(),
            graph_type: "digraph".to_string(),
        }
    }
}

impl DotRenderer {
    pub fn new(node_shape: &str, graph_type: &str) -> Self {
        Self {
            node_shape: node_shape.to_string(),
            graph_type: graph_type.to_string(),
        }
    }
}

impl GraphRenderer for DotRenderer {
    fn render(&self, graph: &FlowGraph) -> Result<String> {
        let dot = Dot::new(&graph.graph);
        Ok(format!("{:?}", dot))
    }
}

pub struct CStyleFlowchartRenderer {
    template: String,
}

impl Default for CStyleFlowchartRenderer {
    fn default() -> Self {
        Self {
            template: r#"
                digraph G {
                    graph [rankdir=TB];
                    node [fontname="Arial", fontsize=10, margin="0.2,0.2"];
                    edge [fontname="Arial", fontsize=10];
                    
                    // 节点样式
                    node [shape=box, style=rounded] [label="BasicBlock"];
                    node [shape=diamond, style=""] [label="Condition"];
                    node [shape=ellipse, style=""] [label="Start/End"];
                    node [shape=hexagon, style=""] [label="Loop"];
                    
                    // 边的样式
                    edge [dir=forward, arrowsize=0.8];
                    true_edge [color="green"];
                    false_edge [color="red"];
                    next_edge [color="black"];
                    loop_edge [color="blue", style=dashed];
                    
                    __NODES__
                    
                    __EDGES__
                }
            "#.to_string()
        }
    }
}

// 节点样式管理
struct NodeStyle;

impl NodeStyle {
    fn get_shape(node: &NodeType) -> &str {
        match node {
            NodeType::Start(_) | NodeType::End(_) => "circle",
            NodeType::BasicBlock(_) => "box",
            NodeType::Condition(_) | NodeType::Loop(_) => "diamond",
        }
    }

    fn get_style(node: &NodeType) -> &str {
        match node {
            NodeType::BasicBlock(_) => "rounded",
            _ => "filled",
        }
    }

    fn get_fillcolor(node: &NodeType) -> &str {
        match node {
            NodeType::Start(_) => "lightgreen",
            NodeType::End(_) => "lightpink",
            NodeType::Condition(_) | NodeType::Loop(_) => "lightyellow",
            NodeType::BasicBlock(_) => "white",
        }
    }

    fn get_label(node: &NodeType) -> String {
        match node {
            NodeType::Start(name) => format!("开始\n{}", name),
            NodeType::End(name) => format!("结束\n{}", name),
            NodeType::BasicBlock(content) => {
                // 简化代码内容
                let content = content.replace("let ", "")
                    .replace(";", "")
                    .replace("println!", "输出")
                    .replace("\"", "'");
                if content.len() > 30 {
                    format!("{}..", &content[..27])
                } else {
                    content
                }
            },
            NodeType::Condition(cond) => {
                // 简化条件表达式
                cond.replace("if ", "")
                    .replace("match ", "匹配 ")
                    .replace(">", "大于")
                    .replace("<", "小于")
                    .replace("==", "等于")
                    .replace("!=", "不等于")
            },
            NodeType::Loop(kind) => {
                match kind.as_str() {
                    "while" => "当条件满足时".to_string(),
                    "loop" => "循环".to_string(),
                    _ => kind.clone(),
                }
            },
        }
    }
}

// 边样式管理
struct EdgeStyle;

impl EdgeStyle {
    fn get_color_and_style(weight: &str) -> (&str, &str) {
        match weight {
            "是" => ("black", "solid"),
            "否" => ("black", "solid"),
            "继续循环" => ("blue", "dashed"),
            "跳出循环" => ("red", "solid"),
            "检查条件" | "进入循环" | "完成分支" | "进入判断" => ("black", "solid"),
            _ => ("black", "solid"),
        }
    }
}

// 模板管理
struct DotTemplate;

impl DotTemplate {
    fn get_template() -> String {
        r#"
            digraph G {
                // 使用自上而下的布局
                graph [
                    rankdir=TB,
                    nodesep=0.5,
                    ranksep=0.7,
                    splines=ortho,  // 使用正交线条
                    concentrate=true // 合并平行边
                ];
                
                // 统一的节点样式
                node [
                    fontname="Arial",
                    fontsize=10,
                    margin="0.2,0.2",
                    height=0.4,
                    width=0.4
                ];
                
                // 统一的边样式
                edge [
                    fontname="Arial",
                    fontsize=10,
                    dir=forward,
                    arrowsize=0.8,
                    penwidth=1
                ];
                
                __NODES__
                
                __EDGES__
                
                // 强制开始节点在最上方
                { rank=source; node [shape=circle]; }
                // 强制结束节点在最下方
                { rank=sink; node [shape=circle]; }
            }
        "#.to_string()
    }
}

// 节点渲染器
struct NodeRenderer;

impl NodeRenderer {
    fn render_node(idx: NodeIndex, node: &NodeType) -> String {
        let shape = NodeStyle::get_shape(node);
        let style = NodeStyle::get_style(node);
        let fillcolor = NodeStyle::get_fillcolor(node);
        
        format!(
            r#"    node_{} [label="{}", shape={}, style="{}", fillcolor="{}"];"#,
            idx.index(), node.label().replace("\"", "\\\""), shape, style, fillcolor
        )
    }
}

// 边渲染器
struct EdgeRenderer;

impl EdgeRenderer {
    fn render_edge(source: NodeIndex, target: NodeIndex, weight: &str) -> String {
        let (color, style) = EdgeStyle::get_color_and_style(weight);
        let label = match weight {
            "是" => "是",
            "否" => "否",
            "继续循环" => "继续",
            "跳出循环" => "跳出",
            "" => "",
            _ => weight,
        };
        
        format!(
            r#"    node_{} -> node_{} [label="{}", color="{}", style="{}"];"#,
            source.index(), target.index(), label, color, style
        )
    }
}

impl GraphRenderer for CStyleFlowchartRenderer {
    fn render(&self, graph: &FlowGraph) -> Result<String> {
        let nodes: Vec<String> = graph.graph.node_indices()
            .zip(graph.graph.node_weights())
            .map(|(idx, node)| NodeRenderer::render_node(idx, node))
            .collect();

        let edges: Vec<String> = graph.graph.edge_indices()
            .map(|edge| {
                let (source, target) = graph.graph.edge_endpoints(edge).unwrap();
                let weight = graph.graph.edge_weight(edge).unwrap();
                EdgeRenderer::render_edge(source, target, weight)
            })
            .collect();
        
        let dot = DotTemplate::get_template()
            .replace("__NODES__", &nodes.join("\n"))
            .replace("__EDGES__", &edges.join("\n"));
            
        Ok(dot)
    }
}

pub fn analyze_file(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path)?;
    let syntax = syn::parse_file(&content)
        .with_context(|| format!("Failed to parse {}", path.display()))?;

    let mut graph = FlowGraph::new();
    let mut visitor = ControlFlowVisitor::new(&mut graph);
    visitor.visit_file(&syntax);

    Ok(graph.to_dot())
}

pub fn analyze_file_with_renderer<R: GraphRenderer>(path: &Path, renderer: &R) -> Result<String> {
    let content = fs::read_to_string(path)?;
    let syntax = syn::parse_file(&content)
        .with_context(|| format!("Failed to parse {}", path.display()))?;

    let mut graph = FlowGraph::new();
    let mut visitor = ControlFlowVisitor::new(&mut graph);
    visitor.visit_file(&syntax);

    graph.render(renderer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_analyze_control_flow() {
        let test_code = r#"
            fn example() {
                let x = 5;
                if x > 0 {
                    println!("positive");
                } else {
                    println!("non-positive");
                }
                
                while x > 0 {
                    println!("countdown");
                }
                
                match x {
                    1 => println!("one"),
                    2 => println!("two"),
                    _ => println!("other"),
                }
            }
        "#;

        let path = PathBuf::from("test.rs");
        std::fs::write(&path, test_code).unwrap();
        
        let result = analyze_file(&path).unwrap();
        assert!(!result.is_empty());
        assert!(result.contains("Start: example"));
        assert!(result.contains("Condition"));
        assert!(result.contains("Loop"));
        
        std::fs::remove_file(path).unwrap();
    }
} 
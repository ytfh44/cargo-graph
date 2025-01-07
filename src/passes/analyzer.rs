use crate::graph::{FlowGraph, NodeType};
use petgraph::graph::NodeIndex;
use syn::{Block, Expr, ExprIf, ExprLoop, ExprMatch, ExprWhile, ItemFn, Stmt, ExprForLoop};
use quote::quote;

pub struct ControlFlowAnalyzerPass<'a> {
    graph: &'a mut FlowGraph,
    current_node: Option<NodeIndex>,
    fn_start_node: Option<NodeIndex>,
    fn_end_node: Option<NodeIndex>,
}

impl<'a> ControlFlowAnalyzerPass<'a> {
    pub fn new(graph: &'a mut FlowGraph) -> Self {
        Self {
            graph,
            current_node: None,
            fn_start_node: None,
            fn_end_node: None,
        }
    }
    
    pub fn analyze_function(&mut self, func: &ItemFn) {
        let fn_name = func.sig.ident.to_string();
        
        // 创建函数开始和结束节点
        let start_node = self.graph.add_node(NodeType::Start(fn_name.clone()));
        let end_node = self.graph.add_node(NodeType::End(fn_name));
        
        self.fn_start_node = Some(start_node);
        self.fn_end_node = Some(end_node);
        self.current_node = Some(start_node);

        // 分析函数体
        let last_node = self.analyze_block(&func.block, None);
        self.graph.add_edge(last_node, end_node, "return".to_string());
    }

    pub fn analyze_block(&mut self, block: &Block, parent: Option<NodeIndex>) -> NodeIndex {
        let mut last_node = parent.unwrap_or_else(|| self.current_node.unwrap());
        
        for stmt in &block.stmts {
            match stmt {
                Stmt::Expr(expr, _) => {
                    match expr {
                        Expr::If(expr_if) => {
                            last_node = self.analyze_if(expr_if, last_node);
                        }
                        Expr::While(expr_while) => {
                            last_node = self.analyze_while(expr_while, last_node);
                        }
                        Expr::Loop(expr_loop) => {
                            last_node = self.analyze_loop(expr_loop, last_node);
                        }
                        Expr::ForLoop(expr_for) => {
                            last_node = self.analyze_for(expr_for, last_node);
                        }
                        Expr::Match(expr_match) => {
                            last_node = self.analyze_match(expr_match, last_node);
                        }
                        _ => {
                            // 创建基本块节点
                            let basic_block = self.graph.add_node(NodeType::BasicBlock(
                                format!("{}", quote!(#expr))
                            ));
                            self.graph.add_edge(last_node, basic_block, "next".to_string());
                            last_node = basic_block;
                        }
                    }
                }
                _ => {
                    // 其他语句类型作为基本块处理
                    let basic_block = self.graph.add_node(NodeType::BasicBlock(
                        format!("{}", quote!(#stmt))
                    ));
                    self.graph.add_edge(last_node, basic_block, "next".to_string());
                    last_node = basic_block;
                }
            }
        }
        
        last_node
    }

    fn analyze_if(&mut self, expr_if: &ExprIf, parent: NodeIndex) -> NodeIndex {
        // 创建条件节点
        let cond_text = format!("{}", quote!(#expr_if.cond));
        let cond_node = self.graph.add_node(NodeType::Condition(cond_text));
        self.graph.add_edge(parent, cond_node, "进入判断".to_string());

        // 处理 then 分支
        let then_node = self.analyze_block(&expr_if.then_branch, Some(cond_node));
        self.graph.add_edge(cond_node, then_node, "是".to_string());

        // 处理 else 分支
        let merge_node = self.graph.add_node(NodeType::BasicBlock("分支合并点".to_string()));
        if let Some((_, else_branch)) = &expr_if.else_branch {
            let else_node = match &**else_branch {
                Expr::Block(block) => self.analyze_block(&block.block, Some(cond_node)),
                Expr::If(else_if) => self.analyze_if(else_if, cond_node),
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

    fn analyze_while(&mut self, expr_while: &ExprWhile, parent: NodeIndex) -> NodeIndex {
        // 创建循环入口节点
        let loop_entry = self.graph.add_node(NodeType::BasicBlock("循环入口".to_string()));
        self.graph.add_edge(parent, loop_entry, "进入循环".to_string());

        // 创建条件节点
        let cond_text = format!("{}", quote!(#expr_while.cond));
        let cond_node = self.graph.add_node(NodeType::Condition(cond_text));
        self.graph.add_edge(loop_entry, cond_node, "检查条件".to_string());

        // 处理循环体
        let body_node = self.analyze_block(&expr_while.body, Some(cond_node));
        self.graph.add_edge(cond_node, body_node, "是".to_string());
        
        // 创建循环回边
        self.graph.add_edge(body_node, loop_entry, "继续循环".to_string());

        // 创建循环出口
        let exit_node = self.graph.add_node(NodeType::BasicBlock("循环结束".to_string()));
        self.graph.add_edge(cond_node, exit_node, "否".to_string());
        
        exit_node
    }

    fn analyze_loop(&mut self, expr_loop: &ExprLoop, parent: NodeIndex) -> NodeIndex {
        // 创建循环入口节点
        let loop_entry = self.graph.add_node(NodeType::Loop("无条件循环".to_string()));
        self.graph.add_edge(parent, loop_entry, "进入循环".to_string());

        // 处理循环体
        let body_node = self.analyze_block(&expr_loop.body, Some(loop_entry));
        
        // 创建循环回边
        self.graph.add_edge(body_node, loop_entry, "继续循环".to_string());

        // 创建循环出口（用于break语句）
        let exit_node = self.graph.add_node(NodeType::BasicBlock("循环结束".to_string()));
        self.graph.add_edge(loop_entry, exit_node, "跳出循环".to_string());
        
        exit_node
    }

    fn analyze_match(&mut self, expr_match: &ExprMatch, parent: NodeIndex) -> NodeIndex {
        let match_node = self.graph.add_node(NodeType::Condition(
            format!("match {}", quote!(#expr_match.expr))
        ));
        self.graph.add_edge(parent, match_node, "next".to_string());

        let merge_node = self.graph.add_node(NodeType::BasicBlock("after_match".to_string()));

        for arm in &expr_match.arms {
            let arm_node = self.graph.add_node(NodeType::BasicBlock(
                format!("case: {}", quote!(#arm.pat))
            ));
            self.graph.add_edge(match_node, arm_node, "case".to_string());

            let body_node = match &*arm.body {
                Expr::Block(block) => self.analyze_block(&block.block, Some(arm_node)),
                expr => {
                    let node = self.graph.add_node(NodeType::BasicBlock(
                        format!("{}", quote!(#expr))
                    ));
                    self.graph.add_edge(arm_node, node, "next".to_string());
                    node
                }
            };
            self.graph.add_edge(body_node, merge_node, "next".to_string());
        }

        merge_node
    }

    fn analyze_for(&mut self, expr_for: &ExprForLoop, parent: NodeIndex) -> NodeIndex {
        // 创建for循环节点，显示迭代器表达式
        let loop_text = format!("for {} in {}", quote!(#expr_for.pat), quote!(#expr_for.expr));
        let loop_node = self.graph.add_node(NodeType::Loop(loop_text));
        self.graph.add_edge(parent, loop_node, "进入循环".to_string());

        // 分析循环体
        let body_node = self.analyze_block(&expr_for.body, Some(loop_node));
        
        // 添加循环返回边
        self.graph.add_edge(body_node, loop_node, "继续循环".to_string());

        // 创建循环出口节点
        let exit_node = self.graph.add_node(NodeType::BasicBlock("循环结束".to_string()));
        self.graph.add_edge(loop_node, exit_node, "退出循环".to_string());

        exit_node
    }
} 
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum LoopKind {
    While(String),     // while 循环，带条件
    For(String),       // for 循环，带迭代器表达式
    Loop,              // 无条件循环
}

impl fmt::Display for LoopKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoopKind::While(cond) => write!(f, "while {}", cond),
            LoopKind::For(expr) => write!(f, "{}", expr),
            LoopKind::Loop => write!(f, "loop"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Start(String, bool),           // 函数开始，bool表示是否是测试函数
    End(String, bool),            // 函数结束，bool表示是否是测试函数
    BasicBlock(String),     // 基本代码块
    Condition(String),      // if/match条件
    Loop(LoopKind),        // 循环结构
}

impl NodeType {
    pub fn label(&self) -> String {
        match self {
            NodeType::Start(name, _) => format!("Start: {}", name),
            NodeType::End(name, _) => format!("End: {}", name),
            NodeType::BasicBlock(content) => {
                let mut result = content.replace(";", ";\n");
                if result.ends_with('\n') {
                    result.pop();
                }
                result
            },
            NodeType::Condition(cond) => format!("Condition: {}", cond),
            NodeType::Loop(kind) => format!("Loop: {}", kind),
        }
    }

    pub fn is_test(&self) -> bool {
        match self {
            NodeType::Start(_, is_test) | NodeType::End(_, is_test) => *is_test,
            _ => false,
        }
    }
} 
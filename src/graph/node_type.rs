#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Start(String),           // 函数开始
    End(String),            // 函数结束
    BasicBlock(String),     // 基本代码块
    Condition(String),      // if/match条件
    Loop(String),          // 循环结构
}

impl NodeType {
    pub fn label(&self) -> String {
        match self {
            NodeType::Start(name) => format!("Start: {}", name),
            NodeType::End(name) => format!("End: {}", name),
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
} 
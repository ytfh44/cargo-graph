use crate::passes::styler::StyledGraph;
use std::collections::HashSet;
use petgraph::graph::NodeIndex;

pub struct DotRendererPass;

impl DotRendererPass {
    pub fn render(graph: &StyledGraph) -> String {
        let mut dot = String::from("digraph G {\n");
        
        // 添加全局属性
        dot.push_str("    graph [\n");
        dot.push_str("        rankdir=TB;\n");        // 从上到下的布局
        dot.push_str("        nodesep=0.8;\n");       // 节点间距
        dot.push_str("        ranksep=1.0;\n");       // 层级间距
        dot.push_str("        splines=ortho;\n");     // 使用正交线
        dot.push_str("        concentrate=false;\n");  // 禁用边的合并
        dot.push_str("        compound=false;\n");     // 禁用复合图
        dot.push_str("        overlap=false;\n");      // 防止重叠
        dot.push_str("        layout=dot;\n");        // 使用dot布局引擎
        dot.push_str("    ];\n\n");

        // 添加全局节点属性
        dot.push_str("    node [\n");
        dot.push_str("        fontname=\"Arial\";\n");
        dot.push_str("        fontsize=10;\n");       // 字体大小
        dot.push_str("        margin=\"0.3\";\n");    // 边距
        dot.push_str("        fixedsize=false;\n");   // 允许节点大小变化
        dot.push_str("    ];\n\n");

        // 添加全局边属性
        dot.push_str("    edge [\n");
        dot.push_str("        fontname=\"Arial\";\n");
        dot.push_str("        fontsize=9;\n");        // 字体大小
        dot.push_str("        dir=forward;\n");
        dot.push_str("        arrowsize=0.7;\n");     // 箭头大小
        dot.push_str("        penwidth=1.0;\n");      // 线宽
        dot.push_str("    ];\n\n");
        
        // 收集所有有效的节点ID
        let valid_nodes: HashSet<NodeIndex> = graph.nodes.iter()
            .map(|node| node.id)
            .collect();
        
        // 添加节点
        for node in &graph.nodes {
            let escaped_label = Self::process_label(&node.label);
            
            // 根据节点类型设置形状和样式
            dot.push_str(&format!(
                "    node_{} [label=\"{}\", shape=\"{}\", style=\"{}\", fillcolor=\"{}\"];\n",
                node.id.index(),
                escaped_label,
                node.shape,
                node.style,
                node.fillcolor
            ));
        }
        
        // 添加边（只添加连接有效节点的边）
        for edge in &graph.edges {
            if valid_nodes.contains(&edge.from) && valid_nodes.contains(&edge.to) {
                let escaped_label = Self::process_label(&edge.label);
                dot.push_str(&format!(
                    "    node_{} -> node_{} [label=\"{}\", color=\"{}\", style=\"{}\"];\n",
                    edge.from.index(),
                    edge.to.index(),
                    escaped_label,
                    edge.color,
                    edge.style
                ));
            }
        }
        
        dot.push_str("}\n");
        dot
    }

    fn process_label(label: &str) -> String {
        // 处理标签中的特殊字符
        let escaped = label
            .replace('\\', "\\\\")  // 必须先转义反斜杠
            .replace('\"', "\\\"")
            .replace('{', "\\{")
            .replace('}', "\\}")
            .replace('<', "\\<")
            .replace('>', "\\>")
            .replace('|', "\\|")
            .replace('\n', "\\n");

        // 如果标签太长，添加换行
        if escaped.len() > 30 {
            let words: Vec<&str> = escaped.split_whitespace().collect();
            let mut result = String::new();
            let mut line_length = 0;
            
            for word in words {
                if line_length + word.len() > 30 {
                    result.push_str("\\n");
                    line_length = 0;
                } else if !result.is_empty() {
                    result.push(' ');
                    line_length += 1;
                }
                result.push_str(word);
                line_length += word.len();
            }
            result
        } else {
            escaped
        }
    }
} 
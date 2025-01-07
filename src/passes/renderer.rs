use crate::passes::styler::{StyledGraph, StyledNode};
use std::collections::{HashSet, HashMap, BTreeMap};
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;

pub struct DotRendererPass;

impl DotRendererPass {
    pub fn render(graph: &StyledGraph) -> String {
        let mut dot = String::from("digraph G {\n");
        
        // 添加全局属性
        dot.push_str("    graph [\n");
        dot.push_str("        rankdir=TB;\n");         // 从上到下的布局
        dot.push_str("        nodesep=0.5;\n");        // 节点水平间距
        dot.push_str("        ranksep=0.5;\n");        // 层级间距
        dot.push_str("        splines=ortho;\n");      // 使用正交线
        dot.push_str("        concentrate=false;\n");   // 禁用边的合并
        dot.push_str("        compound=false;\n");     // 禁用复合图
        dot.push_str("        overlap=false;\n");      // 防止重叠
        dot.push_str("        layout=dot;\n");         // 使用dot布局引擎
        dot.push_str("        newrank=true;\n");       // 使用新的rank分配算法
        dot.push_str("        pad=0.3;\n");           // 图的内边距
        dot.push_str("    ];\n\n");

        // 添加全局节点属性
        dot.push_str("    node [\n");
        dot.push_str("        fontname=\"Arial\";\n");
        dot.push_str("        fontsize=10;\n");
        dot.push_str("        margin=0.2;\n");         // 节点内边距
        dot.push_str("        height=0.4;\n");         // 最小高度
        dot.push_str("        width=0.4;\n");          // 最小宽度
        dot.push_str("        penwidth=1.0;\n");       // 边框宽度
        dot.push_str("        fixedsize=false;\n");    // 允许节点大小根据内容调整
        dot.push_str("    ];\n\n");

        // 添加全局边属性
        dot.push_str("    edge [\n");
        dot.push_str("        fontname=\"Arial\";\n");
        dot.push_str("        fontsize=9;\n");
        dot.push_str("        dir=forward;\n");
        dot.push_str("        arrowsize=0.7;\n");      // 箭头大小
        dot.push_str("        penwidth=1.0;\n");       // 线宽
        dot.push_str("        minlen=1;\n");           // 最小边长度
        dot.push_str("        arrowhead=normal;\n");   // 标准箭头样式
        dot.push_str("        headclip=true;\n");      // 箭头从节点边界开始
        dot.push_str("        tailclip=true;\n");      // 箭头在节点边界结束
        dot.push_str("    ];\n\n");
        
        // 收集所有有效的节点ID
        let valid_nodes: HashSet<NodeIndex> = graph.nodes.iter()
            .map(|node| node.id)
            .collect();

        // 按函数分组节点
        let mut function_nodes: BTreeMap<String, Vec<&StyledNode>> = BTreeMap::new();
        for node in &graph.nodes {
            let func_name = Self::get_function_name(&node.label);
            function_nodes.entry(func_name).or_default().push(node);
        }

        // 添加节点并设置rank约束
        for (func_name, nodes) in &function_nodes {
            // 创建子图以保持函数内的节点在一起
            dot.push_str(&format!("    subgraph cluster_{} {{\n", func_name.replace(" ", "_")));
            dot.push_str("        style=invis;\n");  // 使子图边框不可见

            // 添加函数内的所有节点
            for node in nodes {
                let escaped_label = Self::process_label(&node.label);
                dot.push_str(&format!(
                    "        node_{} [label=\"{}\", shape=\"{}\", style=\"{}\", fillcolor=\"{}\", color=\"black\"];\n",
                    node.id.index(),
                    escaped_label,
                    node.shape,
                    node.style,
                    node.fillcolor
                ));
            }

            // 对Start和End节点进行特殊处理
            let mut start_nodes = Vec::new();
            let mut end_nodes = Vec::new();
            for node in nodes {
                if node.label.starts_with("Start") {
                    start_nodes.push(node);
                } else if node.label.starts_with("End") {
                    end_nodes.push(node);
                }
            }

            // 设置Start节点的rank
            if !start_nodes.is_empty() {
                dot.push_str("        { rank=source; ");
                for node in &start_nodes {
                    dot.push_str(&format!("node_{} ", node.id.index()));
                }
                dot.push_str("}\n");
            }

            // 设置End节点的rank
            if !end_nodes.is_empty() {
                dot.push_str("        { rank=sink; ");
                for node in &end_nodes {
                    dot.push_str(&format!("node_{} ", node.id.index()));
                }
                dot.push_str("}\n");
            }

            dot.push_str("    }\n");
        }

        // 添加边，确保边不会重叠
        for edge in &graph.edges {
            if valid_nodes.contains(&edge.from) && valid_nodes.contains(&edge.to) {
                let escaped_label = Self::process_label(&edge.label);
                dot.push_str(&format!(
                    "    node_{} -> node_{} [label=\"{}\", color=\"{}\", style=\"{}\", weight=1, constraint=true];\n",
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

    fn get_function_name(label: &str) -> String {
        if label.starts_with("Start: ") {
            label["Start: ".len()..].to_string()
        } else if label.starts_with("End: ") {
            label["End: ".len()..].to_string()
        } else {
            // 对于其他节点，尝试从标签中提取函数名
            // 这里可能需要根据实际标签格式进行调整
            label.split('\n').next().unwrap_or("default").to_string()
        }
    }

    fn process_label(label: &str) -> String {
        // 处理标签中的特殊字符
        let escaped = label
            .replace('\\', "\\\\")
            .replace('\"', "\\\"")
            .replace('{', "\\{")
            .replace('}', "\\}")
            .replace('<', "\\<")
            .replace('>', "\\>")
            .replace('|', "\\|")
            .replace('\n', "\\n");

        // 如果标签太长，添加换行
        if escaped.len() > 20 {
            let words: Vec<&str> = escaped.split_whitespace().collect();
            let mut result = String::new();
            let mut line_length = 0;
            
            for word in words {
                if line_length + word.len() > 20 {
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
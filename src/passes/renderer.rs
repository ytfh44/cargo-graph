use crate::passes::styler::StyledGraph;

pub struct DotRendererPass;

impl DotRendererPass {
    pub fn render(graph: &StyledGraph) -> String {
        let mut dot = String::from("digraph G {\n");
        
        // 添加全局属性
        dot.push_str("    graph [\n");
        dot.push_str("        rankdir=TB;\n");
        dot.push_str("        nodesep=1.2;\n");
        dot.push_str("        ranksep=1.5;\n");
        dot.push_str("        splines=ortho;\n");
        dot.push_str("        concentrate=true;\n");
        dot.push_str("        compound=true;\n");
        dot.push_str("        newrank=true\n");
        dot.push_str("    ];\n\n");

        // 添加全局节点属性
        dot.push_str("    node [\n");
        dot.push_str("        fontname=\"Arial\";\n");
        dot.push_str("        fontsize=12;\n");
        dot.push_str("        margin=\"0.5,0.3\";\n");
        dot.push_str("        height=0;\n");
        dot.push_str("        width=0\n");
        dot.push_str("    ];\n\n");

        // 添加全局边属性
        dot.push_str("    edge [\n");
        dot.push_str("        fontname=\"Arial\";\n");
        dot.push_str("        fontsize=10;\n");
        dot.push_str("        dir=forward;\n");
        dot.push_str("        arrowsize=0.8;\n");
        dot.push_str("        penwidth=1;\n");
        dot.push_str("        minlen=2\n");
        dot.push_str("    ];\n\n");
        
        // 添加节点
        for node in &graph.nodes {
            // 处理标签中的特殊字符
            let escaped_label = Self::escape_label(&node.label);
            
            // 根据节点类型设置形状和样式
            dot.push_str(&format!("    node_{} [label=\"{}\", shape=\"{}\", style=\"{}\", fillcolor=\"{}\"];\n",
                node.id.index(), escaped_label, node.shape, node.style, node.fillcolor));
        }
        
        // 添加边
        for edge in &graph.edges {
            // 处理边标签中的特殊字符
            let escaped_label = Self::escape_label(&edge.label);
            
            dot.push_str(&format!("    node_{} -> node_{} [label=\"{}\", color=\"{}\", style=\"{}\"];\n",
                edge.from.index(), edge.to.index(), escaped_label, edge.color, edge.style));
        }
        
        dot.push_str("}\n");
        dot
    }

    /// 转义 DOT 标签中的特殊字符
    fn escape_label(label: &str) -> String {
        label
            .replace('\\', "\\\\")  // 必须先转义反斜杠
            .replace('\"', "\\\"")
            .replace('{', "\\{")
            .replace('}', "\\}")
            .replace('[', "\\[")
            .replace(']', "\\]")
            .replace('<', "\\<")
            .replace('>', "\\>")
            .replace('|', "\\|")
            .replace('(', "\\(")
            .replace(')', "\\)")
            .replace(';', "\\;")
            .replace(':', "\\:")
            .replace('.', "\\.")
            .replace('=', "\\=")
            .replace(',', "\\,")
    }
} 
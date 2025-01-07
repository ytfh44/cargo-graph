use crate::passes::styler::{StyledGraph, StyledNode};
use std::collections::{HashSet, HashMap, BTreeMap};
use petgraph::graph::NodeIndex;
use petgraph::algo::is_cyclic_directed;
use petgraph::Graph;

pub struct DotRendererPass;

impl DotRendererPass {
    pub fn render(graph: &StyledGraph) -> String {
        let mut dot = String::from("digraph G {\n");
        
        // 添加全局属性
        dot.push_str("    graph [\n");
        dot.push_str("        rankdir=LR;\n");        // 从左到右的布局
        dot.push_str("        nodesep=0.5;\n");       // 节点间距
        dot.push_str("        ranksep=0.7;\n");       // 层级间距
        dot.push_str("        splines=polyline;\n");  // 使用简单的直线
        dot.push_str("        concentrate=false;\n");  // 禁用边的合并
        dot.push_str("        compound=false;\n");    // 禁用复合图
        dot.push_str("        overlap=false;\n");     // 防止重叠
        dot.push_str("        layout=dot;\n");        // 使用dot布局引擎
        dot.push_str("        newrank=true;\n");      // 使用新的rank分配算法
        dot.push_str("        ordering=out;\n");      // 根据出边顺序排列节点
        dot.push_str("        packmode=\"graph\";\n"); // 使用图形打包模式
        dot.push_str("        searchsize=50;\n");     // 增加搜索空间
        dot.push_str("    ];\n\n");

        // 添加全局节点属性
        dot.push_str("    node [\n");
        dot.push_str("        fontname=\"Arial\";\n");
        dot.push_str("        fontsize=10;\n");
        dot.push_str("        margin=\"0.2\";\n");
        dot.push_str("        height=0.3;\n");
        dot.push_str("        width=0.3;\n");
        dot.push_str("    ];\n\n");

        // 添加全局边属性
        dot.push_str("    edge [\n");
        dot.push_str("        fontname=\"Arial\";\n");
        dot.push_str("        fontsize=9;\n");
        dot.push_str("        dir=forward;\n");
        dot.push_str("        arrowsize=0.7;\n");
        dot.push_str("        penwidth=0.8;\n");
        dot.push_str("        minlen=1;\n");          // 最小边长度
        dot.push_str("    ];\n\n");
        
        // 收集所有有效的节点ID
        let valid_nodes: HashSet<NodeIndex> = graph.nodes.iter()
            .map(|node| node.id)
            .collect();

        // 构建临时图用于分析
        let mut temp_graph = Graph::<(), ()>::new();
        let node_map: HashMap<NodeIndex, _> = graph.nodes.iter()
            .map(|node| (node.id, temp_graph.add_node(())))
            .collect();

        // 添加边到临时图
        for edge in &graph.edges {
            if let (Some(&from), Some(&to)) = (node_map.get(&edge.from), node_map.get(&edge.to)) {
                temp_graph.add_edge(from, to, ());
            }
        }

        // 检测是否有循环
        let has_cycles = is_cyclic_directed(&temp_graph);

        // 按函数分组节点并排序
        let mut function_nodes: BTreeMap<String, Vec<&StyledNode>> = BTreeMap::new();
        for node in &graph.nodes {
            if let Some(func_name) = Self::get_function_name(&node.label) {
                function_nodes.entry(func_name).or_default().push(node);
            }
        }

        // 首先添加所有节点
        for node in &graph.nodes {
            let escaped_label = Self::process_label(&node.label);
            dot.push_str(&format!(
                "    node_{} [label=\"{}\", shape=\"{}\", style=\"{}\", fillcolor=\"{}\", group=\"{}\"];\n",
                node.id.index(),
                escaped_label,
                node.shape,
                node.style,
                node.fillcolor,
                Self::get_node_group(&node.label) // 添加组属性以改进布局
            ));
        }

        // 使用rank来控制函数的布局
        if !function_nodes.is_empty() {
            // 为每个函数创建一个rank组
            for (func_name, nodes) in &function_nodes {
                // 对节点按ID排序以保持稳定性
                let mut sorted_nodes = nodes.to_vec();
                sorted_nodes.sort_by_key(|node| node.id);
                
                // 创建rank组
                dot.push_str(&format!("    // {} function nodes\n", func_name));
                dot.push_str("    {rank=same;");
                for node in &sorted_nodes {
                    dot.push_str(&format!(" node_{}", node.id.index()));
                }
                dot.push_str("}\n");
                
                // 使用invisible边连接同一函数内的节点，保持它们的相对位置
                for nodes in sorted_nodes.windows(2) {
                    dot.push_str(&format!(
                        "    node_{} -> node_{} [style=invis, weight=100, minlen=2];\n",
                        nodes[0].id.index(),
                        nodes[1].id.index()
                    ));
                }
            }

            // 使用invisible边连接不同函数的起始节点，控制函数的水平顺序
            let start_nodes: Vec<_> = function_nodes.values()
                .filter_map(|nodes| nodes.first())
                .collect();
            
            for nodes in start_nodes.windows(2) {
                dot.push_str(&format!(
                    "    node_{} -> node_{} [style=invis, weight=1, minlen=3];\n",
                    nodes[0].id.index(),
                    nodes[1].id.index()
                ));
            }
        }

        // 添加实际的边
        let mut edge_counts: HashMap<(NodeIndex, NodeIndex), i32> = HashMap::new();
        for edge in &graph.edges {
            if valid_nodes.contains(&edge.from) && valid_nodes.contains(&edge.to) {
                let count = edge_counts.entry((edge.from, edge.to)).or_insert(0);
                *count += 1;
                
                let escaped_label = Self::process_label(&edge.label);
                // 为边添加权重和约束，处理平行边
                dot.push_str(&format!(
                    "    node_{} -> node_{} [label=\"{}\", color=\"{}\", style=\"{}\", weight=2, constraint=true, minlen=2{}];\n",
                    edge.from.index(),
                    edge.to.index(),
                    escaped_label,
                    edge.color,
                    edge.style,
                    if has_cycles { ", samehead=true, sametail=true" } else { "" }
                ));
            }
        }
        
        dot.push_str("}\n");
        dot
    }

    fn get_function_name(label: &str) -> Option<String> {
        if label.starts_with("Start: ") {
            Some(label["Start: ".len()..].to_string())
        } else if label.starts_with("End: ") {
            Some(label["End: ".len()..].to_string())
        } else {
            None
        }
    }

    fn get_node_group(label: &str) -> String {
        // 根据节点标签确定组，用于改进布局
        if let Some(name) = Self::get_function_name(label) {
            name
        } else {
            "default".to_string()
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
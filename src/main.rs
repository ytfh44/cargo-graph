use std::path::{Path, PathBuf};
use std::collections::HashMap;
use walkdir::WalkDir;
use anyhow::{Result, bail};
use clap::Parser;
use cargo_graph::{analyze_file_with_renderer, DotRenderer, CStyleFlowchartRenderer, GraphRenderer};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input: Option<PathBuf>,
    
    #[arg(short, long)]
    output: Option<PathBuf>,
    
    #[arg(short, long, default_value = "svg")]
    format: String,
    
    #[arg(short, long, default_value = "default")]
    style: String,
    
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    Graph,
}

fn get_crate_root() -> Result<PathBuf> {
    let current_dir = std::env::current_dir()?;
    let cargo_toml = current_dir.join("Cargo.toml");
    
    if cargo_toml.exists() {
        Ok(current_dir)
    } else {
        bail!("Could not find Cargo.toml in current directory or any parent directory")
    }
}

fn find_rust_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    
    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().map_or(false, |ext| ext == "rs") &&
            !e.path().to_string_lossy().contains("target") // 排除 target 目录
        })
    {
        files.push(entry.path().to_path_buf());
    }
    
    Ok(files)
}

fn analyze_crate(crate_root: &Path, renderer: &dyn GraphRenderer) -> Result<String> {
    let rust_files = find_rust_files(crate_root)?;
    println!("Found {} Rust files", rust_files.len());
    
    let mut graphs = Vec::new();
    
    // 按模块分组处理文件
    let mut module_files: HashMap<String, Vec<PathBuf>> = HashMap::new();
    
    for file in rust_files {
        let relative_path = file.strip_prefix(crate_root)?.to_str().unwrap().to_string();
        println!("Processing file: {} as module: {}", file.display(), relative_path);
        
        let module_name = relative_path.replace(".rs", "");
        module_files.entry(module_name.clone())
            .or_default()
            .push(file);
    }
    
    // 分析每个模块
    for (module_name, files) in module_files {
        println!("Analyzing module: {} with {} files", module_name, files.len());
        
        for file in files {
            match analyze_file_with_renderer(&file, renderer) {
                Ok(graph) => {
                    println!("Successfully analyzed {}", file.display());
                    graphs.push((module_name.clone(), graph));
                }
                Err(e) => {
                    eprintln!("Warning: Failed to analyze {}: {}", file.display(), e);
                }
            }
        }
    }
    
    println!("Generated {} graphs", graphs.len());
    Ok(merge_graphs(graphs))
}

fn merge_graphs(graphs: Vec<(String, String)>) -> String {
    let mut merged = String::from("digraph G {\n");
    
    // 添加全局属性
    merged.push_str("    graph [\n");
    merged.push_str("        rankdir=TB;\n");
    merged.push_str("        nodesep=1.2;\n");
    merged.push_str("        ranksep=1.5;\n");
    merged.push_str("        splines=ortho;\n");
    merged.push_str("        concentrate=true;\n");
    merged.push_str("        compound=true;\n");
    merged.push_str("        newrank=true\n");
    merged.push_str("    ];\n\n");
    
    // 添加全局节点属性
    merged.push_str("    node [\n");
    merged.push_str("        fontname=\"Arial\";\n");
    merged.push_str("        fontsize=12;\n");
    merged.push_str("        margin=\"0.5,0.3\";\n");
    merged.push_str("        height=0;\n");
    merged.push_str("        width=0\n");
    merged.push_str("    ];\n\n");
    
    // 添加全局边属性
    merged.push_str("    edge [\n");
    merged.push_str("        fontname=\"Arial\";\n");
    merged.push_str("        fontsize=10;\n");
    merged.push_str("        dir=forward;\n");
    merged.push_str("        arrowsize=0.8;\n");
    merged.push_str("        penwidth=1;\n");
    merged.push_str("        minlen=2\n");
    merged.push_str("    ];\n\n");
    
    // 合并所有子图
    for (file_name, graph_content) in graphs {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        
        // 解析子图内容
        for line in graph_content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("digraph") || line.starts_with("}") {
                continue;
            }
            
            if line.contains("->") {
                // 处理边
                edges.push(format!("        {}", line));
            } else if line.contains("node_") && line.contains("[") && line.contains("]") {
                // 处理节点
                let mut node_line = line.to_string();
                
                // 根据节点类型设置不同的形状
                if node_line.contains("Condition:") {
                    node_line = node_line.replace("shape=\"oval\"", "shape=\"diamond\"");
                } else if node_line.contains("Loop:") {
                    node_line = node_line.replace("shape=\"oval\"", "shape=\"hexagon\"");
                } else {
                    node_line = node_line.replace("shape=\"oval\"", "shape=\"box\"");
                }
                
                nodes.push(format!("        {}", node_line));
            }
        }
        
        // 只有当有实际内容时才创建子图
        if !nodes.is_empty() || !edges.is_empty() {
            // 处理文件名，使其适合作为子图名称
            let cluster_name = file_name.replace('\\', "_").replace('/', "_").replace('.', "_");
            let display_name = file_name.replace('\\', "/");
            
            merged.push_str(&format!("    subgraph cluster_{} {{\n", cluster_name));
            merged.push_str(&format!("        label=\"{}\";\n", display_name));
            merged.push_str("        style=rounded;\n");
            merged.push_str("        color=gray;\n");
            merged.push_str("        bgcolor=aliceblue;\n");
            merged.push_str("        fontsize=12;\n");
            merged.push_str("        margin=16;\n");
            merged.push_str("        node [style=filled];\n\n");
            
            // 先添加所有节点
            if !nodes.is_empty() {
                merged.push_str(&nodes.join("\n"));
                merged.push_str("\n");
            }
            
            // 再添加所有边
            if !edges.is_empty() {
                merged.push_str(&edges.join("\n"));
                merged.push_str("\n");
            }
            
            merged.push_str("    }\n\n");
        }
    }
    
    merged.push_str("}\n");
    merged
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    match args.command {
        Some(Commands::Graph) => {
            let renderer: Box<dyn GraphRenderer> = match args.style.as_str() {
                "default" => Box::new(DotRenderer::default()),
                "c-style" => Box::new(CStyleFlowchartRenderer::default()),
                style => bail!("Unsupported style: {}", style),
            };
            
            let output_path = args.output.unwrap_or_else(|| {
                PathBuf::from(format!("crate_flow.{}", args.format))
            });
            
            // 生成 DOT 内容
            let dot_content = if let Some(input_file) = args.input {
                analyze_file_with_renderer(&input_file, &*renderer)?
            } else {
                let crate_root = get_crate_root()?;
                analyze_crate(&crate_root, &*renderer)?
            };
            
            // 创建临时 DOT 文件
            let temp_dot = output_path.with_extension("dot");
            std::fs::write(&temp_dot, dot_content)?;
            
            // 使用 dot 命令转换为 SVG
            let status = std::process::Command::new("dot")
                .args(["-Tsvg", temp_dot.to_str().unwrap(), "-o", output_path.to_str().unwrap()])
                .status()?;
                
            // 删除临时文件
            std::fs::remove_file(temp_dot)?;
            
            if !status.success() {
                bail!("Failed to convert DOT to SVG");
            }
            
            println!("Flow chart saved to: {}", output_path.display());
            Ok(())
        }
        None => {
            println!("Please use 'cargo graph' instead of 'cargo-graph'");
            Ok(())
        }
    }
}

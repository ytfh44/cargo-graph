use anyhow::{Context, Result};
use cargo_graph::{
    analyze_file_with_renderer,
    CStyleFlowchartRenderer,
    DotRenderer,
    GraphRenderer,
};
use clap::Parser;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "cargo-graph")]
#[command(bin_name = "cargo-graph")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    Graph(Args),
}

#[derive(clap::Args)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 要分析的Rust源文件路径（可选，默认分析整个crate）
    #[arg(short = 'i', long)]
    file: Option<PathBuf>,

    /// 输出文件路径（可选，默认输出到标准输出）
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// 输出格式 (dot, svg, png)
    #[arg(short = 'f', long, default_value = "svg")]
    format: String,

    /// 流程图样式 (default, c-style)
    #[arg(short, long, default_value = "c-style")]
    style: String,
}

fn find_rust_files(path: &Path) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().map_or(false, |ext| ext == "rs") &&
            !e.path().to_string_lossy().contains("target") &&
            !e.path().to_string_lossy().contains("tests")  // 排除测试文件
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}

fn find_cargo_toml() -> Result<PathBuf> {
    let mut current_dir = std::env::current_dir()?;
    loop {
        let cargo_toml = current_dir.join("Cargo.toml");
        if cargo_toml.exists() {
            return Ok(cargo_toml);
        }
        if !current_dir.pop() {
            anyhow::bail!("Could not find Cargo.toml in any parent directory");
        }
    }
}

fn get_crate_root() -> Result<PathBuf> {
    let cargo_toml = find_cargo_toml()?;
    Ok(cargo_toml.parent().unwrap().to_path_buf())
}

fn get_module_name(file: &Path, crate_root: &Path) -> String {
    file.strip_prefix(crate_root)
        .ok()
        .and_then(|p| p.to_str())
        .map(|s| s.replace('/', "::").replace(".rs", ""))
        .unwrap_or_else(|| "unknown".to_string())
}

fn merge_graphs(graphs: Vec<(String, String)>) -> String {
    let mut merged = String::from(r#"digraph G {
    // 使用自上而下的布局
    graph [
        rankdir=TB;
        nodesep=0.5;
        ranksep=0.7;
        splines=ortho;
        concentrate=true
    ];
    
    // 统一的节点样式
    node [
        fontname="Arial";
        fontsize=10;
        margin="0.2,0.2";
        height=0.4;
        width=0.4
    ];
    
    // 统一的边样式
    edge [
        fontname="Arial";
        fontsize=10;
        dir=forward;
        arrowsize=0.8;
        penwidth=1
    ];"#);

    // 添加所有子图
    for (module_name, graph) in graphs {
        let subgraph = graph
            .lines()
            .filter(|line| !line.contains("digraph") && !line.contains("graph [") && !line.contains("node [") && !line.contains("edge ["))
            .collect::<Vec<_>>()
            .join("\n");
        
        // 转义模块名中的特殊字符
        let safe_module_name = module_name.replace('-', "_").replace(':', "_");
        
        merged.push_str(&format!(r#"
    subgraph cluster_{} {{
        label="{}";
        style=rounded;
        color=gray;
        {}
    }}"#, safe_module_name, module_name, subgraph));
    }

    merged.push_str("\n}\n");
    merged
}

fn analyze_crate<R: GraphRenderer>(crate_root: &Path, renderer: &R) -> Result<String> {
    let rust_files = find_rust_files(crate_root);
    let mut graphs = Vec::new();
    
    for file in rust_files {
        let module_name = get_module_name(&file, crate_root);
        match analyze_file_with_renderer(&file, renderer) {
            Ok(graph) => graphs.push((module_name, graph)),
            Err(e) => eprintln!("Warning: Failed to analyze {}: {}", file.display(), e),
        }
    }
    
    if graphs.is_empty() {
        anyhow::bail!("No Rust files found or all analyses failed");
    }
    
    Ok(merge_graphs(graphs))
}

fn generate_graph(args: &Args) -> Result<String> {
    match args.style.as_str() {
        "default" => {
            let renderer = DotRenderer::default();
            if let Some(file) = &args.file {
                analyze_file_with_renderer(file, &renderer)
            } else {
                let crate_root = get_crate_root()?;
                analyze_crate(&crate_root, &renderer)
            }
        },
        "c-style" => {
            let renderer = CStyleFlowchartRenderer::default();
            if let Some(file) = &args.file {
                analyze_file_with_renderer(file, &renderer)
            } else {
                let crate_root = get_crate_root()?;
                analyze_crate(&crate_root, &renderer)
            }
        },
        style => anyhow::bail!("Unsupported style: {}", style),
    }
}

fn convert_to_format(dot_content: String, format: &str, output: &PathBuf) -> Result<()> {
    match format {
        "dot" => {
            std::fs::write(output, dot_content)?;
        }
        "svg" | "png" => {
            let temp_dot = output.with_extension("dot");
            std::fs::write(&temp_dot, dot_content)?;
            
            let status = Command::new("dot")
                .arg(format!("-T{}", format))
                .arg(&temp_dot)
                .arg("-o")
                .arg(output)
                .status()
                .context("Failed to execute dot command. Is Graphviz installed?")?;

            std::fs::remove_file(temp_dot)?;
            
            if !status.success() {
                anyhow::bail!("dot command failed");
            }
        }
        _ => anyhow::bail!("Unsupported output format: {}", format),
    }
    Ok(())
}

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();
    let args = if let Some(Commands::Graph(args)) = cli.command {
        args
    } else {
        Args {
            file: None,
            output: None,
            format: "svg".to_string(),
            style: "c-style".to_string(),
        }
    };
    
    // 如果指定了文件，确保文件存在
    if let Some(file) = &args.file {
        if !file.exists() {
            anyhow::bail!("Input file does not exist: {}", file.display());
        }
    }

    // 生成图的DOT描述
    let dot_content = generate_graph(&args)?;

    // 处理输出
    match &args.output {
        Some(output_path) => {
            convert_to_format(dot_content, &args.format, output_path)?;
            println!("Flow chart saved to: {}", output_path.display());
        }
        None => {
            // 如果没有指定输出文件，直接打印DOT内容
            println!("{}", dot_content);
        }
    }

    Ok(())
}

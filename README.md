# Cargo Graph

一个用于生成 Rust 代码控制流图的工具。它能够分析 Rust 源代码，并生成可视化的流程图，帮助开发者更好地理解代码的执行流程。

## 功能特性

- 支持基本的控制流结构：
  - 顺序语句
  - if/else 条件分支
  - match 模式匹配
  - while/for/loop 循环结构
- 自动合并连续的基本代码块
- 支持测试函数的识别和可选显示
- 按函数分组显示控制流
- 清晰的节点布局和箭头指向
- 支持多种节点类型：
  - 开始/结束节点（椭圆形）
  - 基本代码块（矩形）
  - 条件判断（菱形）
  - 循环结构（六边形）

## 安装

确保你的系统已安装 Rust 和 Graphviz。然后通过 Cargo 安装：

```bash
cargo install cargo-graph
```

## 使用方法

### 基本用法

生成当前项目的控制流图：

```bash
cargo graph
```

生成指定文件的控制流图：

```bash
cargo graph path/to/your/file.rs
```

### 配置选项

- `--include-tests`: 包含测试函数在生成的图中
- `--output`: 指定输出文件路径

### 示例

```rust
fn fibonacci(n: u32) -> u32 {
    if n <= 1 {
        return n;
    }
    let mut a = 0;
    let mut b = 1;
    for _ in 2..=n {
        let temp = a + b;
        a = b;
        b = temp;
    }
    b
}
```

生成的流程图将显示：
- 函数的开始和结束节点
- if 条件判断
- for 循环结构
- 基本代码块的内容

## 实现细节

该工具使用以下技术和库：
- `syn`: 用于解析 Rust 源代码
- `petgraph`: 用于构建和操作图结构
- `graphviz`: 用于生成可视化图形
- `quote`: 用于代码片段的格式化

图形渲染采用分层设计：
1. 解析层：将源代码解析为 AST
2. 分析层：构建控制流图
3. 样式层：添加节点和边的视觉属性
4. 渲染层：生成最终的图形

## 贡献

欢迎提交 Issue 和 Pull Request！在提交 PR 之前，请确保：
1. 代码通过所有测试
2. 添加了必要的测试用例
3. 更新了相关文档

## 许可证

AGPL/SSPL License 用户自行选择其中一种，并享有相应的权利和义务。

use syn::{File, ItemFn, visit::{self, Visit}};

pub struct FunctionCollectorPass {
    functions: Vec<ItemFn>,
}

impl Default for FunctionCollectorPass {
    fn default() -> Self {
        Self::new()
    }
}

impl FunctionCollectorPass {
    pub fn new() -> Self {
        Self { functions: Vec::new() }
    }
    
    pub fn collect(file: &File) -> Vec<ItemFn> {
        let mut collector = Self::new();
        collector.visit_file(file);
        collector.functions
    }
}

impl<'ast> Visit<'ast> for FunctionCollectorPass {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        self.functions.push(node.clone());
        visit::visit_item_fn(self, node);
    }
} 
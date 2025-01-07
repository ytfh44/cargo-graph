use anyhow::{Context, Result};
use syn::File;

pub struct ParserPass;

impl ParserPass {
    pub fn parse(source: &str) -> Result<File> {
        syn::parse_str(source).context("Failed to parse source code")
    }
} 
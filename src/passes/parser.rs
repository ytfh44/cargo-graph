use anyhow::{Context, Result};
use syn::{File, Item, ItemFn, Attribute};

pub struct ParserPass;

impl ParserPass {
    pub fn parse(source: &str) -> Result<File> {
        syn::parse_str(source).context("Failed to parse source code")
    }

    pub fn is_test_fn(attrs: &[Attribute]) -> bool {
        attrs.iter().any(|attr| {
            attr.path().is_ident("test") ||
            attr.path().is_ident("tokio::test") ||
            attr.path().is_ident("async_std::test") ||
            attr.path().is_ident("test_case")
        })
    }

    pub fn get_function_info(item: &ItemFn) -> (String, bool) {
        let name = item.sig.ident.to_string();
        let is_test = Self::is_test_fn(&item.attrs);
        (name, is_test)
    }
} 
use std::fs::read_to_string;
use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Default, Debug, Clone, Deserialize)]
pub struct Rag {
    pub rag_embedding_model: String,
    pub rag_top_k: usize,
    pub rag_chunk_size: usize,
    pub rag_chunk_overlap: usize,
    pub documents: Vec<String>,
}

impl Rag {
    pub fn init(rag_name: &str, rag_path: &str) -> Result<Self>  {
        let err = || format!("Failed to load config at '{}'", rag_name);
        let content = read_to_string(rag_path).with_context(err)?;
        let config = serde_yaml::from_str(&content)?;
        Ok(config)
    }
}

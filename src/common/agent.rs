use std::{collections::HashMap, fs::read_to_string};

use anyhow::{bail, Context, Result};
use serde::Deserialize;

use super::{rag::Rag, RAG_TEMPLATE};

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Agent{
    pub name: String,
    pub description: String,
    pub instructions: String,
    pub tools: Vec<String>,
    pub rags: Option<String>,
}

impl Default for Agent  {
    fn default() -> Self {
        Self {
            name: Default::default(),
            description: Default::default(),
            instructions: Default::default(),
            tools: Default::default(),
            rags: Default::default(),
        }
    }
}

impl Agent {
    pub fn init(agent_name: &str, agent_path: &str) -> Result<Self>  {
        let err = || format!("Failed to load config at '{}'", agent_name);
        let content = read_to_string(agent_path).with_context(err)?;
        let config = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn rag(&self, rags: &HashMap<String, String>) -> Result<Option<Rag>> {
        if let Some(rag_name) = &self.rags {
            match rags.get(rag_name) {
                Some(rag_path) => {
                    let rag = Rag::init(rag_name, rag_path)?;
                    return Ok(Some(rag));
                },
                None => {
                    bail!("There is no rag found.");
                }
            }
        }
        Ok(None)
    }

    pub fn rag_template(&self, embeddings: &str, text: &str) -> String {
        if embeddings.is_empty() {
            return text.to_string();
        }
        RAG_TEMPLATE
            // .as_ref()
            // .unwrap_or(RAG_TEMPLATE)
            .replace("__CONTEXT__", embeddings)
            .replace("__INPUT__", text)
    }
}
use std::{collections::HashMap, fs::read_to_string};

use anyhow::{bail, Context, Ok, Result};
use async_openai::types::ChatCompletionTool;
use serde::Deserialize;

use super::{rag::Rag, tool::Tool, RAG_TEMPLATE};

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Agent{
    pub name: String,
    pub description: String,
    pub instructions: String,
    pub tools: Vec<String>,
    pub rags: Option<String>,
    #[serde(skip)]
    pub rag: Rag,
    #[serde(skip)]
    pub tool: Vec<Tool>,
}

impl Default for Agent  {
    fn default() -> Self {
        Self {
            name: Default::default(),
            description: Default::default(),
            instructions: Default::default(),
            tools: Default::default(),
            rags: Default::default(),
            rag: Default::default(),
            tool: Default::default(),
        }
    }
}

impl Agent {
    pub fn init(agent_name: &str, agent_path: &str) -> Result<Self>  {
        let err = || format!("Failed to load config at '{}'", agent_name);
        let content = read_to_string(agent_path).with_context(err)?;
        let agent = serde_yaml::from_str(&content)?;
        Ok(agent)
    }

    pub fn tool(&mut self, tools: &HashMap<String, String>) -> Result<()> {
        let mut vec_tool: Vec<Tool> = vec![];
        for tool_name in self.tools.iter() {
            match tools.get(tool_name) {
                Some(tool_path) => {
                    let mut tool = Tool::init(tool_name, tool_path)?;
                    tool.parse_json()?;
                    vec_tool.push(tool);
                },
                None => {
                    bail!("There is no tool found.");
                }
            }
        }
        self.tool = vec_tool;
        Ok(())
    }

    pub fn rag(&mut self, rags: &HashMap<String, String>) -> Result<()> {
        if let Some(rag_name) = &self.rags {
            match rags.get(rag_name) {
                Some(rag_path) => {
                    let rag = Rag::init(rag_name, rag_path)?;
                    self.rag = rag;
                },
                None => {
                    bail!("There is no rag found.");
                }
            }
        }
        Ok(())
    }

    pub fn echo_tool(&self) -> Result<Vec<ChatCompletionTool>> {
        let mut tools:Vec<ChatCompletionTool> = vec![];
        for tool in self.tool.iter() {
            tools.extend(tool.tool.clone());
        }
        Ok(tools)
    }

    pub fn tool_exec(&self, name: &str) -> Option<&str> {
        for tool in self.tool.iter() {
            if let Some(tool_exec) = tool.tool_exec(name) {
                return Some(tool_exec);
            }
        }
        None
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
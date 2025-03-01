use std::fs::read_to_string;
use std::collections::HashMap;

use anyhow::Context;
use anyhow::Result;

use async_openai::types::ChatCompletionTool;
use serde::Deserialize;

#[derive(Default, Deserialize, Clone)]
pub struct ToolInstance {
    pub tool_exec: HashMap<String, String>,
    pub tool_chat: HashMap<String, ChatCompletionTool>,
    pub tool_embedding_model: HashMap<String, String>,
}

impl ToolInstance {
    pub fn new() -> Self{
        Self{
            tool_exec: Default::default(),
            tool_chat: Default::default(),
            tool_embedding_model: Default::default(),
        }
    }
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct Tool {
    pub tool_json: HashMap<String, String>,
    pub tool_exec: HashMap<String, String>,
    #[serde(skip)]
    pub tool: Vec<ChatCompletionTool>,
}

impl Tool {
    pub fn init(tool_name: &str, tool_path: &str) -> Result<Self> {
        let err = || format!("Failed to load config at '{}'", tool_name);
        let content = read_to_string(tool_path).with_context(err)?;
        let tool = serde_yaml::from_str(&content)?;
        Ok(tool)
    }

    pub fn parse_json(&mut self) -> Result<HashMap<String, ChatCompletionTool>> {
        let mut result: HashMap<String, ChatCompletionTool> = HashMap::new();
        for (tool_name,tool_json) in self.tool_json.iter() {
            let err = || format!("Failed to load json at '{}'", tool_json);
            let content = read_to_string(tool_json).with_context(err)?;
            let config: ChatCompletionTool = serde_json::from_str(&content)?;
            result.insert(tool_name.to_string(), config);
        }
        self.tool = result.iter().map(|(_, tool)| tool.clone()).collect();
        Ok(result)
    }

    pub fn tool_exec(&self, tool_name: &str) -> Option<&str> {
        self.tool_exec.get(tool_name).map(|x| x.as_str())
    }
}

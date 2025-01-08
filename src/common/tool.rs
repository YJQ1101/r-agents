use std::fs::read_to_string;
use std::{collections::HashMap, process::Command};

use anyhow::Context;
use anyhow::Result;

use async_openai::types::ChatCompletionTool;
use serde::Deserialize;
use serde_json::Value;

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

#[derive(Default, Deserialize)]
pub struct Tool {
    pub tool_embedding_model: String,
    pub tool_top_k: usize,
    pub tool_json: HashMap<String, String>,
    pub tool_exec: HashMap<String, String>,
}

impl Tool {
    pub fn init(tool_name: &str, tool_path: &str) -> Result<Self> {
        let err = || format!("Failed to load config at '{}'", tool_name);
        let content = read_to_string(tool_path).with_context(err)?;
        let config = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn parse_json(&self) -> Result<HashMap<String, ChatCompletionTool>> {
        let mut result: HashMap<String, ChatCompletionTool> = HashMap::new();
        for (tool_name,tool_json) in self.tool_json.iter() {
            let err = || format!("Failed to load json at '{}'", tool_json);
            let content = read_to_string(tool_json).with_context(err)?;
            let config: ChatCompletionTool = serde_json::from_str(&content)?;
            result.insert(tool_name.to_string(), config);
        }
        Ok(result)
    }
}

pub async fn call_fn(
    cmd: &str,
    args: &str,
    // envs: Option<HashMap<String, String>>,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = Command::new(cmd)
        .arg(args)
        .output()?;
    // let status = output.status;
    let stdout = std::str::from_utf8(&output.stdout).context("Invalid UTF-8 in stdout")?;

    let function_response = serde_json::from_str(stdout).context(r#"The crawler response is invalid. It should follow the JSON format: `[{"path":"...", "text":"..."}]`."#)?;

    Ok(function_response)
}

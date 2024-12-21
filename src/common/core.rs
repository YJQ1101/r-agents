use std::{collections::HashMap, fs::read_to_string, io::{stdout, Write}, process::Command};

use actix_web::{middleware, web::Data, App, HttpServer};
use anyhow::{Context, Result};
use async_openai::{config::OpenAIConfig, types::{ChatCompletionMessageToolCall, ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage, ChatCompletionRequestToolMessageArgs, ChatCompletionTool, ChatCompletionToolChoiceOption, ChatCompletionToolType, CreateChatCompletionRequestArgs}, Client};
use futures::StreamExt;
use serde::Deserialize;
use serde_json::Value;
use super::{config::ToolsConfig, server::app_config, AppSysConfig};

#[derive(Debug, Clone, Default)]
pub struct Tools {
    pub tools: Vec<ChatCompletionTool>,
}

impl Tools {
    pub fn init(tools_path: Vec<String>) -> Result<Self> {
        let tools: Vec<ChatCompletionTool> = if tools_path.is_empty() {
            vec![]
        } else {   
            tools_path.iter().map(|path| {
                read_to_string(path)
                .with_context(|| format!("Failed to read from file: {}", path))
                .and_then(|content| {
                    serde_json::from_str(&content)
                    .with_context(|| format!("Failed to parse json from file: {}", path))})
                .and_then(|function| Ok(ChatCompletionTool {
                    r#type: ChatCompletionToolType::Function,
                    function,}))
                .expect(&format!("Failed to transform ChatCompletionTool"))}).collect()
        };
        Ok(Self { tools })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Exec {
    pub exec: HashMap<String, String>,
}

impl Exec {
    pub fn init(exec_path: Vec<(String, String)>) -> Result<Self> {
        let exec = exec_path.into_iter().collect();
        Ok(Self { exec })
    }
    pub fn find_exec(&self, name: &str) -> Option<String> {
        self.exec.get(name).cloned()  // Cloning to return an owned String
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Agent{
    pub name: String,
    pub description: String,
    pub version: String,
    pub instructions: String,
    pub tools: Vec<String>,
}

impl Default for Agent  {
    fn default() -> Self {
        Self {
            name: Default::default(),
            description: Default::default(),
            version: Default::default(),
            instructions: Default::default(),
            tools: Default::default(),
        }
    }
}

impl Agent {
    pub fn load_yaml_file(agent_name: &str) -> Result<Self>  {
        let err = || format!("Failed to load config at '{}'", agent_name);
        let content = read_to_string(agent_name).with_context(err)?;
        let config = serde_yaml::from_str(&content)
            .map_err(|err| {
                let err_msg = err.to_string();
                // anyhow!("{err_msg}")
            })
            .unwrap();
    
        Ok(config)
    }

    pub async fn run(&self, msg: Vec<ChatCompletionRequestMessage>, client: &Client<OpenAIConfig>, tools: &Tools, exec: &Exec) -> Result<String, Box<dyn std::error::Error>>  {
        let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(512u32).model("llama3.2").messages(msg.clone()).tools(tools.tools.clone()).tool_choice(ChatCompletionToolChoiceOption::Auto).build()?;

        let response_message = client
        .chat().create(request).await?.choices.first().unwrap().message.clone();

        let mut hh = String::new();
        if let Some(tool_calls) = response_message.tool_calls {
            let mut handles = Vec::new();
            for tool_call in tool_calls {
                if let Some(cmd) = exec.find_exec(&tool_call.function.name) {
                    let args = tool_call.function.arguments.clone();
                    let tool_call_clone = tool_call.clone();
                    let handle =
                    tokio::spawn(async move { call_fn(&cmd, &args).await.unwrap_or_default() });
                    handles.push((handle, tool_call_clone));
                }
            }
    
            let mut function_responses = Vec::new();
    
            for (handle, tool_call_clone) in handles {
                if let Ok(response_content) = handle.await {
                    function_responses.push((tool_call_clone, response_content));
                }
            }

            let mut messages: Vec<ChatCompletionRequestMessage> = msg;

            let tool_calls: Vec<ChatCompletionMessageToolCall> = function_responses
                .iter()
                .map(|(tool_call, _response_content)| tool_call.clone())
                .collect();
    
            let assistant_messages: ChatCompletionRequestMessage =
                ChatCompletionRequestAssistantMessageArgs::default()
                    .tool_calls(tool_calls)
                    .build()?
                    .into();
    
            let tool_messages: Vec<ChatCompletionRequestMessage> = function_responses
                .iter()
                .map(|(tool_call, response_content)| {
                    ChatCompletionRequestToolMessageArgs::default()
                        .content(response_content.to_string())
                        .tool_call_id(tool_call.id.clone())
                        .build()
                        .unwrap()
                        .into()
                })
                .collect();
    
            messages.push(assistant_messages);
            messages.extend(tool_messages);
    
            let subsequent_request = CreateChatCompletionRequestArgs::default()
                .max_tokens(512u32)
                .model("llama3.2")
                .messages(messages)
                .build()
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
            println!("{:?}",subsequent_request);
            let mut stream = client.chat().create_stream(subsequent_request).await?;
    
            let mut lock = stdout().lock();
            while let Some(result) = stream.next().await {
                match result {
                    Ok(response) => {
                        for chat_choice in response.choices.iter() {
                            if let Some(ref content) = chat_choice.delta.content {
                                write!(lock, "{}", content).unwrap();
                                hh.push_str(content);
                            }
                        }
                    }
                    Err(err) => {
                        return Err(Box::new(err) as Box<dyn std::error::Error>);
                    }
                }
            }
        } else {
            hh.push_str(&response_message.content.unwrap());
        }
    
        Ok(hh)
    }
}         

#[actix_web::main]
pub async fn use_agent(agent_name: &str, tools_config: ToolsConfig, sys_config: AppSysConfig) -> Result<()>{
    let agent = Agent::load_yaml_file(agent_name)?;
    let http_addr = sys_config.get_http_addr();
    let api_base = sys_config.get_api_base();
    let api_key = sys_config.get_api_key();

    let client = Client::with_config(
        OpenAIConfig::new()
        .with_api_key(api_key)
        .with_api_base(api_base)
    );
    let tools = Tools::init(agent.tools.iter()
        .filter_map(|tool| tools_config.tools_yaml.get(tool).map(|value| {
        value.to_string()})).collect())?;

    let exec = Exec::init(agent.tools.iter()
        .filter_map(|tool| tools_config.tools_exec.get(tool).map(|value| {
        (tool.to_string(), value.to_string())})).collect::<Vec<_>>())?;

    let server = HttpServer::new(move || {
        let agent = agent.clone();
        let client = client.clone();
        let tools = tools.clone();
        let exec = exec.clone();
        App::new()
            .app_data(Data::new(agent))
            .app_data(Data::new(client))
            .app_data(Data::new(tools))
            .app_data(Data::new(exec))
            .wrap(middleware::Logger::default())
            .configure(app_config)
    });

    server.bind(http_addr)?
    .run()
    .await?;
    Ok(())
}

async fn call_fn(
    cmd: &str,
    args: &str,
    // envs: Option<HashMap<String, String>>,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = Command::new(cmd)
        .arg(args)
        .output()?;
    let status = output.status;
    let stdout = std::str::from_utf8(&output.stdout).context("Invalid UTF-8 in stdout")?;
    let stderr = std::str::from_utf8(&output.stderr).context("Invalid UTF-8 in stderr")?;

    let function_response = serde_json::from_str(stdout).context(r#"The crawler response is invalid. It should follow the JSON format: `[{"path":"...", "text":"..."}]`."#)?;

    Ok(function_response)
}
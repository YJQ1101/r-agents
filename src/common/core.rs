use std::{collections::HashMap, fs::read_to_string, io::{stdout, Write}};

use actix_web::{middleware, web::Data, App, HttpServer};
use anyhow::{Context, Result};
use async_openai::{config::OpenAIConfig, types::{ChatCompletionMessageToolCall, ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage, ChatCompletionRequestToolMessageArgs, ChatCompletionTool, ChatCompletionToolType, CreateChatCompletionRequestArgs}, Client};
use futures::StreamExt;
use rand::{seq::SliceRandom, thread_rng, Rng};
use serde::Deserialize;
use serde_json::{json, Value};
use super::{config::ToolsConfig, server::app_config, AppSysConfig};

#[derive(Debug, Clone, Default)]
pub struct Functions {
    pub declarations: Vec<ChatCompletionTool>,
}

impl Functions {
    pub fn init(declarations_path: Vec<String>) -> Result<Self> {
        let declarations: Vec<ChatCompletionTool> = if declarations_path.is_empty() {
            vec![]
        } else {   
            declarations_path.iter().map(|path| {
                read_to_string(path)
                .with_context(|| format!("Failed to read from file: {}", path))
                .and_then(|content| {
                    serde_yaml::from_str(&content)
                    .with_context(|| format!("Failed to parse YAML from file: {}", path))})
                .and_then(|function| Ok(ChatCompletionTool {
                    r#type: ChatCompletionToolType::Function,
                    function,}))
                .expect(&format!("Failed to transform ChatCompletionTool"))}).collect()
        };
        Ok(Self { declarations })
    }

    // pub fn find(&self, name: &str) -> Option<&FunctionDeclaration> {
    //     self.declarations.iter().find(|v| v.name == name)
    // }

    // pub fn contains(&self, name: &str) -> bool {
    //     self.declarations.iter().any(|v| v.name == name)
    // }

    // pub fn declarations(&self) -> &[FunctionDeclaration] {
    //     &self.declarations
    // }

    // pub fn is_empty(&self) -> bool {
    //     self.declarations.is_empty()
    // }
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct FunctionDeclaration {
//     pub name: String,
//     pub description: String,
//     pub parameters: JsonSchema,
//     #[serde(skip_serializing, default)]
//     pub agent: bool,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct JsonSchema {
//     #[serde(rename = "type")]
//     pub type_value: String,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub description: Option<String>,
//     // #[serde(skip_serializing_if = "Option::is_none")]
//     // pub properties: Option<IndexMap<String, JsonSchema>>,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub items: Option<Box<JsonSchema>>,
//     #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
//     pub enum_value: Option<Vec<String>>,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub required: Option<Vec<String>>,
// }

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

    pub async fn run(&self, mut msg: Vec<ChatCompletionRequestMessage>, client: &Client<OpenAIConfig>, tools: &Functions) -> Result<String, Box<dyn std::error::Error>>  {
        let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(512u32).model("llama3.2:1b").messages(msg.clone()).tools(tools.declarations.clone()).build()?;

        let response_message = client
        .chat().create(request).await?.choices.first().unwrap().message.clone();

        let mut response_content = String::new();
        if let Some(tool_calls) = response_message.tool_calls {
            let mut handles = Vec::new();
            for tool_call in tool_calls {
                let name = tool_call.function.name.clone();
                let args = tool_call.function.arguments.clone();
                let tool_call_clone = tool_call.clone();
    
                let handle =
                    tokio::spawn(async move { call_fn(&name, &args).await.unwrap_or_default() });
                handles.push((handle, tool_call_clone));
            }
    
            let mut function_responses = Vec::new();
    
            for (handle, tool_call_clone) in handles {
                if let Ok(response_content) = handle.await {
                    function_responses.push((tool_call_clone, response_content));
                }
            }
    
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
    
            msg.push(assistant_messages);
            msg.extend(tool_messages);
    
            let subsequent_request = CreateChatCompletionRequestArgs::default()
                .max_tokens(512u32)
                .model("llama3.2:1b")
                .messages(msg)
                .build()
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    
            let mut stream = client.chat().create_stream(subsequent_request).await?;
    
            let mut lock = stdout().lock();
            while let Some(result) = stream.next().await {
                match result {
                    Ok(response) => {
                        for chat_choice in response.choices.iter() {
                            if let Some(ref content) = chat_choice.delta.content {
                                write!(lock, "{}", content).unwrap();
                                response_content.push_str(content);
                            }
                        }
                    }
                    Err(err) => {
                        return Err(Box::new(err) as Box<dyn std::error::Error>);
                    }
                }
            }
        } else {
            response_content.push_str(&response_message.content.unwrap());
        }
    
        Ok(response_content)
    }
}         

#[actix_web::main]
pub async fn use_agent(agent_name: &str, tools_config: ToolsConfig, sys_config: AppSysConfig) -> Result<()>{
    let agent = Agent::load_yaml_file(agent_name)?;
    let http_addr = sys_config.get_http_addr();
    let api_base = sys_config.api_base;
    let api_key = sys_config.api_key;

    let client = Client::with_config(
        OpenAIConfig::new()
        .with_api_key(api_key)
        .with_api_base(api_base)
    );
    let function = Functions::init(agent.tools.iter()
        .filter_map(|tool| tools_config.tools.get(tool).map(|value| {
        value.to_string()})).collect())?;

    let server = HttpServer::new(move || {
        let agent = agent.clone();
        let client = client.clone();
        let function = function.clone();
        //let naming_dal_addr = naming_dal_addr.clone();
        App::new()
            .app_data(Data::new(agent))
            .app_data(Data::new(client))
            .app_data(Data::new(function))
            .wrap(middleware::Logger::default())
            .configure(app_config)
    });

    server.bind(http_addr)?
    .run()
    .await?;
    Ok(())

}

async fn call_fn(name: &str, args: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let mut available_functions: HashMap<&str, fn(&str, &str) -> serde_json::Value> =
        HashMap::new();
    available_functions.insert("get_current_weather", get_current_weather);

    let function_args: serde_json::Value = args.parse().unwrap();

    let location = function_args["location"].as_str().unwrap();
    let unit = function_args["unit"].as_str().unwrap_or("fahrenheit");
    let function = available_functions.get(name).unwrap();
    let function_response = function(location, unit);
    Ok(function_response)
}

fn get_current_weather(location: &str, unit: &str) -> serde_json::Value {
    let mut rng = thread_rng();

    let temperature: i32 = rng.gen_range(20..=55);

    let forecasts = [
        "sunny", "cloudy", "overcast", "rainy", "windy", "foggy", "snowy",
    ];

    let forecast = forecasts.choose(&mut rng).unwrap_or(&"sunny");

    let weather_info = json!({
        "location": location,
        "temperature": temperature.to_string(),
        "unit": unit,
        "forecast": forecast
    });

    weather_info
}
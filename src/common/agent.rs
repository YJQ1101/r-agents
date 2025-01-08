use std::{fs::read_to_string, io::{stdout, Write}};

use anyhow::{Context, Result};
use async_openai::{config::OpenAIConfig, types::{ChatCompletionMessageToolCall, ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage, ChatCompletionRequestToolMessageArgs, ChatCompletionTool, ChatCompletionToolChoiceOption, CreateChatCompletionRequest, CreateChatCompletionRequestArgs, CreateChatCompletionResponse, CreateEmbeddingRequest, CreateEmbeddingResponse}, Client};
use futures::StreamExt;
use serde::Deserialize;

use super::{db::Database, tool::{call_fn, ToolInstance}, RAG_TEMPLATE};

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Agent{
    pub name: String,
    pub description: String,
    pub instructions: String,
    pub tools: Vec<String>,
    pub rags: Option<Vec<String>>,
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
    pub fn init(agent_name: &str) -> Result<Self>  {
        let err = || format!("Failed to load config at '{}'", agent_name);
        let content = read_to_string(agent_name).with_context(err)?;
        let config = serde_yaml::from_str(&content)?;
        Ok(config)
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

    pub async fn chat_completions(&self, mut request: CreateChatCompletionRequest, client: &Client<OpenAIConfig>, db: &Box<dyn Database>, tool_instance: &ToolInstance) -> Result<CreateChatCompletionResponse, Box<dyn std::error::Error>> {
        let model = request.model.clone();
        let max_tokens = request.max_tokens.unwrap_or(512);
        let mut messages = request.messages.clone();

        let mut query: String = String::new();
        for message in request.messages.iter() {
            match message {
                ChatCompletionRequestMessage::System(chat_completion_request_system_message) => query += &serde_json::to_string(&chat_completion_request_system_message.content)?,
                ChatCompletionRequestMessage::User(chat_completion_request_user_message) => query += &serde_json::to_string(&chat_completion_request_user_message.content)?,
                ChatCompletionRequestMessage::Assistant(chat_completion_request_assistant_message) => query += &serde_json::to_string(&chat_completion_request_assistant_message.content)?,
                ChatCompletionRequestMessage::Tool(chat_completion_request_tool_message) => query += &serde_json::to_string(&chat_completion_request_tool_message.content)?,
                _ => query = query,
            }
        }
        println!("{}", query);

        let mut tools: Vec<String> = Vec::new();
        for tool in self.tools.iter() {
            let tmp = db.query_tool(tool, &query, tool_instance.tool_embedding_model.get(tool).map(|x| x.as_str())).await?;
            tools.extend(tmp);
        }
        println!("{:?}", tools);

        let mut docu: Vec<String> = Vec::new();
        if let Some(rags) = &self.rags {
            for rag in rags.iter() {
                let tmp = db.query_rag(rag, &query, Some("bge-large")).await?;
                docu.extend(tmp);
            }
        }
        println!("{:?}", docu);

        let found_chat_tool: Vec<ChatCompletionTool> = tools.into_iter()
        .filter_map(|key| tool_instance.tool_chat.get(&key).map(|value| value.clone())) 
        .collect();
        request.tools = Some(found_chat_tool);
        request.tool_choice = Some(ChatCompletionToolChoiceOption::Auto);
        let response = client
        .chat().create(request).await?;

        let response_message = response.choices.first().unwrap().message.clone();

        let mut hh = String::new();

        if let Some(tool_calls) = response_message.tool_calls {
            let mut handles = Vec::new();
            for tool_call in tool_calls {
                if let Some(cmd) = tool_instance.tool_exec.get(&tool_call.function.name).map(|v| v.clone()) {
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
                .max_tokens(max_tokens)
                .model(model)
                .messages(messages)
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
                                hh.push_str(content);
                            }
                        }
                    }
                    Err(err) => {
                        return Err(Box::new(err) as Box<dyn std::error::Error>);
                    }
                }
            }
        }
    
        Ok(response)
    }

    pub async fn embeddings(&self, request: CreateEmbeddingRequest, client: &Client<OpenAIConfig>) -> Result<CreateEmbeddingResponse, Box<dyn std::error::Error>>{
        let response = client.embeddings().create(request).await?;
        Ok(response)
    }
}      

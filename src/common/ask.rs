use std::io::{stdout, Write};
use std::process::Command;
use std::sync::Arc;
use std::collections::HashMap;
use serde_json::Value;
use tokio::sync::Mutex;

use anyhow::{Context, Result};
use async_openai::types::{ChatCompletionMessageToolCall, ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage, ChatCompletionRequestToolMessageArgs, ChatCompletionToolType, CreateChatCompletionRequest, CreateChatCompletionRequestArgs, FinishReason, FunctionCall};
use futures::StreamExt;

use crate::common::{config::Config, input::Input};

// #[async_recursion::async_recursion]
pub async fn ask(
    config: &Config,
    input: Input,
) -> Result<()> {
    if input.is_empty() {
        return Ok(());
    }

    let request_message = config.write().echo_message(&input)?;
    let request_tool = config.write().echo_tool()?;

    let request: CreateChatCompletionRequest;
    if config.read().working_mode.is_realtime() {
        request = CreateChatCompletionRequestArgs::default()
        .model(config.read().model.clone())
        .messages(request_message)
        .tools(request_tool)
        .build()?;
    } else {
        request = input.request.clone();
    }

    let mut stream = config.read().client.chat().create_stream(request).await?;

    let mut lock = stdout().lock();
    let mut contents:String = String::new();

    config.write().before_chat_completion(&input)?;


    let tool_call_states: Arc<Mutex<HashMap<(u32, u32), ChatCompletionMessageToolCall>>> =
    Arc::new(Mutex::new(HashMap::new()));

    while let Some(result) = stream.next().await {
        match result {
            Ok(response) => {
                println!("{:?}", response);
                for chat_choice in response.choices {
                    let function_responses: Arc<
                        Mutex<Vec<(ChatCompletionMessageToolCall, Value)>>,
                    > = Arc::new(Mutex::new(Vec::new()));

                    if let Some(tool_calls) = chat_choice.delta.tool_calls {
                        for tool_call_chunk in tool_calls.into_iter() {
                            let key = (chat_choice.index, tool_call_chunk.index);
                            let states = tool_call_states.clone();
                            let tool_call_data = tool_call_chunk.clone();

                            let mut states_lock = states.lock().await;
                            let state = states_lock.entry(key).or_insert_with(|| {
                                ChatCompletionMessageToolCall {
                                    id: tool_call_data.id.clone().unwrap_or_default(),
                                    r#type: ChatCompletionToolType::Function,
                                    function: FunctionCall {
                                        name: tool_call_data
                                            .function
                                            .as_ref()
                                            .and_then(|f| f.name.clone())
                                            .unwrap_or_default(),
                                        arguments: tool_call_data
                                            .function
                                            .as_ref()
                                            .and_then(|f| f.arguments.clone())
                                            .unwrap_or_default(),
                                    },
                                }
                            });
                            if let Some(arguments) = tool_call_chunk
                                .function
                                .as_ref()
                                .and_then(|f| f.arguments.as_ref())
                            {
                                state.function.arguments.push_str(arguments);
                            }
                        }
                    }

                    if let Some(finish_reason) = &chat_choice.finish_reason {
                        if matches!(finish_reason, FinishReason::ToolCalls) {
                            let tool_call_states_clone = tool_call_states.clone();

                            let tool_calls_to_process = {
                                let states_lock = tool_call_states_clone.lock().await;
                                states_lock
                                    .iter()
                                    .map(|(_key, tool_call)| {
                                        let name = tool_call.function.name.clone();
                                        let args = tool_call.function.arguments.clone();
                                        let tool_call_clone = tool_call.clone();
                                        (name, args, tool_call_clone)
                                    })
                                    .collect::<Vec<_>>()
                            };

                            let mut handles = Vec::new();
                            for (name, args, tool_call_clone) in tool_calls_to_process {
                                let response_content_clone = function_responses.clone();
                                // let cmd = config.tool_exec(&name);
                                let handle = tokio::spawn(async move {
                                    let response_content = call_fn(&name, &args).await.unwrap();
                                    let mut function_responses_lock =
                                        response_content_clone.lock().await;
                                    function_responses_lock
                                        .push((tool_call_clone, response_content));
                                });
                                handles.push(handle);
                            }

                            for handle in handles {
                                handle.await.unwrap();
                            }

                            let function_responses_clone = function_responses.clone();
                            let function_responses_lock = function_responses_clone.lock().await;
                            
                            let mut messages = config.write().echo_message(&input)?;
                            
                            let tool_calls: Vec<ChatCompletionMessageToolCall> =
                                function_responses_lock
                                    .iter()
                                    .map(|tc| tc.0.clone())
                                    .collect();

                            let assistant_messages: ChatCompletionRequestMessage =
                                ChatCompletionRequestAssistantMessageArgs::default()
                                    .tool_calls(tool_calls)
                                    .build()
                                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
                                    .unwrap()
                                    .into();

                            let tool_messages: Vec<ChatCompletionRequestMessage> =
                                function_responses_lock
                                    .iter()
                                    .map(|tc| {
                                        ChatCompletionRequestToolMessageArgs::default()
                                            .content(tc.1.to_string())
                                            .tool_call_id(tc.0.id.clone())
                                            .build()
                                            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
                                            .unwrap()
                                            .into()
                                    })
                                    .collect();

                            messages.push(assistant_messages);
                            messages.extend(tool_messages);

                            let request = CreateChatCompletionRequestArgs::default()
                                .model(config.read().model.clone())
                                .messages(messages)
                                .build()?;
                            
                            let mut stream = config.read().client.chat().create_stream(request).await?;

                            let mut response_content = String::new();
                            let mut lock = stdout().lock();
                            while let Some(result) = stream.next().await {
                                match result {
                                    Ok(response) => {
                                        for chat_choice in response.choices.iter() {
                                            if let Some(ref content) = chat_choice.delta.content {
                                                contents += content;
                                                write!(lock, "{}", content).unwrap();
                                                response_content.push_str(content);
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        writeln!(lock, "error: {err}").unwrap();
                                        // return Err(Box::new(err) as Box<dyn std::error::Error>);
                                    }
                                }
                            }
                        }
                    }

                    if let Some(content) = &chat_choice.delta.content {
                        let mut lock = stdout().lock();
                        contents += content;
                        write!(lock, "{}", content).unwrap();
                    }
                }
            }
            Err(err) => {
                writeln!(lock, "error: {err}").unwrap();
            }
        }
        stdout().flush()?;
    }
    config.write().after_chat_completion(&input, &contents)?;
    Ok(())
}

pub async fn call_fn(
    cmd: &str,
    args: &str,
    // envs: Option<HashMap<String, String>>,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = Command::new(cmd)
    .arg(args)
    .output()?;
    let stdout = std::str::from_utf8(&output.stdout).context("Invalid UTF-8 in stdout")?;
    let function_response = serde_json::from_str(stdout).context(r#"The crawler response is invalid. It should follow the JSON format: `[{"path":"...", "text":"..."}]`."#)?;
    Ok(function_response)
 }

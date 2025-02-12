use std::io::{stdout, Write};


use anyhow::Result;
use async_openai::types::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequest, CreateChatCompletionRequestArgs};
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
    let request: CreateChatCompletionRequest;
    if config.read().working_mode.is_realtime() {
        request = CreateChatCompletionRequestArgs::default()
        .model(config.read().model.clone())
        .messages([ChatCompletionRequestUserMessageArgs::default()
            .content(input.text.clone())
            .build()?
            .into()])
        .build()?;
    } else {
        request = input.request.clone();
    }

    let mut stream = config.read().client.chat().create_stream(request).await?;

    let mut lock = stdout().lock();
    let mut contents:String = String::new();

    config.write().before_chat_completion(&input)?;

    while let Some(result) = stream.next().await {
        match result {
            Ok(response) => {
                response.choices.iter().for_each(|chat_choice| {
                    if let Some(ref content) = chat_choice.delta.content {
                        write!(lock, "{}", content).unwrap();
                        contents += content;
                    }
                });
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
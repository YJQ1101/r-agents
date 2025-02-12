use std::path::Path;

use super::{ensure_parent_exists, input::Input, TEMP_SESSION_NAME};
use anyhow::{Context, Result};
use async_openai::types::{ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage, ChatCompletionRequestUserMessageArgs};
use fancy_regex::Regex;
use inquire::{validator::Validation, Confirm, Text};
use std::fs::{read_to_string, write};
use serde::{Deserialize, Serialize};

lazy_static::lazy_static! {
    static ref RE_AUTONAME_PREFIX: Regex = Regex::new(r"\d{8}T\d{6}-").unwrap();
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Session {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    // compressed_messages: Vec<Message>,
    messages: Vec<ChatCompletionRequestMessage>,
    #[serde(skip)]
    pub name: String,
    #[serde(skip)]
    path: Option<String>,
    #[serde(skip)]
    pub dirty: bool,
    #[serde(skip)]
    save_session: Option<bool>,
}

impl Session {
    pub fn new(name: &str) -> Self {
        let mut session = Self {
            name: name.to_string(),
            ..Default::default()
        };
        session.dirty = false;
        session
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn load(name: &str, path: &Path) -> Result<Self> {
        let content = read_to_string(path)
            .with_context(|| format!("Failed to load session {} at {}", name, path.display()))?;
        let mut session: Session =
            serde_yaml::from_str(&content).with_context(|| format!("Invalid session {}", name))?;

        // session.model = Model::retrieve_model(config, &session.model_id, ModelType::Chat)?;
        if let Some(autoname) = name.strip_prefix("_/") {
            session.name = TEMP_SESSION_NAME.to_string();
            session.path = None;
            if let Ok(true) = RE_AUTONAME_PREFIX.is_match(autoname) {
                // session.autoname = Some(AutoName::new(autoname[16..].to_string()));
            }
        } else {
            session.name = name.to_string();
            session.path = Some(path.display().to_string());
        }
        Ok(session)
    }

    pub fn save(&mut self, session_name: &str, session_path: &Path, is_realtime: bool) -> Result<()> {
        ensure_parent_exists(session_path)?;

        self.path = Some(session_path.display().to_string());

        let content = serde_yaml::to_string(&self)
            .with_context(|| format!("Failed to serde session '{}'", self.name))?;
        write(session_path, content).with_context(|| {
            format!(
                "Failed to write session '{}' to '{}'",
                self.name,
                session_path.display()
            )
        })?;

        if is_realtime {
            println!("✓ Saved session to '{}'.", session_path.display());
        }

        if self.name != session_name {
            self.name = session_name.to_string()
        }

        self.dirty = false;

        Ok(())
    }
 
    pub fn exit(&mut self, session_dir: &Path, is_realtime: bool) -> Result<()> {
        if self.dirty && self.save_session != Some(false){
            let mut session_dir = session_dir.to_path_buf();
            let mut session_name = self.name.to_string();
            if self.save_session.is_none() {
                if !is_realtime {
                    return Ok(());
                }
                let ans = Confirm::new("Save session?").with_default(false).prompt()?;
                if !ans {
                    return Ok(());
                }
                if session_name == TEMP_SESSION_NAME {
                    session_name = Text::new("Session name:")
                        .with_validator(|input: &str| {
                            let input = input.trim();
                            if input.is_empty() {
                                Ok(Validation::Invalid("This name is required".into()))
                            } else if input == TEMP_SESSION_NAME {
                                Ok(Validation::Invalid("This name is reserved".into()))
                            } else {
                                Ok(Validation::Valid)
                            }
                        })
                        .prompt()?;
                }
            } else if self.save_session == Some(true) && session_name == TEMP_SESSION_NAME {
                session_dir = session_dir.join("_");
                ensure_parent_exists(&session_dir).with_context(|| {
                    format!("Failed to create directory '{}'", session_dir.display())
                })?;

                let now = chrono::Local::now();
                session_name = now.format("%Y%m%dT%H%M%S").to_string();
                if let Some(autoname) = Some("32423532") {
                    session_name = format!("{session_name}-{autoname}")
                }
            }
            let session_path = session_dir.join(format!("{session_name}.yaml"));
            self.save(&session_name, &session_path, is_realtime)?;
        }
        Ok(())
    }


    pub fn add_message(&mut self, input: &Input, output: &str) -> Result<()> {
        // if input.continue_output().is_some() {
        //     if let Some(message) = self.messages.last_mut() {
        //         match message {
        //             ChatCompletionRequestMessage::Assistant(text) => *text = format!("{text}{output}"),
        //             ChatCompletionRequestMessage::Tool(text) => query += &serde_json::to_string(&chat_completion_request_tool_message.content)?,
        //             _ => todo!(),
        //         }
        //         if let CreateMessageRequestContent::Content(text) = &mut message {
        //             *text = format!("{text}{output}");
        //         }
        //     }
        // } else {
            // if self.messages.is_empty() {
            //     if self.name == TEMP_SESSION_NAME {
                    // let raw_input = input.raw();
                    // let chat_history = format!("USER: {raw_input}\nASSISTANT: {output}\n");
                    // self.autoname = Some(AutoName::new_from_chat_history(chat_history));
                // }
                // self.messages.extend(input.role().build_messages(input));
            // } else {
                self.messages
                    .push(ChatCompletionRequestUserMessageArgs::default().content(input.message_content()).build()?.into());
            // }
            // self.data_urls.extend(input.data_urls());
            // if let Some(tool_calls) = input.tool_calls() {
            //     self.messages.push(ChatCompletionRequestToolMessageArgs::default().content(tool_calls).build()?.into());
            // }
            self.messages.push(ChatCompletionRequestAssistantMessageArgs::default().content(output).build()?.into());
        // }
        self.dirty = true;
        Ok(())
    }

    pub fn clear_messages(&mut self) {
        self.messages.clear();
        self.dirty = true;
    }

}


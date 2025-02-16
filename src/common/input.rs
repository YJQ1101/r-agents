use std::collections::HashMap;

use async_openai::types::CreateChatCompletionRequest;

#[derive(Debug, Clone)]
pub struct Input {
    pub text: String,
    pub request: CreateChatCompletionRequest,
    patched_text: Option<String>,
    continue_output: Option<String>,
    regenerate: bool,
    medias: Vec<String>,
    data_urls: HashMap<String, String>,
    // tool_calls: Option<MessageContentToolCalls>,
    rag_name: Option<String>,
    // role: Role,
    // with_session: bool,
    // with_agent: bool,
}

impl Input {
    pub fn from_str(text: &str) -> Self {
        // let (role, with_session, with_agent) = resolve_role(&config.read(), role);
        Self {
            // config: config.clone(),
            text: text.to_string(),
            request: Default::default(),
            patched_text: None,
            continue_output: None,
            regenerate: false,
            medias: Default::default(),
            data_urls: Default::default(),
            // tool_calls: None,
            rag_name: None,
            // with_session: todo!(),
            // with_agent: todo!(),
            // role,
            // with_session,
            // with_agent,
        }
    }
    pub fn from_web(request: &CreateChatCompletionRequest) -> Self {
        // let (role, with_session, with_agent) = resolve_role(&config.read(), role);
        Self {
            // config: config.clone(),
            text: Default::default(),
            request: request.clone(),
            patched_text: None,
            continue_output: None,
            regenerate: false,
            medias: Default::default(),
            data_urls: Default::default(),
            // tool_calls: None,
            rag_name: None,
            // with_session: todo!(),
            // with_agent: todo!(),
            // role,
            // with_session,
            // with_agent,
        }
    }


    pub fn is_empty(&self) -> bool {
        self.text.is_empty() && self.medias.is_empty()
    }

    pub fn message_content(&self) -> &str {
        &self.text
    }

    pub fn regenerate(&self) -> bool {
        self.regenerate
    }
}
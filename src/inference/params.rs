use serde::{Deserialize, Serialize};

use super::instance::ChatInstance;

#[derive(Debug,Clone)]
pub struct EmbeddingsParams {
    pub input:String,
    pub model:String,
    pub encoding_format:String,
    pub user:String,
}

impl Default for EmbeddingsParams {
    fn default() -> Self {
        Self {
            input:Default::default(),
            model:Default::default(),
            encoding_format:Default::default(),
            user:Default::default(),
        }
    }
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub enum Role {
    system,
    user,
    assistant,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Messages {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Serialize,Deserialize)]
pub struct ChatWebParams {
    pub messages:Option<Vec<Messages>>,
    pub model:Option<String>,
    pub seed: Option<u64>,
    pub temp: Option<f64>,
    pub top_p: Option<f64>,
    pub repeat_penalty: Option<f32>,
    // pub repeat_last_n: usize,
}

impl ChatWebParams {
    fn messages_template(msg_list: &Vec<Messages>)->String {
        let mut history = String::new();
        for msg in msg_list {
            history.push_str("<|im_start|>");
            if msg.role == Role::system {
               history.push_str("system\n");
            } else if msg.role == Role::user {
               history.push_str("user\n");
            } else {
               history.push_str("assistant\n");
            }
            history.push_str(msg.content.as_str());
            history.push_str("<|im_end|>\n");
        }
        history.push_str("<|im_start|>assistant\n");
        history
    }

    pub fn to_instance(&self) -> Result<ChatInstance,String> {
        let mut instance = ChatInstance::default();
        if let Some(promote) = &self.messages{
            instance.promote = Self::messages_template(promote);
        }
        else{
            return Err("promote should provide!".to_owned());
        }

        if let Some(model) = &self.model{
            instance.model = model.to_string();
        }
        else{
            return Err("model should provide!".to_owned());
        }

        instance.seed = self.seed.unwrap_or(1u64);
        instance.temp = None;
        instance.repeat_penalty = self.repeat_penalty.unwrap_or(1f32);
        Ok(instance)
    }
}

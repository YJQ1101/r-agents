use actix_web::{web, HttpResponse, Responder};
use async_openai::{config::OpenAIConfig, types::{ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequest}, Client};
use serde::Deserialize;

use super::core::{Agent, Exec, Tools};

#[derive(PartialEq, Debug, Deserialize)]
pub enum Role {
    system,
    user,
    assistant,
}

#[derive(Debug, Deserialize)]
pub struct Messages {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct ChatWebParams {
    pub messages:Vec<Messages>,
    pub model:String,
    pub seed: Option<u64>,
    pub temp: Option<f64>,
    pub top_p: Option<f64>,
    pub repeat_penalty: Option<f32>,
}

impl ChatWebParams{
    pub fn to_instance(&self) -> Vec<ChatCompletionRequestMessage> {
        let request: Vec<ChatCompletionRequestMessage> = self.messages.iter().map(|msg| {
            if msg.role == Role::user {
                ChatCompletionRequestUserMessageArgs::default()
                .content(msg.content.clone())
                .build().unwrap()
                .into()
            } else if msg.role == Role::system {
                ChatCompletionRequestSystemMessageArgs::default()
                .content(msg.content.clone())
                .build().unwrap()
                .into()
            } else {
                ChatCompletionRequestAssistantMessageArgs::default()
                .content(msg.content.clone())
                .build().unwrap()
                .into()
            }
        }).collect();
        request
    }
}

pub async fn run(a:web::Json<ChatWebParams>,agent:web::Data<Agent>, client:web::Data<Client<OpenAIConfig>>, function:web::Data<Tools>, exec:web::Data<Exec>) -> impl Responder {
    let msg = a.0.to_instance();

    match agent.run(msg, &client, &function, &exec).await {
        Ok(response) => {
            HttpResponse::Ok().body(response)
        }
        Err(e) => {
            println!("Error occurred: {:?}", e); 
            HttpResponse::InternalServerError().body("no")
        }
    }
}

pub fn app_config(config:&mut web::ServiceConfig){
    config.service(
        web::scope("/r-agents/v1")
            .service(web::resource("/chat/completions")
                .route( web::post().to(run))
            )
    );
}

// curl http://localhost:8848/r-agents/v1/chat/completions \
// -H "Content-Type: application/json" \
//   -d '{
//     "model": "llama3.2:1b",
//     "messages": [
//       {
//         "role": "system",
//         "content": "You are a helpful assistant."
//       },
//       {
//         "role": "user",
//         "content": "Hello!"
//       }
//     ]
//   }'
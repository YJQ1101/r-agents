use actix_web::{web, HttpResponse, Responder};
use async_openai::{config::OpenAIConfig, types::{ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestToolMessageArgs, ChatCompletionRequestUserMessageArgs}, Client};
use serde::Deserialize;

use super::core::{Agent, Exec, Tools};

#[derive(PartialEq, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
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
    pub fn to_instance(&self, instructions: &str) -> Vec<ChatCompletionRequestMessage> {
        let mut request: Vec<ChatCompletionRequestMessage> = vec![
            ChatCompletionRequestSystemMessageArgs::default()
            .content(instructions)
            .build().unwrap()
            .into()];
        self.messages.iter().for_each(|msg| {
            if msg.role == Role::User {
                request.push(ChatCompletionRequestUserMessageArgs::default()
                .content(msg.content.clone())
                .build().unwrap()
                .into());
            } else if msg.role == Role::Assistant {
                request.push(ChatCompletionRequestAssistantMessageArgs::default()
                .content(msg.content.clone())
                .build().unwrap()
                .into());
            } else if msg.role == Role::Tool {
                request.push(ChatCompletionRequestToolMessageArgs::default()
                .content(msg.content.clone())
                .build().unwrap()
                .into());
            }
        });
        request
    }
}

pub async fn run(a:web::Json<ChatWebParams>,agent:web::Data<Agent>, client:web::Data<Client<OpenAIConfig>>, function:web::Data<Tools>, exec:web::Data<Exec>) -> impl Responder {
    let msg = a.0.to_instance(&agent.instructions);
    let model = &a.0.model;
    match agent.run(model, msg, &client, &function, &exec).await {
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

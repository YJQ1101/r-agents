use actix_web::{web::{self}, HttpResponse, Responder};
use async_openai::{config::OpenAIConfig, types::{CreateChatCompletionRequest, CreateEmbeddingRequest}, Client};

use super::{agent::Agent, db::Database, tool::ToolInstance};

pub async fn chat_completions(a:web::Json<CreateChatCompletionRequest>, agent:web::Data<Agent>, client:web::Data<Client<OpenAIConfig>>, db:web::Data<Box<dyn Database>>, tool_instance:web::Data<ToolInstance>) -> impl Responder {
    match agent.chat_completions(a.0, &client, &db, &tool_instance).await {
        Ok(response) => {
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            println!("Error occurred: {:?}", e); 
            HttpResponse::InternalServerError().body("error")
        }
    }
}

pub async fn embeddings(a:web::Json<CreateEmbeddingRequest>, agent:web::Data<Agent>, client:web::Data<Client<OpenAIConfig>>) -> impl Responder {
    match agent.embeddings(a.0, &client).await {
        Ok(response) => {
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            println!("Error occurred: {:?}", e); 
            HttpResponse::InternalServerError().body("error")
        }
    }
}

pub fn app_config(config:&mut web::ServiceConfig){
    config.service(
        web::scope("/r-agents/v1")
            .service(web::resource("/chat/completions")
                .route( web::post().to(chat_completions))
            )
            .service(web::resource("/embeddings")
                .route( web::post().to(embeddings))
            )
    );
}

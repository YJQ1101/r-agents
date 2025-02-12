use actix_web::{web::{self}, HttpResponse, Responder};
use async_openai::types::CreateChatCompletionRequest;
use serde::Deserialize;

use crate::common::{ask::ask, config::Config, input::Input};
pub async fn chat_completions(a:web::Json<CreateChatCompletionRequest>, config:web::Data<Config>) -> impl Responder {
    let input = Input::from_web(&a.0);
    match ask(&config, input).await {
        Ok(response) => {
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            println!("Error occurred: {:?}", e); 
            HttpResponse::InternalServerError().body("error")
        }
    }
}

pub async fn info(config:web::Data<Config>) -> impl Responder {
    let info = config.read().sysinfo();
    match info {
        Ok(response) => {
            HttpResponse::Ok().body(response)
        }
        Err(e) => {
            println!("Error occurred: {:?}", e); 
            HttpResponse::InternalServerError().body("error")
        }
    }
}

pub async fn regenerate(config:web::Data<Config>) -> impl Responder {
    let (input, _) = match config.read().last_message.clone() {
        Some(v) => v,
        None => return HttpResponse::InternalServerError().body("error"),
    };
    match ask(&config, input).await {
        Ok(response) => {
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            println!("Error occurred: {:?}", e); 
            HttpResponse::InternalServerError().body("error")
        }
    }
}

#[derive(Debug,Deserialize,Default)]
pub struct SessionWebParams {
    pub name: String,
}

pub async fn session(param:web::Query<SessionWebParams>, config:web::Data<Config>) -> impl Responder {
    let name = &param.0.name;
    match config.write().use_session(Some(&name)) {
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
        web::scope("/r-agents")
            .service(web::resource("/v1/chat/completions")
                .route( web::post().to(chat_completions))
            )
            .service(web::resource("/info")
                .route( web::post().to(info))
            )
            .service(web::resource("/regenerate")
                .route( web::post().to(regenerate))
            )
            .service(web::resource("/session")
                .route( web::get().to(session))
            )
    );
}

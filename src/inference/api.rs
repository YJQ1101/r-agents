use actix::Addr;
use actix_web::{web, HttpResponse, Responder};
use super::core::{InferenceActor, Openai};
use super::params::ChatWebParams;
// params:web::Json<ChatWebParams>, 
pub async fn chat(params:web::Json<ChatWebParams>, inference_actor:web::Data<Addr<InferenceActor>>) -> impl Responder {
    let params = params.to_instance();
    match params {
        Ok(instance) => {
            match inference_actor.send(Openai::Chat(instance)).await {
                Ok(res) => {
                    HttpResponse::Ok().body("good");
                },
                Err(_) => todo!(),
            }
        }
        Err(e) => {

            HttpResponse::InternalServerError().body(e);
        }
    } 
    HttpResponse::Ok().body("good")
}

pub fn inference_config(config:&mut web::ServiceConfig) {
    config.service(web::scope("/rllama")
        .service(web::resource("/chat").route(web::post().to(chat)))
    );
}

use std::error::Error;
use actix::Actor;
use actix_web::{web::Data, App, HttpServer};

use crate::inference::core::InferenceActor;
use crate::models::get_inference_context;

use super::config::AppSysConfig;
use super::config::app_config;

#[actix_web::main]
pub async fn load_model(model_name: String) -> Result<(), Box<dyn Error>>  {
    let sys_config = AppSysConfig::init_from_env();
    let http_addr = sys_config.get_http_addr();
    println!("{}", http_addr);
    log::info!("http server addr:{}",&http_addr);
    let inference_context = get_inference_context(model_name);
    let inference_actor = InferenceActor::new(inference_context).start();

    let mut server = HttpServer::new(move || {
        let inference_actor = inference_actor.clone();
        App::new()
            .app_data(Data::new(inference_actor))
            .configure(app_config)
    });
    if let Some(num) = sys_config.http_workers {
        server=server.workers(num);
    }

    server.bind(http_addr)?
    .run()
    .await?;
    Ok(())
}
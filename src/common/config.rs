use actix_web::web;
use crate::{inference::api::inference_config, web::index::index_config};

#[derive(Default,Clone,Debug)]
pub struct AppSysConfig{
    // pub config_db_file:String,
    pub http_port:u16,
    pub http_workers:Option<usize>,
    // pub grpc_port:u16,
}

impl AppSysConfig {
    pub fn init_from_env() -> Self {
        let http_port=std::env::var("RllamaR_HTTP_PORT")
            .unwrap_or("8848".to_owned()).parse()
            .unwrap_or(8848);
        let http_workers = std::env::var("RllamaR_HTTP_WORKERS")
            .unwrap_or("".to_owned())
            .parse().ok();
        Self { 
            http_port,
            http_workers,
        }
    }

    pub fn get_http_addr(&self) -> String {
        format!("0.0.0.0:{}",&self.http_port)
    }
}

pub fn app_config(config:&mut web::ServiceConfig){
    index_config(config);
    inference_config(config);
}

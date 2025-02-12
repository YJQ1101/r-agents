
use std::sync::Arc;

use actix_web::{middleware, web::Data, App, HttpServer};
use anyhow::Result;
use parking_lot::RwLock;
use crate::{common::{config::{CConfig, Config}, WorkingMode}, serve::route::app_config};

#[derive(Clone)]
pub struct Server {
    config: Config,
}

impl Server {
    pub fn init(config: &str) -> Result<Self> {
        let config = Arc::new(RwLock::new(CConfig::init(config.into())?));
        Ok(Server {
            config
        })
    }
    
    fn boot(&mut self) -> Result<()> {
        self.config.write().working_mode = WorkingMode::Serve;
        self.config.write().create_client()?;
        Ok(())
    }

    pub async fn run(&mut self) -> Result<()>{
        self.boot()?;
        let http_addr = self.config.read().get_http_addr()?;
        let config = self.config.clone();
        log::info!("http server addr:{}",&http_addr);
        let server = HttpServer::new(move || {
            let config = config.clone();
            App::new()
                .app_data(Data::new(config))
                .wrap(middleware::Logger::default())
                .configure(app_config)
        });

        server.bind(http_addr)?
        .run()
        .await?;
        Ok(())
    }
}
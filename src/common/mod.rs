pub mod cli;
pub mod config;
pub mod core;
pub mod server;

#[derive(Default,Clone,Debug)]
pub struct AppSysConfig{
    pub api_base:String,
    pub api_key:String,
    pub http_port:u16,
}

impl AppSysConfig {
    pub fn init_from_env() -> anyhow::Result<Self> {
        let api_base = std::env::var("OPENAI_API_BASE")
            .unwrap_or_else(|_| "".to_string())
            .into();
        let api_key = std::env::var("OPENAI_API_KEY")
            .unwrap_or_else(|_| "".to_string())
            .into();
        let http_port=std::env::var("HTTP_PORT")
        .unwrap_or("8848".to_owned())
        .parse().unwrap_or(8848);
        Ok(Self { 
            api_base,
            api_key,
            http_port,
        })
    }
    pub fn get_http_addr(&self) -> String {
        format!("0.0.0.0:{}",&self.http_port)
    }
}
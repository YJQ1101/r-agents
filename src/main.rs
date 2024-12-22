use std::error::Error;
use actix_web::web::Data;
use actix_web::{middleware, App, HttpServer};
use async_openai::config::OpenAIConfig;
use async_openai::Client;
use clap::Parser;
use ragents::common::cli::Cli;
use ragents::common::config::{AgentsConfig, ToolsConfig};
use ragents::common::core::{Agent, Exec, Tools};
use ragents::common::server::app_config;
use ragents::common::AppSysConfig;
use anyhow::Result;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let cli = Cli::parse();
    let agents_config = AgentsConfig::init(cli.agents.into())?;
    let tools_config = ToolsConfig::init(cli.tools.into())?;
    let sys_config = AppSysConfig::init_from_env()?;

    let agent = Agent::init(agents_config.agents.get(&cli.agent)
        .expect(&format!("Error: Agent {} not found .",&cli.agent)))?;

    println!("\x1b[32;1mAgent Found:\x1b[0m {}", &cli.agent);

    let http_addr = sys_config.get_http_addr();
    let api_base = sys_config.get_api_base();
    let api_key = sys_config.get_api_key();
    log::info!("http server addr:{}",&http_addr);
    log::info!("http server addr:{}",&api_base);
    log::info!("http server addr:{}",&api_key);
    println!("\x1b[32;1mYou are using:\x1b[0m {} ", &api_base);

    let client = Client::with_config(
        OpenAIConfig::new()
        .with_api_key(api_key)
        .with_api_base(api_base)
    );

    let tools = Tools::init(agent.tools.iter()
        .filter_map(|tool| tools_config.tools_yaml.get(tool).map(|value| {
        value.to_string()})).collect())?;

    let exec = Exec::init(agent.tools.iter()
        .filter_map(|tool| tools_config.tools_exec.get(tool).map(|value| {
        (tool.to_string(), value.to_string())})).collect::<Vec<_>>())?;

    let server = HttpServer::new(move || {
        let agent = agent.clone();
        let client = client.clone();
        let tools = tools.clone();
        let exec = exec.clone();
        App::new()
            .app_data(Data::new(agent))
            .app_data(Data::new(client))
            .app_data(Data::new(tools))
            .app_data(Data::new(exec))
            .wrap(middleware::Logger::default())
            .configure(app_config)
    });

    server.bind(http_addr)?
    .run()
    .await?;
    Ok(())
}
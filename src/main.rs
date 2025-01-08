use std::error::Error;
use actix_web::web::Data;
use actix_web::{middleware, App, HttpServer};
use async_openai::config::OpenAIConfig;
use async_openai::Client;
use clap::Parser;
use ragents::common::cli::CliMain;
use ragents::common::config::{AgentsConfig, ToolsConfig};
use ragents::common::agent::Agent;
use ragents::common::db::create_database;
use ragents::common::server::app_config;
use ragents::common::tool::{Tool, ToolInstance};
use ragents::common::AppSysConfig;
use anyhow::Result;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let cli = CliMain::parse();
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
    log::info!("api_base addr:{}",&api_base);
    log::info!("api_key addr:{}",&api_key);
    println!("\x1b[32;1mYou are using:\x1b[0m {} ", &api_base);

    let mut tool_instance = ToolInstance::new();
    for use_tool in agent.tools.iter() {
        let tool = Tool::init(use_tool, tools_config.tools.get(use_tool).expect("no tools find"))?;
        tool_instance.tool_chat.extend(tool.parse_json()?);
        tool_instance.tool_exec.extend(tool.tool_exec);
        tool_instance.tool_embedding_model.insert(use_tool.to_string(), tool.tool_embedding_model);
    }

    let client = Client::with_config(
        OpenAIConfig::new()
        .with_api_key(api_key)
        .with_api_base(api_base)
    );

    let server = HttpServer::new(move || {
        let agent = agent.clone();
        let client = client.clone();
        let db = create_database(cli.database.as_str());
        let tool_instance = tool_instance.clone();
        App::new()
            .app_data(Data::new(agent))
            .app_data(Data::new(client))
            .app_data(Data::new(db))
            .app_data(Data::new(tool_instance))
            .wrap(middleware::Logger::default())
            .configure(app_config)
    });

    server.bind(http_addr)?
    .run()
    .await?;
    Ok(())
}
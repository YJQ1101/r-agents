use std::error::Error;
use clap::Parser;
use RllamaR::common::cli::Cli;
use RllamaR::common::config::{AgentsConfig, ToolsConfig};
use RllamaR::common::core::use_agent;
use RllamaR::common::AppSysConfig;
use anyhow::Result;

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let agents_config = AgentsConfig::init("./agents.yaml".into())?;
    let tools_config = ToolsConfig::init("./tools.yaml".into())?;
    let sys_config = AppSysConfig::init_from_env()?;

    if let Some(agent) = &cli.agent {
        agents_config.agents.get(agent).map(|value| {
            println!("Found: {} -> {}", agent, value);
            use_agent(value, tools_config, sys_config);
        });
    }
    Ok(())
}
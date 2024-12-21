use std::{collections::HashMap, fs::read_to_string, path::PathBuf};
use actix_web::web;
use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AgentsConfig {
    pub agents: HashMap<String, String>,
}

impl Default for AgentsConfig  {
    fn default() -> Self {
        Self {
            agents: Default::default(),
        }
    }
}

impl AgentsConfig {
    pub fn init(config_file: PathBuf) -> Result<Self> {
        // Self::load_from_file(&config_file)?
        let err = || format!("Failed to load config at '{}'", config_file.display());
        let content = read_to_string(&config_file).with_context(err)?;
        let config: Self = serde_yaml::from_str(&content)
            .map_err(|err| {
                let err_msg = err.to_string();
                // anyhow!("{err_msg}")
            })
            .unwrap();

        Ok(config)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ToolsConfig {
    pub tools_yaml: HashMap<String, String>,
    pub tools_exec: HashMap<String, String>,
}

impl Default for ToolsConfig  {
    fn default() -> Self {
        Self {
            tools_yaml: Default::default(),
            tools_exec: Default::default(),
        }
    }
}

impl ToolsConfig {
    pub fn init(config_file: PathBuf) -> Result<Self> {
        // Self::load_from_file(&config_file)?
        let err = || format!("Failed to load config at '{}'", config_file.display());
        let content = read_to_string(&config_file).with_context(err)?;
        let config: Self = serde_yaml::from_str(&content)
            .map_err(|err| {
                let err_msg = err.to_string();
                // anyhow!("{err_msg}")
            })
            .unwrap();

        Ok(config)
    }
}

use std::{collections::HashMap, fs::read_to_string, path::PathBuf};
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
        let err = || format!("Failed to load config at '{}'", config_file.display());
        let content = read_to_string(&config_file).with_context(err)?;
        let config = serde_yaml::from_str(&content)?;
        Ok(config)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ToolsConfig {
    pub tools: HashMap<String, String>,
}

impl Default for ToolsConfig  {
    fn default() -> Self {
        Self {
            tools: Default::default(),
        }
    }
}

impl ToolsConfig {
    pub fn init(config_file: PathBuf) -> Result<Self> {
        let err = || format!("Failed to load config at '{}'", config_file.display());
        let content = read_to_string(&config_file).with_context(err)?;
        let config = serde_yaml::from_str(&content)?;
        Ok(config)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct RagsConfig {
    pub rags: HashMap<String, String>,
}

impl Default for RagsConfig  {
    fn default() -> Self {
        Self {
            rags: Default::default(),
        }
    }
}

impl RagsConfig {
    pub fn init(config_file: PathBuf) -> Result<Self> {
        let err = || format!("Failed to load config at '{}'", config_file.display());
        let content = read_to_string(&config_file).with_context(err)?;
        let config = serde_yaml::from_str(&content)?;
        Ok(config)
    }
}

use std::{collections::HashMap, env, fs::{read_to_string, remove_file}, path::PathBuf, sync::Arc};
use anyhow::{bail, Context, Result};
use async_openai::{config::OpenAIConfig, types::{ChatCompletionRequestMessage, ChatCompletionRequestUserMessageArgs}, Client};
use parking_lot::RwLock;
use serde::Deserialize;
use crate::realtime::prompt::render_prompt;

use super::{agent::Agent, get_env_name, input::Input, normalize_env_name, rag::Rag, session::Session, WorkingMode, AGENTS_DIR_NAME, LEFT_PROMPT, RIGHT_PROMPT, SESSIONS_DIR_NAME, TEMP_SESSION_NAME};

pub type Config = Arc<RwLock<CConfig>>;

#[derive(Debug, Clone, Deserialize)]
pub struct CConfig {
    pub api_base: String,
    pub api_key: Option<String>,
    pub http_port: Option<u16>,

    pub model: String,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,

    pub agents: HashMap<String, String>,
    pub tools: HashMap<String, String>,
    pub rags: HashMap<String, String>,

    #[serde(skip)]
    pub agent: Option<Agent>,
    #[serde(skip)]
    pub rag: Option<Rag>,
    #[serde(skip)]
    pub session: Option<Session>,
    #[serde(skip)]
    pub last_message: Option<(Input, String)>,
    #[serde(skip)]
    pub client: Client<OpenAIConfig>,
    #[serde(skip)]
    pub working_mode: WorkingMode,
}

impl CConfig {
    pub fn init(config_file: PathBuf) -> Result<Self> {
        let err = || format!("Failed to load config at '{}'", config_file.display());
        let content = read_to_string(&config_file).with_context(err)?;
        let config = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn create_client(&mut self) -> Result<()> {
        let api_base = self.get_api_base();
        let api_key = self.get_api_key();
        log::info!("api_base addr:{}",&api_base);
        log::info!("api_key addr:{}",&api_key);
        let client: Client<OpenAIConfig> = Client::with_config(
            OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(api_base)
        );
        self.client = client;
        Ok(())
    }

    pub fn get_http_addr(&self) -> Result<String> {
        let http_port = self.http_port.unwrap_or(8848);
        Ok(format!("0.0.0.0:{}",http_port))
    }

    pub fn get_api_base(&self) -> String {
        let api_base = self.api_base.clone();
        api_base
    }

    pub fn get_api_key(&self) -> String {
        let api_key = self.api_key.clone().unwrap_or("".to_string());
        api_key
    }

    pub fn sysinfo(&self) -> Result<String> {
        let items = vec![
            ("api_base", self.api_base.clone()),
            ("model", self.model.clone()),
        ];
        let output = items
            .iter()
            .map(|(name, value)| format!("{name:<24}{value}\n"))
            .collect::<Vec<String>>()
            .join("");
        Ok(output)
    }

    pub fn config_dir() -> PathBuf {
        if let Ok(v) = env::var(get_env_name("config_dir")) {
            PathBuf::from(v)
        } else if let Ok(v) = env::var("XDG_CONFIG_HOME") {
            PathBuf::from(v).join(env!("CARGO_CRATE_NAME"))
        } else {
            let dir = dirs::config_dir().expect("No user's config directory");
            dir.join(env!("CARGO_CRATE_NAME"))
        }
    }

    pub fn local_path(name: &str) -> PathBuf {
        Self::config_dir().join(name)
    }

    pub fn agents_data_dir() -> PathBuf {
        Self::local_path(AGENTS_DIR_NAME)
    }

    pub fn agent_data_dir(name: &str) -> PathBuf {
        match env::var(format!("{}_DATA_DIR", normalize_env_name(name))) {
            Ok(value) => PathBuf::from(value),
            Err(_) => Self::agents_data_dir().join(name),
        }
    }

    pub fn render_prompt_left(&self) -> String {
        let variables = self.generate_prompt_context();
        let left_prompt = LEFT_PROMPT;
        render_prompt(left_prompt, &variables)
    }

    pub fn render_prompt_right(&self) -> String {
        let variables = self.generate_prompt_context();
        let right_prompt = RIGHT_PROMPT;
        render_prompt(right_prompt, &variables)
    }

    fn generate_prompt_context(&self) -> HashMap<&str, String> {
        let mut output = HashMap::new();
        output.insert("model", self.model.clone());
        // output.insert("client_name", role.model().client_name().to_string());
        // output.insert("model_name", role.model().name().to_string());
        // output.insert(
        //     "max_input_tokens",
        //     role.model()
        //         .max_input_tokens()
        //         .unwrap_or_default()
        //         .to_string(),
        // );
        if let Some(temperature) = self.temperature {
            if temperature != 0.0 {
                output.insert("temperature", temperature.to_string());
            }
        }
        if let Some(top_p) = self.top_p {
            if top_p != 0.0 {
                output.insert("top_p", top_p.to_string());
            }
        }
        if let Some(session) = &self.session {
            output.insert("session", session.name.to_string());
            // if let Some(autoname) = session.autoname() {
            //     output.insert("session_autoname", autoname.to_string());
            // }
            output.insert("dirty", session.dirty.to_string());
            // let (tokens, percent) = session.tokens_usage();
            // output.insert("consume_tokens", tokens.to_string());
            // output.insert("consume_percent", percent.to_string());
            // output.insert("user_messages_len", session.user_messages_len().to_string());
        }
        if let Some(rag) = &self.rag {
            output.insert("rag", rag.rag_embedding_model.to_string());
        }
        if let Some(agent) = &self.agent {
            output.insert("agent", agent.name.to_string());
        }

        output.insert("color.reset", "\u{1b}[0m".to_string());
        output.insert("color.black", "\u{1b}[30m".to_string());
        output.insert("color.dark_gray", "\u{1b}[90m".to_string());
        output.insert("color.red", "\u{1b}[31m".to_string());
        output.insert("color.light_red", "\u{1b}[91m".to_string());
        output.insert("color.green", "\u{1b}[32m".to_string());
        output.insert("color.light_green", "\u{1b}[92m".to_string());
        output.insert("color.yellow", "\u{1b}[33m".to_string());
        output.insert("color.light_yellow", "\u{1b}[93m".to_string());
        output.insert("color.blue", "\u{1b}[34m".to_string());
        output.insert("color.light_blue", "\u{1b}[94m".to_string());
        output.insert("color.purple", "\u{1b}[35m".to_string());
        output.insert("color.light_purple", "\u{1b}[95m".to_string());
        output.insert("color.magenta", "\u{1b}[35m".to_string());
        output.insert("color.light_magenta", "\u{1b}[95m".to_string());
        output.insert("color.cyan", "\u{1b}[36m".to_string());
        output.insert("color.light_cyan", "\u{1b}[96m".to_string());
        output.insert("color.white", "\u{1b}[37m".to_string());
        output.insert("color.light_gray", "\u{1b}[97m".to_string());

        output
    }

    pub fn before_chat_completion(&mut self, input: &Input) -> Result<()> {
        self.last_message = Some((input.clone(), String::new()));
        Ok(())
    }

    pub fn after_chat_completion(&mut self, input: &Input, output: &str) -> Result<()> {
        self.last_message = Some((input.clone(), output.to_string()));
        self.save_message(input, output)?;
        Ok(())
    }

    fn save_message(&mut self, input: &Input, output: &str) -> Result<()> {
        if let Some(session) = &mut self.session {
            session.add_message(&input, output)?;
            return Ok(());
        }
        if output.is_empty() {
            return Ok(());
        }
        Ok(())
    }

    pub fn echo_message(&mut self, input: &Input) -> Result<Vec<ChatCompletionRequestMessage>> {
        if let Some(session) = &mut self.session {
            session.echo_messages(&input)
        } else {
            Ok(vec![ChatCompletionRequestUserMessageArgs::default().content(input.message_content()).build()?.into()])
        }   
    }

    pub fn sessions_dir(&self) -> PathBuf {
        match &self.agent {
            None => match env::var(get_env_name("sessions_dir")) {
                Ok(value) => PathBuf::from(value),
                Err(_) => Self::local_path(SESSIONS_DIR_NAME),
            },
            Some(agent) => Self::agent_data_dir(&agent.name).join(SESSIONS_DIR_NAME),
        }
    }

    pub fn session_file(&self, name: &str) -> PathBuf {
        match name.split_once("/") {
            Some((dir, name)) => self.sessions_dir().join(dir).join(format!("{name}.yaml")),
            None => self.sessions_dir().join(format!("{name}.yaml")),
        }
    }

    pub fn exit_session(&mut self) -> Result<()> {
        if let Some(mut session) = self.session.take() {
            let sessions_dir = self.sessions_dir();
            session.exit(&sessions_dir, self.working_mode.is_realtime())?;
            self.last_message = None;
        }
        Ok(())
    }

    pub fn save_session(&mut self, name: Option<&str>) -> Result<()> {
        let session_name = match &self.session {
            Some(session) => match name {
                Some(v) => v.to_string(),
                None => "".to_string(),
            },
            None => bail!("No session"),
        };
        let session_path = self.session_file(&session_name);
        if let Some(session) = self.session.as_mut() {
            session.save(&session_name, &session_path, self.working_mode.is_realtime())?;
        }
        Ok(())
    }

    pub fn use_session(&mut self, session_name: Option<&str>) -> Result<()> {
        if self.session.is_some() {
            bail!(
                "Already in a session, please run '.exit session' first to exit the current session."
            );
        }
        let mut session;
        match session_name {
            None | Some(TEMP_SESSION_NAME) => {
                let session_file = self.session_file(TEMP_SESSION_NAME);
                if session_file.exists() {
                    remove_file(session_file).with_context(|| {
                        format!("Failed to cleanup previous '{TEMP_SESSION_NAME}' session")
                    })?;
                }
                session = Some(Session::new(TEMP_SESSION_NAME));
            }
            Some(name) => {
                let session_path = self.session_file(name);
                if !session_path.exists() {
                    session = Some(Session::new(name));
                } else {
                    session = Some(Session::load(name, &session_path)?);
                }
            }
        }
        // if let Some(session) = session.as_mut() {
        //     if session.is_empty() {
        //         if let Some((input, output)) = &self.last_message {
        //             if self.agent.is_some() == input.with_agent() {
        //                 let ans = Confirm::new(
        //                     "Start a session that incorporates the last question and answer?",
        //                 )
        //                 .with_default(false)
        //                 .prompt()?;
        //                 if ans {
        //                     session.add_message(input, output)?;
        //                 }
        //             }
        //         }
        //     }
        // }
        self.session = session;
        // self.init_agent_session_variables()?;
        Ok(())
    }

    pub fn empty_session(&mut self) -> Result<()> {
        if let Some(session) = self.session.as_mut() {
            // if let Some(agent) = self.agent.as_ref() {
            //     session.sync_agent(agent);
            // }
            session.clear_messages();
        } else {
            bail!("No session")
        }
        self.last_message = None;
        Ok(())
    }

    pub async fn use_agent(&mut self, agent_name: &str, session_name: Option<&str>) -> Result<()> {
        if self.agent.is_some() {
            bail!("Already in a agent, please run '.exit agent' first to exit the current agent.");
        }
        match self.agents.get(agent_name) {
            Some(agent_path) => {
                let agent = Agent::init(agent_name, agent_path)?;
                self.rag = agent.rag(&self.rags)?;
                self.agent = Some(agent);
            },
            None => {
                bail!("No this agent");
            }
        }
        Ok(())
    }

    pub fn exit_agent(&mut self) -> Result<()> {
        self.exit_session()?;
        if self.agent.take().is_some() {
            self.rag.take();
            self.last_message = None;
        }
        Ok(())
    }
}

use std::{fs::create_dir_all, path::Path};

use anyhow::{anyhow, Context, Result};

pub mod config;
pub mod agent;
pub mod rag;
pub mod tool;
pub mod session;
pub mod input;
pub mod ask;
pub mod loader;
pub mod db;

const TEMP_SESSION_NAME: &str = "temp";
const LEFT_PROMPT: &str = "{color.green}{?session {?agent {agent}>}{session}{?role /}}{!session {?agent {agent}>}}{role}{?rag @{rag}}{color.cyan}{?session )}{!session >}{color.reset} ";
const RIGHT_PROMPT: &str = "{color.purple}{?session {?consume_tokens {consume_tokens}({consume_percent}%)}{!consume_tokens {consume_tokens}}}{color.reset}";
const SESSIONS_DIR_NAME: &str = "sessions";
const AGENTS_DIR_NAME: &str = "agents";

const RAG_TEMPLATE: &str = r#"Answer the query based on the context while respecting the rules. (user query, some textual context and rules, all inside xml tags)

<context>
__CONTEXT__
</context>

<rules>
- If you don't know, just say so.
- If you are not sure, ask for clarification.
- Answer in the same language as the user query.
- If the context appears unreadable or of poor quality, tell the user then answer as best as you can.
- If the answer is not in the context but you think you know the answer, explain that to the user then answer with your own knowledge.
- Answer directly and without using xml tags.
</rules>

<user_query>
__INPUT__
</user_query>"#;

pub fn get_env_name(key: &str) -> String {
    format!("{}_{key}", env!("CARGO_CRATE_NAME"),).to_ascii_uppercase()
}

pub fn normalize_env_name(value: &str) -> String {
    value.replace('-', "_").to_ascii_uppercase()
}

pub fn ensure_parent_exists(path: &Path) -> Result<()> {
    if path.exists() {
        return Ok(());
    }
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("Failed to write to '{}', No parent path", path.display()))?;
    if !parent.exists() {
        create_dir_all(parent).with_context(|| {
            format!(
                "Failed to write to '{}', Cannot create parent directory",
                path.display()
            )
        })?;
    }
    Ok(())
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkingMode {
    Realtime,
    Serve,
}

impl WorkingMode {
    pub fn is_realtime(&self) -> bool {
        *self == WorkingMode::Realtime
    }
    pub fn is_serve(&self) -> bool {
        *self == WorkingMode::Serve
    }
}
impl Default for WorkingMode {
    fn default() -> Self {
        WorkingMode::Realtime
    }
}

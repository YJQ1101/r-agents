pub mod cli;
pub mod config;
pub mod agent;
pub mod server;
pub mod rag;
pub mod tool;
pub mod db;

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

    pub fn get_api_base(&self) -> String {
        self.api_base.clone()
    }

    pub fn get_api_key(&self) -> String {
        self.api_key.clone()
    }
}

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
use std::error::Error;
use chromadb::v2::{collection::QueryOptions, embeddings::openai::{OpenAIConfig, OpenAIEmbeddings}, ChromaClient};
use async_trait::async_trait;
use anyhow::Result;

#[async_trait]
pub trait Database {
    async fn query_tool(&self, name: &str, query: &str, embedding_function: Option<&str>) -> Result<Vec<String>, Box<dyn Error>>;
    async fn query_rag(&self, name: &str, query: &str, embedding_function: Option<&str>) -> Result<Vec<String>, Box<dyn Error>>;
}

pub struct Chroma {
    client: ChromaClient,
}

impl Chroma {
    pub fn new() -> Self {
        let client = ChromaClient::new(Default::default());
        Chroma {
            client,
        }
    }
}

#[async_trait]
impl Database for Chroma {
    async fn query_tool(&self, name: &str, query: &str, embedding_function: Option<&str>) -> Result<Vec<String>, Box<dyn Error>>{
        let collection = self.client.get_collection(name).await?;
            let query = QueryOptions {
                query_texts: Some(vec![query]),
                query_embeddings: None,
                where_metadata: None,
                where_document: None,
                n_results: Some(5),
                include: None,
            };
    
            let openai_embeddings = OpenAIEmbeddings::new(OpenAIConfig{ 
                api_endpoint: format!("{}{}", std::env::var("OPENAI_API_BASE").unwrap_or_else(|_| "".to_string()), "/embeddings"),
                api_key: std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "".to_string()).into(),
                model: embedding_function.unwrap_or("beg-large").to_string(),
            });
            let query_result = collection.query(query, Some(Box::new(openai_embeddings))).await?;
        Ok(query_result.ids.into_iter().flatten().collect())
    }

    async fn query_rag(&self, name: &str, query: &str, embedding_function: Option<&str>) -> Result<Vec<String>, Box<dyn Error>> {
        let collection = self.client.get_collection(name).await?;
            let query = QueryOptions {
                query_texts: Some(vec![query]),
                query_embeddings: None,
                where_metadata: None,
                where_document: None,
                n_results: Some(5),
                include: None,
            };
    
            let openai_embeddings = OpenAIEmbeddings::new(OpenAIConfig{ 
                api_endpoint: format!("{}{}", std::env::var("OPENAI_API_BASE").unwrap_or_else(|_| "".to_string()), "/embeddings"),
                api_key: std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "".to_string()).into(),
                model: embedding_function.unwrap_or("beg-large").to_string(),
            });
            let query_result = collection.query(query, Some(Box::new(openai_embeddings))).await?;
        let documents = query_result.documents.unwrap_or(vec![]);
        Ok(documents.into_iter().flatten().collect())
    }
}

pub fn create_database(database: &str) -> Box<dyn Database> {
    match database {
        "chromadb" => {
            Box::new(Chroma::new())
        }
        _ => {
            eprintln!("Unknown database type: {}", database);
            Box::new(Chroma::new())
        }
    }
}
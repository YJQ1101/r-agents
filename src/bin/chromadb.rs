use std::{error::Error, fs::{self}};
use chromadb::v2::{collection::CollectionEntries, embeddings::openai::{OpenAIConfig, OpenAIEmbeddings}, ChromaClient, ChromaCollection};
use anyhow::Result;
use clap::Parser;
use ragents::common::{cli::CliChromadb, config::{RagsConfig, ToolsConfig}, rag::Rag, tool::Tool};
use tokenizers::Tokenizer;
   
pub async fn rags_load_chromadb(name: &str, rag: Rag) -> Result<(),  Box<dyn Error>> {
    let client: ChromaClient = ChromaClient::new(Default::default());
    let collection: ChromaCollection = client.get_or_create_collection(name, None).await?;

    // Get the UUID of the collection
    let collection_uuid = collection.id();
    println!("\x1b[32;1mCollection UUID:\x1b[0m {}", collection_uuid);

    let mut chunks = Vec::new();
    let mut ids = Vec::new();
    
    let tokenizer = Tokenizer::from_pretrained("bert-base-cased", None).unwrap();
    for document in rag.documents.iter() {
        let content = fs::read_to_string(document)?;
        let encoding = tokenizer.encode(content, false).unwrap();
        let tokens: Vec<&str>= encoding.get_tokens().iter().map(|s| s.as_str()).collect();
        let mut start = 0;

        while start < tokens.len() {
            let end = std::cmp::min(start + rag.rag_chunk_size, tokens.len());
            let chunk = tokens[start..end].join(" ");  // Use slice directly for chunks
            
            chunks.push(chunk);
            ids.push(format!("{}{}{}", document, "-id-", start));
            start += rag.rag_chunk_size - rag.rag_chunk_overlap;
        }
    }
    println!("{:?}", chunks);
    println!("{:?}", ids);

    // Upsert some embeddings with documents and no metadata.
    let collection_entries = CollectionEntries {
        ids: ids.iter().map(|s| s.as_str()).collect(),
        embeddings: None,
        metadatas: None,
        documents: Some(chunks.iter().map(|s| s.as_str()).collect())
    };

    let openai_embeddings = OpenAIEmbeddings::new(OpenAIConfig{ 
        api_endpoint: format!("{}{}", std::env::var("OPENAI_API_BASE").unwrap_or_else(|_| "".to_string()), "/embeddings"),
        api_key: std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "".to_string()).into(),
        model: rag.rag_embedding_model,
    });
    collection.upsert(collection_entries, Some(Box::new(openai_embeddings))).await?;
    println!("\x1b[32;1mSuccessful!\x1b[0m");
    Ok(())
}

pub async fn tools_load_chromadb(name: &str, tool: Tool) -> Result<(),  Box<dyn Error>> {
    let client = ChromaClient::new(Default::default());
    let collection = client.get_or_create_collection(name, None).await?;

    // Get the UUID of the collection
    let collection_uuid = collection.id();
    println!("\x1b[32;1mCollection UUID:\x1b[0m {}", collection_uuid);

    let mut chunks = Vec::new();
    let mut ids = Vec::new();
    
    let result = tool.parse_json()?;
    for (tool_name,tool_json) in result.into_iter() {
        ids.push(tool_name);
        chunks.push(tool_json.function.description.unwrap_or(tool_json.function.name));
    }
    println!("{:?}", chunks);
    println!("{:?}", ids);

    let collection_entries = CollectionEntries {
        ids: ids.iter().map(|s| s.as_str()).collect(),
        embeddings: None,
        metadatas: None,
        documents: Some(chunks.iter().map(|s| s.as_str()).collect())
    };

    let openai_embeddings = OpenAIEmbeddings::new(OpenAIConfig{ 
        api_endpoint: format!("{}{}", std::env::var("OPENAI_API_BASE").unwrap_or_else(|_| "".to_string()), "/embeddings"),
        api_key: std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "".to_string()).into(),
        model: tool.tool_embedding_model,
    });
    collection.upsert(collection_entries, Some(Box::new(openai_embeddings))).await?;
    println!("\x1b[32;1mSuccessful!\x1b[0m");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(),  Box<dyn Error>> {
    let cli = CliChromadb::parse();

    let tools_config = ToolsConfig::init(cli.tools.into())?;
    let rags_config = RagsConfig::init(cli.rags.into())?;

    for (name, path) in rags_config.rags.iter() {
        let rag = Rag::init(name, path).expect(&format!("Error: Rag {} not found .",name));
        rags_load_chromadb(name, rag).await?;
    }

    for (name, path) in tools_config.tools.iter() {
        let tool = Tool::init(name, path).expect(&format!("Error: Rag {} not found .",name));
        tools_load_chromadb(name, tool).await?;
    }
    Ok(())
}

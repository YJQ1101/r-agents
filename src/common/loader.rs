use std::collections::HashMap;

pub const EXTENSION_METADATA: &str = "__extension__";

pub type DocumentMetadata = HashMap<String, String>;

pub const DEFAULT_EXTENSION: &str = "txt";

#[derive(Debug, Clone)]
pub struct LoadedDocument {
    pub path: String,
    pub contents: String,
    pub metadata: DocumentMetadata,
}

impl LoadedDocument {
    pub fn new(path: String, contents: String, metadata: DocumentMetadata) -> Self {
        Self {
            path,
            contents,
            metadata,
        }
    }
}

// pub async fn load_file(loaders: &HashMap<String, String>, path: &str) -> Result<LoadedDocument> {
//     let extension = get_patch_extension(path).unwrap_or_else(|| DEFAULT_EXTENSION.into());
//     match loaders.get(&extension) {
//         Some(loader_command) => load_with_command(path, &extension, loader_command),
//         None => load_plain(path, &extension).await,
//     }
// }

// pub async fn load_url(loaders: &HashMap<String, String>, path: &str) -> Result<LoadedDocument> {
//     let (contents, extension) = fetch(loaders, path, false).await?;
//     let mut metadata: DocumentMetadata = Default::default();
//     metadata.insert(EXTENSION_METADATA.into(), extension);
//     Ok(LoadedDocument::new(path.into(), contents, metadata))
// }

// async fn load_plain(path: &str, extension: &str) -> Result<LoadedDocument> {
//     let contents = tokio::fs::read_to_string(path).await?;
//     let mut metadata: DocumentMetadata = Default::default();
//     metadata.insert(EXTENSION_METADATA.into(), extension.to_string());
//     Ok(LoadedDocument::new(path.into(), contents, metadata))
// }

// fn load_with_command(path: &str, extension: &str, loader_command: &str) -> Result<LoadedDocument> {
//     let contents = run_loader_command(path, extension, loader_command)?;
//     let mut metadata: DocumentMetadata = Default::default();
//     metadata.insert(EXTENSION_METADATA.into(), DEFAULT_EXTENSION.to_string());
//     Ok(LoadedDocument::new(path.into(), contents, metadata))
// }

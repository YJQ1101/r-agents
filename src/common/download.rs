use hf_hub::{api::sync::Api, Repo, RepoType};

pub fn download_model(model_name: String) {
    let api = Api::new().unwrap();

    let repo = api.repo(Repo::with_revision(
        model_name,
        RepoType::Model,
        "main".to_string(),
    ));

    let config_file = repo.get("config.json").unwrap();
    let tokenizer_filename = repo.get("tokenizer.json").unwrap();
    let filename = repo.get("model.safetensors").unwrap();

    // let filenames = vec![repo.get("model.safetensors")?];
    println!("\x1b[1;32mDownload successful!\x1b[0m");
    println!("\x1b[1;32mConfig.json path: \x1b[0m{:?}", config_file.as_os_str());
    println!("\x1b[1;32mTokenizer.json path: \x1b[0m{:?}", tokenizer_filename.as_os_str());
    println!("\x1b[1;32mmodel.safetensors path: \x1b[0m{:?}", filename.as_os_str());
}

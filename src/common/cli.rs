use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// agents yaml file
    #[clap(short = 'a', long)]
    pub agents: String,
    /// tools yaml file
    #[clap(short = 't', long)]
    pub tools: String,
    /// Start a agent
    #[clap(long)]
    pub agent: String,
    /// Input text
    #[clap(trailing_var_arg = true)]
    text: Vec<String>,
}

impl Cli {
    pub fn text(&self) -> Option<String> {
        let text = self
            .text
            .iter()
            .map(|x| x.trim().to_string())
            .collect::<Vec<String>>()
            .join(" ");
        if text.is_empty() {
            return None;
        }
        Some(text)
    }
}

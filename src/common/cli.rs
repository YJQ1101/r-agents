use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliMain {
    /// agents yaml file
    #[clap(short = 'a', long)]
    pub agents: String,
    /// Start a agent
    #[clap(long)]
    pub agent: String,
    /// tools yaml file
    #[clap(short = 't', long)]
    pub tools: String,
    /// Start a database
    #[clap(short = 'd', long)]
    pub database: String,
    /// Input text
    #[clap(trailing_var_arg = true)]
    text: Vec<String>,
}

#[derive(Parser, Debug)]
pub struct CliChromadb {
    /// tools yaml file
    #[clap(short = 't', long)]
    pub tools: String,
    /// rags yaml file
    #[clap(short = 'r', long)]
    pub rags: String,
}

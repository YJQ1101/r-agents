pub mod common;
pub mod realtime;
pub mod serve;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliMain {
    #[command(subcommand)]
    pub mode: Mode,
}

#[derive(Subcommand, Debug)]
pub enum Mode {
    /// Start the program in Realtime mode
    Realtime {
        /// Specify the config file for Realtime mode
        #[arg(long)]
        config: String,
    },
    /// Start the program in Server mode
    Serve {
        /// Specify the config file for Server mode
        #[arg(long)]
        config: String,
    },
}

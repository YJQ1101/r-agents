use std::error::Error;
use clap::Parser;
use ragents::{realtime::realtime::Realtime, serve::server::Server};
use anyhow::Result;
use ragents::{CliMain, Mode};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = CliMain::parse();
    match &cli.mode {
        Mode::Realtime{ config } => {
            println!("Starting in Realtime mode...");
            let mut realtime = Realtime::init(config)?;
            if let Err(err) = realtime.run().await {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        }

        Mode::Serve{ config } => {
            println!("Starting in Server mode...");
            let mut server = Server::init(config)?;
            if let Err(err) = server.run().await {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        }
    }
    Ok(())
}
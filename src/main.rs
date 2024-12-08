use RllamaR::common::download::download_model;
use RllamaR::common::load::load_model;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long,short = 'p')]
    pull: Option<String>,

    #[arg(long,short = 'r')]
    run: Option<String>,
}

fn main(){
    let args = Args::parse();
    
    let provided_fields = vec![
        args.pull.is_some(),
        args.run.is_some(),
    ];

    let count = provided_fields.iter().filter(|&&b| b).count();

    if count > 1 || count == 0 {
        eprintln!("\x1b[1;31merror:\x1b[0m Please provide only one of the following fields: --pull, --run");
        return;
    }  

    if let Some(model_name) = args.pull {
        println!("\x1b[1;32mPulling with model: \x1b[0m{}", model_name);
        download_model(model_name);
    }  

    if let Some(model_name) = args.run {
        println!("\x1b[1;32mLoading model: \x1b[0m{}", model_name);
        load_model(model_name);
    } 
}

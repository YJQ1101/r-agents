use std::sync::Arc;

use anyhow::{bail, Result};
use parking_lot::RwLock;
use reedline::{Reedline, Signal};

use crate::{common::{ask::ask, config::{CConfig, Config}, input::Input, WorkingMode}, realtime::{dump_realtime_help, parse_command, split_args, unknown_command, MULTILINE_RE}};

use super::{abort::{create_abort_signal, AbortSignal}, editor::create_editor, prompt::RealtimePrompt};

pub struct Realtime {
    editor: Reedline,
    config: Config,
    prompt: RealtimePrompt,
    abort_signal: AbortSignal,
}

impl Realtime {
    pub fn init(config: &str) -> Result<Self> {
        let editor = create_editor()?;
        let config = Arc::new(RwLock::new(CConfig::init(config.into())?));
        let prompt = RealtimePrompt::new(&config);
        let abort_signal = create_abort_signal();

        Ok(Self {
            editor,
            config,
            prompt,
            abort_signal,
        })
    }

    fn boot(&mut self) -> Result<()> {
        self.config.write().working_mode = WorkingMode::Realtime;
        self.config.write().create_client()?;
        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        self.boot()?;
        loop {
            if self.abort_signal.aborted_ctrld() {
                break;
            }
            let sig = self.editor.read_line(&self.prompt);
            match sig {
                Ok(Signal::Success(line)) => {
                    self.abort_signal.reset();
                    match self.handle(&line).await {
                        Ok(exit) => {
                            if exit {
                                break;
                            }
                        }
                        Err(err) => {
                            println!("{}", err)
                        }
                    }
                }
                Ok(Signal::CtrlC) => {
                    self.abort_signal.set_ctrlc();
                    println!("(To exit, press Ctrl+D or enter \".exit\")\n");
                }
                Ok(Signal::CtrlD) => {
                    self.abort_signal.set_ctrld();
                    break;
                }
                _ => {}
            }
        }
        self.config.write().exit_session()?;
        Ok(())
    }

    async fn handle(&mut self, mut line: &str) -> Result<bool> {
        if let Ok(Some(captures)) = MULTILINE_RE.captures(line) {
            if let Some(text_match) = captures.get(1) {
                line = text_match.as_str();
            }
        }
        match parse_command(line) {
            Some((cmd, args)) => match cmd {
                ".help" => {
                    dump_realtime_help();
                }
                ".info" => {
                    let output = self.config.read().sysinfo()?;
                    print!("{}", output);
                }
                ".list" => match args {
                    Some("session") => {
                        // let info = self.config.read().session_info()?;
                        // print!("{}", info);
                    }
                    Some("agent") => {
                        let info = &self.config.read().agents;
                        print!("{:?}", info);
                    }
                    Some(_) => unknown_command()?,
                    None => println!(r#"Usage: .list <session|rag|agent|tool>"#),
                },
                ".agent" => match split_args(args) {
                    Some((agent_name, session_name)) => {
                        self.config.write().use_agent(agent_name, session_name).await?;
                    }
                    None => println!(r#"Usage: .agent <agent-name> [session-name]"#),
                },
                ".regenerate" => {
                    let (input, _) = match self.config.read().last_message.clone() {
                        Some(v) => v,
                        None => bail!("Unable to regenerate the last response"),
                    };
                    ask(&self.config, input).await?;
                }
                ".session" => {
                    self.config.write().use_session(args)?;
                }
                ".empty" => match args {
                    Some("session") => {
                        self.config.write().empty_session()?;
                    }
                    _ => {
                        println!(r#"Usage: .empty <session> [name]"#)
                    }
                },
                ".exit" => match args {
                    Some("session") => {
                        self.config.write().exit_session()?;
                    }
                    _ => {
                        println!(r#"Usage: .exit <session> [name]"#)
                    }
                },
                _ => unknown_command()?,
            },
            None => {
                let input = Input::from_str( line);
                ask(&self.config, input).await?;
            }
        }
        println!();
        Ok(false)
    }
}

#[derive(Debug, Clone)]
pub struct RealtimeCommand {
    pub name: &'static str,
    pub description: &'static str,
}

impl RealtimeCommand {
    pub fn new(name: &'static str, desc: &'static str) -> Self {
        Self {
            name,
            description: desc,
        }
    }
}

use fancy_regex::Regex;
use realtime::RealtimeCommand;
use anyhow::{bail, Result};

pub mod realtime;
pub mod abort;
pub mod editor;
pub mod prompt;
pub mod highlighter;

lazy_static::lazy_static! {
    static ref REALTIME_COMMANDS: [RealtimeCommand; 15] = [
        RealtimeCommand::new(".help", "Show this help message"),
        RealtimeCommand::new(".info", "View system info"),
        RealtimeCommand::new(
            ".prompt",
            "Create a temporary role using a prompt",
        ),
        RealtimeCommand::new(
            ".role",
            "Create or switch to a specific role",
        ),
        RealtimeCommand::new(
            ".session",
            "Begin a session",
        ),
        RealtimeCommand::new(
            ".empty session",
            "Erase messages in the current session",
        ),
        RealtimeCommand::new(
            ".save session",
            "Save the current session to file",
        ),
        RealtimeCommand::new(
            ".exit session",
            "End the session",
        ),
        RealtimeCommand::new(".agent", "Use a agent"),
        RealtimeCommand::new(
            ".exit agent",
            "Leave the agent",
        ),
        RealtimeCommand::new(
            ".rag",
            "Init or use the RAG",
        ),
        RealtimeCommand::new(
            ".exit rag",
            "Leave the RAG",
        ),
        RealtimeCommand::new(".continue", "Continue the response"),
        RealtimeCommand::new(
            ".regenerate",
            "Regenerate the last response",
        ),
        RealtimeCommand::new(".exit", "Exit the Realtime"),
    ];
    static ref COMMAND_RE: Regex = Regex::new(r"^\s*(\.\S*)\s*").unwrap();
    static ref MULTILINE_RE: Regex = Regex::new(r"(?s)^\s*:::\s*(.*)\s*:::\s*$").unwrap();
}

fn parse_command(line: &str) -> Option<(&str, Option<&str>)> {
    match COMMAND_RE.captures(line) {
        Ok(Some(captures)) => {
            let cmd = captures.get(1)?.as_str();
            let args = line[captures[0].len()..].trim();
            let args = if args.is_empty() { None } else { Some(args) };
            Some((cmd, args))
        }
        _ => None,
    }
}

pub fn unknown_command() -> Result<()> {
    bail!(r#"Unknown command. Type ".help" for additional help."#);
}

pub fn split_args(args: Option<&str>) -> Option<(&str, Option<&str>)> {
    args.map(|v| match v.split_once(' ') {
        Some((subcmd, args)) => (subcmd, Some(args.trim())),
        None => (v, None),
    })
}

fn dump_realtime_help() {
    let head = REALTIME_COMMANDS
        .iter()
        .map(|cmd| format!("{:<24} {}", cmd.name, cmd.description))
        .collect::<Vec<String>>()
        .join("\n");
    println!(
        r###"{head}

Type ::: to start multi-line editing, type ::: to finish it.
Press Ctrl+O to open an editor for editing the input buffer.
Press Ctrl+C to cancel the response, Ctrl+D to exit the Realtime."###,
    );
}


use nu_ansi_term::{Color, Style};
use reedline::{Highlighter, StyledText};

use super::REALTIME_COMMANDS;

const DEFAULT_COLOR: Color = Color::Default;
const MATCH_COLOR: Color = Color::Green;

pub struct RealtimeHighlighter;

impl RealtimeHighlighter {
    pub fn new() -> Self {
        Self
    }
}

impl Highlighter for RealtimeHighlighter {
    fn highlight(&self, line: &str, _cursor: usize) -> StyledText {
        let mut styled_text = StyledText::new();

        if REALTIME_COMMANDS.iter().any(|cmd| line.contains(cmd.name)) {
            let matches: Vec<&str> = REALTIME_COMMANDS
                .iter()
                .filter(|cmd| line.contains(cmd.name))
                .map(|cmd| cmd.name)
                .collect();
            let longest_match = matches.iter().fold(String::new(), |acc, &item| {
                if item.len() > acc.len() {
                    item.to_string()
                } else {
                    acc
                }
            });
            let buffer_split: Vec<&str> = line.splitn(2, &longest_match).collect();

            styled_text.push((Style::new().fg(DEFAULT_COLOR), buffer_split[0].to_string()));
            styled_text.push((Style::new().fg(MATCH_COLOR), longest_match));
            styled_text.push((Style::new().fg(DEFAULT_COLOR), buffer_split[1].to_string()));
        } else {
            styled_text.push((Style::new().fg(DEFAULT_COLOR), line.to_string()));
        }

        styled_text
    }
}

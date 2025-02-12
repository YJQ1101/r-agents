use reedline::{Prompt, PromptHistorySearch, PromptHistorySearchStatus};
use std::{borrow::Cow, collections::HashMap};

use crate::common::config::Config;

#[derive(Clone)]
pub struct RealtimePrompt {
    config: Config,
}

impl RealtimePrompt {
    pub fn new(config: &Config) -> Self {
        Self {
            config : config.clone(),
        }
    }
}

impl Prompt for RealtimePrompt {
    fn render_prompt_left(&self) -> Cow<str> {
        Cow::Owned(self.config.read().render_prompt_left())
    }

    fn render_prompt_right(&self) -> Cow<str> {
        Cow::Owned(self.config.read().render_prompt_right())
    }

    fn render_prompt_indicator(&self, _prompt_mode: reedline::PromptEditMode) -> Cow<str> {
        Cow::Borrowed("")
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<str> {
        Cow::Borrowed("... ")
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: PromptHistorySearch,
    ) -> Cow<str> {
        let prefix = match history_search.status {
            PromptHistorySearchStatus::Passing => "",
            PromptHistorySearchStatus::Failing => "failing ",
        };
        // NOTE: magic strings, given there is logic on how these compose I am not sure if it
        // is worth extracting in to static constant
        Cow::Owned(format!(
            "({}reverse-search: {}) ",
            prefix, history_search.term
        ))
    }
}

pub fn render_prompt(template: &str, variables: &HashMap<&str, String>) -> String {
    let exprs = parse_template(template);
    eval_exprs(&exprs, variables)
}

fn parse_template(template: &str) -> Vec<Expr> {
    let chars: Vec<char> = template.chars().collect();
    let mut exprs = vec![];
    let mut current = vec![];
    let mut balances = vec![];
    for ch in chars.iter().cloned() {
        if !balances.is_empty() {
            if ch == '}' {
                balances.pop();
                if balances.is_empty() {
                    if !current.is_empty() {
                        let block = parse_block(&mut current);
                        exprs.push(block)
                    }
                } else {
                    current.push(ch);
                }
            } else if ch == '{' {
                balances.push(ch);
                current.push(ch);
            } else {
                current.push(ch);
            }
        } else if ch == '{' {
            balances.push(ch);
            add_text(&mut exprs, &mut current);
        } else {
            current.push(ch)
        }
    }
    add_text(&mut exprs, &mut current);
    exprs
}

fn parse_block(current: &mut Vec<char>) -> Expr {
    let value: String = current.drain(..).collect();
    match value.split_once(' ') {
        Some((name, tail)) => {
            if let Some(name) = name.strip_prefix('?') {
                let block_exprs = parse_template(tail);
                Expr::Block(BlockType::Yes, name.to_string(), block_exprs)
            } else if let Some(name) = name.strip_prefix('!') {
                let block_exprs = parse_template(tail);
                Expr::Block(BlockType::No, name.to_string(), block_exprs)
            } else {
                Expr::Text(format!("{{{value}}}"))
            }
        }
        None => Expr::Variable(value),
    }
}

fn eval_exprs(exprs: &[Expr], variables: &HashMap<&str, String>) -> String {
    let mut output = String::new();
    for part in exprs {
        match part {
            Expr::Text(text) => output.push_str(text),
            Expr::Variable(variable) => {
                let value = variables
                    .get(variable.as_str())
                    .cloned()
                    .unwrap_or_default();
                output.push_str(&value);
            }
            Expr::Block(typ, variable, block_exprs) => {
                let value = variables
                    .get(variable.as_str())
                    .cloned()
                    .unwrap_or_default();
                match typ {
                    BlockType::Yes => {
                        if truly(&value) {
                            let block_output = eval_exprs(block_exprs, variables);
                            output.push_str(&block_output)
                        }
                    }
                    BlockType::No => {
                        if !truly(&value) {
                            let block_output = eval_exprs(block_exprs, variables);
                            output.push_str(&block_output)
                        }
                    }
                }
            }
        }
    }
    output
}

fn add_text(exprs: &mut Vec<Expr>, current: &mut Vec<char>) {
    if current.is_empty() {
        return;
    }
    let value: String = current.drain(..).collect();
    exprs.push(Expr::Text(value));
}

fn truly(value: &str) -> bool {
    !(value.is_empty() || value == "0" || value == "false")
}

#[derive(Debug)]
enum Expr {
    Text(String),
    Variable(String),
    Block(BlockType, String, Vec<Expr>),
}

#[derive(Debug)]
enum BlockType {
    Yes,
    No,
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_render {
        ($template:expr, [$(($key:literal, $value:literal),)*], $expect:literal) => {
            let data = HashMap::from([
                $(($key, $value.into()),)*
            ]);
            assert_eq!(render_prompt($template, &data), $expect);
        };
    }

    #[test]
    fn test_render() {
        let prompt = "{?session {session}{?role /}}{role}{?session )}{!session >}";
        assert_render!(prompt, [], ">");
        assert_render!(prompt, [("role", "coder"),], "coder>");
        assert_render!(prompt, [("session", "temp"),], "temp)");
        assert_render!(
            prompt,
            [("session", "temp"), ("role", "coder"),],
            "temp/coder)"
        );
    }
}

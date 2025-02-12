use reedline::{default_emacs_keybindings, ColumnarMenu, EditCommand, EditMode, Emacs, KeyCode, KeyModifiers, Keybindings, MenuBuilder, Reedline, ReedlineEvent, ReedlineMenu};
use anyhow::Result;

use super::highlighter::RealtimeHighlighter;

fn extra_keybindings(keybindings: &mut Keybindings) {
    keybindings.add_binding(
        KeyModifiers::NONE,
        KeyCode::Tab,
        ReedlineEvent::UntilFound(vec![
            ReedlineEvent::Menu("completion_menu".to_string()),
            ReedlineEvent::MenuNext,
        ]),
    );
    keybindings.add_binding(
        KeyModifiers::SHIFT,
        KeyCode::BackTab,
        ReedlineEvent::MenuPrevious,
    );
    keybindings.add_binding(
        KeyModifiers::CONTROL,
        KeyCode::Enter,
        ReedlineEvent::Edit(vec![EditCommand::InsertNewline]),
    );
    keybindings.add_binding(
        KeyModifiers::SHIFT,
        KeyCode::Enter,
        ReedlineEvent::Edit(vec![EditCommand::InsertNewline]),
    );
    keybindings.add_binding(
        KeyModifiers::ALT,
        KeyCode::Enter,
        ReedlineEvent::Edit(vec![EditCommand::InsertNewline]),
    );
}

pub fn create_editor() -> Result<Reedline> {
    let highlighter = RealtimeHighlighter::new();
    let menu = {
        let completion_menu = ColumnarMenu::default().with_name("completion_menu");
        ReedlineMenu::EngineCompleter(Box::new(completion_menu))
    };

    let edit_mode: Box<dyn EditMode> = {
        let mut keybindings = default_emacs_keybindings();
        extra_keybindings(&mut keybindings);
        Box::new(Emacs::new(keybindings))
    };
    let editor = Reedline::create()
        .with_highlighter(Box::new(highlighter))
        .with_menu(menu)
        .with_edit_mode(edit_mode)
        .with_quick_completions(true)
        .with_partial_completions(true)
        .use_bracketed_paste(true)
        .with_ansi_colors(true);

    Ok(editor)
}
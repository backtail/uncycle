use crate::app;

use ratatui::{
    layout::Rect,
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crossterm::event::KeyCode;

use app::App;

pub fn render_help_tab(f: &mut Frame, app: &App, area: Rect) {
    // Controls block
    let controls_text: Vec<Line> = app
        .keybindings
        .get_bindings_for_display()
        .iter()
        .map(|binding| {
            let key_str = match binding.key {
                KeyCode::Char(' ') => "[Space]".to_string(),
                KeyCode::Char(c) => format!("[{}]", c.to_uppercase()),
                _ => format!("[{:?}]", binding.key),
            };
            Line::from(format!("{} - {}", key_str, binding.description))
        })
        .collect();

    let controls_block = Block::default().title("Controls").borders(Borders::ALL);
    let controls = Paragraph::new(controls_text)
        .block(controls_block)
        .wrap(Wrap { trim: true });
    f.render_widget(controls, area);
}

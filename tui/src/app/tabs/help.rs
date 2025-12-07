use crate::app::keybindings::Keybindings;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::Line,
    widgets::{Block, Paragraph, Widget, Wrap},
};

use crossterm::event::KeyCode;

#[derive(Default)]
pub struct HelpMenu {
    keybindings: Keybindings,
}

impl Widget for HelpMenu {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let controls_text: Vec<Line> = self
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

        let controls_block = Block::default();

        Paragraph::new(controls_text)
            .block(controls_block)
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }
}

use crate::app;

use ratatui::{
    layout::Rect,
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use app::App;

pub fn render_settings_tab(f: &mut Frame, _app: &App, area: Rect) {
    let settings_text = vec![
        Line::from("MIDI Configuration:"),
        Line::from("  • Input port: Auto-detected"),
    ];

    let settings_block = Block::default().title("Settings").borders(Borders::ALL);
    let settings = Paragraph::new(settings_text)
        .block(settings_block)
        .wrap(Wrap { trim: true });
    f.render_widget(settings, area);
}

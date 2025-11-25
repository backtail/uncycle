use crate::app;

use ratatui::{
    layout::Rect,
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use app::App;

const NO_INPUT: &str = "Not set";

pub fn render_settings_tab(f: &mut Frame, _app: &App, area: Rect) {
    let state = _app.midi_state.lock().unwrap();
    let settings_text = vec![
        Line::from(format!(
            "In port:  {}",
            state.port_in_name.as_ref().unwrap_or(&NO_INPUT.to_string())
        )),
        Line::from(format!(
            "Out port: {}",
            state
                .port_out_name
                .as_ref()
                .unwrap_or(&NO_INPUT.to_string())
        )),
    ];

    let settings_block = Block::default()
        .title("MIDI Configuration")
        .borders(Borders::ALL);
    let settings = Paragraph::new(settings_text)
        .block(settings_block)
        .wrap(Wrap { trim: true });
    f.render_widget(settings, area);
}

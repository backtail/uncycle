use crate::app;

use ratatui::{
    layout::Rect,
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use app::App;

pub fn render_main_tab(f: &mut Frame, app: &App, area: Rect) {
    let status_text;
    {
        if let Ok(midi_state) = app.midi_state.lock() {
            status_text = vec![Line::from(format!("BPM: {}", midi_state.clock_bpm))];
        } else {
            status_text = vec![];
        }
    }

    let status_block = Block::default().title("Status").borders(Borders::ALL);
    let status = Paragraph::new(status_text)
        .block(status_block)
        .wrap(Wrap { trim: true });
    f.render_widget(status, area);
}

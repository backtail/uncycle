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
        let mut state = app.midi_state.lock().unwrap();
        status_text = vec![
            Line::from(format!("BPM: {}", state.get_bpm())),
            Line::from(format!("Step: {}", state.get_step_number() + 1)),
        ];
    }

    let status_block = Block::default().title("Status").borders(Borders::ALL);
    let status = Paragraph::new(status_text)
        .block(status_block)
        .wrap(Wrap { trim: true });
    f.render_widget(status, area);
}

use crate::app;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use app::App;

pub fn render_main_tab(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // Status
            Constraint::Min(5),    // Controls
        ])
        .split(area);

    // Status block
    let midi_state = app.midi_state.lock().unwrap();
    let status_text = vec![
        Line::from(vec![
            // Span::raw("Status: "),
            Span::styled(
                app.state.status(),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(format!("Loop count: {}", app.state.loop_count)),
        Line::from(format!("MIDI: {}", midi_state.get_note_display())),
        Line::from(format!("Total notes: {}", midi_state.note_count)),
    ];

    let status_block = Block::default().title("Status").borders(Borders::ALL);
    let status = Paragraph::new(status_text)
        .block(status_block)
        .wrap(Wrap { trim: true });
    f.render_widget(status, chunks[0]);
}

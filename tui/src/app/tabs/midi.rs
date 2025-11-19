use crate::app;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use app::App;

pub fn render_midi_tab(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // Current status
            Constraint::Min(5),    // Message log
        ])
        .split(area);

    let midi_state = app.midi_state.lock().unwrap();

    // Current MIDI status
    let status_text = vec![
        Line::from(format!("Last note: {}", midi_state.get_note_display())),
        Line::from(format!("Total notes received: {}", midi_state.note_count)),
    ];

    let status_block = Block::default().title("MIDI Status").borders(Borders::ALL);
    let status = Paragraph::new(status_text)
        .block(status_block)
        .wrap(Wrap { trim: true });
    f.render_widget(status, chunks[0]);

    // Message log
    let log_items: Vec<ListItem> = midi_state
        .message_log
        .iter()
        .rev() // Show newest first
        .take(50) // Limit to 50 items
        .map(|msg| ListItem::new(Line::from(msg.as_str())))
        .collect();

    let log_block = Block::default()
        .title("MIDI Message Log")
        .borders(Borders::ALL);
    let log = List::new(log_items)
        .block(log_block)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow));
    f.render_widget(log, chunks[1]);
}

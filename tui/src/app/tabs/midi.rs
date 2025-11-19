use crate::{app, midi};

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use std::time::Duration;

use app::App;
use midi::MidiState;

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
    let active_notes_count = midi_state.active_notes.iter().filter(|n| n.active).count();
    let status_text = vec![
        Line::from(format!("Active notes: {}", active_notes_count)),
        Line::from(format!("Last note: {}", midi_state.get_note_display())),
        Line::from(format!("Total notes received: {}", midi_state.note_count)),
        Line::from(""),
        Line::from("Recent active notes:"),
    ];

    let recent_notes: Vec<Line> = midi_state
        .active_notes
        .iter()
        .filter(|n| n.active && n.timestamp.elapsed() < Duration::from_secs(2))
        .take(10)
        .map(|n| {
            let note_name = MidiState::get_note_name(n.note);
            Line::from(format!(
                "  ♪ {} ({}), vel: {}",
                note_name, n.note, n.velocity
            ))
        })
        .collect();

    let mut all_status_text = status_text;
    all_status_text.extend(recent_notes);

    let status_block = Block::default().title("MIDI Status").borders(Borders::ALL);
    let status = Paragraph::new(all_status_text)
        .block(status_block)
        .wrap(Wrap { trim: true });
    f.render_widget(status, chunks[0]);

    // Message log
    let log_items: Vec<ListItem> = midi_state
        .message_log
        .iter()
        .rev() // Show newest first
        .take(20) // Limit to 20 items
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

use crate::app;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use app::App;

pub fn render_midi_tab(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2); 3])
        .split(area);

    let logger = app.midi_logger.lock().unwrap();

    // Message log
    let incoming_notes: Vec<ListItem> = logger
        .in_note_log
        .iter()
        .rev() // Show newest first
        .take(200) // Limit to 200 items
        .map(|msg| ListItem::new(Line::from(msg.as_str())))
        .collect();

    let incoming_cc: Vec<ListItem> = logger
        .in_cc_log
        .iter()
        .rev() // Show newest first
        .take(200) // Limit to 200 items
        .map(|msg| ListItem::new(Line::from(msg.as_str())))
        .collect();

    let misc: Vec<ListItem> = logger
        .in_other_log
        .iter()
        .rev() // Show newest first
        .take(200) // Limit to 200 items
        .map(|msg| ListItem::new(Line::from(msg.as_str())))
        .collect();

    let log_block = Block::default().title("Note Log").borders(Borders::ALL);
    let log = List::new(incoming_notes)
        .block(log_block)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow));
    f.render_widget(log, chunks[0]);

    let log_block = Block::default().title("CC Log").borders(Borders::ALL);
    let log = List::new(incoming_cc)
        .block(log_block)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow));
    f.render_widget(log, chunks[1]);

    let log_block = Block::default().title("Misc").borders(Borders::ALL);
    let log = List::new(misc)
        .block(log_block)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow));
    f.render_widget(log, chunks[2]);
}

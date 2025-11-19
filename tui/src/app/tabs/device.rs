use crate::app;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::Stylize,
    style::{Color, Style},
    symbols::{self},
    widgets::{Block, Paragraph},
    Frame,
};

use app::App;

const STEPS: usize = 16;
const STEP_NAME: [&'static str; STEPS] = [
    "BD", "SD", "LT", "MT", "HT", "RS", "HC", "CH", "OH", "CC", "RC", "", "", "", "", "",
];

pub fn render_device_tab(f: &mut Frame, _app: &App, area: Rect) {
    let ratio = STEPS as u32;

    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Ratio(1, 3),
            // Constraint::Ratio(1, 3),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Ratio(1, 3),
        ])
        .split(area);

    display_div_line::<6>(f, vert[2]);
    display_div_line::<3>(f, vert[3]);
    display_div_line::<4>(f, vert[4]);
    display_div_line::<2>(f, vert[5]);

    let steps = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, ratio); STEPS])
        .split(vert[6]);

    for i in 0..steps.len() {
        let c;
        match i as u16 {
            0..=3 => c = Color::Red,
            4..=7 => c = Color::Yellow,
            8..=11 => c = Color::LightYellow,
            12..=u16::MAX => c = Color::White,
        }
        let b;
        if i % 2 == 0 {
            b = active_step(STEP_NAME[i], c);
        } else {
            b = unactive_step(STEP_NAME[i], c);
        }

        let p = Paragraph::default().block(b);
        f.render_widget(p, steps[i]);
    }
}

fn active_step<'a>(title: &'a str, c: Color) -> Block<'a> {
    Block::bordered()
        .title_bottom(title)
        .title_alignment(Alignment::Center)
        .border_set(symbols::border::QUADRANT_OUTSIDE)
        .border_style(Style::reset().fg(c).reversed())
        .style(Style::default().fg(c).bg(c))
}

fn unactive_step<'a>(title: &'a str, c: Color) -> Block<'a> {
    Block::bordered()
        .title_bottom(title)
        .title_alignment(Alignment::Center)
        .border_set(symbols::border::QUADRANT_OUTSIDE)
        .border_style(Style::reset().fg(c).reversed())
        .style(Style::default().fg(c))
}

fn white_line<'a>() -> Block<'a> {
    Block::bordered()
        .border_set(symbols::border::QUADRANT_OUTSIDE)
        .border_style(Style::reset().fg(Color::White).reversed())
        .style(Style::default().bg(Color::White))
}

fn display_div_line<const DIV: u32>(f: &mut Frame, area: Rect) {
    let lines = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Ratio(1, DIV); DIV as usize])
        .split(area);

    for i in 0..lines.len() {
        f.render_widget(white_line(), lines[i]);
    }
}

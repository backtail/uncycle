use ratatui::{
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, BorderType, Tabs, Widget},
};

use tui_big_text::{BigText, PixelSize};

/// Provides custom tabs
pub fn ui_tabs(n: usize) -> impl Widget {
    Tabs::new(vec![
        "[1] Main",
        "[2] Device",
        "[3] MIDI Monitor",
        "[4] Settings",
        "[5] Help",
    ])
    .select(n)
    .padding(" ", " ")
    .block(Block::default())
    .style(Style::default().fg(Color::DarkGray))
    .highlight_style(
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    )
}

pub fn border_rounded<'a>(title: &'a str, c: Color) -> impl Widget + 'a {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .style(c)
        .title(title)
        .title_style(c)
}

pub fn main_text<'a>(lines: Vec<Line<'a>>) -> impl Widget + 'a {
    BigText::builder()
        .pixel_size(PixelSize::Quadrant)
        .lines(lines)
        .centered()
        .build()
}

use ratatui::{
    style::Color,
    text::Line,
    widgets::{Block, BorderType, Widget},
};

use tui_big_text::{BigText, PixelSize};

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

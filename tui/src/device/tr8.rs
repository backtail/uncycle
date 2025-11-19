use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::Stylize,
    style::{Color, Style},
    symbols::{self},
    widgets::{
        canvas::{self, Canvas},
        Block, Widget,
    },
    Frame,
};

use crate::app::App;

const TR_8_INTRUMENTS: usize = 11;

const TR_8_BD_NOTE: u8 = 36;
const TR_8_SD_NOTE: u8 = 38;
const TR_8_LT_NOTE: u8 = 43;
const TR_8_MT_NOTE: u8 = 47;
const TR_8_HT_NOTE: u8 = 50;
const TR_8_RS_NOTE: u8 = 37;
const TR_8_HC_NOTE: u8 = 39;
const TR_8_CH_NOTE: u8 = 42;
const TR_8_OH_NOTE: u8 = 46;
const TR_8_CC_NOTE: u8 = 49;
const TR_8_RC_NOTE: u8 = 51;

const TR_8_BD2_NOTE: u8 = 35;
const TR_8_SD2_NOTE: u8 = 40;
const TR_8_CB_NOTE: u8 = 56;
const TR_8_TB_NOTE: u8 = 54;

const TR_8_STEPS: usize = TR_8_INTRUMENTS + 5;
const TR_8_STEP_NAME: [&'static str; TR_8_STEPS] = [
    "BD", "SD", "LT", "MT", "HT", "RS", "HC", "CH", "OH", "CC", "RC", "", "", "", "", "",
];

const TR_8_NOTES: [u8; TR_8_STEPS] = [
    TR_8_BD_NOTE,
    TR_8_SD_NOTE,
    TR_8_LT_NOTE,
    TR_8_MT_NOTE,
    TR_8_HT_NOTE,
    TR_8_RS_NOTE,
    TR_8_HC_NOTE,
    TR_8_CH_NOTE,
    TR_8_OH_NOTE,
    TR_8_CC_NOTE,
    TR_8_RC_NOTE,
    TR_8_BD2_NOTE,
    TR_8_SD2_NOTE,
    TR_8_CB_NOTE,
    TR_8_TB_NOTE,
    0,
];

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    // MIDI data extraction
    ///////////////////////

    let mut current_active_steps: [bool; TR_8_STEPS] = [false; TR_8_STEPS];
    let current_volume: [u8; TR_8_INTRUMENTS] = [0_u8; TR_8_INTRUMENTS];

    {
        let mut midi_state = app.midi_state.lock().unwrap();

        for i in 0..TR_8_STEPS {
            current_active_steps[i] = midi_state.find_active_note(TR_8_NOTES[i])
        }
    }

    // Vertical Rendering
    /////////////////////

    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            // volume
            Constraint::Ratio(1, 3),
            // placeholder
            Constraint::Length(2),
            // scale lines
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            //steps
            Constraint::Ratio(1, 3),
        ])
        .split(area);

    render_instruments(f, vert[0]);
    render_lines::<6>(f, vert[2]);
    render_lines::<3>(f, vert[3]);
    render_lines::<4>(f, vert[4]);
    render_lines::<2>(f, vert[5]);
    render_steps(f, vert[6], &current_active_steps);
}

// Horizontal rendering
///////////////////////

fn render_instruments(f: &mut Frame, area: Rect) {
    let steps = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, TR_8_INTRUMENTS as u32); TR_8_INTRUMENTS])
        .split(area);

    for i in 0..steps.len() {
        f.render_widget(fader(), steps[i]);
    }
}

fn render_lines<const DIV: u32>(f: &mut Frame, area: Rect) {
    let lines = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Ratio(1, DIV); DIV as usize])
        .split(area);

    for i in 0..lines.len() {
        f.render_widget(white_line(), lines[i]);
    }
}

fn render_steps(f: &mut Frame, area: Rect, is_active: &[bool; TR_8_STEPS]) {
    let steps = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, TR_8_STEPS as u32); TR_8_STEPS])
        .split(area);

    for i in 0..TR_8_STEPS {
        f.render_widget(tr8_step(TR_8_STEP_NAME[i], i, is_active[i]), steps[i]);
    }
}

// Helper functions
///////////////////

fn fader() -> impl Widget {
    Canvas::default()
        .paint(|ctx| {
            ctx.draw(&canvas::Line::new(1.0, 0.0, 1.0, 1.0, Color::White));
            ctx.draw(&canvas::Line::new(0.5, 0.5, 1.5, 0.5, Color::White));
        })
        .marker(symbols::Marker::Block)
        .x_bounds([0.0, 2.0])
        .y_bounds([0.0, 1.0])
}

fn tr8_step<'a>(title: &'a str, step: usize, is_active: bool) -> impl Widget + 'a {
    let c;
    match step as u16 {
        0..=3 => c = Color::Red,
        4..=7 => c = Color::Yellow,
        8..=11 => c = Color::LightYellow,
        12..=u16::MAX => c = Color::White,
    }

    let b = Block::bordered()
        .title_bottom(title)
        .title_alignment(Alignment::Center)
        .border_set(symbols::border::QUADRANT_OUTSIDE)
        .border_style(Style::reset().fg(c).reversed())
        .style(Style::default().fg(c));

    if is_active {
        return b.bg(c);
    } else {
        return b;
    }
}

fn white_line<'a>() -> Block<'a> {
    Block::bordered()
        .border_set(symbols::border::QUADRANT_OUTSIDE)
        .border_style(Style::reset().fg(Color::White).reversed())
        .style(Style::default().bg(Color::White))
}

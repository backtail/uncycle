use std::{
    f64::consts::{FRAC_PI_3, PI},
    str::FromStr,
};

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    prelude::Stylize,
    style::{Color, Style},
    symbols::{self},
    widgets::{
        block::Position,
        canvas::{self, Canvas},
        Block, Widget,
    },
    Frame,
};

use crate::app::App;

/// (number: u8, name: &'static str)
type RichMidiCC = (u8, &'static str);

const TR_8_INTRUMENTS: usize = 11;
const TR_8_STEPS: usize = TR_8_INTRUMENTS + 5;
const TR_8_PARAM_ELEMS: usize = TR_8_INTRUMENTS + 2;

// relevant notes to check

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
    //
    // | these might be unsupported if 727 update has not been flashed on device
    // v
    TR_8_BD2_NOTE,
    TR_8_SD2_NOTE,
    TR_8_CB_NOTE,
    TR_8_TB_NOTE,
    0,
];

// relevant CC numbers to check

const TR_8_CC_FADER: [RichMidiCC; TR_8_INTRUMENTS] = [
    (24, "BD"),
    (29, "SD"),
    (48, "LT"),
    (51, "MT"),
    (54, "HT"),
    (57, "RS"),
    (60, "HC"),
    (63, "CH"),
    (82, "OH"),
    (85, "CC"),
    (88, "RC"),
];

const KNOB_TURN_RADIANS: f64 = -5.0 * FRAC_PI_3;
const KNOB_TURN_OFFSET: f64 = PI + FRAC_PI_3;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    // MIDI data extraction
    ///////////////////////

    let mut current_active_steps: [bool; TR_8_STEPS] = [false; TR_8_STEPS];
    let mut current_volume: [u8; TR_8_INTRUMENTS] = [0_u8; TR_8_INTRUMENTS];

    {
        let mut midi_state = app.midi_state.lock().unwrap();

        for i in 0..TR_8_STEPS {
            current_active_steps[i] = midi_state.find_active_note(TR_8_NOTES[i])
        }

        for i in 0..TR_8_INTRUMENTS {
            current_volume[i] = midi_state.get_cc_val_of(TR_8_CC_FADER[i].0);
        }
    }

    // Vertical Rendering
    /////////////////////

    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            // volume
            Constraint::Length(20),
            // placeholder
            Constraint::Length(1),
            // scale lines
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            //steps
            Constraint::Length(10),
            // placeholder
            Constraint::Fill(1),
        ])
        .margin(6)
        .split(area);

    f.render_widget(
        Block::bordered()
            .green()
            .border_set(symbols::border::QUADRANT_OUTSIDE),
        area.inner(Margin::new(2, 2)),
    );
    render_instruments(f, vert[0], &current_volume);
    render_lines::<6>(f, vert[2]);
    render_lines::<3>(f, vert[3]);
    render_lines::<4>(f, vert[4]);
    render_lines::<2>(f, vert[5]);
    render_steps(f, vert[6], &current_active_steps);
}

// Horizontal rendering
///////////////////////

fn render_instruments(f: &mut Frame, area: Rect, volume: &[u8; TR_8_INTRUMENTS]) {
    // Split horizonzally
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(50),
        ])
        .split(area);

    let first_row_knobs = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, TR_8_PARAM_ELEMS as u32); TR_8_PARAM_ELEMS])
        .split(vert[0]);

    let second_row_knobs = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, TR_8_PARAM_ELEMS as u32); TR_8_PARAM_ELEMS])
        .split(vert[1]);

    let steps = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(2, TR_8_PARAM_ELEMS as u32), // BD has more area
            Constraint::Ratio(2, TR_8_PARAM_ELEMS as u32), // SD has more area
            Constraint::Ratio(1, TR_8_PARAM_ELEMS as u32),
            Constraint::Ratio(1, TR_8_PARAM_ELEMS as u32),
            Constraint::Ratio(1, TR_8_PARAM_ELEMS as u32),
            Constraint::Ratio(1, TR_8_PARAM_ELEMS as u32),
            Constraint::Ratio(1, TR_8_PARAM_ELEMS as u32),
            Constraint::Ratio(1, TR_8_PARAM_ELEMS as u32),
            Constraint::Ratio(1, TR_8_PARAM_ELEMS as u32),
            Constraint::Ratio(1, TR_8_PARAM_ELEMS as u32),
            Constraint::Ratio(1, TR_8_PARAM_ELEMS as u32),
        ])
        .split(vert[2]);

    for i in 0..first_row_knobs.len() {
        let wrap = i as f64 / first_row_knobs.len() as f64;
        f.render_widget(knob(wrap), first_row_knobs[i]);
    }

    for i in 0..second_row_knobs.len() {
        f.render_widget(knob(0.0), second_row_knobs[i]);
    }

    for i in 0..steps.len() {
        let pos = volume[i] as f64 / 127.0_f64;
        f.render_widget(fader(TR_8_CC_FADER[i].1, pos), steps[i]);
    }
}

fn render_lines<const DIV: u32>(f: &mut Frame, area: Rect) {
    let lines = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Ratio(1, DIV); DIV as usize])
        .split(area);

    for i in 0..lines.len() {
        f.render_widget(
            Block::bordered()
                .border_set(symbols::border::QUADRANT_OUTSIDE)
                .border_style(Style::reset().fg(Color::DarkGray).reversed()),
            lines[i],
        );
    }
}

fn render_steps(f: &mut Frame, area: Rect, is_active: &[bool; TR_8_STEPS]) {
    let steps = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, TR_8_STEPS as u32); TR_8_STEPS])
        .split(area);

    for i in 0..TR_8_STEPS {
        f.render_widget(tr8_step(i, is_active[i]), steps[i]);
    }
}

// Helper functions
///////////////////

fn knob(pos: f64) -> impl Widget {
    Canvas::default()
        .block(
            Block::new()
                .title("TEST")
                .title_alignment(Alignment::Center)
                .title_position(Position::Bottom),
        )
        .paint(move |ctx| {
            ctx.draw(&canvas::Circle {
                x: 0.0,
                y: 0.0,
                radius: 1.0,
                color: Color::White,
            });
            ctx.draw(&canvas::Line::new(
                0.0,
                0.0,
                (pos * KNOB_TURN_RADIANS + KNOB_TURN_OFFSET).cos(),
                (pos * KNOB_TURN_RADIANS + KNOB_TURN_OFFSET).sin(),
                Color::White,
            ));
        })
        .marker(symbols::Marker::Braille)
        .x_bounds([-2.0, 2.0])
        .y_bounds([-2.0, 2.0])
}

fn fader<'a>(title: &'a str, pos: f64) -> impl Widget + 'a {
    Canvas::default()
        .block(
            Block::new()
                .title(title)
                .title_alignment(Alignment::Center)
                .title_position(Position::Bottom),
        )
        .paint(move |ctx| {
            ctx.draw(&canvas::Line::new(0.0, 0.0, 0.0, 1.0, Color::Green));
            ctx.draw(&canvas::Line::new(-0.5, pos, 0.5, pos, Color::White));
        })
        .marker(symbols::Marker::Braille)
        .x_bounds([-1.0, 1.0])
        .y_bounds([-0.1, 1.1])
}

fn tr8_step(step: usize, is_active: bool) -> impl Widget {
    let c;
    match step as u16 {
        0..=3 => c = Color::Red,
        4..=7 => c = Color::from_str("#ff8800ff").unwrap_or(Color::Yellow),
        8..=11 => c = Color::LightYellow,
        12..=u16::MAX => c = Color::White,
    }

    let b = Block::bordered()
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

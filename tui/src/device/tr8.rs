use core::f64;
use std::{
    f64::consts::{FRAC_PI_3, PI},
    str::FromStr,
    usize,
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

const TR_8_NOTES: [u8; TR_8_STEPS] = [
    36, // BD
    38, // SD
    43, // LT
    47, // MT
    50, // HT
    37, // RS
    39, // HC
    42, // CH
    46, // OH
    49, // CC
    51, // RC
    //
    // | these might be unsupported if 727 update has not been flashed on device
    // v
    35, // BD2
    40, // SD2
    56, // CB
    54, // TB
    0,  // unused
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

const TR_8_CC_PARAMS_1ST_ROW: [RichMidiCC; TR_8_PARAM_ELEMS] = [
    (20, "TUNE"),   // BD
    (21, "ATTACK"), // BD
    (25, "TUNE"),   // SD
    (26, "SNAPPY"), // SD
    (46, "TUNE"),   // LT
    (49, "TUNE"),   // MT
    (52, "TUNE"),   // HT
    (55, "TUNE"),   // RS
    (58, "TUNE"),   // HC
    (61, "TUNE"),   // CH
    (80, "TUNE"),   // OH
    (83, "TUNE"),   // CC
    (86, "TUNE"),   // RC
];

const TR_8_CC_PARAMS_2ND_ROW: [RichMidiCC; TR_8_PARAM_ELEMS] = [
    (22, "COMP"),  // BD
    (23, "DECAY"), // BD
    (27, "COMP"),  // SD
    (28, "DECAY"), // SD
    (47, "DECAY"), // LT
    (50, "DECAY"), // MT
    (53, "DECAY"), // HT
    (56, "DECAY"), // RS
    (59, "DECAY"), // HC
    (62, "DECAY"), // CH
    (81, "DECAY"), // OH
    (84, "DECAY"), // CC
    (87, "DECAY"), // RC
];

const KNOB_TURN_RADIANS: f64 = -5.0 * FRAC_PI_3;
const KNOB_TURN_OFFSET: f64 = PI + FRAC_PI_3;
const GOLDEN_RATIO: f64 = 1.61803398875;

const BG_COLOR: Color = Color::Black;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    // MIDI data extraction
    ///////////////////////

    let mut current_active_steps: [bool; TR_8_STEPS] = [false; TR_8_STEPS];
    let mut current_volume: [u8; TR_8_INTRUMENTS] = [0_u8; TR_8_INTRUMENTS];
    let mut current_param_1st: [u8; TR_8_PARAM_ELEMS] = [0_u8; TR_8_PARAM_ELEMS];
    let mut current_param_2nd: [u8; TR_8_PARAM_ELEMS] = [0_u8; TR_8_PARAM_ELEMS];

    {
        let mut midi_state = app.midi_state.lock().unwrap();

        for i in 0..(TR_8_STEPS - 1) {
            match i {
                0..TR_8_INTRUMENTS => {
                    current_volume[i] = midi_state.get_cc_val_of(TR_8_CC_FADER[i].0);
                    current_active_steps[i] = midi_state.find_active_note(TR_8_NOTES[i]);
                    current_param_1st[i] = midi_state.get_cc_val_of(TR_8_CC_PARAMS_1ST_ROW[i].0);
                    current_param_2nd[i] = midi_state.get_cc_val_of(TR_8_CC_PARAMS_2ND_ROW[i].0);
                }
                TR_8_INTRUMENTS..TR_8_PARAM_ELEMS => {
                    current_active_steps[i] = midi_state.find_active_note(TR_8_NOTES[i]);
                    current_param_1st[i] = midi_state.get_cc_val_of(TR_8_CC_PARAMS_1ST_ROW[i].0);
                    current_param_2nd[i] = midi_state.get_cc_val_of(TR_8_CC_PARAMS_2ND_ROW[i].0);
                }
                TR_8_PARAM_ELEMS..TR_8_STEPS => {
                    current_active_steps[i] = midi_state.find_active_note(TR_8_NOTES[i]);
                }
                _ => {}
            }
        }
    }

    // Vertical Rendering
    /////////////////////

    let h = area.height;
    let w = ((2 * h) as f64 * GOLDEN_RATIO) as u16; // characters are themselves have a ratio of 2:1

    let tr8 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Fill(1), Constraint::Min(w), Constraint::Fill(1)])
        .split(
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Fill(1), Constraint::Min(h), Constraint::Fill(1)])
                .split(area)[1],
        )[1];

    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            // volume
            Constraint::Fill(1),
            // placeholder
            // Constraint::Max(2),
            // scale lines
            Constraint::Max(1),
            Constraint::Max(1),
            Constraint::Max(1),
            Constraint::Max(1),
            //steps
            Constraint::Percentage(20),
        ])
        .margin(2)
        .split(tr8);

    // Border outline
    f.render_widget(
        Block::bordered()
            .green()
            .border_set(symbols::border::QUADRANT_OUTSIDE)
            .bg(BG_COLOR),
        area.clamp(tr8).inner(Margin::new(0, 0)),
    );

    render_instruments(
        f,
        vert[0],
        &current_volume,
        &current_param_1st,
        &current_param_2nd,
    );
    render_lines::<6>(f, vert[1]);
    render_lines::<3>(f, vert[2]);
    render_lines::<4>(f, vert[3]);
    render_lines::<2>(f, vert[4]);
    render_steps(f, vert[5], &current_active_steps);
}

// Horizontal rendering
///////////////////////

fn render_instruments(
    f: &mut Frame,
    area: Rect,
    volume: &[u8; TR_8_INTRUMENTS],
    param_1st: &[u8; TR_8_PARAM_ELEMS],
    param_2nd: &[u8; TR_8_PARAM_ELEMS],
) {
    // Split horizonzally
    let row = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(23),
            Constraint::Percentage(23),
            Constraint::Percentage(4),
            Constraint::Percentage(46),
            Constraint::Percentage(4),
        ])
        .split(area);

    let knobs_1st_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, TR_8_PARAM_ELEMS as u32); TR_8_PARAM_ELEMS])
        .split(row[0]);

    let knobs_2nd_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, TR_8_PARAM_ELEMS as u32); TR_8_PARAM_ELEMS])
        .split(row[1]);

    let faders = Layout::default()
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
        .split(row[3]);

    for i in 0..TR_8_PARAM_ELEMS {
        // let wrap = i as f64 / knobs_1st_row.len() as f64;
        let pos_1st_row = param_1st[i] as f64 / 127.0_f64;
        let pos_2nd_row = param_2nd[i] as f64 / 127.0_f64;
        f.render_widget(
            knob(TR_8_CC_PARAMS_1ST_ROW[i].1, pos_1st_row),
            knobs_1st_row[i],
        );
        f.render_widget(
            knob(TR_8_CC_PARAMS_2ND_ROW[i].1, pos_2nd_row),
            knobs_2nd_row[i],
        );
    }

    for i in 0..faders.len() {
        let pos = volume[i] as f64 / 127.0_f64;
        f.render_widget(fader(TR_8_CC_FADER[i].1, pos), faders[i]);
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
                .border_style(Style::reset().fg(Color::DarkGray).bg(BG_COLOR).reversed()),
            lines[i].inner(Margin {
                horizontal: 1,
                vertical: 0,
            }),
        );
    }
}

fn render_steps(f: &mut Frame, area: Rect, is_active: &[bool; TR_8_STEPS]) {
    let steps = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, TR_8_STEPS as u32); TR_8_STEPS])
        .split(area);

    for i in 0..TR_8_STEPS {
        f.render_widget(tr8_step(i, is_active[i]), steps[i].inner(Margin::new(1, 1)));
    }
}

// Helper functions
///////////////////

fn knob<'a>(title: &'a str, pos: f64) -> impl Widget + 'a {
    Canvas::default()
        .block(
            Block::new()
                .title(title)
                .title_alignment(Alignment::Center)
                .title_position(Position::Bottom)
                .gray(),
        )
        .paint(move |ctx| {
            ctx.draw(&canvas::Circle {
                x: 0.0,
                y: 0.0,
                radius: 1.0,
                color: Color::DarkGray,
            });
            ctx.draw(&canvas::Line::new(
                (pos * KNOB_TURN_RADIANS + KNOB_TURN_OFFSET).cos() * 0.4,
                (pos * KNOB_TURN_RADIANS + KNOB_TURN_OFFSET).sin() * 0.4,
                (pos * KNOB_TURN_RADIANS + KNOB_TURN_OFFSET).cos(),
                (pos * KNOB_TURN_RADIANS + KNOB_TURN_OFFSET).sin(),
                Color::White,
            ));
        })
        .marker(symbols::Marker::HalfBlock)
        .x_bounds([-2.0, 2.0])
        .y_bounds([-2.0, 2.0])
        .background_color(BG_COLOR)
}

fn fader<'a>(title: &'a str, pos: f64) -> impl Widget + 'a {
    Canvas::default()
        .block(
            Block::new()
                .title_bottom(title)
                .title_alignment(Alignment::Center)
                .gray(),
        )
        .paint(move |ctx| {
            ctx.draw(&canvas::Line::new(0.0, 0.0, 0.0, 1.0, Color::Green));
            ctx.draw(&canvas::Line::new(-0.5, pos, 0.5, pos, Color::White));
        })
        .marker(symbols::Marker::HalfBlock)
        .x_bounds([-1.0, 1.0])
        .y_bounds([-0.1, 1.1])
        .background_color(BG_COLOR)
}

fn tr8_step(step: usize, is_active: bool) -> impl Widget {
    let c;
    match step as u16 {
        0..=3 => c = Color::from_str("#ff0000").unwrap_or(Color::Red),
        4..=7 => c = Color::from_str("#FF5C00").unwrap_or(Color::Yellow),
        8..=11 => c = Color::from_str("#ffcc00ff").unwrap_or(Color::LightYellow),
        12..=u16::MAX => c = Color::White,
    }

    let b = Block::bordered()
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::reset().fg(c).bg(BG_COLOR))
        .style(Style::default().fg(c));

    if !is_active {
        return b.bg(c);
    } else {
        return b;
    }
}

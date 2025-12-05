use crate::app;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Stylize},
    text::ToLine,
    Frame,
};

use app::App;

pub fn render_main_tab(f: &mut Frame, app: &App, area: Rect) {
    let current_step;
    let loop_steps;
    let bpm;
    let is_running;
    {
        let mut locked = app.core.lock().unwrap();

        current_step = locked.get_step_number() + 1;
        loop_steps = locked.looper.loop_steps;
        bpm = locked.get_bpm();
        is_running = locked.is_running();
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Fill(1)])
        .split(area);

    let higher = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Fill(1), Constraint::Fill(1)])
        .split(chunks[0]);

    let lower = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Fill(1), Constraint::Fill(1)])
        .split(chunks[1]);

    let record_state = app.core.lock().unwrap().looper.record;
    let overdub_state = app.core.lock().unwrap().looper.overdub;

    let rec_text;

    if record_state {
        rec_text = "Recording".to_line().red();
    } else if overdub_state {
        rec_text = "Overdubbing".to_line().red();
    } else {
        rec_text = "".to_line();
    }

    let running_text;

    if is_running {
        running_text = "Running".to_line().green();
    } else {
        running_text = "Stopped".to_line().white();
    }

    let bpm_text = vec!["BPM".to_line().dark_gray(), bpm.to_line().magenta()];

    let recording_text = vec!["Status".to_line().dark_gray(), running_text, rec_text];

    let current_step_text = vec![
        "TR-8".to_line().dark_gray(),
        "step".to_line().dark_gray(),
        current_step.to_line().green(),
    ];

    let rec_loop_text = vec![
        "loop".to_line().dark_gray(),
        "steps".to_line().dark_gray(),
        loop_steps.to_line().red(),
    ];

    f.render_widget(app::widgets::border_rounded("", Color::DarkGray), higher[0]);
    f.render_widget(app::widgets::main_text(bpm_text), higher[0]);

    f.render_widget(app::widgets::border_rounded("", Color::Gray), higher[1]);
    f.render_widget(app::widgets::main_text(recording_text), higher[1]);

    f.render_widget(app::widgets::border_rounded("", Color::Gray), lower[0]);
    f.render_widget(app::widgets::main_text(current_step_text), lower[0]);

    f.render_widget(app::widgets::border_rounded("", Color::Gray), lower[1]);
    f.render_widget(app::widgets::main_text(rec_loop_text), lower[1]);
}

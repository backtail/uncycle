use crate::app::widgets;

use super::{connection::setup_midi_socket, keybindings, log::Logger, tabs::*};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Margin},
    Frame, Terminal,
};

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use keybindings::{Action, Keybindings};

use uncycle_core::prelude::*;

#[derive(PartialEq)]
pub enum AppTab {
    Main = 1,
    Device = 2,
    Midi = 3,
    Settings = 4,
    Help = 5,
}

pub struct App {
    pub keybindings: Keybindings,
    pub core: Arc<Mutex<UncycleCore>>,
    pub log: Arc<Mutex<Logger>>,
    pub current_tab: AppTab,
    should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            keybindings: Keybindings::new(),
            core: Arc::new(Mutex::new(UncycleCore::new())),
            log: Arc::new(Mutex::new(Logger::new())),
            current_tab: AppTab::Main,
            should_quit: false,
        }
    }

    fn on_key(&mut self, key: KeyCode) {
        if let Some(action) = self.keybindings.find_action(key) {
            match action {
                Action::Quit => self.should_quit = true,
                Action::IncreaseBPM => self.core.lock().unwrap().increase_bpm_by(1.0),
                Action::DecreaseBPM => self.core.lock().unwrap().decrease_bpm_by(1.0),
                Action::CycleTabs => self.cycle_tabs(),
                Action::RevCycleTabs => self.rev_cycle_tabs(),
                Action::ToggleSequence => self.core.lock().unwrap().start_stop_sequence(),
                Action::StartRecording => self.core.lock().unwrap().start_recording(),
                Action::DeleteRecording => self.core.lock().unwrap().delete_recording(),
                Action::HalfLoopLen => self.core.lock().unwrap().half_loop_len(),
                Action::DoubleLoopLen => self.core.lock().unwrap().double_loop_len(),
            }
        } else {
            // Handle tab switching
            match key {
                KeyCode::Char('1') => self.current_tab = AppTab::Main,
                KeyCode::Char('2') => self.current_tab = AppTab::Device,
                KeyCode::Char('3') => self.current_tab = AppTab::Midi,
                KeyCode::Char('4') => self.current_tab = AppTab::Settings,
                KeyCode::Char('5') => self.current_tab = AppTab::Help,
                _ => {}
            }
        }
    }

    fn cycle_tabs(&mut self) {
        match self.current_tab {
            AppTab::Main => self.current_tab = AppTab::Device,
            AppTab::Device => self.current_tab = AppTab::Midi,
            AppTab::Midi => self.current_tab = AppTab::Settings,
            AppTab::Settings => self.current_tab = AppTab::Help,
            AppTab::Help => self.current_tab = AppTab::Main,
        }
    }

    fn rev_cycle_tabs(&mut self) {
        match self.current_tab {
            AppTab::Main => self.current_tab = AppTab::Help,
            AppTab::Device => self.current_tab = AppTab::Main,
            AppTab::Midi => self.current_tab = AppTab::Device,
            AppTab::Settings => self.current_tab = AppTab::Midi,
            AppTab::Help => self.current_tab = AppTab::Settings,
        }
    }
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    setup_midi_socket(app.core.clone(), app.log.clone());

    while !app.should_quit {
        terminal.draw(|f| ui(f, app))?;

        // Handle events with timeout to check for MIDI redraw signals (~60Hz)
        if crossterm::event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.on_key(key.code);
                }
            }
        }
    }

    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    // Layout

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(2), // Tabs
            Constraint::Min(10),   // Main content
        ])
        .split(f.area());

    let first_row = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Ratio(1, 2); 2])
        .split(chunks[0]);

    let main_area = chunks[1].inner(Margin {
        horizontal: 1,
        vertical: 0,
    });

    // Rendering

    match app.current_tab {
        AppTab::Main => {
            f.render_widget(widgets::ui_tabs(0), first_row[0]);
            render_main_tab(f, app, main_area);
        }
        AppTab::Device => {
            f.render_widget(widgets::ui_tabs(1), first_row[0]);
            render_device_tab(f, app, main_area);
        }
        AppTab::Midi => {
            f.render_widget(widgets::ui_tabs(2), first_row[0]);
            render_midi_tab(f, app, main_area);
        }
        AppTab::Settings => {
            f.render_widget(widgets::ui_tabs(3), first_row[0]);
            render_settings_tab(f, app, main_area);
        }
        AppTab::Help => {
            f.render_widget(widgets::ui_tabs(4), first_row[0]);
            render_help_tab(f, app, main_area);
        }
    }
}

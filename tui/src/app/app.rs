use super::{connection::setup_midi_socket, keybindings, log::Logger, tabs::*};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    widgets::{Block, Tabs},
    Frame, Terminal,
};

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use keybindings::{Action, Keybindings};

use uncycle_core::UncycleCore;

#[derive(PartialEq)]
enum AppTab {
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
    current_tab: AppTab,
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
                Action::KillConnection => {
                    let mut locked = self.core.lock().unwrap();
                    locked.kill_rx_conn = true;
                    locked.kill_tx_conn = true;
                }
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
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(3), // Tabs
            Constraint::Min(10),   // Main content
        ])
        .split(f.area());

    // Tabs
    let tabs = Tabs::new(vec![
        "[1] Main",
        "[2] Device",
        "[3] MIDI Monitor",
        "[4] Settings",
        "[5] Help",
    ])
    .padding(" ", " ")
    .block(Block::default())
    .select(match app.current_tab {
        AppTab::Main => 0,
        AppTab::Device => 1,
        AppTab::Midi => 2,
        AppTab::Settings => 3,
        AppTab::Help => 4,
    })
    .style(Style::default().fg(Color::DarkGray))
    .highlight_style(
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(tabs, chunks[0]);

    let main_area = chunks[1].inner(Margin {
        horizontal: 1,
        vertical: 0,
    });

    // Main content based on current tab
    match app.current_tab {
        AppTab::Main => render_main_tab(f, app, main_area),
        AppTab::Device => render_device_tab(f, app, main_area),
        AppTab::Midi => render_midi_tab(f, app, main_area),
        AppTab::Settings => render_settings_tab(f, app, main_area),
        AppTab::Help => render_help_tab(f, app, main_area),
    }
}

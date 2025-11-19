pub mod connection;
pub mod tabs;

use crate::{keybindings, midi, state};

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
    sync::{mpsc, Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use keybindings::{Action, Keybindings};
use state::UncycleState;

use connection::update_midi_clock;
use midi::MidiState;
use tabs::*;

#[derive(PartialEq)]
enum AppTab {
    Main = 1,
    Device = 2,
    Midi = 3,
    Settings = 4,
    Help = 5,
}

pub struct App {
    state: UncycleState,
    keybindings: Keybindings,
    pub midi_state: Arc<Mutex<MidiState>>,
    current_tab: AppTab,
    should_quit: bool,
}

impl App {
    pub fn new(
        state: UncycleState,
        keybindings: Keybindings,
        midi_state: Arc<Mutex<MidiState>>,
    ) -> Self {
        Self {
            state,
            keybindings,
            midi_state,
            current_tab: AppTab::Main,
            should_quit: false,
        }
    }

    fn on_key(&mut self, key: KeyCode) {
        if let Some(action) = self.keybindings.find_action(key) {
            match action {
                Action::Quit => self.should_quit = true,
                Action::StartRecording => self.state.start_recording(),
                Action::StopRecording => {
                    self.state.stop_recording();
                    self.state.stop_playback();
                }
                Action::StartPlayback => self.state.start_playback(),
                Action::TogglePlayback => self.state.toggle_playback(),
                Action::ClearLoop => self.state.loop_count = 0,
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
}

pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    redraw_rx: mpsc::Receiver<()>,
) -> Result<()> {
    let mut last_clock_check = Instant::now();
    let clock_check_interval = Duration::from_millis(1); // Check every 1ms for accuracy

    while !app.should_quit {
        // Draw UI
        terminal.draw(|f| ui(f, app))?;

        // Handle events with timeout to check for MIDI redraw signals
        if crossterm::event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.on_key(key.code);
                }
            }
        } else {
            // Check for MIDI redraw signals
            if redraw_rx.try_recv().is_ok() {
                // Force redraw on next iteration
                continue;
            }
        }

        if last_clock_check.elapsed() >= clock_check_interval {
            last_clock_check = Instant::now();
            update_midi_clock(&app.midi_state)?;
        }

        // Simulate loop progression
        if app.state.is_playing {
            app.state.loop_count += 1;
            thread::sleep(Duration::from_millis(100));
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
        vertical: 1,
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

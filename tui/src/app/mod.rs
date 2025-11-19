pub mod connection;
pub mod tabs;

use crate::{keybindings, midi, state};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame, Terminal,
};

use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};

use keybindings::{Action, Keybindings};
use state::UncycleState;

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
    while !app.should_quit {
        // Draw UI
        terminal.draw(|f| ui(f, app))?;

        // Handle events with timeout to check for MIDI redraw signals
        if crossterm::event::poll(Duration::from_millis(1))? {
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
            Constraint::Length(3), // Header
            Constraint::Length(3), // Tabs
            Constraint::Min(10),   // Main content
            Constraint::Length(3), // Footer
        ])
        .split(f.area());

    // Header
    let header = Paragraph::new("uncycle v0.1.0_alpha")
        .style(Style::default().add_modifier(Modifier::BOLD))
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    // Tabs
    let tabs = Tabs::new(vec![
        "[1] Main",
        "[2] Device",
        "[3] MIDI Monitor",
        "[4] Settings",
        "[5] Help",
    ])
    .block(Block::default().borders(Borders::ALL))
    .select(match app.current_tab {
        AppTab::Main => 0,
        AppTab::Device => 1,
        AppTab::Midi => 2,
        AppTab::Settings => 3,
        AppTab::Help => 4,
    })
    .style(Style::default().fg(Color::White))
    .highlight_style(
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(tabs, chunks[1]);

    // Main content based on current tab
    match app.current_tab {
        AppTab::Main => render_main_tab(f, app, chunks[2]),
        AppTab::Device => render_device_tab(f, app, chunks[2]),
        AppTab::Midi => render_midi_tab(f, app, chunks[2]),
        AppTab::Settings => render_settings_tab(f, app, chunks[2]),
        AppTab::Help => render_help_tab(f, app, chunks[2]),
    }

    // Footer
    let footer = Paragraph::new("Q to quit")
        .style(Style::default().fg(Color::Gray))
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(footer, chunks[3]);
}

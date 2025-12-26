use super::{connection::setup_midi_socket, keybindings, log::Logger, menu::*, tabs::*};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use keybindings::{Action, Keybindings};
use ratatui::prelude::*;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use uncycle_core::{devices::TR8, prelude::*};

const DEFAULT_BPM: f32 = 120.0;

#[derive(PartialEq, Clone)]
pub enum AppTab {
    Main = 1,
    Device = 2,
    Midi = 3,
}

#[derive(Clone)]
pub struct App {
    pub keybindings: Keybindings,
    pub core: Arc<Mutex<UncycleCore>>,
    pub log: Arc<Mutex<Logger>>,
    pub tab: AppTab,
    pub menu: PopupMenu,
    should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            keybindings: Keybindings::new(),
            core: Arc::new(Mutex::new(UncycleCore::new(DEFAULT_BPM))),
            log: Arc::new(Mutex::new(Logger::new())),
            tab: AppTab::Main,
            menu: PopupMenu::new(),
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
                Action::ToggleMenu => self.toggle_tab_menu(PopupTab::Menu),
                Action::ToggleHelp => self.toggle_tab_menu(PopupTab::Help),
                Action::MenuMoveDown => match self.menu.settings.focus {
                    FocusArea::Settings => self.menu.settings.next_setting(),
                    FocusArea::Options => {
                        self.menu.settings.next_option();
                        self.menu.settings.apply_current_setting();
                    }
                },
                Action::MenuMoveUp => match self.menu.settings.focus {
                    FocusArea::Settings => self.menu.settings.previous_setting(),
                    FocusArea::Options => {
                        self.menu.settings.previous_option();
                        self.menu.settings.apply_current_setting();
                    }
                },
                Action::MenuEnter => self.menu.settings.switch_focus(),

                Action::MenuExit => self.menu.settings.switch_focus(),
            }
        } else {
            // Handle tab switching
            match key {
                KeyCode::Char('1') => self.tab = AppTab::Main,
                KeyCode::Char('2') => self.tab = AppTab::Device,
                KeyCode::Char('3') => self.tab = AppTab::Midi,
                _ => {}
            }
        }
    }

    fn cycle_tabs(&mut self) {
        match self.tab {
            AppTab::Main => self.tab = AppTab::Device,
            AppTab::Device => self.tab = AppTab::Midi,
            AppTab::Midi => self.tab = AppTab::Main,
        }
    }

    fn rev_cycle_tabs(&mut self) {
        match self.tab {
            AppTab::Main => self.tab = AppTab::Midi,
            AppTab::Device => self.tab = AppTab::Main,
            AppTab::Midi => self.tab = AppTab::Device,
        }
    }

    fn toggle_tab_menu(&mut self, tab: PopupTab) {
        if !(self.menu.is_active && self.menu.tab != tab) {
            self.menu.is_active ^= true;
        }

        self.menu.tab = tab;
    }
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    setup_midi_socket(app.core.clone(), app.log.clone());

    // default to TR-8 for now
    app.core
        .lock()
        .unwrap()
        .set_device(SupportedDevice::from(TR8::init()));

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

fn ui(f: &mut Frame, app: &mut App) {
    // Layout
    let main_area = f.area().inner(Margin {
        horizontal: 1,
        vertical: 1,
    });

    // Rendering
    match app.tab {
        AppTab::Main => render_main_tab(f, app, main_area),
        AppTab::Device => render_device_tab(f, app, main_area),
        AppTab::Midi => render_midi_tab(f, app, main_area),
    }

    render_popup_menu(f, app, f.area());
}

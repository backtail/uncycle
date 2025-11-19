mod app;
mod keybindings;
mod midi;
mod state;

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io,
    sync::{mpsc, Arc, Mutex},
};

use keybindings::Keybindings;
use state::UncycleState;

use midi::MidiState;

use app::connection::setup_midi_socket;
use app::{run_app, App};

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let state = UncycleState::new();
    let keybindings = Keybindings::new();
    let midi_state = Arc::new(Mutex::new(MidiState::new()));
    let (redraw_tx, redraw_rx) = mpsc::channel();

    setup_midi_socket(midi_state.clone(), redraw_tx);

    let mut app = App::new(state, keybindings, midi_state);

    let result = run_app(&mut terminal, &mut app, redraw_rx);

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

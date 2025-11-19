mod keybindings;
mod state;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, size},
    cursor::{MoveTo, Hide, Show},
};
use std::io;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use keybindings::{Keybindings, Action};
use state::UncycleState;

// MIDI note representation
#[derive(Clone)]
struct MidiNote {
    pub note: u8,
    pub velocity: u8,
    pub timestamp: Instant,
    pub active: bool,
}

impl MidiNote {
    fn new(note: u8, velocity: u8) -> Self {
        Self {
            note,
            velocity,
            timestamp: Instant::now(),
            active: true,
        }
    }
}

struct MidiState {
    pub active_notes: Vec<MidiNote>,
    pub last_note: Option<MidiNote>,
    pub error: Option<String>,
}

impl MidiState {
    fn new() -> Self {
        Self {
            active_notes: Vec::new(),
            last_note: None,
            error: None,
        }
    }

    fn add_note(&mut self, note: u8, velocity: u8) {
        let midi_note = MidiNote::new(note, velocity);
        self.active_notes.push(midi_note.clone());
        self.last_note = Some(midi_note);
        
        // Clean up old notes (keep only from last 2 seconds)
        let now = Instant::now();
        self.active_notes.retain(|n| now.duration_since(n.timestamp) < Duration::from_secs(2));
    }

    fn remove_note(&mut self, note: u8) {
        if let Some(found_note) = self.active_notes.iter_mut().find(|n| n.note == note) {
            found_note.active = false;
        }
    }

    fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    fn get_note_display(&self) -> String {
        if let Some(error) = &self.error {
            return format!("MIDI Error: {}", error);
        }

        if let Some(last_note) = &self.last_note {
            if last_note.timestamp.elapsed() < Duration::from_secs(1) {
                let note_name = Self::get_note_name(last_note.note);
                return format!("♪ {} ({}) vel:{}", note_name, last_note.note, last_note.velocity);
            }
        }
        
        // Show active notes if any
        let active_count = self.active_notes.iter().filter(|n| n.active).count();
        if active_count > 0 {
            format!("♪ {} note(s) active", active_count)
        } else {
            "No MIDI input".to_string()
        }
    }

    fn get_note_name(note: u8) -> String {
        let notes = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
        let octave = (note / 12) as i8 - 1;
        let note_index = (note % 12) as usize;
        format!("{}{}", notes[note_index], octave)
    }
}

struct TerminalBuffer {
    width: u16,
    height: u16,
    buffer: Vec<Vec<char>>,
}

impl TerminalBuffer {
    fn new(width: u16, height: u16) -> Self {
        let buffer = vec![vec![' '; width as usize]; height as usize];
        Self { width, height, buffer }
    }

    fn clear(&mut self) {
        for row in &mut self.buffer {
            for cell in row {
                *cell = ' ';
            }
        }
    }

    fn write_str(&mut self, x: u16, y: u16, text: &str) {
        if y >= self.height { return; }
        
        for (i, ch) in text.chars().enumerate() {
            let pos_x = x + i as u16;
            if pos_x < self.width && y < self.height {
                self.buffer[y as usize][pos_x as usize] = ch;
            }
        }
    }

    fn render(&self, previous: &TerminalBuffer) -> Result<()> {
        let mut stdout = io::stdout();
        
        for y in 0..self.height {
            for x in 0..self.width {
                let current_char = self.buffer[y as usize][x as usize];
                let previous_char = previous.buffer[y as usize][x as usize];
                
                if current_char != previous_char {
                    execute!(stdout, MoveTo(x, y))?;
                    print!("{}", current_char);
                }
            }
        }
        
        stdout.flush()?;
        Ok(())
    }
}

fn setup_midi_input(midi_state: Arc<Mutex<MidiState>>) {
    thread::spawn(move || {
        midi_input_thread(midi_state);
    });
}

fn midi_input_thread(midi_state: Arc<Mutex<MidiState>>) {
    let input = match midir::MidiInput::new("uncycle_midi_input") {
        Ok(input) => input,
        Err(e) => {
            let mut state = midi_state.lock().unwrap();
            state.set_error(format!("Failed to create MIDI input: {}", e));
            return;
        }
    };
    
    let in_ports = input.ports();
    
    if in_ports.is_empty() {
        let mut state = midi_state.lock().unwrap();
        state.set_error("No MIDI input ports available".to_string());
        return;
    }

    // // eprintln!("Available MIDI ports:");
    // for (i, port) in in_ports.iter().enumerate() {
        // eprintln!("{}: {}", i, input.port_name(port).unwrap_or_else(|_| "Unknown".to_string()));
    // }

    // Use the first available port
    let in_port = &in_ports[0];
    let port_name = match input.port_name(in_port) {
        Ok(name) => name,
        Err(e) => {
            let mut state = midi_state.lock().unwrap();
            state.set_error(format!("Failed to get port name: {}", e));
            return;
        }
    };
    
    // eprintln!("Using MIDI port: {}", port_name);

    // Connect without using ? operator - handle the Result manually
    let conn_result = input.connect(
        in_port,
        "uncycle-midi-in",
        move |_timestamp, message, _| {
            if message.len() >= 3 {
                let status = message[0];
                let data1 = message[1];
                let data2 = message[2];
                
                // Note On (0x90-0x9F) or Note Off (0x80-0x8F)
                let message_type = status & 0xF0;
                
                let mut state = midi_state.lock().unwrap();
                
                match message_type {
                    0x90 => { // Note On
                        if data2 > 0 { // Velocity > 0
                            state.add_note(data1, data2);
                        } else { // Note Off with velocity 0
                            state.remove_note(data1);
                        }
                    }
                    0x80 => { // Note Off
                        state.remove_note(data1);
                    }
                    _ => {
                        // Other MIDI messages (CC, pitch bend, etc.)
                    }
                }
            }
        },
        (),
    );

    match conn_result {
        Ok(conn) => {
            // Keep the connection alive - the connection will be dropped when this thread ends
            loop {
                thread::sleep(Duration::from_secs(1));
            }
        }
        Err(e) => {
            // let mut state = midi_state.lock().unwrap();
            // state.set_error(format!("Failed to connect to MIDI port: {}", e));
        }
    }
}

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;
    
    // Create state and keybindings
    let mut state = UncycleState::new();
    let keybindings = Keybindings::new();
    
    // Create MIDI state
    let midi_state = Arc::new(Mutex::new(MidiState::new()));
    
    // Setup MIDI input
    setup_midi_input(midi_state.clone());
    
    // Run the application
    let result = run_app(&mut state, &keybindings, midi_state);
    
    // Cleanup
    execute!(stdout, Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    
    result
}

fn run_app(state: &mut UncycleState, keybindings: &Keybindings, midi_state: Arc<Mutex<MidiState>>) -> Result<()> {
    let mut is_running = true;
    let terminal_size = size()?;
    
    // Create double buffers
    let mut current_buffer = TerminalBuffer::new(terminal_size.0, terminal_size.1);
    let mut previous_buffer = TerminalBuffer::new(terminal_size.0, terminal_size.1);
    
    // Initial draw
    draw_ui(state, &keybindings, &mut current_buffer, terminal_size, &midi_state)?;
    current_buffer.render(&previous_buffer)?;
    std::mem::swap(&mut current_buffer, &mut previous_buffer);
    
    while is_running {
        // Check for terminal resize
        let new_size = size()?;
        if new_size != terminal_size {
            // On resize, we need to recreate buffers and do a full redraw
            let mut new_current = TerminalBuffer::new(new_size.0, new_size.1);
            let new_previous = TerminalBuffer::new(new_size.0, new_size.1);
            
            draw_ui(state, &keybindings, &mut new_current, new_size, &midi_state)?;
            new_current.render(&new_previous)?;
            
            current_buffer = new_current;
            previous_buffer = new_previous;
        } else {
            // Always redraw when we have MIDI input to show real-time updates
            let needs_redraw = state.needs_redraw(new_size) || {
                let midi_state = midi_state.lock().unwrap();
                midi_state.last_note.is_some() && 
                midi_state.last_note.as_ref().unwrap().timestamp.elapsed() < Duration::from_millis(100)
            };
            
            if needs_redraw {
                current_buffer.clear();
                draw_ui(state, &keybindings, &mut current_buffer, new_size, &midi_state)?;
                current_buffer.render(&previous_buffer)?;
                std::mem::swap(&mut current_buffer, &mut previous_buffer);
            }
        }
        
        // Handle input
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                // Only process key presses (not releases)
                if key.kind == KeyEventKind::Press {
                    is_running = handle_input(key.code, state, &keybindings);
                }
            }
        } else {
            // Small sleep when no input to reduce CPU usage
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        
        // Simulate loop progression when playing
        if state.is_playing {
            // In a real app, this would be driven by MIDI clock
            state.loop_count += 1;
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
    
    Ok(())
}

fn draw_ui(
    state: &UncycleState, 
    keybindings: &Keybindings, 
    buffer: &mut TerminalBuffer, 
    (cols, rows): (u16, u16),
    midi_state: &Arc<Mutex<MidiState>>
) -> Result<()> {
    // Draw static header - centered
    let title = "uncycle - MIDI Looper";
    let title_x = if cols > title.len() as u16 {
        (cols - title.len() as u16) / 2
    } else {
        0
    };
    
    buffer.write_str(title_x, 0, &format!("┌{}┐", "─".repeat(title.len() + 2)));
    buffer.write_str(title_x, 1, &format!("│ {} │", title));
    buffer.write_str(title_x, 2, &format!("└{}┘", "─".repeat(title.len() + 2)));
    
    // Draw dynamic status
    let status_y = 4;
    buffer.write_str(0, status_y, &format!("Status: {}", state.status()));
    buffer.write_str(0, status_y + 1, &format!("Loop count: {}", state.loop_count));
    
    // Draw MIDI status
    let midi_state_guard = midi_state.lock().unwrap();
    let midi_display = midi_state_guard.get_note_display();
    buffer.write_str(0, status_y + 2, &format!("MIDI: {}", midi_display));
    buffer.write_str(0, status_y + 3, &format!("Terminal: {}x{}", cols, rows));
    
    // Draw controls section
    let controls_y = 8;
    buffer.write_str(0, controls_y, "Controls:");
    
    // Draw keybindings
    let display_bindings = keybindings.get_bindings_for_display();
    for (i, binding) in display_bindings.iter().enumerate() {
        let y = controls_y + 1 + i as u16;
        if y < rows - 2 {
            let key_str = match binding.key {
                KeyCode::Char(' ') => "[Space]".to_string(),
                KeyCode::Char(c) => format!("[{}]", c.to_uppercase()),
                _ => format!("[{:?}]", binding.key),
            };
            buffer.write_str(2, y, &format!("{} - {}", key_str, binding.description));
        }
    }
    
    // Update footer
    if rows > 15 {
        buffer.write_str(0, rows - 1, &format!("Press any key... ({}x{})", cols, rows));
    }
    
    Ok(())
}

fn handle_input(key: KeyCode, state: &mut UncycleState, keybindings: &Keybindings) -> bool {
    if let Some(action) = keybindings.find_action(key) {
        match action {
            Action::Quit => false,
            Action::StartRecording => {
                state.start_recording();
                true
            }
            Action::StopRecording => {
                state.stop_recording();
                state.stop_playback();
                true
            }
            Action::StartPlayback => {
                state.start_playback();
                true
            }
            Action::StopPlayback => {
                state.stop_playback();
                true
            }
            Action::TogglePlayback => {
                state.toggle_playback();
                true
            }
            Action::ClearLoop => {
                state.loop_count = 0;
                true
            }
        }
    } else {
        true
    }
}
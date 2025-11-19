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

use keybindings::{Keybindings, Action};
use state::UncycleState;

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

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;
    
    // Create state and keybindings
    let mut state = UncycleState::new();
    let keybindings = Keybindings::new();
    
    // Run the application
    let result = run_app(&mut state, &keybindings);
    
    // Cleanup
    execute!(stdout, Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    
    result
}

fn run_app(state: &mut UncycleState, keybindings: &Keybindings) -> Result<()> {
    let mut is_running = true;
    let terminal_size = size()?;
    
    // Create double buffers
    let mut current_buffer = TerminalBuffer::new(terminal_size.0, terminal_size.1);
    let mut previous_buffer = TerminalBuffer::new(terminal_size.0, terminal_size.1);
    
    // Initial draw
    draw_ui(state, &keybindings, &mut current_buffer, terminal_size)?;
    current_buffer.render(&previous_buffer)?;
    std::mem::swap(&mut current_buffer, &mut previous_buffer);
    
    while is_running {
        // Check for terminal resize
        let new_size = size()?;
        if new_size != terminal_size {
            // On resize, we need to recreate buffers and do a full redraw
            let mut new_current = TerminalBuffer::new(new_size.0, new_size.1);
            let new_previous = TerminalBuffer::new(new_size.0, new_size.1);
            
            draw_ui(state, &keybindings, &mut new_current, new_size)?;
            new_current.render(&new_previous)?;
            
            current_buffer = new_current;
            previous_buffer = new_previous;
        } else {
            // Only redraw if something changed
            if state.needs_redraw(new_size) {
                current_buffer.clear();
                draw_ui(state, &keybindings, &mut current_buffer, new_size)?;
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

fn draw_ui(state: &UncycleState, keybindings: &Keybindings, buffer: &mut TerminalBuffer, (cols, rows): (u16, u16)) -> Result<()> {
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
    buffer.write_str(0, status_y + 2, &format!("Terminal: {}x{}", cols, rows));
    
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

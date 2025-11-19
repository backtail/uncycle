use std::time::{Duration, Instant};

// MIDI note representation
#[derive(Clone)]
pub struct MidiNote {
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

pub struct MidiState {
    pub active_notes: Vec<MidiNote>,
    pub last_note: Option<MidiNote>,
    pub error: Option<String>,
    pub note_count: u32,
    pub message_log: Vec<String>,
}

impl MidiState {
    pub fn new() -> Self {
        Self {
            active_notes: Vec::new(),
            last_note: None,
            error: None,
            note_count: 0,
            message_log: Vec::new(),
        }
    }

    pub fn add_note(&mut self, note: u8, velocity: u8) {
        let midi_note = MidiNote::new(note, velocity);
        self.active_notes.push(midi_note.clone());
        self.last_note = Some(midi_note);
        self.note_count += 1;

        let note_name = Self::get_note_name(note);
        self.message_log
            .push(format!("♪ {} ({}) vel:{}", note_name, note, velocity));

        // Keep only last 50 messages
        if self.message_log.len() > 50 {
            self.message_log.remove(0);
        }

        // Clean up old notes
        let now = Instant::now();
        self.active_notes
            .retain(|n| now.duration_since(n.timestamp) < Duration::from_secs(2));
    }

    pub fn remove_note(&mut self, note: u8) {
        if let Some(found_note) = self.active_notes.iter_mut().find(|n| n.note == note) {
            found_note.active = false;
        }
    }

    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    pub fn add_message(&mut self, message: String) {
        self.message_log.push(message);
        if self.message_log.len() > 50 {
            self.message_log.remove(0);
        }
    }

    pub fn get_note_display(&self) -> String {
        if let Some(error) = &self.error {
            return format!("MIDI Error: {}", error);
        }

        if let Some(last_note) = &self.last_note {
            if last_note.timestamp.elapsed() < Duration::from_secs(1) {
                let note_name = Self::get_note_name(last_note.note);
                return format!(
                    "♪ {} ({}) vel:{}",
                    note_name, last_note.note, last_note.velocity
                );
            }
        }

        let active_count = self.active_notes.iter().filter(|n| n.active).count();
        if active_count > 0 {
            format!("♪ {} note(s) active", active_count)
        } else {
            "No MIDI input".to_string()
        }
    }

    pub fn get_note_name(note: u8) -> String {
        let notes = [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ];
        let octave = (note / 12) as i8 - 1;
        let note_index = (note % 12) as usize;
        format!("{}{}", notes[note_index], octave)
    }
}

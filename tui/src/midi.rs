use std::time::Instant;

// MIDI note representation
#[derive(Clone)]
pub struct MidiNote {
    pub note: u8,
    pub velocity: u8,
    pub active: bool,
}

// MIDI CC representation
#[derive(Clone)]
pub struct MidiCC {
    pub cc_num: u8,
    pub cc_val: u8,
}

impl MidiNote {
    fn new(note: u8, velocity: u8) -> Self {
        Self {
            note,
            velocity,
            active: true,
        }
    }
}

impl MidiCC {
    fn new(cc_num: u8, cc_val: u8) -> Self {
        Self { cc_num, cc_val }
    }
}

pub struct MidiState {
    pub active_notes: Vec<MidiNote>,
    pub last_cc: Vec<MidiCC>,
    pub last_note: Option<MidiNote>,
    pub error: Option<String>,
    pub note_count: u32,
    pub message_log: Vec<String>,

    pub output_connection: Option<midir::MidiOutputConnection>,
    pub _input_connected: bool,
    pub _output_connected: bool,
    pub _clock_running: bool,
    pub clock_bpm: f64,
    pub last_clock_time: Option<Instant>,
    pub clock_pulse_count: u32,
}

impl MidiState {
    pub fn new() -> Self {
        Self {
            active_notes: Vec::new(),
            last_cc: Vec::new(),
            last_note: None,
            error: None,
            note_count: 0,
            message_log: Vec::new(),

            output_connection: None,
            _input_connected: false,
            _output_connected: false,
            _clock_running: false,
            clock_bpm: 120.0,
            last_clock_time: None,
            clock_pulse_count: 0,
        }
    }

    pub fn add_note(&mut self, note: u8, velocity: u8) {
        let midi_note = MidiNote::new(note, velocity);
        self.active_notes.push(midi_note.clone());
        self.last_note = Some(midi_note);
        self.note_count += 1;
    }

    pub fn update_cc(&mut self, cc_num: u8, cc_val: u8) {
        if let Some(old) = self.last_cc.iter_mut().find(|n| n.cc_num == cc_num) {
            old.cc_val = cc_val
        } else {
            self.last_cc.push(MidiCC::new(cc_num, cc_val));
        }
    }

    pub fn remove_note(&mut self, note: u8) {
        self.active_notes.retain(|n| n.note != note);
    }

    pub fn find_active_note(&mut self, note: u8) -> bool {
        self.active_notes.iter().find(|n| n.note == note).is_some()
    }

    pub fn get_cc_val_of(&mut self, cc_num: u8) -> u8 {
        if let Some(exists) = self.last_cc.iter().find(|n| n.cc_num == cc_num) {
            exists.cc_val
        } else {
            0
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
            let note_name = Self::get_note_name(last_note.note);
            return format!(
                "♪ {} ({}) vel:{}",
                note_name, last_note.note, last_note.velocity
            );
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

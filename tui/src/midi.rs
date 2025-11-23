use anyhow::Result;
use midir::MidiOutputConnection;
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

// MIDI note representation
#[derive(Clone)]
pub struct MidiNote {
    pub note: u8,
    pub _velocity: u8,
}

// MIDI CC representation
#[derive(Clone)]
pub struct MidiCC {
    pub cc_num: u8,
    pub cc_val: u8,
}

impl MidiNote {
    fn new(note: u8, _velocity: u8) -> Self {
        Self { note, _velocity }
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

    // logging incoming data for convenience
    pub in_note_log: Vec<String>,
    pub in_cc_log: Vec<String>,
    pub in_other_log: Vec<String>,

    pub output_connection: Option<MidiOutputConnection>,
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
            in_note_log: Vec::new(),
            in_cc_log: Vec::new(),
            in_other_log: Vec::new(),

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

    pub fn log_incoming_note(&mut self, message: String) {
        self.in_note_log.push(message);
        if self.in_note_log.len() > 200 {
            self.in_note_log.remove(0);
        }
    }

    pub fn log_incoming_cc(&mut self, message: String) {
        self.in_cc_log.push(message);
        if self.in_cc_log.len() > 200 {
            self.in_cc_log.remove(0);
        }
    }

    pub fn log_misc(&mut self, message: String) {
        self.in_other_log.push(message);
        if self.in_other_log.len() > 200 {
            self.in_other_log.remove(0);
        }
    }

    pub fn increase_bpm_by(&mut self, amount: f64) {
        self.clock_bpm += amount;

        if self.clock_bpm >= 200.0 {
            self.clock_bpm = 200.0;
        }
    }

    pub fn decrease_bpm_by(&mut self, amount: f64) {
        self.clock_bpm -= amount;

        if self.clock_bpm <= 40.0 {
            self.clock_bpm = 40.0;
        }
    }
}

pub fn midi_rx_callback(midi_state: &Arc<Mutex<MidiState>>, message: &[u8]) -> Result<()> {
    if message.len() >= 3 {
        let status = message[0];
        let data1 = message[1];
        let data2 = message[2];

        let message_type = status & 0xF0;
        let mut state = midi_state.lock().unwrap();

        match message_type {
            0x90 => {
                // Note On
                if data2 > 0 {
                    state.add_note(data1, data2);
                    state.log_incoming_note(format!("NOTE ON:  {:02} {:02}", data1, data2));
                } else {
                    state.remove_note(data1);
                    state.log_incoming_note(format!("NOTE OFF: {:02}", data1));
                }
            }

            0x80 => {
                // Note Off
                state.remove_note(data1);
                state.log_incoming_note(format!("NOTE OFF: {:02}", data1));
            }

            0xB0 => {
                // Control Change
                state.update_cc(data1, data2);
                state.log_incoming_cc(format!("CC:       {:02} {:02}", data1, data2));
            }

            _ => {
                state.log_misc(format!("MIDI: {:02X} {:02X} {:02X}", status, data1, data2));
            }
        }
    }
    Ok(())
}

pub fn midi_tx_callback(midi_state: &Arc<Mutex<MidiState>>) -> Result<()> {
    let mut state = midi_state.lock().unwrap();

    if let Some(last_time) = state.last_clock_time {
        let interval = Duration::from_micros((60_000_000.0 / (state.clock_bpm * 24.0)) as u64);

        if last_time.elapsed() >= interval {
            if let Some(ref mut conn) = state.output_connection {
                const MIDI_CLOCK: &[u8] = &[0xF8];
                conn.send(MIDI_CLOCK)
                    .map_err(|e| anyhow::anyhow!("Failed to send MIDI Clock: {}", e))?;
                state.last_clock_time = Some(Instant::now());
                state.clock_pulse_count = state.clock_pulse_count.wrapping_add(1);
            }
        }
    } else {
        state.last_clock_time = Some(Instant::now());
    }
    Ok(())
}

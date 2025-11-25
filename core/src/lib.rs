pub mod devices;

use anyhow::{Error, Result};
use midir::MidiOutputConnection;
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use heapless::Vec;

const N_NOTES: usize = 128;
const N_CC_NUMBERS: usize = 128;
const MESSAGE_BUFFER_LEN: usize = 256;

const MIDI_CLOCK: u8 = 0xF8;
const MIDI_START: u8 = 0xFA;
const MIDI_STOP: u8 = 0xFC;

pub struct MidiState {
    /// allocate space for all possible values
    active_notes: [Option<u8>; N_NOTES],
    /// allocate space for all possible values
    last_cc: [Option<u8>; N_CC_NUMBERS],

    // logging data for convenience
    pub in_note_log: Vec<String, MESSAGE_BUFFER_LEN>,
    pub in_cc_log: Vec<String, MESSAGE_BUFFER_LEN>,
    pub in_other_log: Vec<String, MESSAGE_BUFFER_LEN>,

    pub port_in_name: Option<String>,
    pub port_out_name: Option<String>,

    pub clock_running: bool,
    pub start_flag: bool,
    pub stop_flag: bool,
    pub clock_bpm: f64,
    pub last_clock_time: Option<Instant>,
    pub clock_pulse_count: u32,

    pub kill_rx_conn: bool,
    pub kill_tx_conn: bool,
}

impl MidiState {
    pub fn new() -> Self {
        Self {
            active_notes: [None; N_NOTES],
            last_cc: [None; N_CC_NUMBERS],

            in_note_log: Vec::new(),
            in_cc_log: Vec::new(),
            in_other_log: Vec::new(),

            port_in_name: None,
            port_out_name: None,

            clock_running: false,
            start_flag: false,
            stop_flag: false,
            clock_bpm: 120.0,
            last_clock_time: None,
            clock_pulse_count: 0,

            kill_rx_conn: false,
            kill_tx_conn: false,
        }
    }

    pub fn update_note(&mut self, note: u8, velocity: u8) {
        self.active_notes[note as usize] = Some(velocity);
    }

    pub fn update_cc(&mut self, cc_num: u8, cc_val: u8) {
        self.last_cc[cc_num as usize] = Some(cc_val);
    }

    pub fn remove_note(&mut self, note: u8) {
        self.active_notes[note as usize] = None;
    }

    pub fn find_active_note(&mut self, note: u8) -> bool {
        self.active_notes[note as usize].is_some()
    }

    pub fn get_cc_val_of(&mut self, cc_num: u8) -> u8 {
        self.last_cc[cc_num as usize].unwrap_or(0)
    }

    pub fn log_incoming_note(&mut self, message: String) {
        if self.in_note_log.is_full() {
            self.in_note_log.remove(0);
        }

        self.in_note_log.push(message).unwrap();
    }

    pub fn log_incoming_cc(&mut self, message: String) {
        if self.in_cc_log.is_full() {
            self.in_cc_log.remove(0);
        }

        self.in_cc_log.push(message).unwrap();
    }

    pub fn log_misc(&mut self, message: String) {
        if self.in_other_log.is_full() {
            self.in_other_log.remove(0);
        }

        self.in_other_log.push(message).unwrap();
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

    pub fn start_stop_sequence(&mut self) {
        if self.clock_running {
            self.stop_flag = true;
        } else {
            self.start_flag = true;
        }
    }

    pub fn get_step_number(&mut self) -> u8 {
        if self.clock_running {
            ((self.clock_pulse_count / 6) % 16) as u8
        } else {
            0
        }
    }
}

// is executed in dedicated thread
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
                    state.update_note(data1, data2);
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

// is executed in dedicated thread
pub fn midi_tx_callback(
    midi_state: &Arc<Mutex<MidiState>>,
    conn: &mut MidiOutputConnection,
) -> Result<()> {
    let mut state = midi_state.lock().unwrap();

    // MIDI Start
    if state.start_flag {
        state.start_flag = false;
        state.clock_running = true;

        conn.send(&[MIDI_START]).unwrap();
        state.clock_pulse_count = 0;
        state.log_misc(format!("Send: 0x{:02X} (MIDI Start)", MIDI_START));
    }

    // MIDI Stop
    if state.stop_flag {
        state.stop_flag = false;
        state.clock_running = false;

        conn.send(&[MIDI_STOP]).unwrap();
        state.log_misc(format!("Send: 0x{:02X} (MIDI Stop)", MIDI_STOP));
    }

    // MIDI Clock
    if let Some(last_time) = state.last_clock_time {
        let interval = Duration::from_micros((60_000_000.0 / (state.clock_bpm * 24.0)) as u64);

        if last_time.elapsed() >= interval {
            conn.send(&[MIDI_CLOCK]).unwrap();

            state.last_clock_time = Some(Instant::now());
            state.clock_pulse_count = state.clock_pulse_count.wrapping_add(1);
        }
    } else {
        state.last_clock_time = Some(Instant::now());
    }

    if !state.kill_tx_conn {
        state.kill_tx_conn = false;
        Ok(())
    } else {
        Err(Error::msg("MIDI TX connection killed"))
    }
}

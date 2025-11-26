#![no_std]

#[cfg(feature = "std")]
extern crate std;

pub mod devices;

use heapless::Vec;

const N_NOTES: usize = 128;
const N_CC_NUMBERS: usize = 128;

const MIDI_CLOCK: u8 = 0xF8;
const MIDI_START: u8 = 0xFA;
const MIDI_STOP: u8 = 0xFC;

const MIDI_NOTE_ON: u8 = 0x90;
const MIDI_NOTE_OFF: u8 = 0x80;
const MIDI_CONTORL_CHANGE: u8 = 0xB0;

const TX_MIDI_Q_LEN: usize = 16;

pub struct UncycleCore {
    /// allocate space for all possible values
    active_notes: [Option<u8>; N_NOTES],
    /// allocate space for all possible values
    last_cc: [Option<u8>; N_CC_NUMBERS],

    clock_running: bool,
    start_flag: bool,
    stop_flag: bool,
    clock_bpm: f32,
    last_clock_time: u64, // in microseconds
    clock_pulse_count: u32,

    pub kill_rx_conn: bool,
    pub kill_tx_conn: bool,
}

impl UncycleCore {
    pub fn new() -> Self {
        Self {
            active_notes: [None; N_NOTES],
            last_cc: [None; N_CC_NUMBERS],

            clock_running: false,
            start_flag: false,
            stop_flag: false,
            clock_bpm: 120.0,
            last_clock_time: 0,
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

    pub fn increase_bpm_by(&mut self, amount: f32) {
        self.clock_bpm += amount;

        if self.clock_bpm >= 200.0 {
            self.clock_bpm = 200.0;
        }
    }

    pub fn decrease_bpm_by(&mut self, amount: f32) {
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

    pub fn get_bpm(&self) -> f32 {
        self.clock_bpm
    }

    // timestamp is time elapsed since beginning of program start in microseconds
    pub fn midi_tx_callback(&mut self, elapsed: u64) -> Vec<u8, TX_MIDI_Q_LEN> {
        let mut tx_q = Vec::new();

        // MIDI Start
        if self.start_flag {
            self.start_flag = false;
            self.clock_running = true;

            tx_q.push(MIDI_START).ok();
            self.clock_pulse_count = 0;
        }

        // MIDI Stop
        if self.stop_flag {
            self.stop_flag = false;
            self.clock_running = false;

            tx_q.push(MIDI_STOP).ok();
        }

        // MIDI Clock
        let interval = (60_000_000.0 / (self.clock_bpm * 24.0)) as u64;

        if elapsed - self.last_clock_time >= interval {
            self.last_clock_time = elapsed;
            self.clock_pulse_count = self.clock_pulse_count.wrapping_add(1);

            tx_q.push(MIDI_CLOCK).ok();
        }

        tx_q
    }

    pub fn midi_rx_callback(&mut self, message: &[u8]) {
        if let Some(msg) = parse_midi_message(message) {
            let _status = message[0];
            let data1 = message[1];
            let data2 = message[2];

            match msg {
                MIDI_NOTE_ON => self.update_note(data1, data2),
                MIDI_NOTE_OFF => self.remove_note(data1),
                MIDI_CONTORL_CHANGE => self.update_cc(data1, data2),
                _ => {}
            }
        }
    }
}

pub fn parse_midi_message(msg: &[u8]) -> Option<u8> {
    let mut result = None;

    if msg.len() >= 3 {
        let status = msg[0];
        let _data1 = msg[1];
        let data2 = msg[2];

        let message_type = status & 0xF0;

        match message_type {
            MIDI_NOTE_ON => {
                // Note On
                if data2 > 0 {
                    result = Some(MIDI_NOTE_ON);
                } else {
                    result = Some(MIDI_NOTE_OFF);
                }
            }

            MIDI_NOTE_OFF => {
                // Note Off
                result = Some(MIDI_NOTE_OFF);
            }

            MIDI_CONTORL_CHANGE => {
                result = Some(MIDI_CONTORL_CHANGE);
            }

            _ => {}
        }
    }

    result
}

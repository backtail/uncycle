use super::{looper::Looper, midi::*};
use heapless::Vec;

const TX_MIDI_Q_LEN: usize = 16;

pub struct UncycleCore {
    /// allocate space for all possible values
    active_notes: [Option<u8>; N_NOTES],
    /// allocate space for all possible values
    last_cc: [Option<u8>; N_CC_NUMBERS],

    looper: Looper,

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

            looper: Looper::new(),

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

    pub fn start_recording(&mut self) {
        self.looper.start_recording();
    }

    fn handle_looper_playback(&mut self, now: u64, tx_q: &mut Vec<u8, TX_MIDI_Q_LEN>) {
        for bytes in self.looper.play_back_recording(now) {
            for byte in bytes {
                tx_q.push(*byte).ok();
            }

            self.last_cc[bytes[1] as usize] = Some(bytes[2]);
        }
    }

    /// `now` is time elapsed since beginning of program start in microseconds
    pub fn midi_rx_callback(&mut self, now: u64, message: &[u8]) {
        if let Some(in_type) = parse_midi_message(message) {
            let bytes: MidiMsg = [message[0], message[1], message[2]];

            match in_type {
                MIDI_NOTE_ON => self.update_note(bytes[1], bytes[2]),
                MIDI_NOTE_OFF => self.remove_note(bytes[1]),
                MIDI_CONTORL_CHANGE => {
                    self.update_cc(bytes[1], bytes[2]);
                    self.looper.record_cc(now, &bytes);
                }
                _ => {}
            };
        }
    }

    /// `now` is time elapsed since beginning of program start in microseconds
    pub fn midi_tx_callback(&mut self, now: u64) -> Vec<u8, TX_MIDI_Q_LEN> {
        let mut tx_q = Vec::new();

        if self.looper.check_if_started(now) {
            tx_q.push(MIDI_START).ok();
        }

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

        if now - self.last_clock_time >= interval {
            self.last_clock_time = now;
            self.clock_pulse_count = self.clock_pulse_count.wrapping_add(1);

            self.looper.handle_timing(now);

            tx_q.push(MIDI_CLOCK).ok();
        }

        self.handle_looper_playback(now, &mut tx_q);

        tx_q
    }
}

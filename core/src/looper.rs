use crate::midi::MidiMsg;

use heapless::Vec;

#[derive(Clone, Copy)]
pub struct RecordedMidiMsg {
    time: u32,
    msg: MidiMsg,
}

pub struct Looper {
    pub current_loop_cc: Vec<RecordedMidiMsg, 1024>,
    pub recorded_cc: Vec<RecordedMidiMsg, 1024>,
    pub recording_ongoing: bool,
    pub recording_started: Option<u64>,
    pub recording_clock_tick_counter: u16,
    pub recorded_loop_length: Option<u32>,
}

impl Looper {
    pub fn new() -> Self {
        Self {
            current_loop_cc: Vec::new(),
            recorded_cc: Vec::new(),
            recording_ongoing: false,
            recording_started: None,
            recording_clock_tick_counter: 0,
            recorded_loop_length: None,
        }
    }

    pub fn start_recording(&mut self) {
        self.recording_ongoing = true;
    }

    pub fn check_if_started(&mut self, now: u64) -> bool {
        if self.recording_ongoing && self.recording_started.is_none() {
            self.recording_started = Some(now);

            true
        } else {
            false
        }
    }

    /// Must be called for every incoming CC message
    pub fn record_cc(&mut self, now: u64, cc_msg: &MidiMsg) {
        if self.recording_ongoing {
            if let Some(start_time) = self.recording_started {
                self.recorded_cc
                    .push(RecordedMidiMsg {
                        msg: *cc_msg,
                        time: (now - start_time) as u32,
                    })
                    .ok();
            }
        }
    }

    /// Must be called for MIDI clock tick
    pub fn handle_timing(&mut self, now: u64) {
        if self.recording_started.is_some() {
            self.recording_clock_tick_counter += 1;

            if self.recording_ongoing {
                if self.recording_clock_tick_counter >= (16 / 4) * 24 {
                    self.recording_ongoing = false;
                    self.recorded_loop_length =
                        Some((now - self.recording_started.unwrap()) as u32);
                }
            }

            // end of loop
            if self.recording_clock_tick_counter % (16 / 4) * 24 == 0 {
                for cc in &self.recorded_cc {
                    self.current_loop_cc.push(*cc).ok();
                }
            }
        }
    }

    pub fn play_back_recording(&mut self, now: u64) -> Vec<MidiMsg, 1> {
        let mut result = Vec::new();
        if let Some(loop_time) = self.recorded_loop_length {
            self.current_loop_cc.retain(|cc| {
                if cc.time as u64 <= (now - self.recording_started.unwrap()) % loop_time as u64 {
                    result.push(cc.msg).ok();

                    false
                } else {
                    true
                }
            });
        }

        result
    }
}

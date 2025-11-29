use crate::midi::{MidiMsg, N_CC_NUMBERS};

use heapless::Vec;

#[derive(Clone, Copy)]
pub struct RecordedMidiMsg {
    time: u32,
    msg: MidiMsg,
}

pub struct Looper {
    /// statically allocated buffer for 128 possible CC messages
    playback_buffer: Vec<MidiMsg, N_CC_NUMBERS>,

    time_last_checked: u64,

    pub recorded_cc: Vec<RecordedMidiMsg, 1024>,
    pub recording_ongoing: bool,
    pub recording_started: Option<u64>,
    pub recording_clock_tick_counter: u16,
    pub recorded_loop_length: Option<u32>,
}

impl Looper {
    pub fn new() -> Self {
        Self {
            playback_buffer: Vec::new(),

            time_last_checked: 0,

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

    pub fn delete_recording(&mut self) {
        self.recorded_cc.clear();
        self.recording_ongoing = false;
        self.recording_started = None;
        self.recording_clock_tick_counter = 0;
        self.recorded_loop_length = None;
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
                if self.recording_clock_tick_counter > (16 / 4) * 24 {
                    self.recording_ongoing = false;
                    self.recorded_loop_length =
                        Some((now - self.recording_started.unwrap()) as u32);
                }
            }

            // end of loop
            if self.recording_clock_tick_counter % (16 / 4) * 24 == 0 {
                // signal end of loop
            }
        }
    }

    /// Returns a heapless vector with pre-allocated 128 possible items of type `MidiMsg`, since it only returns the
    /// lastest event in the relevant time frame from each CC number. This tries to limit the bandwith
    /// of dubbed recordings and also acts as the worst case scenario.
    pub fn play_back_recording(&mut self, now: u64) -> &Vec<MidiMsg, N_CC_NUMBERS> {
        self.playback_buffer.clear();

        if let Some(loop_time) = self.recorded_loop_length {
            self.recorded_cc.iter().for_each(|cc| {
                let rec_start = self.recording_started.unwrap();

                if is_in_time_frame(
                    cc.time,
                    self.time_last_checked - rec_start,
                    now - rec_start,
                    loop_time,
                ) {
                    self.playback_buffer
                        .push(cc.msg) // add this recorded event
                        .ok(); // if vec is full, drop it
                }
            });
        }

        self.time_last_checked = now;

        &self.playback_buffer
    }
}

fn is_in_time_frame(check: u32, frame_begin: u64, frame_end: u64, loop_len: u32) -> bool {
    check >= (frame_begin % loop_len as u64) as u32 // lower bound
    && check <= (frame_end % loop_len as u64) as u32 // upper bound
}

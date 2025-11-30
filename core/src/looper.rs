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

    recorded_cc: Vec<RecordedMidiMsg, 1024>,
    record: bool,
    rec_start: Option<u64>,
    loop_len: Option<u32>,
}

impl Looper {
    pub fn new() -> Self {
        Self {
            playback_buffer: Vec::new(),

            time_last_checked: 0,

            recorded_cc: Vec::new(),
            record: false,
            rec_start: None,
            loop_len: None,
        }
    }

    /// Engage in recording CC messages by providing a non-zero `loop_len` in µs
    pub fn start_recording(&mut self, loop_len: u32) {
        assert!(loop_len != 0);

        if self.rec_start.is_none() && !self.record {
            self.record = true;
            self.loop_len = Some(loop_len);
        }
    }

    pub fn delete_recording(&mut self) {
        self.recorded_cc.clear();
        self.record = false;
        self.rec_start = None;
        self.loop_len = None;
    }

    pub fn check_if_started(&mut self, now: u64) -> bool {
        if self.record && self.rec_start.is_none() {
            self.rec_start = Some(now);

            true
        } else {
            false
        }
    }

    /// Must be called for every incoming CC message
    pub fn record_cc(&mut self, now: u64, cc_msg: &MidiMsg) {
        if self.record {
            if let Some(start_time) = self.rec_start {
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
        if let Some(start) = self.rec_start {
            if self.record {
                if let Some(len) = self.loop_len {
                    if (now - start) as u32 >= len {
                        self.record = false;
                    }
                }
            }
        }
    }

    /// Returns a heapless vector with pre-allocated 128 possible items of type `MidiMsg`, since it only returns the
    /// lastest event in the relevant time frame from each CC number. This tries to limit the bandwith
    /// of dubbed recordings and also acts as the worst case scenario.
    pub fn play_back_recording(&mut self, now: u64) -> &Vec<MidiMsg, N_CC_NUMBERS> {
        self.playback_buffer.clear();

        if !self.record {
            if let Some(loop_time) = self.loop_len {
                let rec_start = self.rec_start.unwrap();

                self.recorded_cc.iter().for_each(|cc| {
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
        }
        self.time_last_checked = now;

        &self.playback_buffer
    }
}

fn is_in_time_frame(check: u32, frame_begin: u64, frame_end: u64, loop_len: u32) -> bool {
    check >= (frame_begin % loop_len as u64) as u32 // lower bound
    && check <= (frame_end % loop_len as u64) as u32 // upper bound
}

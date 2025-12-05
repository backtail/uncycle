use crate::midi::{MidiMsg, N_CC_NUMBERS};

use heapless::Vec;

const DEFAULT_REC_LEN_STEPS: u16 = 32;

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
    pub record: bool,
    rec_start: Option<u64>,

    pub loop_steps: u16,

    /// in microseconds
    loop_len: u32,

    pub overdub: bool,
    overdub_start: Option<u64>,
}

impl Looper {
    pub fn new() -> Self {
        Self {
            playback_buffer: Vec::new(),

            time_last_checked: 0,

            recorded_cc: Vec::new(),
            record: false,
            rec_start: None,

            loop_steps: DEFAULT_REC_LEN_STEPS,
            loop_len: bpm_to_us(120.0, DEFAULT_REC_LEN_STEPS),

            overdub: false,
            overdub_start: None,
        }
    }

    // set loop length from 4 to U16_MAX
    pub fn set_loop_steps(&mut self, steps: u16) {
        assert!(steps >= 4);

        let n_old_steps = self.loop_steps;
        let ratio = steps as f32 / n_old_steps as f32;
        self.loop_len = (self.loop_len as f32 * ratio) as u32;

        self.loop_steps = steps;
    }

    pub fn update_loop_len(&mut self, bpm: f32) {
        self.loop_len = bpm_to_us(bpm, self.loop_steps);
    }

    /// Engage in recording CC messages by providing a non-zero `loop_len` in µs
    pub fn start_recording(&mut self, now: u64) {
        if !self.record {
            if self.rec_start.is_none() {
                self.record = true;
                self.rec_start = Some(now);
            } else {
                if !self.overdub {
                    self.overdub_start = Some(now);
                    self.overdub = true;
                }
            }
        }
    }

    pub fn delete_recording(&mut self) {
        self.recorded_cc.clear();
        self.record = false;
        self.rec_start = None;
        self.overdub = false;
        self.overdub_start = None;
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

        if self.overdub {
            if let Some(start_time) = self.overdub_start {
                self.recorded_cc
                    .push(RecordedMidiMsg {
                        msg: *cc_msg,
                        time: (now - start_time) as u32,
                    })
                    .ok();
            }
        }
    }

    /// End of loop handling
    pub fn handle_eol(&mut self, now: u64) {
        if let Some(start) = self.rec_start {
            if self.record {
                if (now - start) as u32 >= self.loop_len {
                    self.record = false;
                }
            }
        }

        if let Some(start) = self.overdub_start {
            if self.overdub {
                if (now - start) as u32 >= self.loop_len {
                    self.overdub = false;
                    self.overdub_start = None;
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
            if let Some(start) = self.rec_start {
                self.recorded_cc.iter().for_each(|cc| {
                    if is_in_time_frame(
                        cc.time,
                        self.time_last_checked - start,
                        now - start,
                        self.loop_len,
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

/// Returns `n_steps` time for current bpm in µs
///
/// BPM stands for *Beat per Minute* or more accurately **Quarter Note per Minute**
/// - time per quarter note: 60 s / `clock_bpm`
/// - `n_steps` are in sixteenths
fn bpm_to_us(bpm: f32, n_steps: u16) -> u32 {
    assert!(bpm != 0.0);

    let quarter_note = 60.0 / bpm; // in s

    (n_steps as f32 * (quarter_note / 4.0) * 10E5) as u32 // in µs
}

// Simple state machine - no external library needed for now
pub struct UncycleState {
    pub is_recording: bool,
    pub is_playing: bool,
    pub loop_count: u32,
}

impl UncycleState {
    pub fn new() -> Self {
        Self {
            is_recording: false,
            is_playing: false,
            loop_count: 0,
        }
    }

    pub fn start_recording(&mut self) {
        self.is_recording = true;
        self.is_playing = false;
    }

    pub fn stop_recording(&mut self) {
        self.is_recording = false;
    }

    pub fn start_playback(&mut self) {
        self.is_playing = true;
        self.is_recording = false;
    }

    pub fn stop_playback(&mut self) {
        self.is_playing = false;
    }

    pub fn toggle_playback(&mut self) {
        if self.is_playing {
            self.stop_playback();
        } else {
            self.start_playback();
        }
    }

    pub fn status(&self) -> &str {
        if self.is_recording {
            "⏺ RECORDING"
        } else if self.is_playing {
            "▶ PLAYING"
        } else {
            "⏸ STOPPED"
        }
    }
}

impl Default for UncycleState {
    fn default() -> Self {
        Self::new()
    }
}

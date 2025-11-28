use heapless::Vec;

const MESSAGE_BUFFER_LEN: usize = 256;

pub struct Logger {
    pub in_note_log: Vec<String, MESSAGE_BUFFER_LEN>,
    pub in_cc_log: Vec<String, MESSAGE_BUFFER_LEN>,
    pub in_other_log: Vec<String, MESSAGE_BUFFER_LEN>,

    pub out_cc_log: Vec<String, MESSAGE_BUFFER_LEN>,

    pub port_in_name: Option<String>,
    pub port_out_name: Option<String>,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            in_note_log: Vec::new(),
            in_cc_log: Vec::new(),
            in_other_log: Vec::new(),

            out_cc_log: Vec::new(),

            port_in_name: None,
            port_out_name: None,
        }
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

    pub fn log_outgoing_cc(&mut self, message: String) {
        if self.out_cc_log.is_full() {
            self.out_cc_log.remove(0);
        }

        self.out_cc_log.push(message).unwrap();
    }
}

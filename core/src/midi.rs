pub const N_NOTES: usize = 128;
pub const N_CC_NUMBERS: usize = 128;

pub const MIDI_CLOCK: u8 = 0xF8;
pub const MIDI_START: u8 = 0xFA;
pub const MIDI_CONTINUE: u8 = 0xFB;
pub const MIDI_STOP: u8 = 0xFC;

pub const MIDI_NOTE_ON: u8 = 0x90;
pub const MIDI_NOTE_OFF: u8 = 0x80;
pub const MIDI_CONTORL_CHANGE: u8 = 0xB0;

pub type MidiMsg = [u8; 3];

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

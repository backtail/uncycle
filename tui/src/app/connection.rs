use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use uncycle_core::{parse_midi_message, MidiState};

use crate::app::MidiLogger;

pub fn setup_midi_socket(midi_state: Arc<Mutex<MidiState>>, midi_logger: Arc<Mutex<MidiLogger>>) {
    let midi_state_output = Arc::clone(&midi_state);
    let midi_logger_output = Arc::clone(&midi_logger);

    thread::spawn(move || {
        midi_input_thread(midi_state, midi_logger);
    });

    thread::spawn(move || {
        midi_output_thread(midi_state_output, midi_logger_output);
    });
}

pub fn midi_input_thread(midi_state: Arc<Mutex<MidiState>>, midi_logger: Arc<Mutex<MidiLogger>>) {
    let input = match midir::MidiInput::new("uncycle_midi_input") {
        Ok(input) => input,
        Err(e) => {
            midi_logger
                .lock()
                .unwrap()
                .log_misc(format!("Failed to create MIDI input: {}", e));
            return;
        }
    };

    let in_ports = input.ports();

    if in_ports.is_empty() {
        midi_logger
            .lock()
            .unwrap()
            .log_misc("No MIDI input ports available".to_string());
        return;
    }

    // Use the first available port
    let mut in_port = &in_ports[0];

    // However, try to connect to TR-8 if possible
    for (i, port) in in_ports.iter().enumerate() {
        if input.port_name(port).unwrap().contains("TR-8") {
            in_port = &in_ports[i];
            break;
        }
    }

    // callback function is defined before entering loop
    let conn_result = input.connect(
        in_port,
        "uncycle-midi-in",
        move |_timestamp, message, _| {
            // first handle midi logic
            midi_state.lock().unwrap().midi_rx_callback(message);

            // then handle logging
            if let Some(msg) = parse_midi_message(message) {
                let status = message[0];
                let data1 = message[1];
                let data2 = message[2];

                const MIDI_NOTE_ON: u8 = 0x90;
                const MIDI_NOTE_OFF: u8 = 0x80;
                const MIDI_CONTORL_CHANGE: u8 = 0xB0;

                match msg {
                    MIDI_NOTE_ON => midi_logger
                        .lock()
                        .unwrap()
                        .log_incoming_note(format!("NOTE ON:  {:02} {:02}", data1, data2)),

                    MIDI_NOTE_OFF => {}

                    MIDI_CONTORL_CHANGE => midi_logger
                        .lock()
                        .unwrap()
                        .log_incoming_cc(format!("CC:       {:02} {:02}", data1, data2)),

                    _ => midi_logger
                        .lock()
                        .unwrap()
                        .log_misc(format!("MIDI: {:02X} {:02X} {:02X}", status, data1, data2)),
                }
            }
        },
        (),
    );

    match conn_result {
        Ok(_conn) => loop {
            thread::sleep(Duration::from_millis(16));
        },

        Err(_e) => {}
    }
}

pub fn midi_output_thread(midi_state: Arc<Mutex<MidiState>>, midi_logger: Arc<Mutex<MidiLogger>>) {
    let output = match midir::MidiOutput::new("uncycle_midi_output") {
        Ok(output) => output,
        Err(e) => {
            midi_logger
                .lock()
                .unwrap()
                .log_misc(format!("Failed to create MIDI output: {}", e));
            return;
        }
    };

    let out_ports = output.ports();

    if out_ports.is_empty() {
        midi_logger
            .lock()
            .unwrap()
            .log_misc("No MIDI output ports available".to_string());
        return;
    }

    let mut out_port = &out_ports[0];

    for (i, port) in out_ports.iter().enumerate() {
        if output.port_name(port).unwrap().contains("TR-8") {
            out_port = &out_ports[i];
            break;
        }
    }

    let port_name = match output.port_name(out_port) {
        Ok(name) => name,
        Err(e) => {
            midi_logger
                .lock()
                .unwrap()
                .log_misc(format!("Failed to get output port name: {}", e));
            return;
        }
    };

    let conn_result = output.connect(out_port, "uncycle-midi-in");

    match conn_result {
        Ok(mut conn) => {
            {
                let mut state = midi_logger.lock().unwrap();
                state.port_out_name = Some(port_name.clone());
                state.log_misc(format!("Connected to MIDI out port: {}", port_name));
            }

            let start_time = Instant::now();

            loop {
                // poll @ 1kHz, thread timing accuracy does not matter since we pass time as paramter to callback
                thread::sleep(Duration::from_millis(1));

                let bytes;
                let elapsed = start_time.elapsed().as_micros() as u64;

                {
                    bytes = midi_state.lock().unwrap().midi_tx_callback(elapsed);
                }

                // send MIDI outside of lock
                conn.send(&bytes).ok();

                // log after sending
                const MIDI_START: u8 = 0xFA;
                const MIDI_STOP: u8 = 0xFC;

                for byte in &bytes {
                    if *byte == MIDI_START {
                        midi_logger
                            .lock()
                            .unwrap()
                            .log_misc(format!("Send: 0x{:02X} (MIDI Start)", MIDI_START));
                    }

                    if *byte == MIDI_STOP {
                        midi_logger
                            .lock()
                            .unwrap()
                            .log_misc(format!("Send: 0x{:02X} (MIDI Stop)", MIDI_STOP));
                    }
                }
            }
        }

        Err(e) => {
            midi_logger
                .lock()
                .unwrap()
                .log_misc(format!("Failed to connect to MIDI port: {}", e));
        }
    }
}

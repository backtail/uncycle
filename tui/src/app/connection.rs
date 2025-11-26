use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use uncycle_core::prelude::*;

use super::log::Logger;

pub fn setup_midi_socket(core: Arc<Mutex<UncycleCore>>, log: Arc<Mutex<Logger>>) {
    let core_arc_clone = Arc::clone(&core);
    let log_arc_clone = Arc::clone(&log);

    thread::spawn(move || {
        midi_input_thread(core, log);
    });

    thread::spawn(move || {
        midi_output_thread(core_arc_clone, log_arc_clone);
    });
}

pub fn midi_input_thread(core: Arc<Mutex<UncycleCore>>, log: Arc<Mutex<Logger>>) {
    let input = match midir::MidiInput::new("uncycle_midi_input") {
        Ok(input) => input,
        Err(e) => {
            log.lock()
                .unwrap()
                .log_misc(format!("Failed to create MIDI input: {}", e));
            return;
        }
    };

    let in_ports = input.ports();

    if in_ports.is_empty() {
        log.lock()
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
            core.lock().unwrap().midi_rx_callback(message);

            // then handle logging
            if let Some(msg) = parse_midi_message(message) {
                let status = message[0];
                let data1 = message[1];
                let data2 = message[2];

                const MIDI_NOTE_ON: u8 = 0x90;
                const MIDI_NOTE_OFF: u8 = 0x80;
                const MIDI_CONTORL_CHANGE: u8 = 0xB0;

                match msg {
                    MIDI_NOTE_ON => log
                        .lock()
                        .unwrap()
                        .log_incoming_note(format!("NOTE ON:  {:02} {:02}", data1, data2)),

                    MIDI_NOTE_OFF => {}

                    MIDI_CONTORL_CHANGE => log
                        .lock()
                        .unwrap()
                        .log_incoming_cc(format!("CC:       {:02} {:02}", data1, data2)),

                    _ => log
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

pub fn midi_output_thread(core: Arc<Mutex<UncycleCore>>, log: Arc<Mutex<Logger>>) {
    let output = match midir::MidiOutput::new("uncycle_midi_output") {
        Ok(output) => output,
        Err(e) => {
            log.lock()
                .unwrap()
                .log_misc(format!("Failed to create MIDI output: {}", e));
            return;
        }
    };

    let out_ports = output.ports();

    if out_ports.is_empty() {
        log.lock()
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
            log.lock()
                .unwrap()
                .log_misc(format!("Failed to get output port name: {}", e));
            return;
        }
    };

    let conn_result = output.connect(out_port, "uncycle-midi-in");

    match conn_result {
        Ok(mut conn) => {
            {
                let mut locked = log.lock().unwrap();
                locked.port_out_name = Some(port_name.clone());
                locked.log_misc(format!("Connected to MIDI out port: {}", port_name));
            }

            let start_time = Instant::now();

            loop {
                // poll @ 1kHz, thread timing accuracy does not matter since we pass time as paramter to callback
                thread::sleep(Duration::from_millis(1));

                let bytes;
                let elapsed = start_time.elapsed().as_micros() as u64;

                {
                    bytes = core.lock().unwrap().midi_tx_callback(elapsed);
                }

                // send MIDI outside of lock
                conn.send(&bytes).ok();

                // log after sending
                const MIDI_START: u8 = 0xFA;
                const MIDI_STOP: u8 = 0xFC;

                for byte in &bytes {
                    if *byte == MIDI_START {
                        log.lock()
                            .unwrap()
                            .log_misc(format!("Send: 0x{:02X} (MIDI Start)", MIDI_START));
                    }

                    if *byte == MIDI_STOP {
                        log.lock()
                            .unwrap()
                            .log_misc(format!("Send: 0x{:02X} (MIDI Stop)", MIDI_STOP));
                    }
                }
            }
        }

        Err(e) => {
            log.lock()
                .unwrap()
                .log_misc(format!("Failed to connect to MIDI port: {}", e));
        }
    }
}

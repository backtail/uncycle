use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::midi::{midi_rx_callback, midi_tx_callback, MidiState};

pub fn setup_midi_socket(midi_state: Arc<Mutex<MidiState>>) {
    let midi_state_output = Arc::clone(&midi_state);

    thread::spawn(move || {
        midi_input_thread(midi_state);
    });

    thread::spawn(move || {
        midi_output_thread(midi_state_output);
    });
}

fn midi_input_thread(midi_state: Arc<Mutex<MidiState>>) {
    let input = match midir::MidiInput::new("uncycle_midi_input") {
        Ok(input) => input,
        Err(e) => {
            let mut state = midi_state.lock().unwrap();
            state.set_error(format!("Failed to create MIDI input: {}", e));
            return;
        }
    };

    let in_ports = input.ports();

    if in_ports.is_empty() {
        let mut state = midi_state.lock().unwrap();
        state.set_error("No MIDI input ports available".to_string());
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

    let port_name = match input.port_name(in_port) {
        Ok(name) => name,
        Err(e) => {
            let mut state = midi_state.lock().unwrap();
            state.set_error(format!("Failed to get port name: {}", e));
            return;
        }
    };

    let midi_state_clone = Arc::clone(&midi_state);

    // callback function is defined before entering loop
    let conn_result = input.connect(
        in_port,
        "uncycle-midi-in",
        move |_timestamp, message, _| {
            midi_rx_callback(&midi_state_clone, message).unwrap();
        },
        (),
    );

    match conn_result {
        Ok(_conn) => {
            {
                let mut state = midi_state.lock().unwrap();
                state.log_misc(format!("Connected to MIDI in port: {}", port_name));
            }

            // Keep thread alive
            loop {
                thread::sleep(Duration::from_secs(1));
            }
        }
        Err(e) => {
            let mut state = midi_state.lock().unwrap();
            state.set_error(format!("Failed to connect to MIDI port: {}", e));
        }
    }
}

fn midi_output_thread(midi_state: Arc<Mutex<MidiState>>) {
    let output = match midir::MidiOutput::new("uncycle_midi_output") {
        Ok(output) => output,
        Err(e) => {
            let mut state = midi_state.lock().unwrap();
            state.set_error(format!("Failed to create MIDI output: {}", e));
            return;
        }
    };

    let out_ports = output.ports();

    if out_ports.is_empty() {
        let mut state = midi_state.lock().unwrap();
        state.set_error("No MIDI output ports available".to_string());
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
            let mut state = midi_state.lock().unwrap();
            state.set_error(format!("Failed to get output port name: {}", e));
            return;
        }
    };

    let conn_result = output.connect(out_port, "uncycle-midi-in");

    match conn_result {
        Ok(mut conn) => {
            {
                let mut state = midi_state.lock().unwrap();
                state.log_misc(format!("Connected to MIDI out port: {}", port_name));

                // auto-start for ease of debug
                const MIDI_START: &[u8] = &[0xFA];
                conn.send(MIDI_START).unwrap();

                state.output_connection = Some(conn);
            }

            loop {
                // poll @ 1kHz
                thread::sleep(Duration::from_millis(1));

                // callback function is being called inside loop via polling
                midi_tx_callback(&midi_state).unwrap();
            }
        }

        Err(e) => {
            let mut state = midi_state.lock().unwrap();
            state.set_error(format!("Failed to connect to MIDI port: {}", e));
        }
    }
}

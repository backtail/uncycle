use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use uncycle_core::MidiState;

pub fn setup_midi_socket(midi_state: Arc<Mutex<MidiState>>) {
    let midi_state_output = Arc::clone(&midi_state);

    thread::spawn(move || {
        midi_input_thread(midi_state);
    });

    thread::spawn(move || {
        midi_output_thread(midi_state_output);
    });
}

pub fn midi_input_thread(midi_state: Arc<Mutex<MidiState>>) {
    let input = match midir::MidiInput::new("uncycle_midi_input") {
        Ok(input) => input,
        Err(e) => {
            let mut state = midi_state.lock().unwrap();
            state.log_misc(format!("Failed to create MIDI input: {}", e));
            return;
        }
    };

    let in_ports = input.ports();

    if in_ports.is_empty() {
        let mut state = midi_state.lock().unwrap();
        state.log_misc("No MIDI input ports available".to_string());
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
            let mut state = midi_state.lock().unwrap();
            state.midi_rx_callback(message);
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

pub fn midi_output_thread(midi_state: Arc<Mutex<MidiState>>) {
    let output = match midir::MidiOutput::new("uncycle_midi_output") {
        Ok(output) => output,
        Err(e) => {
            let mut state = midi_state.lock().unwrap();
            state.log_misc(format!("Failed to create MIDI output: {}", e));
            return;
        }
    };

    let out_ports = output.ports();

    if out_ports.is_empty() {
        let mut state = midi_state.lock().unwrap();
        state.log_misc("No MIDI output ports available".to_string());
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
            state.log_misc(format!("Failed to get output port name: {}", e));
            return;
        }
    };

    let conn_result = output.connect(out_port, "uncycle-midi-in");

    match conn_result {
        Ok(mut conn) => {
            {
                let mut state = midi_state.lock().unwrap();
                state.port_out_name = Some(port_name.clone());
                state.log_misc(format!("Connected to MIDI out port: {}", port_name));
            }

            loop {
                // poll @ 1kHz
                thread::sleep(Duration::from_millis(1));

                let bytes;

                {
                    let mut state = midi_state.lock().unwrap();
                    bytes = state.midi_tx_callback();
                }

                conn.send(&bytes).ok();
            }
        }

        Err(e) => {
            let mut state = midi_state.lock().unwrap();
            state.log_misc(format!("Failed to connect to MIDI port: {}", e));
        }
    }
}

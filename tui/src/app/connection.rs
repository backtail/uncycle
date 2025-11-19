use crate::midi;

use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};

use midi::MidiState;

pub fn setup_midi_socket(midi_state: Arc<Mutex<MidiState>>, redraw_tx: mpsc::Sender<()>) {
    thread::spawn(move || {
        midi_input_thread(midi_state, redraw_tx);
    });
}

fn midi_input_thread(midi_state: Arc<Mutex<MidiState>>, redraw_tx: mpsc::Sender<()>) {
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
            continue;
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

    {
        let mut state = midi_state.lock().unwrap();
        state.add_message(format!("Connected to MIDI port: {}", port_name));
    }

    let midi_state_clone = Arc::clone(&midi_state);
    let redraw_tx_clone = redraw_tx.clone();

    let conn_result = input.connect(
        in_port,
        "uncycle-midi-in",
        move |_timestamp, message, _| {
            if message.len() >= 3 {
                let status = message[0];
                let data1 = message[1];
                let data2 = message[2];

                let message_type = status & 0xF0;
                let mut state = midi_state_clone.lock().unwrap();

                match message_type {
                    0x90 => {
                        // Note On
                        if data2 > 0 {
                            state.add_note(data1, data2);
                            state.add_message(format!("NOTE ON:  {:02} {:02}", data1, data2));
                        } else {
                            state.remove_note(data1);
                            state.add_message(format!("NOTE OFF: {:02}", data1));
                        }
                    }

                    0x80 => {
                        // Note Off
                        state.remove_note(data1);
                        state.add_message(format!("NOTE OFF: {:02}", data1));
                    }

                    0xB0 => {
                        // Control Change
                        state.update_cc(data1, data2);
                        state.add_message(format!("CC:       {:02} {:02}", data1, data2));
                    }

                    _ => {
                        state.add_message(format!(
                            "MIDI: {:02X} {:02X} {:02X}",
                            status, data1, data2
                        ));
                    }
                }

                let _ = redraw_tx_clone.send(());
            }
        },
        (),
    );

    match conn_result {
        Ok(_conn) => {
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

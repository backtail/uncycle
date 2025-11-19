use crate::midi;

use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use anyhow::Result;

use midi::MidiState;

pub fn setup_midi_socket(midi_state: Arc<Mutex<MidiState>>, redraw_tx: mpsc::Sender<()>) {
    let midi_state_output = Arc::clone(&midi_state);

    thread::spawn(move || {
        midi_input_thread(midi_state, redraw_tx);
    });

    thread::spawn(move || {
        midi_output_thread(midi_state_output);
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

    {
        let mut state = midi_state.lock().unwrap();
        state.add_message(format!("Connected to MIDI in port: {}", port_name));
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
        Ok(mut _conn) => {
            // Keep thread alive
            {
                let mut state = midi_state.lock().unwrap();
                state.add_message(format!("Connected to MIDI out port: {}", port_name));
                state.output_connection = Some(_conn);
            }
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

pub fn update_midi_clock(midi_state: &Arc<Mutex<MidiState>>) -> Result<()> {
    let mut state = midi_state.lock().unwrap();

    // if state.clock_running {
    if let Some(last_time) = state.last_clock_time {
        let interval = Duration::from_micros((60_000_000.0 / (state.clock_bpm * 24.0)) as u64);

        if last_time.elapsed() >= interval {
            if let Some(ref mut conn) = state.output_connection {
                const MIDI_CLOCK: &[u8] = &[0xF8];
                conn.send(MIDI_CLOCK)
                    .map_err(|e| anyhow::anyhow!("Failed to send MIDI Clock: {}", e))?;
                state.last_clock_time = Some(Instant::now());
                state.clock_pulse_count = state.clock_pulse_count.wrapping_add(1);
            }
        }
    } else {
        state.last_clock_time = Some(Instant::now());
    }
    // }
    Ok(())
}

use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use midir::{MidiIO, MidiInput, MidiOutput, MidiOutputConnection};
use uncycle_core::prelude::*;

use super::log::Logger;

pub fn setup_midi_socket(core: Arc<Mutex<UncycleCore>>, log: Arc<Mutex<Logger>>) {
    let core_arc_clone = Arc::clone(&core);
    let log_arc_clone = Arc::clone(&log);

    let now = Instant::now();

    thread::spawn(move || {
        midi_input_thread(core, log, now);
    });

    thread::spawn(move || {
        midi_output_thread(core_arc_clone, log_arc_clone, now);
    });
}

/////////////////////////////////////////////////////////////////////
// MIDI INPUT
/////////////////////////////////////////////////////////////////////

/// Tries to reconnect to a specific input port every 1 second, based on chosen device
pub fn midi_input_thread(core: Arc<Mutex<UncycleCore>>, log: Arc<Mutex<Logger>>, now: Instant) {
    loop {
        let app_input = match midir::MidiInput::new("uncycle_midi_input") {
            Ok(app_input) => app_input,
            Err(e) => {
                log.lock()
                    .unwrap()
                    .log_misc(format!("Failed to create MIDI input: {}", e));
                return;
            }
        };

        if core.lock().unwrap().device.is_some() {
            autoconnect_input(app_input, &core, now, &log);
        }

        // no need to keep track of time, if no device is actually connected
        thread::sleep(Duration::from_secs(1));
    }
}

fn autoconnect_input(
    app_input: MidiInput,
    core: &Arc<Mutex<UncycleCore>>,
    now: Instant,
    log: &Arc<Mutex<Logger>>,
) {
    if let Ok(device_in_port) = get_device_port(&app_input, &core, &log) {
        if let Ok(port_name) = get_port_name(&app_input, &device_in_port, &log) {
            let log_rx_callback = log.clone();
            let core_rx_callback = core.clone();

            match app_input.connect(
                &device_in_port,
                "uncycle-midi-in",
                move |_timestamp, message, _| {
                    input_callback(message, now, &core_rx_callback, &log_rx_callback)
                },
                (),
            ) {
                Err(e) => {
                    log.lock()
                        .unwrap()
                        .log_misc(format!("Unable to connect: {}", e));
                }

                Ok(_conn) => {
                    log_in_port(&log, port_name);
                    loop {
                        // higher precision time keeping
                        thread::sleep(Duration::from_millis(16));

                        if core.lock().unwrap().kill_rx_conn {
                            continue;
                        }
                    }
                }
            }
        }
    }
}

fn input_callback(
    message: &[u8],
    now: Instant,
    core: &Arc<Mutex<UncycleCore>>,
    log: &Arc<Mutex<Logger>>,
) {
    // first handle midi logic
    let elapsed = now.elapsed().as_micros() as u64;

    core.lock().unwrap().midi_rx_callback(message);

    // then handle logging
    if let Some(msg) = parse_midi_message(message) {
        let status = message[0];
        let data1 = message[1];
        let data2 = message[2];

        match msg {
            MIDI_NOTE_ON => log
                .lock()
                .unwrap()
                .log_incoming_note(format!("NOTE ON:  {:02} {:02}", data1, data2)),

            MIDI_NOTE_OFF => {}

            MIDI_CONTORL_CHANGE => log.lock().unwrap().log_incoming_cc(format!(
                "[{} ms {:3} ns] {} {}",
                elapsed / 1000,
                elapsed % 1000,
                data1,
                data2,
            )),

            _ => log
                .lock()
                .unwrap()
                .log_misc(format!("MIDI: {:02X} {:02X} {:02X}", status, data1, data2)),
        }
    }
}

fn log_in_port(log: &Arc<Mutex<Logger>>, port_name: String) {
    let mut locked = log.lock().unwrap();
    locked.port_in_name = Some(port_name.clone());
    locked.log_misc(format!("Connected to in port: {}", port_name));
}

/////////////////////////////////////////////////////////////////////
// MIDI OUTPUT
/////////////////////////////////////////////////////////////////////

/// Tries to reconnect to a specific output port every 1 second, based on chosen device
pub fn midi_output_thread(core: Arc<Mutex<UncycleCore>>, log: Arc<Mutex<Logger>>, now: Instant) {
    loop {
        let app_output = match midir::MidiOutput::new("uncycle_midi_output") {
            Ok(app_output) => app_output,
            Err(e) => {
                log.lock()
                    .unwrap()
                    .log_misc(format!("Failed to create MIDI output: {}", e));
                return;
            }
        };

        if core.lock().unwrap().device.is_some() {
            autoconnect_output(app_output, &core, now, &log);
        }

        // no need to keep track of time, if no device is actually connected
        thread::sleep(Duration::from_secs(1));
    }
}

fn autoconnect_output(
    app_output: MidiOutput,
    core: &Arc<Mutex<UncycleCore>>,
    now: Instant,
    log: &Arc<Mutex<Logger>>,
) {
    if let Ok(device_out_port) = get_device_port(&app_output, &core, &log) {
        if let Ok(port_name) = get_port_name(&app_output, &device_out_port, &log) {
            match app_output.connect(&device_out_port, "uncycle-midi-out") {
                Err(e) => {
                    log.lock()
                        .unwrap()
                        .log_misc(format!("Failed to connect to MIDI port: {}", e));
                }

                Ok(mut conn) => {
                    log_out_port(&log, port_name);

                    loop {
                        output_callback(&mut conn, now, &core, &log);

                        if core.lock().unwrap().kill_tx_conn {
                            continue;
                        }

                        // poll @ 1kHz, thread timing accuracy does not matter too much since we pass time as paramter to callback
                        // therefor poll rate is what we care about
                        thread::sleep(Duration::from_micros(100));
                    }
                }
            }
        }
    }
}

fn output_callback(
    conn: &mut MidiOutputConnection,
    now: Instant,
    core: &Arc<Mutex<UncycleCore>>,
    log: &Arc<Mutex<Logger>>,
) {
    let elapsed = now.elapsed().as_micros() as u64;

    {
        core.lock().unwrap().update_time(elapsed);
    }

    let bytes;

    {
        bytes = core.lock().unwrap().midi_tx_callback();
    }

    // send MIDI outside of lock
    conn.send(&bytes).ok();

    // log after sending

    for byte in &bytes {
        if *byte == MIDI_START {
            log.lock()
                .unwrap()
                .log_misc(format!("Send: 0x{:02X} (MIDI Start)", MIDI_START));
        }

        if *byte == MIDI_CONTINUE {
            log.lock()
                .unwrap()
                .log_misc(format!("Send: 0x{:02X} (MIDI Continue)", MIDI_CONTINUE));
        }

        if *byte == MIDI_STOP {
            log.lock()
                .unwrap()
                .log_misc(format!("Send: 0x{:02X} (MIDI Stop)", MIDI_STOP));
        }

        if (*byte) & 0xF0 == MIDI_CONTORL_CHANGE {
            log.lock().unwrap().log_outgoing_cc(format!(
                "[{} ms {:3} ns] CC",
                elapsed / 1000,
                elapsed % 1000,
            ));
        }
    }
}

fn log_out_port(log: &Arc<Mutex<Logger>>, port_name: String) {
    let mut locked = log.lock().unwrap();
    locked.port_out_name = Some(port_name.clone());
    locked.log_misc(format!("Connected to out port: {}", port_name));
}

/////////////////////////////////////////////////////////////////////
// HELPERS
/////////////////////////////////////////////////////////////////////

fn get_device_name(core: &Arc<Mutex<UncycleCore>>) -> Option<String> {
    if let Some(device) = &core.lock().unwrap().device {
        Some(device.name_to_str().to_string())
    } else {
        None
    }
}

fn get_device_port<M: MidiIO>(
    app_input: &M,
    core: &Arc<Mutex<UncycleCore>>,
    log: &Arc<Mutex<Logger>>,
) -> Result<M::Port, ()> {
    let other_in_ports = app_input.ports();

    if other_in_ports.is_empty() {
        log.lock()
            .unwrap()
            .log_misc("No MIDI ports available".to_string());
        return Err(());
    }

    if let Some(name) = get_device_name(&core).as_ref() {
        for (i, port) in other_in_ports.iter().enumerate() {
            if app_input.port_name(port).unwrap().contains(name) {
                return Ok(other_in_ports[i].clone());
            }
        }

        log.lock()
            .unwrap()
            .log_misc(format!("No MIDI ports named {} available", name));
    }

    Err(())
}

fn get_port_name<M: MidiIO>(
    app_input: &M,
    device_in_port: &M::Port,
    log: &Arc<Mutex<Logger>>,
) -> Result<String, ()> {
    match app_input.port_name(device_in_port) {
        Ok(name) => Ok(name),
        Err(e) => {
            log.lock()
                .unwrap()
                .log_misc(format!("Failed to get port name: {}", e));
            Err(())
        }
    }
}

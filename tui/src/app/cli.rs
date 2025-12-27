use std::{fmt::Debug, process::exit};

use clap::{Parser, ValueEnum};

use crate::app::{menu::Setting};
use uncycle_core::{devices::{SupportedDevice, TR8}, prelude::{DeviceInterface, UncycleCore}};
use strum::{EnumIter, IntoEnumIterator};

#[derive(Debug, Parser)]
#[command(version, long_about = None)]
struct Cli {
    #[arg(short, long, value_enum, default_value_t = RecMode::OneShot)]
    rec_mode: RecMode,

    #[arg(short, long, help = "Select desired device", default_value_t = SupportedDevice::TR8(TR8::default()))]
    device: SupportedDevice,

    #[arg(required = false, short, long, help = "Lists all supported devices")]
    list_devices: bool,

}

#[derive(Debug, Clone, ValueEnum, EnumIter, PartialEq, Eq)]
pub enum RecMode {
    #[value(alias("one-shot"))]
    OneShot,

    #[value(alias("continous"))]
    Continous,
}

pub fn parse_cli() -> Vec<Setting> {
    let args = Cli::parse();

    if args.list_devices == true {
        show_supported_devices() // exits program
    }  

    let mut settings = Vec::new();

    parse_mode(&args, &mut settings);
    parse_device(&args, &mut settings);

    settings
}

fn show_supported_devices() -> ! {
    println!("Supported devices:");

    for device in SupportedDevice::iter() {
        println!("  {}", device.id_to_str());
    }

    exit(0);
}

fn parse_mode(args: &Cli, settings_vec: &mut Vec<Setting>) {
    let mut index = 0;
    let mut options = Vec::new();

    for (i, mode) in RecMode::iter().enumerate() {
        options.push(format!("{:?}", mode));

        if mode == args.rec_mode {
            index = i;
        }
    }

    settings_vec.push(
        Setting {
            name: "Recording".to_string(), 
            description: "Select Recording Mode".to_string(), 
            options: options, 
            selected_option: index,
            apply_fn: nop,
        }
    );
}

fn parse_device(args: &Cli, settings_vec: &mut Vec<Setting>) {
    let mut index = 0;
    let mut options = Vec::new();

    for (i, device) in SupportedDevice::iter().enumerate() {
        options.push(device.manufacturer_to_str().to_string() + " " + &device.name_to_str());

        if device.id_to_str().eq(&args.device.id_to_str()) {
            index = i;
        }
    }

    settings_vec.push(
        Setting {
            name: "Device".to_string(), 
            description: "Select Supported Device".to_string(), 
            options: options, 
            selected_option: index,
            apply_fn: change_device,
        }
    );
}

fn nop(_core: &mut UncycleCore, _setting: &Setting) {}

fn change_device(core: &mut UncycleCore, setting: &Setting) {
     for device in SupportedDevice::iter() {
        // very ugly... but works
        if setting.options[setting.selected_option].eq(&(device.manufacturer_to_str().to_string() + " " + &device.name_to_str())) {
            core.set_device(device);
        }
    }
}
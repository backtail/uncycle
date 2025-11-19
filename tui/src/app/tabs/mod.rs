mod device;
mod help;
mod main;
mod midi;
mod settings;

pub use device::render_device_tab;
pub use help::render_help_tab;
pub use main::render_main_tab;
pub use midi::render_midi_tab;
pub use settings::render_settings_tab;

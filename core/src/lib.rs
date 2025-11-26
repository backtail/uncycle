#![no_std]

#[cfg(feature = "std")]
extern crate std;

pub mod devices;

mod core;
mod midi;

pub mod prelude {
    pub use crate::core::UncycleCore;
    pub use crate::midi::parse_midi_message;
}

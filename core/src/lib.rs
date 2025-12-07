#![no_std]

#[cfg(feature = "std")]
extern crate std;

pub mod devices;

mod core;
mod looper;
mod midi;

pub mod prelude {
    pub use crate::core::UncycleCore;
    pub use crate::devices::{DeviceInterface, SupportedDevice};
    pub use crate::midi::*;
}

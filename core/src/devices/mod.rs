pub mod tr8;

pub use tr8::TR8;

#[cfg(feature = "std")]
use std::{string::ToString, str::FromStr};

#[cfg(feature = "std")]
impl ToString for SupportedDevice {
    fn to_string(&self) -> std::string::String {
        self.id_to_str().to_string()
    }
}

#[cfg(feature = "std")]
impl FromStr for SupportedDevice {
    type Err = std::fmt::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for device in SupportedDevice::iter() {
            if device.id_to_str().eq(s) {
                return Ok(device);
            }
        }
        
        Err(std::fmt::Error)
    }
}

use strum::EnumIter;
use strum::IntoEnumIterator;
use enum_dispatch::enum_dispatch;
use heapless::String;


#[enum_dispatch(DeviceInterface)]
#[derive(Clone, Debug, EnumIter)]
pub enum SupportedDevice {
    TR8(TR8),
}


#[enum_dispatch]
pub trait DeviceInterface {
    fn run(&mut self);
    fn stop(&mut self);
    fn is_running(&self) -> bool;

    fn name_to_str(&self) -> String<64>;
    fn manufacturer_to_str(&self) -> String<64>;
    fn id_to_str(&self) -> String<64>;
}
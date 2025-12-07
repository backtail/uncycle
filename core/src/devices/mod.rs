pub mod tr8;

pub use tr8::TR8;

use enum_dispatch::enum_dispatch;
use heapless::String;

#[enum_dispatch(DeviceInterface)]
#[derive(Clone)]
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
}

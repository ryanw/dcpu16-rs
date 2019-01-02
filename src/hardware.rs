use super::Processor;
use downcast_rs::{impl_downcast, Downcast};

pub trait HardwareDevice: Downcast {
    fn id(&self) -> u32;
    fn version(&self) -> u16;
    fn manufacturer(&self) -> u32;
    fn handle_interrupt(&mut self, _processor: &Processor) {}
}
impl_downcast!(HardwareDevice);

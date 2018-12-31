use super::*;

pub struct Monitor {
    pub screen_addr: u16,
    pub font_addr: u16,
    pub palette_addr: u16,
    pub border_color: u16,
}

impl Monitor {
    pub fn new() -> Monitor {
        Monitor {
            screen_addr: 0x0,
            font_addr: 0x0,
            palette_addr: 0x0,
            border_color: 0x0,
        }
    }
}

impl HardwareDevice for Monitor {
    fn id(&self) -> u32 {
        0x7349F615
    }
    fn version(&self) -> u16 {
        0x1802
    }
    fn manufacturer(&self) -> u32 {
        0x1C6C8B36
    }
    fn handle_interrupt(&mut self, processor: &Processor) {
        let op = processor.get_register(A);
        let param = processor.get_register(B);

        match op {
            0x00 => {
                self.screen_addr = param;
            }
            _ => {}
        }
    }
}

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

    pub fn render_ansi(&self, processor: &Processor) -> String {
        let mut output = "".to_owned();
        output += "\x1b[0;0H";
        for y in 0..12 {
            if y > 0 {
                output += "\n";
            }
            for x in 0..32 {
                let i = x + y * 32;
                let cell = processor.get_memory(self.screen_addr + i);
                let c = (cell & 0b0000000001111111) as u8;
                let f = ((cell & 0b1111000000000000) >> 12) as u8;
                let b = ((cell & 0b0000111100000000) >> 8) as u8;
                let x = ((cell & 0b0000000010000000) >> 7) as u8;
                let fg = self.get_ansi_color(processor, f);
                let bg = self.get_ansi_color(processor, b);
                let tile = "  ";
                output += &format!("\x1b[48;5;{}m{}", fg, tile);
                // Back 2, down 1
                output += "\x1b[2D\x1b[B";
                output += &format!("\x1b[48;5;{}m{}", bg, tile);
                // Up 1
                output += "\x1b[A";
            }
            // Down 1
            output += "\x1b[B";
        }

        output
    }

    pub fn render_24bit_ansi(&self, processor: &Processor) -> String {
        let mut output = "".to_owned();
        output += "\x1b[0;0H";
        for y in 0..12 {
            if y > 0 {
                output += "\n";
            }
            for x in 0..32 {
                let i = x + y * 32;
                let cell = processor.get_memory(self.screen_addr + i);
                let c = (cell & 0b0000000001111111) as u8;
                let f = ((cell & 0b1111000000000000) >> 12) as u8;
                let b = ((cell & 0b0000111100000000) >> 8) as u8;
                let x = ((cell & 0b0000000010000000) >> 7) as u8;
                let fg = self.get_24bit_ansi_color(processor, f);
                let bg = self.get_24bit_ansi_color(processor, b);
                let tile = " ";
                output += &format!("\x1b[48;2;{}m{}", fg, tile);
                // Back 2, down 1
                output += "\x1b[1D\x1b[B";
                output += &format!("\x1b[48;2;{}m{}", bg, tile);
                // Up 1
                output += "\x1b[A";
            }
            // Down 1
            output += "\x1b[B";
        }

        output
    }

    pub fn get_ansi_color(&self, processor: &Processor, index: u8) -> u16 {
        let color = processor.get_memory(self.palette_addr + index as u16);
        let r = ((color & 0b0000111100000000) >> 8) / 3;
        let g = ((color & 0b0000000011110000) >> 4) / 3;
        let b = (color & 0b0000000000001111) / 3;

        let ansi_color = 16 + 36 * r + 6 * g + b;
        ansi_color
    }

    pub fn get_24bit_ansi_color(&self, processor: &Processor, index: u8) -> String {
        let color = processor.get_memory(self.palette_addr + index as u16);
        let r = ((color & 0b0000111100000000) >> 8) * 16;
        let g = ((color & 0b0000000011110000) >> 4) * 16;
        let b = (color & 0b0000000000001111) * 16;

        format!("{};{};{}", r, g, b)
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
            0x00 => self.screen_addr = param,
            0x01 => self.font_addr = param,
            0x02 => self.palette_addr = param,
            0x03 => self.border_color = param,
            _ => {}
        }
    }
}

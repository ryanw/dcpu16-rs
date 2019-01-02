use super::hardware::HardwareDevice;
use super::processor::Processor;
use super::processor::Register::*;

const DEFAULT_FONT: [u16; 256] = [
    0x000f, 0x0808, 0x080f, 0x0808, 0x08f8, 0x0808, 0x00ff, 0x0808, 0x0808, 0x0808, 0x08ff, 0x0808,
    0x00ff, 0x1414, 0xff00, 0xff08, 0x1f10, 0x1714, 0xfc04, 0xf414, 0x1710, 0x1714, 0xf404, 0xf414,
    0xff00, 0xf714, 0x1414, 0x1414, 0xf700, 0xf714, 0x1417, 0x1414, 0x0f08, 0x0f08, 0x14f4, 0x1414,
    0xf808, 0xf808, 0x0f08, 0x0f08, 0x001f, 0x1414, 0x00fc, 0x1414, 0xf808, 0xf808, 0xff08, 0xff08,
    0x14ff, 0x1414, 0x080f, 0x0000, 0x00f8, 0x0808, 0xffff, 0xffff, 0xf0f0, 0xf0f0, 0xffff, 0x0000,
    0x0000, 0xffff, 0x0f0f, 0x0f0f, 0x0000, 0x0000, 0x005f, 0x0000, 0x0300, 0x0300, 0x3e14, 0x3e00,
    0x266b, 0x3200, 0x611c, 0x4300, 0x3629, 0x7650, 0x0002, 0x0100, 0x1c22, 0x4100, 0x4122, 0x1c00,
    0x2a1c, 0x2a00, 0x083e, 0x0800, 0x4020, 0x0000, 0x0808, 0x0800, 0x0040, 0x0000, 0x601c, 0x0300,
    0x3e41, 0x3e00, 0x427f, 0x4000, 0x6259, 0x4600, 0x2249, 0x3600, 0x0f08, 0x7f00, 0x2745, 0x3900,
    0x3e49, 0x3200, 0x6119, 0x0700, 0x3649, 0x3600, 0x2649, 0x3e00, 0x0024, 0x0000, 0x4024, 0x0000,
    0x0814, 0x2241, 0x1414, 0x1400, 0x4122, 0x1408, 0x0259, 0x0600, 0x3e59, 0x5e00, 0x7e09, 0x7e00,
    0x7f49, 0x3600, 0x3e41, 0x2200, 0x7f41, 0x3e00, 0x7f49, 0x4100, 0x7f09, 0x0100, 0x3e49, 0x3a00,
    0x7f08, 0x7f00, 0x417f, 0x4100, 0x2040, 0x3f00, 0x7f0c, 0x7300, 0x7f40, 0x4000, 0x7f06, 0x7f00,
    0x7f01, 0x7e00, 0x3e41, 0x3e00, 0x7f09, 0x0600, 0x3e41, 0xbe00, 0x7f09, 0x7600, 0x2649, 0x3200,
    0x017f, 0x0100, 0x7f40, 0x7f00, 0x1f60, 0x1f00, 0x7f30, 0x7f00, 0x7708, 0x7700, 0x0778, 0x0700,
    0x7149, 0x4700, 0x007f, 0x4100, 0x031c, 0x6000, 0x0041, 0x7f00, 0x0201, 0x0200, 0x8080, 0x8000,
    0x0001, 0x0200, 0x2454, 0x7800, 0x7f44, 0x3800, 0x3844, 0x2800, 0x3844, 0x7f00, 0x3854, 0x5800,
    0x087e, 0x0900, 0x4854, 0x3c00, 0x7f04, 0x7800, 0x447d, 0x4000, 0x2040, 0x3d00, 0x7f10, 0x6c00,
    0x417f, 0x4000, 0x7c18, 0x7c00, 0x7c04, 0x7800, 0x3844, 0x3800, 0x7c14, 0x0800, 0x0814, 0x7c00,
    0x7c04, 0x0800, 0x4854, 0x2400, 0x043e, 0x4400, 0x3c40, 0x7c00, 0x1c60, 0x1c00, 0x7c30, 0x7c00,
    0x6c10, 0x6c00, 0x4c50, 0x3c00, 0x6454, 0x4c00, 0x0836, 0x4100, 0x0077, 0x0000, 0x4136, 0x0800,
    0x0201, 0x0201, 0x704c, 0x7000,
];

const DEFAULT_PALETTE: [u16; 16] = [
    0x000, 0x00a, 0x0a0, 0x0aa, 0xa00, 0xa0a, 0xa50, 0xaaa, 0x555, 0x55f, 0x5f5, 0x5ff, 0xf55,
    0xf5f, 0xff5, 0xfff,
];

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
                output += "\x1b[3B\n";
            }
            for x in 0..32 {
                let i = x + y * 32;
                let cell = processor.get_memory(self.screen_addr + i);
                let c = (cell & 0b0000000001111111) as u16;
                let f = ((cell & 0b1111000000000000) >> 12) as u8;
                let b = ((cell & 0b0000111100000000) >> 8) as u8;
                // FIXME support blinking
                let x = ((cell & 0b0000000010000000) >> 7) as u8;
                let fg = self.get_ansi_color(processor, f);
                let bg = self.get_ansi_color(processor, b);
                let tile = self.get_font_char(processor, c);
                output += &format!("\x1b[38;5;{}m\x1b[48;5;{}m{}", fg, bg, tile);
            }
        }

        output
    }

    pub fn render_24bit_ansi(&self, processor: &Processor) -> String {
        let mut output = "".to_owned();
        output += "\x1b[0;0H";
        for y in 0..12 {
            if y > 0 {
                output += "\x1b[3B\n";
            }
            for x in 0..32 {
                let i = x + y * 32;
                let cell = processor.get_memory(self.screen_addr + i);
                let c = (cell & 0b0000000001111111) as u16;
                let f = ((cell & 0b1111000000000000) >> 12) as u8;
                let b = ((cell & 0b0000111100000000) >> 8) as u8;
                // FIXME support blinking
                let x = ((cell & 0b0000000010000000) >> 7) as u8;
                let fg = self.get_24bit_ansi_color(processor, f);
                let bg = self.get_24bit_ansi_color(processor, b);
                let tile = self.get_font_char(processor, c);
                output += &format!("\x1b[38;2;{}m\x1b[48;2;{}m{}", fg, bg, tile);
            }
        }

        output
    }

    pub fn get_ansi_color(&self, processor: &Processor, index: u8) -> u16 {
        let color = if self.palette_addr > 0 {
            processor.get_memory(self.palette_addr + index as u16)
        } else {
            DEFAULT_PALETTE[index as usize]
        };
        let r = ((color & 0b0000111100000000) >> 8) / 3;
        let g = ((color & 0b0000000011110000) >> 4) / 3;
        let b = (color & 0b0000000000001111) / 3;

        let ansi_color = 16 + 36 * r + 6 * g + b;
        ansi_color
    }

    pub fn get_24bit_ansi_color(&self, processor: &Processor, index: u8) -> String {
        let color = if self.palette_addr > 0 {
            processor.get_memory(self.palette_addr + index as u16)
        } else {
            DEFAULT_PALETTE[index as usize]
        };
        let r = ((color & 0b0000111100000000) >> 8) * 16;
        let g = ((color & 0b0000000011110000) >> 4) * 16;
        let b = (color & 0b0000000000001111) * 16;

        format!("{};{};{}", r, g, b)
    }

    pub fn get_font_char(&self, processor: &Processor, index: u16) -> String {
        let addr = index * 2; // 2 words per char
        let word0 = if self.font_addr > 0 {
            processor.get_memory(self.font_addr + addr)
        } else {
            DEFAULT_FONT[addr as usize]
        };
        let word1 = if self.font_addr > 0 {
            processor.get_memory(self.font_addr + addr + 1)
        } else {
            DEFAULT_FONT[addr as usize + 1]
        };

        let pixels = ((word0 as u32) << 16) + word1 as u32;
        self.get_wide_8x4_char(pixels)
    }

    pub fn get_8x4_char(&self, pixels: u32) -> String {
        let col0 = ((pixels & 0xFF000000) >> 24) as u16;
        let col1 = ((pixels & 0x00FF0000) >> 16) as u16;
        let col2 = ((pixels & 0x0000FF00) >> 8) as u16;
        let col3 = ((pixels & 0x000000FF) >> 0) as u16;

        let block0 = (col0 & 0b00000011) + ((col1 & 0b00000011) << 2);
        let block1 = (col2 & 0b00000011) + ((col3 & 0b00000011) << 2);

        let block2 = ((col0 & 0b00001100) >> 2) + ((col1 & 0b00001100) << 0);
        let block3 = ((col2 & 0b00001100) >> 2) + ((col3 & 0b00001100) << 0);

        let block4 = ((col0 & 0b00110000) >> 4) + ((col1 & 0b00110000) >> 2);
        let block5 = ((col2 & 0b00110000) >> 4) + ((col3 & 0b00110000) >> 2);

        let block6 = ((col0 & 0b11000000) >> 6) + ((col1 & 0b11000000) >> 4);
        let block7 = ((col2 & 0b11000000) >> 6) + ((col3 & 0b11000000) >> 4);

        format!(
            "{}{}\x1b[2D\x1b[B{}{}\x1b[2D\x1b[B{}{}\x1b[2D\x1b[B{}{}\x1b[3A",
            self.get_2x2_char(block0),
            self.get_2x2_char(block1),
            self.get_2x2_char(block2),
            self.get_2x2_char(block3),
            self.get_2x2_char(block4),
            self.get_2x2_char(block5),
            self.get_2x2_char(block6),
            self.get_2x2_char(block7),
        )
    }

    pub fn get_wide_8x4_char(&self, pixels: u32) -> String {
        let col0 = ((pixels & 0xFF000000) >> 24) as u16;
        let col1 = ((pixels & 0x00FF0000) >> 16) as u16;
        let col2 = ((pixels & 0x0000FF00) >> 8) as u16;
        let col3 = ((pixels & 0x000000FF) >> 0) as u16;

        let block0 = (col0 & 0b00000011) + ((col1 & 0b00000011) << 2);
        let block1 = (col2 & 0b00000011) + ((col3 & 0b00000011) << 2);

        let block2 = ((col0 & 0b00001100) >> 2) + ((col1 & 0b00001100) << 0);
        let block3 = ((col2 & 0b00001100) >> 2) + ((col3 & 0b00001100) << 0);

        let block4 = ((col0 & 0b00110000) >> 4) + ((col1 & 0b00110000) >> 2);
        let block5 = ((col2 & 0b00110000) >> 4) + ((col3 & 0b00110000) >> 2);

        let block6 = ((col0 & 0b11000000) >> 6) + ((col1 & 0b11000000) >> 4);
        let block7 = ((col2 & 0b11000000) >> 6) + ((col3 & 0b11000000) >> 4);

        format!(
            "{}{}\x1b[4D\x1b[B{}{}\x1b[4D\x1b[B{}{}\x1b[4D\x1b[B{}{}\x1b[3A",
            self.get_wide_2x2_char(block0),
            self.get_wide_2x2_char(block1),
            self.get_wide_2x2_char(block2),
            self.get_wide_2x2_char(block3),
            self.get_wide_2x2_char(block4),
            self.get_wide_2x2_char(block5),
            self.get_wide_2x2_char(block6),
            self.get_wide_2x2_char(block7),
        )
    }

    pub fn get_2x2_char(&self, pixels: u16) -> String {
        match pixels {
            0x0 => " ".to_owned(),

            0x1 => "▘".to_owned(),

            0x2 => "▖".to_owned(),

            0x3 => "▌".to_owned(),

            0x4 => "▝".to_owned(),

            0x5 => "▀".to_owned(),

            0x6 => "▞".to_owned(),

            0x7 => "▛".to_owned(),

            0x8 => "▗".to_owned(),

            0x9 => "▚".to_owned(),

            0xa => "▄".to_owned(),

            0xb => "▙".to_owned(),

            0xc => "▐".to_owned(),

            0xd => "▜".to_owned(),

            0xe => "▟".to_owned(),

            0xF => "\x1b[7m \x1b[27m".to_owned(),

            _ => "X".to_owned(),
        }
    }

    pub fn get_wide_2x2_char(&self, pixels: u16) -> String {
        match pixels {
            0x0 => "  ".to_owned(),

            0x1 => "▀ ".to_owned(),

            0x2 => "▄ ".to_owned(),

            0x3 => "\x1b[7m \x1b[27m ".to_owned(),

            0x4 => " ▀".to_owned(),

            0x5 => "▀▀".to_owned(),

            0x6 => "▄▀".to_owned(),

            0x7 => "\x1b[7m \x1b[27m▀".to_owned(),

            0x8 => " ▄".to_owned(),

            0x9 => "▀▄".to_owned(),

            0xa => "▄▄".to_owned(),

            0xb => "\x1b[7m \x1b[27m▄".to_owned(),

            0xc => " \x1b[7m \x1b[27m".to_owned(),

            0xd => "▀\x1b[7m \x1b[27m".to_owned(),

            0xe => "▄\x1b[7m \x1b[27m".to_owned(),

            0xF => "\x1b[7m  \x1b[27m".to_owned(),

            _ => "X".to_owned(),
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
            0x00 => self.screen_addr = param,
            0x01 => self.font_addr = param,
            0x02 => self.palette_addr = param,
            0x03 => self.border_color = param,
            _ => {}
        }
    }
}

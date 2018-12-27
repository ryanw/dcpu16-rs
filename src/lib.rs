use std::mem;

#[derive(Copy, Clone, Debug)]
pub enum Registers {
    A = 0x0000,
    B,
    C,
    X,
    Y,
    Z,
    I,
    J,
    PC,
    SP,
    EX,
    IA,
}
use self::Registers::*;

pub struct Processor {
    memory: Memory,
    registers: [u16; 12],
    cycle_wait: u8,
}

impl Processor {
    pub fn new() -> Processor {
        let mut cpu = Processor {
            memory: Memory::new(),
            registers: [0; 12],
            cycle_wait: 0,
        };
        // Start stack pointer at the end
        cpu.registers[SP as usize] = 0xffff;

        cpu
    }

    pub fn tick(&mut self) {
        if self.cycle_wait > 0 {
            self.cycle_wait -= 1;
            return;
        }
    }

    pub fn get_register(&self, register: Registers) -> u16 {
        self.registers[register as usize]
    }

    pub fn get_signed_register(&self, register: Registers) -> i16 {
        unsafe { mem::transmute(self.registers[register as usize]) }
    }

    pub fn execute_set(&mut self, register: Registers, value: u16) {
        self.registers[register as usize] = value;
    }

    pub fn execute_add(&mut self, register: Registers, value: u16) {
        self.cycle_wait += 1;

        let old_value = self.registers[register as usize];
        let (new_value, overflowed) = old_value.overflowing_add(value);
        self.registers[register as usize] = new_value;
        if overflowed {
            self.registers[EX as usize] = 0x0001;
        } else {
            self.registers[EX as usize] = 0x0000;
        }
    }

    pub fn execute_sub(&mut self, register: Registers, value: u16) {
        self.cycle_wait += 1;

        let old_value = self.registers[register as usize];
        let (new_value, overflowed) = old_value.overflowing_sub(value);
        self.registers[register as usize] = new_value;
        if overflowed {
            self.registers[EX as usize] = 0xFFFF;
        } else {
            self.registers[EX as usize] = 0x0000;
        }
    }

    pub fn execute_mul(&mut self, register: Registers, value: u16) {
        self.cycle_wait += 1;

        let old_value = self.registers[register as usize];
        let new_value = old_value.wrapping_mul(value);
        self.registers[register as usize] = new_value;
        self.registers[EX as usize] = (((old_value as u32 * value as u32) >> 16) & 0xFFFF) as u16;
    }

    pub fn execute_div(&mut self, register: Registers, value: u16) {
        self.cycle_wait += 2;

        if value == 0 {
            self.registers[register as usize] = 0;
            self.registers[EX as usize] = 0;
            return;
        }
        let old_value = self.registers[register as usize];
        let new_value = old_value.wrapping_div(value);
        self.registers[register as usize] = new_value;
        self.registers[EX as usize] = ((((old_value as u32) << 16) / value as u32) & 0xFFFF) as u16;
    }

    pub fn execute_mli(&mut self, register: Registers, value: i16) {
        self.cycle_wait += 1;

        let old_value: i16 = unsafe { mem::transmute(self.registers[register as usize]) };
        let new_value: i16 = old_value.wrapping_mul(value);
        self.registers[register as usize] = unsafe { mem::transmute(new_value) };
        self.registers[EX as usize] =
            unsafe { mem::transmute((((old_value as i32 * value as i32) >> 16) & 0xFFFF) as i16) };
    }

    pub fn execute_dvi(&mut self, register: Registers, value: i16) {
        self.cycle_wait += 2;

        if value == 0 {
            self.registers[register as usize] = 0;
            self.registers[EX as usize] = 0;
            return;
        }
        let old_value: i16 = unsafe { mem::transmute(self.registers[register as usize]) };
        let new_value: i16 = old_value.wrapping_div(value);
        self.registers[register as usize] = unsafe { mem::transmute(new_value) };
        self.registers[EX as usize] = unsafe {
            mem::transmute(((((old_value as i32) << 16) / value as i32) & 0xFFFF) as i16)
        };
    }

    pub fn execute_mod(&mut self, register: Registers, value: u16) {
        self.cycle_wait += 2;

        if value == 0 {
            self.registers[register as usize] = 0;
            return;
        }

        let old_value = self.registers[register as usize];
        self.registers[register as usize] = old_value % value;
    }

    pub fn execute_mdi(&mut self, register: Registers, value: i16) {
        self.cycle_wait += 2;

        if value == 0 {
            self.registers[register as usize] = 0;
            return;
        }

        let old_value: i16 = unsafe { mem::transmute(self.registers[register as usize]) };
        self.registers[register as usize] = unsafe { mem::transmute(old_value % value) };
    }
}

pub struct Memory([u16; 0x10000]);
impl Memory {
    pub fn new() -> Memory {
        Memory([0; 0x10000])
    }
}
pub struct Register(u16);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_value_into_the_a_register() {
        let mut machine = Processor::new();
        machine.execute_set(A, 0x000E);
        assert_eq!(machine.get_register(A), 0x000E);
    }

    #[test]
    fn add_value_to_the_a_register() {
        let mut machine = Processor::new();
        machine.execute_set(A, 0x0003);
        machine.execute_add(A, 0x0002);
        assert_eq!(machine.get_register(A), 0x0005);
        assert_eq!(machine.get_register(EX), 0x0000);
    }

    #[test]
    fn add_overflow_value_to_the_a_register() {
        let mut machine = Processor::new();
        machine.execute_set(A, 0xFFFD);
        machine.execute_add(A, 0x0005);
        assert_eq!(machine.get_register(A), 0x0002);
        assert_eq!(machine.get_register(EX), 0x0001);
    }

    #[test]
    fn sub_value_from_the_a_register() {
        let mut machine = Processor::new();
        machine.execute_set(A, 0x0005);
        machine.execute_sub(A, 0x0002);
        assert_eq!(machine.get_register(A), 0x0003);
        assert_eq!(machine.get_register(EX), 0x0000);
    }

    #[test]
    fn sub_underflow_value_from_the_a_register() {
        let mut machine = Processor::new();
        machine.execute_set(A, 0x0002);
        machine.execute_sub(A, 0x0005);
        assert_eq!(machine.get_register(A), 0xFFFD);
        assert_eq!(machine.get_register(EX), 0xFFFF);
    }

    #[test]
    fn mul_value_from_the_a_register() {
        let mut machine = Processor::new();
        machine.execute_set(A, 0x0005);
        machine.execute_mul(A, 0x0002);
        assert_eq!(machine.get_register(A), 0x000A);
        assert_eq!(machine.get_register(EX), 0x0000);
    }

    #[test]
    fn mul_underflow_value_from_the_a_register() {
        let mut machine = Processor::new();
        machine.execute_set(A, 0xFFF0);
        machine.execute_mul(A, 0x0003);
        assert_eq!(machine.get_register(A), 0xFFD0);
        assert_eq!(machine.get_register(EX), 0x0002);
    }

    #[test]
    fn div_whole_value_from_the_a_register() {
        let mut machine = Processor::new();
        machine.execute_set(A, 0x000A);
        machine.execute_div(A, 0x0005);
        assert_eq!(machine.get_register(A), 0x0002);
        assert_eq!(machine.get_register(EX), 0x0000);
    }

    #[test]
    fn div_remainder_value_from_the_a_register() {
        let mut machine = Processor::new();
        machine.execute_set(A, 0x000B);
        machine.execute_div(A, 0x0004);
        assert_eq!(machine.get_register(A), 0x0002);
        assert_eq!(machine.get_register(EX), 0xC000); // 0.75
    }

    #[test]
    fn div_the_a_register_by_zero() {
        let mut machine = Processor::new();
        machine.execute_set(EX, 0x0001);
        machine.execute_set(A, 0x0006);
        machine.execute_div(A, 0x0000);
        assert_eq!(machine.get_register(A), 0x0000);
        assert_eq!(machine.get_register(EX), 0x0000);
    }

    #[test]
    fn mli_value_from_the_a_register() {
        let mut machine = Processor::new();
        machine.execute_set(A, unsafe { mem::transmute(-0x0005 as i16) });
        machine.execute_mli(A, 0x0002);
        assert_eq!(machine.get_signed_register(A), -0x000A);
        assert_eq!(machine.get_register(EX), 0xFFFF);
    }

    #[test]
    fn mli_underflow_value_from_the_a_register() {
        let mut machine = Processor::new();
        machine.execute_set(A, unsafe { mem::transmute(-0x7000 as i16) });
        machine.execute_mli(A, 0x0006);
        assert_eq!(machine.get_signed_register(A), 0x6000);
        assert_eq!(machine.get_register(EX), 0xFFFD);
    }

    #[test]
    fn dvi_remainder_value_from_the_a_register() {
        let mut machine = Processor::new();
        machine.execute_set(A, unsafe { mem::transmute(-0x000B as i16) });
        machine.execute_dvi(A, 0x0004);
        assert_eq!(machine.get_signed_register(A), -0x0002);
        assert_eq!(machine.get_register(EX), 0x4000); // 0.25
    }

    #[test]
    fn dvi_the_a_register_by_zero() {
        let mut machine = Processor::new();
        machine.execute_set(EX, 0x0001);
        machine.execute_set(A, unsafe { mem::transmute(-0x000B as i16) });
        machine.execute_dvi(A, 0x0000);
        assert_eq!(machine.get_register(A), 0x0000);
        assert_eq!(machine.get_register(EX), 0x0000);
    }

    #[test]
    fn mod_the_a_register() {
        let mut machine = Processor::new();
        machine.execute_set(A, 0x000B);
        machine.execute_mod(A, 0x0004);
        assert_eq!(machine.get_register(A), 0x0003);
    }

    #[test]
    fn mod_the_a_register_by_zero() {
        let mut machine = Processor::new();
        machine.execute_set(A, 0x000B);
        machine.execute_mod(A, 0x0000);
        assert_eq!(machine.get_register(A), 0x0000);
    }

    #[test]
    fn mdi_the_a_register() {
        let mut machine = Processor::new();
        machine.execute_set(A, unsafe { mem::transmute(-0x0007 as i16) });
        machine.execute_mdi(A, 0x0004);
        assert_eq!(machine.get_signed_register(A), -0x0003);
    }

    #[test]
    fn mdi_the_a_register_by_zero() {
        let mut machine = Processor::new();
        machine.execute_set(A, unsafe { mem::transmute(-0x0007 as i16) });
        machine.execute_mdi(A, 0x0000);
        assert_eq!(machine.get_register(A), 0x0000);
    }
}

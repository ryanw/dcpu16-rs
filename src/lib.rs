#[derive(Copy, Clone, Debug)]
pub enum Registers {
    A = 0x00,
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
}

impl Processor {
    pub fn new() -> Processor {
        Processor {
            memory: Memory::new(),
            registers: [0; 12],
        }
    }

    pub fn execute_set(&mut self, register: Registers, value: u16) {
        self.registers[register as usize] = value;
    }

    pub fn execute_add(&mut self, register: Registers, value: u16) {
        let old_value: u16 = self.registers[register as usize];
        let (new_value, overflowed) = old_value.overflowing_add(value);
        self.registers[register as usize] = new_value;
        if overflowed {
            self.registers[EX as usize] = 0x0001;
        }
        else {
            self.registers[EX as usize] = 0x0000;
        }
    }

    pub fn execute_sub(&mut self, register: Registers, value: u16) {
        let old_value: u16 = self.registers[register as usize];
        let (new_value, overflowed) = old_value.overflowing_sub(value);
        self.registers[register as usize] = new_value;
        if overflowed {
            self.registers[EX as usize] = 0xFFFF;
        }
        else {
            self.registers[EX as usize] = 0x0000;
        }
    }

    pub fn execute_mul(&mut self, register: Registers, value: u16) {
        let old_value: u16 = self.registers[register as usize];
        let new_value = old_value.wrapping_mul(value);
        self.registers[register as usize] = new_value;
        self.registers[EX as usize] = (((old_value as u32 * value as u32) >> 16) & 0xFFFF) as u16;
    }

    pub fn execute_div(&mut self, register: Registers, value: u16) {
        if value == 0 {
            self.registers[register as usize] = 0;
            self.registers[EX as usize] = 0;
            return;
        }
        let old_value: u16 = self.registers[register as usize];
        let new_value = old_value.wrapping_div(value);
        self.registers[register as usize] = new_value;
        self.registers[EX as usize] = ((((old_value as u32) << 16) / value as u32) & 0xFFFF) as u16;
    }

    pub fn get_register(&self, register: Registers) -> u16 {
        self.registers[register as usize]
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
        machine.execute_set(A, 0x0E);
        assert_eq!(machine.get_register(A), 0x0E);
    }

    #[test]
    fn add_value_to_the_a_register() {
        let mut machine = Processor::new();
        machine.execute_set(A, 0x03);
        machine.execute_add(A, 0x02);
        assert_eq!(machine.get_register(A), 0x05);
        assert_eq!(machine.get_register(EX), 0x0000);
    }

    #[test]
    fn add_overflow_value_to_the_a_register() {
        let mut machine = Processor::new();
        machine.execute_set(A, 0xFFFD);
        machine.execute_add(A, 0x05);
        assert_eq!(machine.get_register(A), 0x02);
        assert_eq!(machine.get_register(EX), 0x0001);
    }

    #[test]
    fn sub_value_from_the_a_register() {
        let mut machine = Processor::new();
        machine.execute_set(A, 0x05);
        machine.execute_sub(A, 0x02);
        assert_eq!(machine.get_register(A), 0x03);
        assert_eq!(machine.get_register(EX), 0x0000);
    }

    #[test]
    fn sub_underflow_value_from_the_a_register() {
        let mut machine = Processor::new();
        machine.execute_set(A, 0x02);
        machine.execute_sub(A, 0x05);
        assert_eq!(machine.get_register(A), 0xFFFD);
        assert_eq!(machine.get_register(EX), 0xFFFF);
    }

    #[test]
    fn mul_value_from_the_a_register() {
        let mut machine = Processor::new();
        machine.execute_set(A, 0x05);
        machine.execute_mul(A, 0x02);
        assert_eq!(machine.get_register(A), 0x0A);
        assert_eq!(machine.get_register(EX), 0x0000);
    }

    #[test]
    fn mul_underflow_value_from_the_a_register() {
        let mut machine = Processor::new();
        machine.execute_set(A, 0xFFF0);
        machine.execute_mul(A, 0x03);
        assert_eq!(machine.get_register(A), 0xFFD0);
        assert_eq!(machine.get_register(EX), 0x0002);
    }

    #[test]
    fn div_value_from_the_a_register() {
        let mut machine = Processor::new();
        machine.execute_set(A, 0x0A);
        machine.execute_div(A, 0x02);
        assert_eq!(machine.get_register(A), 0x05);
        assert_eq!(machine.get_register(EX), 0x0000);
    }

    #[test]
    fn div_underflow_value_from_the_a_register() {
        let mut machine = Processor::new();
        machine.execute_set(A, 0xFFFF);
        machine.execute_div(A, 0x0A);
        assert_eq!(machine.get_register(A), 0x1999);
        assert_eq!(machine.get_register(EX), 0x8000);
    }

    #[test]
    fn div_the_a_register_by_zero() {
        let mut machine = Processor::new();
        machine.execute_set(EX, 0x01);
        machine.execute_set(A, 0x06);
        machine.execute_div(A, 0x00);
        assert_eq!(machine.get_register(A), 0x00);
        assert_eq!(machine.get_register(EX), 0x00);
    }
}

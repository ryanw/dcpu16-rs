mod opcodes;
use self::opcodes::*;
use std::mem;
use std::ops::{Index, IndexMut};

pub struct Instruction {
    op: OpCode,
    b: Value,
    a: Value,
}

impl Instruction {
    pub fn new(op: OpCode, b: Value, a: Value) -> Instruction {
        Instruction { op, b, a }
    }

    pub fn new_from_u16(op: u16, b: u16, a: u16) -> Instruction {
        Instruction {
            op: op as OpCode,
            b: Value::from(b),
            a: Value::from(a),
        }
    }

    pub fn words(&self, processor: &mut Processor) -> Vec<u16> {
        let mut words = Vec::with_capacity(3);
        let a = self.a.get_a(processor);
        let b = self.b.get_b(processor);
        let word = self.op | a | b;

        words.push(word);

        // Larger literals use an extra word
        if a == 0x1F {
            match self.a {
                Value::Literal(val) => words.push(val),
                //Value::NextWord => words.push(),
                _ => {}
            }
        }

        words
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Value {
    Register(Register),
    AtRegister(Register),
    AfterRegister(Register),
    Push,
    Pop,
    Peek,
    Pick,
    AtNextWord,
    NextWord,
    Literal(u16),
}
impl Value {
    pub fn to_u16(&self) -> u16 {
        u16::from(*self)
    }

    pub fn get_a(&self, processor: &mut Processor) -> u16 {
        self.to_u16() << 10
    }

    pub fn get_b(&self, processor: &mut Processor) -> u16 {
        self.to_u16() << 5
    }
}

impl From<u16> for Value {
    fn from(value: u16) -> Value {
        match value {
            0x00..=0x07 => Value::Register(Register::from(value)),
            0x08..=0x0f => Value::AtRegister(Register::from(value - 0x08)),
            0x10..=0x17 => Value::AtRegister(Register::from(value - 0x10)),
            0x18 => Value::Push,
            0x19 => Value::Peek,
            0x1A => Value::Pick,
            0x1B => Value::Register(Register::SP),
            0x1C => Value::Register(Register::PC),
            0x1D => Value::Register(Register::PC),
            0x1E => Value::AtNextWord,
            0x1F => Value::NextWord,
            0x20..=0x3f => Value::Literal(value - 0x21),
            _ => panic!("Invalid value code: {}", value),
        }
    }
}
impl From<Value> for u16 {
    fn from(value: Value) -> u16 {
        match value {
            Value::Register(reg) if (reg as u16) < 0x08 => reg as u16,
            Value::Register(reg) => reg as u16 - 0x08 + 0x1B,
            Value::AtRegister(reg) => reg as u16 + 0x08,
            Value::AfterRegister(reg) => reg as u16 + 0x10,
            Value::Push | Value::Pop => 0x18,
            Value::Peek => 0x19,
            Value::Pick => 0x1A,
            Value::AtNextWord => 0x1E,
            Value::NextWord => 0x1F,
            Value::Literal(literal) if literal > 0x1E => 0x1F,
            Value::Literal(literal) => literal + 0x21,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Register {
    A,
    B,
    C,
    X,
    Y,
    Z,
    I,
    J,

    SP,
    PC,
    EX,
    IA,
}
impl From<u16> for Register {
    fn from(value: u16) -> Register {
        match value {
            0x00..=0x07 => unsafe { mem::transmute(value as u8) },
            _ => panic!("Invalid register: {}", value),
        }
    }
}

use self::Register::*;

pub struct Processor {
    memory: Memory,
    registers: [u16; 12],
    cycle_wait: u8,
}

impl Processor {
    pub fn new() -> Processor {
        Processor {
            memory: Memory::new(),
            registers: [0; 12],
            cycle_wait: 0,
        }
    }

    pub fn tick(&mut self) {
        if self.cycle_wait > 0 {
            self.cycle_wait -= 1;
            return;
        }

        self.execute_next();
    }

    pub fn execute_next(&mut self) {
        let addr = self.get_register(PC);
        let instruction = self.memory.get_instruction(addr);
        self.inc(PC);
        self.execute_instruction(instruction);
    }

    pub fn execute_instruction(&mut self, instruction: Instruction) {
        let Instruction { op, b, a } = instruction;
        let a_value = match a {
            Value::NextWord => {
                self.cycle_wait += 1;
                self.next_word()
            }
            _ => panic!("Invalid 'a' value: {:?}", a),
        };
        match op {
            SET => match b {
                Value::Register(reg) => {
                    self.registers[reg as usize] = a_value;
                },
                _ => panic!("Invalid 'a' value: {:?}", a),
            },
            _ => panic!("Invalid op code {}", op),
        }
    }

    pub fn next_word(&mut self) -> u16 {
        let addr = self.get_register(PC);
        let value = self.memory[addr];
        self.inc(PC);

        value
    }

    pub fn next_value(&mut self) -> Value {
        let word = self.next_word();
        let value = Value::from(word);
        println!("GOT VALUE: {:?}", value);

        value
    }

    pub fn inc(&mut self, register: Register) {
        let old_value = self.registers[register as usize];
        self.registers[register as usize] = old_value.wrapping_add(1);
    }

    pub fn dec(&mut self, register: Register) {
        let old_value = self.registers[register as usize];
        self.registers[register as usize] = old_value.wrapping_sub(1);
    }

    pub fn get(&mut self, value: Value) -> u16 {
        match value {
            _ => panic!("Unknown value: {:?}", value),
        }
    }

    pub fn get_register(&self, register: Register) -> u16 {
        self.registers[register as usize]
    }

    pub fn get_signed_register(&self, register: Register) -> i16 {
        unsafe { mem::transmute(self.registers[register as usize]) }
    }

    pub fn execute_set(&mut self, register: Register, value: u16) {
        self.registers[register as usize] = value;
    }

    pub fn execute_add(&mut self, register: Register, value: u16) {
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

    pub fn execute_sub(&mut self, register: Register, value: u16) {
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

    pub fn execute_mul(&mut self, register: Register, value: u16) {
        self.cycle_wait += 1;

        let old_value = self.registers[register as usize];
        let new_value = old_value.wrapping_mul(value);
        self.registers[register as usize] = new_value;
        self.registers[EX as usize] = (((old_value as u32 * value as u32) >> 16) & 0xFFFF) as u16;
    }

    pub fn execute_div(&mut self, register: Register, value: u16) {
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

    pub fn execute_mli(&mut self, register: Register, value: i16) {
        self.cycle_wait += 1;

        let old_value: i16 = unsafe { mem::transmute(self.registers[register as usize]) };
        let new_value: i16 = old_value.wrapping_mul(value);
        self.registers[register as usize] = unsafe { mem::transmute(new_value) };
        self.registers[EX as usize] =
            unsafe { mem::transmute((((old_value as i32 * value as i32) >> 16) & 0xFFFF) as i16) };
    }

    pub fn execute_dvi(&mut self, register: Register, value: i16) {
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

    pub fn execute_mod(&mut self, register: Register, value: u16) {
        self.cycle_wait += 2;

        if value == 0 {
            self.registers[register as usize] = 0;
            return;
        }

        let old_value = self.registers[register as usize];
        self.registers[register as usize] = old_value % value;
    }

    pub fn execute_mdi(&mut self, register: Register, value: i16) {
        self.cycle_wait += 2;

        if value == 0 {
            self.registers[register as usize] = 0;
            return;
        }

        let old_value: i16 = unsafe { mem::transmute(self.registers[register as usize]) };
        self.registers[register as usize] = unsafe { mem::transmute(old_value % value) };
    }

    pub fn execute_and(&mut self, register: Register, value: u16) {
        let old_value = self.registers[register as usize];
        self.registers[register as usize] = old_value & value;
    }

    pub fn execute_bor(&mut self, register: Register, value: u16) {
        let old_value = self.registers[register as usize];
        self.registers[register as usize] = old_value | value;
    }

    pub fn execute_xor(&mut self, register: Register, value: u16) {
        let old_value = self.registers[register as usize];
        self.registers[register as usize] = old_value ^ value;
    }

    pub fn execute_shr(&mut self, register: Register, value: u8) {
        let old_value = self.registers[register as usize];
        self.registers[register as usize] = old_value >> value;
        self.registers[EX as usize] = (((old_value as u32) << 16) >> value) as u16;
    }

    pub fn execute_asr(&mut self, register: Register, value: u8) {
        let old_value: i16 = unsafe { mem::transmute(self.registers[register as usize]) };
        self.registers[register as usize] = unsafe { mem::transmute(old_value >> value) };
        self.registers[EX as usize] =
            unsafe { mem::transmute((((old_value as i32) << 16) >> value) as i16) };
    }

    pub fn execute_shl(&mut self, register: Register, value: u8) {
        let old_value = self.registers[register as usize];
        self.registers[register as usize] = old_value << value;
        self.registers[EX as usize] = (((old_value as u32) << value) >> 16) as u16;
    }
}

pub struct Memory([u16; 0x10000]);
impl Memory {
    pub fn new() -> Memory {
        Memory([0; 0x10000])
    }

    pub fn set(&mut self, addr: u16, value: u16) {
        self.0[addr as usize] = value;
    }

    pub fn get(&mut self, addr: u16) -> u16 {
        self.0[addr as usize]
    }

    pub fn get_instruction(&self, addr: u16) -> Instruction {
        let word = self.0[addr as usize];
        let op = word & 0b0000000000011111;
        let b = (word & 0b0000001111100000) >> 5;
        let a = (word & 0b1111110000000000) >> 10;

        Instruction::new_from_u16(op, b, a)
    }
}
impl Index<u16> for Memory {
    type Output = u16;

    fn index(&self, addr: u16) -> &u16 {
        &self.0[addr as usize]
    }
}
impl IndexMut<u16> for Memory {
    fn index_mut(&mut self, addr: u16) -> &mut u16 {
        &mut self.0[addr as usize]
    }
}

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

    #[test]
    fn and_the_a_register() {
        let mut machine = Processor::new();
        machine.execute_set(A, 0x0006);
        machine.execute_and(A, 0x0003);
        assert_eq!(machine.get_register(A), 0x0002);
    }

    #[test]
    fn bor_the_a_register() {
        let mut machine = Processor::new();
        machine.execute_set(A, 0x0006);
        machine.execute_bor(A, 0x0003);
        assert_eq!(machine.get_register(A), 0x0007);
    }

    #[test]
    fn xor_the_a_register() {
        let mut machine = Processor::new();
        machine.execute_set(A, 0x0006);
        machine.execute_xor(A, 0x0003);
        assert_eq!(machine.get_register(A), 0x0005);
    }

    #[test]
    fn shr_the_a_register() {
        let mut machine = Processor::new();
        machine.execute_set(A, 0xF0AA);
        machine.execute_shr(A, 0x0002);
        assert_eq!(machine.get_register(A), 0x3C2A);
        assert_eq!(machine.get_register(EX), 0x8000);
    }

    #[test]
    fn asr_the_a_register() {
        let mut machine = Processor::new();
        machine.execute_set(A, 0xF0AA);
        machine.execute_asr(A, 0x0002);
        assert_eq!(machine.get_register(A), 0xFC2A);
        assert_eq!(machine.get_register(EX), 0x8000);
    }

    #[test]
    fn shl_the_a_register() {
        let mut machine = Processor::new();
        machine.execute_set(A, 0xF0AA);
        machine.execute_shl(A, 0x0002);
        assert_eq!(machine.get_register(A), 0xC2A8);
        assert_eq!(machine.get_register(EX), 0x0003);
    }

    #[test]
    fn executes_instruction_at_pc() {
        let mut machine = Processor::new();
        let mut words =
            Instruction::new(SET, Value::Register(A), Value::NextWord).words(&mut machine);
        words.push(0xDEAD);
        for (i, &word) in words.iter().enumerate() {
            machine.memory[i as u16] = word;
        }
        assert_eq!(machine.get_register(A), 0x0000);
        machine.tick();
        assert_eq!(machine.get_register(A), 0xDEAD);
    }
}

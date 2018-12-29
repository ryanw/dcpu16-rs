#[cfg(test)]
mod tests;

mod opcodes;
use self::opcodes::*;
use std::mem;
use std::ops::{Index, IndexMut};

fn to_signed(val: u16) -> i16 {
    unsafe { mem::transmute(val) }
}

fn to_unsigned(val: i16) -> u16 {
    unsafe { mem::transmute(val) }
}

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

    pub fn execute(&self, processor: &mut Processor) {
        let a = self.get_a(processor);
        self.set_b(processor, a);
    }

    pub fn get_a(&self, processor: &mut Processor) -> u16 {
        match self.a {
            Value::Register(reg) => processor.get_register(reg),
            Value::AtRegister(reg) => {
                let addr = processor.get_register(reg);
                processor.memory[addr]
            }
            Value::AfterRegister(reg) => {
                let addr = processor.get_register(reg) + processor.next_word();
                processor.memory[addr]
            }
            Value::Push | Value::Pop => {
                // A is always POP
                processor.pop()
            }
            Value::Peek => processor.peek(),
            Value::Pick => {
                let addr = processor.get_register(SP) + processor.next_word();
                processor.memory[addr]
            }
            Value::AtNextWord => {
                let addr = processor.next_word();
                processor.memory[addr]
            }
            Value::NextWord => processor.next_word(),
            Value::Literal(literal) => literal,
        }
    }

    pub fn get_b(&self, processor: &mut Processor) -> u16 {
        match self.b {
            Value::Register(reg) => processor.get_register(reg),
            Value::AtRegister(reg) => {
                let addr = processor.get_register(reg);
                processor.memory[addr]
            }
            Value::AfterRegister(reg) => {
                let addr = processor.get_register(reg) + processor.next_word();
                processor.memory[addr]
            }
            Value::Push | Value::Pop => {
                // B is always PUSH
                processor.inc(SP);
                let addr = processor.get_register(SP);
                processor.memory[addr]
            }
            Value::Peek => processor.peek(),
            Value::Pick => {
                let addr = processor.get_register(SP) + processor.next_word();
                processor.memory[addr]
            }
            Value::AtNextWord => {
                let addr = processor.next_word();
                processor.memory[addr]
            }
            Value::NextWord => processor.next_word(),
            Value::Literal(literal) => literal + 0x21,
        }
    }

    /// Returns the value in `b` without modifing any registers or using cycles
    pub fn peek_b(&self, processor: &Processor) -> u16 {
        match self.b {
            Value::Register(reg) => processor.get_register(reg),
            Value::AtRegister(reg) => {
                let addr = processor.get_register(reg);
                processor.memory[addr]
            }
            Value::AfterRegister(reg) => {
                let addr = processor.get_register(reg) + processor.peek_next_word();
                processor.memory[addr]
            }
            Value::Push | Value::Pop => {
                // B is always PUSH
                let addr = processor.get_register(SP);
                processor.memory[addr]
            }
            Value::Peek => processor.peek(),
            Value::Pick => {
                let addr = processor.get_register(SP) + processor.peek_next_word();
                processor.memory[addr]
            }
            Value::AtNextWord => {
                let addr = processor.peek_next_word();
                processor.memory[addr]
            }
            Value::NextWord => processor.peek_next_word(),
            Value::Literal(literal) => literal + 0x21,
        }
    }

    pub fn set_b(&self, processor: &mut Processor, a: u16) {
        // Get current `b` value to apply the operation to
        let b = self.peek_b(processor);
        let mut ex = processor.get_register(EX);
        let new_value = match self.op {
            SET => a,
            ADD => {
                processor.cycle_wait += 1;
                let (value, overflowed) = b.overflowing_add(a);
                if overflowed {
                    ex = 0x0001;
                } else {
                    ex = 0x0000;
                }

                value
            }
            SUB => {
                processor.cycle_wait += 1;
                let (value, overflowed) = b.overflowing_sub(a);
                if overflowed {
                    ex = 0xFFFF;
                } else {
                    ex = 0x0000;
                }

                value
            }
            MUL => {
                processor.cycle_wait += 1;
                ex = (((b as u32 * a as u32) >> 16) & 0xFFFF) as u16;
                b.wrapping_mul(a)
            }
            MLI => {
                processor.cycle_wait += 1;
                let signed_b = to_signed(b);
                let signed_a = to_signed(a);
                ex = to_unsigned((((signed_b as i32 * signed_a as i32) >> 16) & 0xFFFF) as i16);
                to_unsigned(signed_b.wrapping_mul(signed_a))
            }
            DIV => {
                processor.cycle_wait += 2;

                if a == 0 {
                    ex = 0;
                    0
                } else {
                    ex = ((((b as u32) << 16) / a as u32) & 0xFFFF) as u16;
                    b.wrapping_div(a)
                }
            }
            DVI => {
                processor.cycle_wait += 2;

                if a == 0 {
                    ex = 0;
                    0
                } else {
                    let signed_b = to_signed(b);
                    let signed_a = to_signed(a);
                    ex = to_unsigned(
                        ((((signed_b as i32) << 16) / signed_a as i32) & 0xFFFF) as i16,
                    );
                    to_unsigned(signed_b.wrapping_div(signed_a))
                }
            }
            MOD => {
                processor.cycle_wait += 2;
                if a == 0 {
                    0
                } else {
                    b % a
                }
            }
            MDI => {
                processor.cycle_wait += 2;
                if a == 0 {
                    0
                } else {
                    let signed_b = to_signed(b);
                    let signed_a = to_signed(a);
                    to_unsigned(signed_b % signed_a)
                }
            }
            AND => b & a,
            BOR => b | a,
            XOR => b ^ a,
            SHR => {
                ex = (((b as u32) << 16) >> a) as u16;
                b >> a
            }
            ASR => {
                let signed_b = to_signed(b);
                let signed_a = to_signed(a);
                ex = (((signed_b as u32) << 16) >> signed_a) as u16;
                to_unsigned(signed_b >> signed_a)
            }
            SHL => {
                ex = (((b as u32) << a) >> 16) as u16;
                b << a
            }
            _ => panic!("Invalid op code {}", self.op),
        };

        processor.set_register(EX, ex);

        // Write to `b`
        match self.b {
            Value::Register(reg) => {
                processor.registers[reg as usize] = new_value;
            }
            Value::AtRegister(reg) => {
                let addr = processor.get_register(reg);
                processor.memory[addr] = new_value;
            }
            Value::AfterRegister(reg) => {}
            Value::Push | Value::Pop => {
                // B is always PUSH
                let addr = processor.get_register(SP);
                processor.memory[addr] = new_value;
            }
            Value::Peek => {}
            Value::Pick => {}
            Value::AtNextWord => {}
            Value::NextWord => {}
            Value::Literal(literal) => {}
        }
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
        instruction.execute(self);
    }

    pub fn next_word(&mut self) -> u16 {
        let word = self.peek_next_word();
        self.inc(PC);
        self.cycle_wait += 1;

        word
    }

    pub fn peek_next_word(&self) -> u16 {
        let addr = self.get_register(PC);
        self.memory[addr]
    }

    pub fn next_value(&mut self) -> Value {
        let word = self.next_word();
        let value = Value::from(word);

        value
    }

    pub fn push(&mut self, value: u16) {
        let addr = self.get_register(SP);
        self.dec(SP);
        self.memory[addr] = value;
    }

    pub fn pop(&mut self) -> u16 {
        let addr = self.get_register(SP);
        self.inc(SP);
        self.memory[addr]
    }

    pub fn peek(&self) -> u16 {
        let addr = self.get_register(SP);
        self.memory[addr]
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

    pub fn set_register(&mut self, register: Register, value: u16) {
        self.registers[register as usize] = value;
    }

    pub fn get_signed_register(&self, register: Register) -> i16 {
        to_signed(self.registers[register as usize])
    }

    pub fn set_signed_register(&mut self, register: Register, value: i16) {
        self.registers[register as usize] = to_unsigned(value);
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

        let old_value = to_signed(self.registers[register as usize]);
        let new_value = old_value.wrapping_mul(value);
        self.registers[register as usize] = to_unsigned(new_value);
        self.registers[EX as usize] =
            to_unsigned((((old_value as i32 * value as i32) >> 16) & 0xFFFF) as i16);
    }

    pub fn execute_dvi(&mut self, register: Register, value: i16) {
        self.cycle_wait += 2;

        if value == 0 {
            self.registers[register as usize] = 0;
            self.registers[EX as usize] = 0;
            return;
        }
        let old_value = to_signed(self.registers[register as usize]);
        let new_value = old_value.wrapping_div(value);
        self.registers[register as usize] = to_unsigned(new_value);
        self.registers[EX as usize] =
            to_unsigned(((((old_value as i32) << 16) / value as i32) & 0xFFFF) as i16);
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

        let old_value = to_signed(self.registers[register as usize]);
        self.registers[register as usize] = to_unsigned(old_value % value);
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
        let old_value = to_signed(self.registers[register as usize]);
        self.registers[register as usize] = to_unsigned(old_value >> value);
        self.registers[EX as usize] = to_unsigned((((old_value as i32) << 16) >> value) as i16);
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
        self[addr] = value;
    }

    pub fn get(&mut self, addr: u16) -> u16 {
        self[addr]
    }

    pub fn get_instruction(&self, addr: u16) -> Instruction {
        let word = self[addr];
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

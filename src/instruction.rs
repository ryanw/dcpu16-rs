use super::opcodes::*;
use super::processor::Processor;
use super::processor::Register::*;
use super::value::Value;
use std::mem;

fn to_signed(val: u16) -> i16 {
    unsafe { mem::transmute(val) }
}

fn to_unsigned(val: i16) -> u16 {
    unsafe { mem::transmute(val) }
}

#[derive(Debug)]
pub struct Instruction {
    op: OpCode,
    b: Value,
    a: Value,
}

impl From<u16> for Instruction {
    fn from(word: u16) -> Instruction {
        let op = word & 0b0000000000011111;
        let b = (word & 0b0000001111100000) >> 5;
        let a = (word & 0b1111110000000000) >> 10;

        // Specials
        if op == 0x00 {
            Instruction::new(SPL, Value::OpCode(b), Value::from(a))
        } else {
            Instruction::new(op as OpCode, Value::from(b), Value::from(a))
        }
    }
}

impl Instruction {
    pub fn new(op: OpCode, b: Value, a: Value) -> Instruction {
        Instruction { op, b, a }
    }

    pub fn words(&self) -> Vec<u16> {
        let mut words = Vec::with_capacity(3);
        let a = self.a.get_a();
        let b = self.b.get_b();
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
        match self.op {
            // Specials
            0x00 => self.execute_special(processor),

            // Setters
            0x01..=0x0F | 0x1A..=0x1F => {
                let a = self.get_a(processor);
                self.set_b(processor, a);
            }

            // Conditionals
            0x10..=0x17 => {
                let a = self.get_a(processor);
                self.test_condition(processor, a);
            }

            // Bad
            _ => panic!("Invalid op code {}", self.op),
        }
    }

    pub fn execute_special(&self, processor: &mut Processor) {
        let a = self.get_a(processor);
        let op = self.peek_b(processor);
        match op {
            JSR => {
                processor.cycle_wait += 2;
                let pc = processor.get_register(PC);
                processor.push(pc);
                processor.set_register(PC, a);
            }
            INT => {
                processor.cycle_wait += 3;
                processor.trigger_interrupt(a);
            }
            IAG => {
                let value = processor.get_register(IA);
                self.set_value(processor, self.a, value);
            }
            IAS => {
                processor.set_register(IA, a);
            }
            RFI => {
                processor.cycle_wait += 2;
                processor.return_from_interrupt();
            }
            IAQ => {
                processor.cycle_wait += 1;
                if a == 0 {
                    processor.is_queuing_interrupts = false;
                } else {
                    processor.is_queuing_interrupts = true;
                }
            }
            HWN => {
                processor.cycle_wait += 1;
                let hardware_count = processor.hardware_count();
                self.set_value(processor, self.a, hardware_count);
            }
            HWQ => {
                processor.cycle_wait += 3;
                if let Some(rc) = processor.get_hardware(a) {
                    if let Ok(hardware) = rc.try_borrow() {
                        let a = (hardware.id() & 0xFFFF) as u16;
                        let b = (hardware.id() >> 16 & 0xFFFF) as u16;
                        let c = hardware.version();
                        let x = (hardware.manufacturer() & 0xFFFF) as u16;
                        let y = (hardware.manufacturer() >> 16 & 0xFFFF) as u16;
                        processor.set_register(A, a);
                        processor.set_register(B, b);
                        processor.set_register(C, c);
                        processor.set_register(X, x);
                        processor.set_register(Y, y);
                    } else {
                        panic!("TODO");
                    }
                } else {
                    processor.set_register(A, 0x00);
                    processor.set_register(B, 0x00);
                    processor.set_register(C, 0x00);
                    processor.set_register(X, 0x00);
                    processor.set_register(Y, 0x00);
                }
            }
            HWI => {
                processor.cycle_wait += 3;
                if let Some(rc) = processor.get_hardware(a) {
                    if let Ok(mut hardware) = rc.try_borrow_mut() {
                        hardware.handle_interrupt(processor);
                    }
                }
            }
            _ => panic!("Invalid special op code {}", op),
        }
    }

    pub fn get_a(&self, processor: &mut Processor) -> u16 {
        match self.a {
            Value::Register(reg) => processor.get_register(reg),
            Value::RegisterPointer(reg) => {
                let addr = processor.get_register(reg);
                processor.memory[addr]
            }
            Value::RegisterPointerOffset(reg) => {
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
            Value::NextWordPointer => {
                let addr = processor.next_word();
                processor.memory[addr]
            }
            Value::NextWord => processor.next_word(),
            Value::Literal(literal) => literal,
            Value::OpCode(op) => op,
        }
    }

    pub fn get_b(&self, processor: &mut Processor) -> u16 {
        match self.b {
            Value::Register(reg) => processor.get_register(reg),
            Value::RegisterPointer(reg) => {
                let addr = processor.get_register(reg);
                processor.memory[addr]
            }
            Value::RegisterPointerOffset(reg) => {
                let addr = processor.get_register(reg) + processor.next_word();
                processor.memory[addr]
            }
            Value::Push | Value::Pop => {
                // B is always PUSH
                let addr = processor.get_register(SP).wrapping_sub(1);
                processor.memory[addr]
            }
            Value::Peek => processor.peek(),
            Value::Pick => {
                let addr = processor.get_register(SP) + processor.next_word();
                processor.memory[addr]
            }
            Value::NextWordPointer => {
                let addr = processor.next_word();
                processor.memory[addr]
            }
            Value::NextWord => processor.next_word(),
            Value::Literal(literal) => literal,
            Value::OpCode(op) => op,
        }
    }

    /// Returns the value in `b` without modifing any registers or using cycles
    pub fn peek_b(&self, processor: &Processor) -> u16 {
        match self.b {
            Value::Register(reg) => processor.get_register(reg),
            Value::RegisterPointer(reg) => {
                let addr = processor.get_register(reg);
                processor.memory[addr]
            }
            Value::RegisterPointerOffset(reg) => {
                let addr = processor.get_register(reg) + processor.peek_next_word();
                processor.memory[addr]
            }
            Value::Push | Value::Pop => {
                // B is always PUSH
                let addr = processor.get_register(SP).wrapping_sub(1);
                processor.memory[addr]
            }
            Value::Peek => processor.peek(),
            Value::Pick => {
                let addr = processor.get_register(SP) + processor.peek_next_word();
                processor.memory[addr]
            }
            Value::NextWordPointer => {
                let addr = processor.peek_next_word();
                processor.memory[addr]
            }
            Value::NextWord => processor.peek_next_word(),
            Value::Literal(literal) => literal,
            Value::OpCode(op) => op,
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
            ADX => {
                processor.cycle_wait += 2;
                let (value1, overflowed1) = b.overflowing_add(a);
                let (value2, overflowed2) = value1.overflowing_add(ex);
                if overflowed1 || overflowed2 {
                    ex = 0x0001;
                } else {
                    ex = 0x0000;
                }
                value2
            }
            SBX => {
                processor.cycle_wait += 2;
                let (value1, overflowed1) = b.overflowing_sub(a);
                let (value2, overflowed2) = value1.overflowing_add(ex);
                if overflowed1 || overflowed2 {
                    ex = 0xFFFF;
                } else {
                    ex = 0x0000;
                }
                value2
            }
            STI => {
                processor.cycle_wait += 1;
                processor.inc(I);
                processor.inc(J);
                a
            }
            STD => {
                processor.cycle_wait += 1;
                processor.dec(I);
                processor.dec(J);
                a
            }
            _ => panic!("Invalid op code {}", self.op),
        };

        processor.set_register(EX, ex);

        self.set_value(processor, self.b, new_value);
    }

    pub fn set_value(&self, processor: &mut Processor, target: Value, value: u16) {
        match target {
            Value::Register(reg) => {
                processor.registers[reg as usize] = value;
            }
            Value::RegisterPointer(reg) => {
                let addr = processor.get_register(reg);
                processor.memory[addr] = value;
            }
            Value::RegisterPointerOffset(reg) => {
                let offset = processor.next_word();
                let addr = processor.get_register(reg).wrapping_add(offset);
                processor.memory[addr] = value;
            }
            Value::Push | Value::Pop => {
                // B is always PUSH
                processor.dec(SP);
                let addr = processor.get_register(SP);
                processor.memory[addr] = value;
            }
            Value::Peek => {}
            Value::Pick => {}
            Value::NextWordPointer => {
                let addr = processor.next_word();
                processor.set_memory(addr, value);
            }
            Value::NextWord => {}
            Value::Literal(_) => {}
            Value::OpCode(_) => {}
        }
    }

    pub fn test_condition(&self, processor: &mut Processor, a: u16) {
        let b = self.get_b(processor);
        match self.op {
            IFB => {
                processor.cycle_wait += 1;
                if b & a == 0 {
                    self.condition_failure(processor);
                }
            }
            IFC => {
                processor.cycle_wait += 1;
                if b & a != 0 {
                    self.condition_failure(processor);
                }
            }
            IFE => {
                processor.cycle_wait += 1;
                if b != a {
                    self.condition_failure(processor);
                }
            }
            IFN => {
                processor.cycle_wait += 1;
                if b == a {
                    self.condition_failure(processor);
                }
            }
            IFG => {
                processor.cycle_wait += 1;
                if b <= a {
                    self.condition_failure(processor);
                }
            }
            IFA => {
                let signed_b = to_signed(b);
                let signed_a = to_signed(a);

                processor.cycle_wait += 1;
                if signed_b <= signed_a {
                    self.condition_failure(processor);
                }
            }
            IFL => {
                processor.cycle_wait += 1;
                if b >= a {
                    self.condition_failure(processor);
                }
            }
            IFU => {
                let signed_b = to_signed(b);
                let signed_a = to_signed(a);

                processor.cycle_wait += 1;
                if signed_b >= signed_a {
                    self.condition_failure(processor);
                }
            }
            _ => panic!("Invalid op code {}", self.op),
        };
    }

    pub fn condition_failure(&self, processor: &mut Processor) {
        // Condition failed, skip next instruction
        let addr = processor.get_register(PC);
        let Instruction { op, b, a } = processor.memory.get_instruction(addr);

        // Skip extra word in longer instructions
        match b {
            Value::RegisterPointerOffset(_) | Value::NextWordPointer | Value::NextWord => {
                processor.inc(PC);
                processor.cycle_wait += 1;
            }
            _ => {}
        }
        match a {
            Value::RegisterPointerOffset(_) | Value::NextWordPointer | Value::NextWord => {
                processor.inc(PC);
                processor.cycle_wait += 1;
            }
            _ => {}
        }
        match op {
            // Chaining IFn conditions
            0x10..=0x17 => {
                processor.inc(PC);
                processor.cycle_wait += 1;
            }
            _ => {}
        }
        processor.inc(PC);
    }
}

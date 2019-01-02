use super::opcodes::OpCode;
use super::processor::Register;

#[derive(Copy, Clone, Debug)]
pub enum Value {
    Register(Register),
    RegisterPointer(Register),
    RegisterPointerOffset(Register),
    Push,
    Pop,
    Peek,
    Pick,
    NextWordPointer,
    NextWord,
    Literal(u16),
    OpCode(OpCode),
}
impl Value {
    pub fn to_u16(&self) -> u16 {
        u16::from(*self)
    }

    pub fn get_a(&self) -> u16 {
        self.to_u16() << 10
    }

    pub fn get_b(&self) -> u16 {
        self.to_u16() << 5
    }
}

impl From<u16> for Value {
    fn from(value: u16) -> Value {
        match value {
            0x00..=0x07 => Value::Register(Register::from(value)),
            0x08..=0x0f => Value::RegisterPointer(Register::from(value - 0x08)),
            0x10..=0x17 => Value::RegisterPointerOffset(Register::from(value - 0x10)),
            0x18 => Value::Push,
            0x19 => Value::Peek,
            0x1A => Value::Pick,
            0x1B => Value::Register(Register::SP),
            0x1C => Value::Register(Register::PC),
            0x1D => Value::Register(Register::EX),
            0x1E => Value::NextWordPointer,
            0x1F => Value::NextWord,
            0x20..=0x3f => Value::Literal(value.wrapping_sub(0x21)),
            _ => panic!("Invalid value code: {}", value),
        }
    }
}
impl From<Value> for u16 {
    fn from(value: Value) -> u16 {
        match value {
            Value::Register(reg) if (reg as u16) < 0x08 => reg as u16,
            Value::Register(reg) => reg as u16 - 0x08 + 0x1B,
            Value::RegisterPointer(reg) => reg as u16 + 0x08,
            Value::RegisterPointerOffset(reg) => reg as u16 + 0x10,
            Value::Push | Value::Pop => 0x18,
            Value::Peek => 0x19,
            Value::Pick => 0x1A,
            Value::NextWordPointer => 0x1E,
            Value::NextWord => 0x1F,
            Value::Literal(literal) if literal > 0x1E => 0x1F,
            Value::Literal(literal) => literal + 0x21,
            Value::OpCode(op) => op,
        }
    }
}

use super::instruction::Instruction;
use super::opcodes::OpCode;
use super::value::Value;

pub struct Program(Vec<u16>);
impl Program {
    pub fn new() -> Program {
        Program(Vec::with_capacity(64))
    }

    pub fn add(&mut self, op: OpCode, b: Value, a: Value) {
        let inst = Instruction::new(op, b, a);
        self.0.append(&mut inst.words());
    }

    pub fn add_word(&mut self, word: u16) {
        self.0.push(word);
    }

    pub fn words(&self) -> &Vec<u16> {
        &self.0
    }
}

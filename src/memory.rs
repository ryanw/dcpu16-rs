use super::instruction::Instruction;
use super::program::Program;
use std::ops::{Index, IndexMut};

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
        Instruction::from(self[addr])
    }

    pub fn load_program(&mut self, addr: u16, program: &Program) {
        for (i, &word) in program.words().iter().enumerate() {
            self[addr + i as u16] = word;
        }
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

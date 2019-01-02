use self::Register::*;
use super::hardware::HardwareDevice;
use super::memory::Memory;
use super::value::Value;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::mem;
use std::rc::Rc;

fn to_signed(val: u16) -> i16 {
    unsafe { mem::transmute(val) }
}

fn to_unsigned(val: i16) -> u16 {
    unsafe { mem::transmute(val) }
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

pub struct Processor {
    pub(crate) memory: Memory,
    pub(crate) registers: [u16; 12],
    pub(crate) cycle_wait: u8,
    pub(crate) is_queuing_interrupts: bool,
    cycle: usize,
    interrupt_queue: VecDeque<u16>,
    is_on_fire: bool,
    hardware: Vec<Rc<RefCell<dyn HardwareDevice>>>,
}

impl Processor {
    pub fn new() -> Processor {
        Processor {
            memory: Memory::new(),
            registers: [0; 12],
            cycle_wait: 0,
            cycle: 0,
            is_queuing_interrupts: false,
            interrupt_queue: VecDeque::with_capacity(256),
            is_on_fire: false,
            hardware: vec![],
        }
    }

    pub fn tick(&mut self) {
        if self.is_on_fire {
            return;
        }

        self.cycle = self.cycle.wrapping_add(1);
        if self.cycle_wait > 0 {
            self.cycle_wait -= 1;
            return;
        }

        self.execute_next();
        self.process_interrupt_queue();
    }

    pub fn cycle(&self) -> usize {
        self.cycle
    }

    pub fn get_memory(&self, addr: u16) -> u16 {
        self.memory[addr]
    }

    pub fn set_memory(&mut self, addr: u16, value: u16) {
        self.memory[addr] = value;
    }

    pub fn connect_hardware<T: 'static + HardwareDevice>(&mut self, hardware: T) {
        self.hardware.push(Rc::new(RefCell::new(hardware)));
    }

    pub fn hardware_count(&self) -> u16 {
        self.hardware.len() as u16
    }

    pub fn get_hardware(&self, index: u16) -> Option<Rc<RefCell<dyn HardwareDevice>>> {
        if let Some(rc) = self.hardware.get(index as usize) {
            Some(rc.clone())
        } else {
            None
        }
    }

    pub fn with_hardware<T: HardwareDevice, F: FnMut(&T, &Processor)>(
        &self,
        index: u16,
        mut closure: F,
    ) {
        if let Some(rc) = self.get_hardware(index) {
            if let Ok(hardware) = rc.try_borrow_mut() {
                if let Some(device) = hardware.downcast_ref::<T>() {
                    closure(device, self);
                }
            }
        }
    }

    pub fn with_hardware_mut<T: HardwareDevice, F: FnMut(&mut T, &mut Processor)>(
        &mut self,
        index: u16,
        mut closure: F,
    ) {
        if let Some(rc) = self.get_hardware(index) {
            if let Ok(mut hardware) = rc.try_borrow_mut() {
                if let Some(device) = hardware.downcast_mut::<T>() {
                    closure(device, self);
                }
            }
        }
    }

    pub fn execute_next(&mut self) {
        let addr = self.get_register(PC);
        let instruction = self.memory.get_instruction(addr);
        self.inc(PC);
        instruction.execute(self);
    }

    pub fn process_interrupt_queue(&mut self) {
        if self.is_queuing_interrupts {
            return;
        }
        if let Some(message) = self.interrupt_queue.pop_front() {
            self.handle_interrupt(message);
        }
    }

    pub fn handle_interrupt(&mut self, message: u16) {
        let handler_addr = self.get_register(IA);

        self.is_queuing_interrupts = true;
        let pc = self.get_register(PC);
        let a = self.get_register(A);
        self.push(pc);
        self.push(a);
        self.set_register(PC, handler_addr);
        self.set_register(A, message);
    }

    pub fn trigger_interrupt(&mut self, message: u16) {
        if self.get_register(IA) == 0 {
            return;
        }

        if self.is_queuing_interrupts {
            self.queue_interrupt(message);
            return;
        }

        self.handle_interrupt(message);
    }

    pub fn return_from_interrupt(&mut self) {
        self.is_queuing_interrupts = false;
        let a = self.pop();
        let pc = self.pop();
        self.set_register(A, a);
        self.set_register(PC, pc);
    }

    pub fn queue_interrupt(&mut self, message: u16) {
        if self.interrupt_queue.len() >= 256 {
            self.is_on_fire = true;
            return;
        }

        self.interrupt_queue.push_back(message);
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
        self.dec(SP);
        let addr = self.get_register(SP);
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
}

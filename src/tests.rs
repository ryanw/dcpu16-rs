use super::*;
use super::Register::*;
use super::opcodes::*;
use std::mem;

fn to_unsigned(val: i16) -> u16 {
    unsafe { mem::transmute(val) }
}

#[test]
fn set_register_to_next_word() {
    let mut machine = Processor::new();
    let mut words = Instruction::new(SET, Value::Register(A), Value::NextWord).words();
    words.push(0xDEAD);
    for (i, &word) in words.iter().enumerate() {
        machine.memory[i as u16] = word;
    }
    assert_eq!(machine.get_register(A), 0x0000);
    machine.tick();
    assert_eq!(machine.get_register(A), 0xDEAD);
    assert_eq!(machine.get_register(PC), 0x0002);
    assert_eq!(machine.cycle_wait, 1);
}

// Setting

#[test]
fn set_register_to_literal() {
    let mut machine = Processor::new();
    let words = Instruction::new(SET, Value::Register(A), Value::Literal(0x05)).words();
    for (i, &word) in words.iter().enumerate() {
        machine.memory[i as u16] = word;
    }
    assert_eq!(machine.get_register(A), 0x0000);
    machine.tick();
    assert_eq!(machine.get_register(A), 0x0005);
    assert_eq!(machine.get_register(PC), 0x0001);
    assert_eq!(machine.get_register(EX), 0x0000);
    assert_eq!(machine.cycle_wait, 0);
}

#[test]
fn set_register_to_register() {
    let mut machine = Processor::new();
    let words = Instruction::new(SET, Value::Register(B), Value::Register(A)).words();
    for (i, &word) in words.iter().enumerate() {
        machine.memory[i as u16] = word;
    }
    machine.set_register(A, 0x1234);
    machine.set_register(B, 0x4444);
    machine.tick();
    assert_eq!(machine.get_register(A), 0x1234);
    assert_eq!(machine.get_register(B), 0x1234);
    assert_eq!(machine.get_register(PC), 0x0001);
    assert_eq!(machine.get_register(EX), 0x0000);
    assert_eq!(machine.cycle_wait, 0);
}

// Addition

#[test]
fn add_register_to_literal() {
    let mut machine = Processor::new();
    let words = Instruction::new(ADD, Value::Register(A), Value::Literal(0x05)).words();
    for (i, &word) in words.iter().enumerate() {
        machine.memory[i as u16] = word;
    }
    machine.set_register(A, 0x1111);
    machine.tick();
    assert_eq!(machine.get_register(A), 0x1116);
    assert_eq!(machine.get_register(PC), 0x0001);
    assert_eq!(machine.get_register(EX), 0x0000);
    assert_eq!(machine.cycle_wait, 1);
}

#[test]
fn add_register_to_register() {
    let mut machine = Processor::new();
    let words = Instruction::new(ADD, Value::Register(B), Value::Register(A)).words();
    for (i, &word) in words.iter().enumerate() {
        machine.memory[i as u16] = word;
    }
    machine.set_register(A, 0x1234);
    machine.set_register(B, 0x0005);
    machine.tick();
    assert_eq!(machine.get_register(A), 0x1234);
    assert_eq!(machine.get_register(B), 0x1239);
    assert_eq!(machine.get_register(PC), 0x0001);
    assert_eq!(machine.get_register(EX), 0x0000);
    assert_eq!(machine.cycle_wait, 1);
}

// Subtraction
// TODO check overflows
#[test]
fn sub_register_to_literal() {
    let mut machine = Processor::new();
    let words = Instruction::new(SUB, Value::Register(A), Value::Literal(0x05)).words();
    for (i, &word) in words.iter().enumerate() {
        machine.memory[i as u16] = word;
    }
    machine.set_register(A, 0x1111);
    machine.tick();
    assert_eq!(machine.get_register(A), 0x110c);
    assert_eq!(machine.get_register(PC), 0x0001);
    assert_eq!(machine.get_register(EX), 0x0000);
    assert_eq!(machine.cycle_wait, 1);
}

#[test]
fn sub_register_to_register() {
    let mut machine = Processor::new();
    let words = Instruction::new(SUB, Value::Register(B), Value::Register(A)).words();
    for (i, &word) in words.iter().enumerate() {
        machine.memory[i as u16] = word;
    }
    machine.set_register(A, 0x0005);
    machine.set_register(B, 0x1234);
    machine.tick();
    assert_eq!(machine.get_register(A), 0x0005);
    assert_eq!(machine.get_register(B), 0x122f);
    assert_eq!(machine.get_register(PC), 0x0001);
    assert_eq!(machine.get_register(EX), 0x0000);
    assert_eq!(machine.cycle_wait, 1);
}

// Multiplication
#[test]
fn mul_register_by_literal() {
    let mut machine = Processor::new();
    let words = Instruction::new(MUL, Value::Register(A), Value::Literal(0x02)).words();
    for (i, &word) in words.iter().enumerate() {
        machine.memory[i as u16] = word;
    }
    machine.set_register(A, 0x0005);
    machine.tick();
    assert_eq!(machine.get_register(A), 0x000A);
    assert_eq!(machine.get_register(PC), 0x0001);
    assert_eq!(machine.get_register(EX), 0x0000);
    assert_eq!(machine.cycle_wait, 1);
}

#[test]
fn mul_register_by_literal_with_overflow() {
    let mut machine = Processor::new();
    let words = Instruction::new(MUL, Value::Register(A), Value::Literal(0x03)).words();
    for (i, &word) in words.iter().enumerate() {
        machine.memory[i as u16] = word;
    }
    machine.set_register(A, 0xFFF0);
    machine.tick();
    assert_eq!(machine.get_register(A), 0xFFD0);
    assert_eq!(machine.get_register(PC), 0x0001);
    assert_eq!(machine.get_register(EX), 0x0002);
    assert_eq!(machine.cycle_wait, 1);
}

#[test]
fn mul_register_by_register() {
    let mut machine = Processor::new();
    let words = Instruction::new(MUL, Value::Register(A), Value::Register(B)).words();
    for (i, &word) in words.iter().enumerate() {
        machine.memory[i as u16] = word;
    }
    machine.set_register(A, 0x0005);
    machine.set_register(B, 0x0002);
    machine.tick();
    assert_eq!(machine.get_register(A), 0x000A);
    assert_eq!(machine.get_register(B), 0x0002);
    assert_eq!(machine.get_register(PC), 0x0001);
    assert_eq!(machine.get_register(EX), 0x0000);
    assert_eq!(machine.cycle_wait, 1);
}

// Multiplication - Signed
#[test]
fn mli_register_by_literal() {
    let mut machine = Processor::new();
    let words = Instruction::new(MLI, Value::Register(A), Value::Literal(0x02)).words();
    for (i, &word) in words.iter().enumerate() {
        machine.memory[i as u16] = word;
    }
    machine.set_signed_register(A, -0x0005);
    machine.tick();
    assert_eq!(machine.get_signed_register(A), -0x000A);
    assert_eq!(machine.get_register(PC), 0x0001);
    assert_eq!(machine.get_register(EX), 0xFFFF);
    assert_eq!(machine.cycle_wait, 1);
}

#[test]
fn mli_register_by_literal_with_overflow() {
    let mut machine = Processor::new();
    let words = Instruction::new(MLI, Value::Register(A), Value::Literal(0x06)).words();
    for (i, &word) in words.iter().enumerate() {
        machine.memory[i as u16] = word;
    }
    machine.set_signed_register(A, -0x7000);
    machine.tick();
    assert_eq!(machine.get_signed_register(A), 0x6000);
    assert_eq!(machine.get_register(PC), 0x0001);
    assert_eq!(machine.get_register(EX), 0xFFFD);
    assert_eq!(machine.cycle_wait, 1);
}

#[test]
fn mli_register_by_register() {
    let mut machine = Processor::new();
    let words = Instruction::new(MLI, Value::Register(A), Value::Register(B)).words();
    for (i, &word) in words.iter().enumerate() {
        machine.memory[i as u16] = word;
    }
    machine.set_signed_register(A, -0x0005);
    machine.set_signed_register(B, 0x0002);
    machine.tick();
    assert_eq!(machine.get_signed_register(A), -0x000A);
    assert_eq!(machine.get_signed_register(B), 0x0002);
    assert_eq!(machine.get_register(PC), 0x0001);
    assert_eq!(machine.get_register(EX), 0xFFFF);
    assert_eq!(machine.cycle_wait, 1);
}

// Division
#[test]
fn div_register_by_literal() {
    let mut machine = Processor::new();
    let words = Instruction::new(DIV, Value::Register(A), Value::Literal(0x05)).words();
    for (i, &word) in words.iter().enumerate() {
        machine.memory[i as u16] = word;
    }
    machine.set_register(A, 0x000A);
    machine.tick();
    assert_eq!(machine.get_register(A), 0x0002);
    assert_eq!(machine.get_register(PC), 0x0001);
    assert_eq!(machine.get_register(EX), 0x0000);
    assert_eq!(machine.cycle_wait, 2);
}

#[test]
fn div_register_by_literal_with_remainder() {
    let mut machine = Processor::new();
    let words = Instruction::new(DIV, Value::Register(A), Value::Literal(0x04)).words();
    for (i, &word) in words.iter().enumerate() {
        machine.memory[i as u16] = word;
    }
    machine.set_register(A, 0x000B);
    machine.tick();
    assert_eq!(machine.get_register(A), 0x0002);
    assert_eq!(machine.get_register(PC), 0x0001);
    assert_eq!(machine.get_register(EX), 0xC000);
    assert_eq!(machine.cycle_wait, 2);
}

#[test]
fn div_register_by_register() {
    let mut machine = Processor::new();
    let words = Instruction::new(DIV, Value::Register(A), Value::Register(B)).words();
    for (i, &word) in words.iter().enumerate() {
        machine.memory[i as u16] = word;
    }
    machine.set_register(A, 0x000A);
    machine.set_register(B, 0x0005);
    machine.tick();
    assert_eq!(machine.get_register(A), 0x0002);
    assert_eq!(machine.get_register(B), 0x0005);
    assert_eq!(machine.get_register(PC), 0x0001);
    assert_eq!(machine.get_register(EX), 0x0000);
    assert_eq!(machine.cycle_wait, 2);
}

#[test]
fn div_register_by_zero() {
    let mut machine = Processor::new();
    let words = Instruction::new(DIV, Value::Register(A), Value::Literal(0x00)).words();
    for (i, &word) in words.iter().enumerate() {
        machine.memory[i as u16] = word;
    }
    machine.set_register(A, 0x000A);
    machine.tick();
    assert_eq!(machine.get_register(A), 0x0000);
    assert_eq!(machine.get_register(PC), 0x0001);
    assert_eq!(machine.get_register(EX), 0x0000);
    assert_eq!(machine.cycle_wait, 2);
}

// Division - Signed
#[test]
fn dvi_register_by_literal_with_remainder() {
    let mut machine = Processor::new();
    let words = Instruction::new(DVI, Value::Register(A), Value::Literal(0x04)).words();
    for (i, &word) in words.iter().enumerate() {
        machine.memory[i as u16] = word;
    }
    machine.set_signed_register(A, -0x000B);
    machine.tick();
    assert_eq!(machine.get_signed_register(A), -0x0002);
    assert_eq!(machine.get_register(PC), 0x0001);
    assert_eq!(machine.get_register(EX), 0x4000);
    assert_eq!(machine.cycle_wait, 2);
}

#[test]
fn dvi_register_by_register_with_remainder() {
    let mut machine = Processor::new();
    let words = Instruction::new(DVI, Value::Register(A), Value::Register(B)).words();
    for (i, &word) in words.iter().enumerate() {
        machine.memory[i as u16] = word;
    }
    machine.set_signed_register(A, -0x000B);
    machine.set_signed_register(B, 0x0004);
    machine.tick();
    assert_eq!(machine.get_signed_register(A), -0x0002);
    assert_eq!(machine.get_signed_register(B), 0x0004);
    assert_eq!(machine.get_register(PC), 0x0001);
    assert_eq!(machine.get_register(EX), 0x4000);
    assert_eq!(machine.cycle_wait, 2);
}

#[test]
fn dvi_register_by_zero() {
    let mut machine = Processor::new();
    let words = Instruction::new(DVI, Value::Register(A), Value::Literal(0x00)).words();
    for (i, &word) in words.iter().enumerate() {
        machine.memory[i as u16] = word;
    }
    machine.set_signed_register(A, -0x000B);
    machine.tick();
    assert_eq!(machine.get_signed_register(A), 0x0000);
    assert_eq!(machine.get_register(PC), 0x0001);
    assert_eq!(machine.get_register(EX), 0x0000);
    assert_eq!(machine.cycle_wait, 2);
}

// Modulus
#[test]
fn mod_register_by_literal() {
    let mut machine = Processor::new();
    let words = Instruction::new(MOD, Value::Register(A), Value::Literal(0x04)).words();
    for (i, &word) in words.iter().enumerate() {
        machine.memory[i as u16] = word;
    }
    machine.set_register(A, 0x000B);
    machine.tick();
    assert_eq!(machine.get_register(A), 0x0003);
    assert_eq!(machine.get_register(PC), 0x0001);
    assert_eq!(machine.get_register(EX), 0x0000);
    assert_eq!(machine.cycle_wait, 2);
}

#[test]
fn mdi_register_by_literal() {
    let mut machine = Processor::new();
    let words = Instruction::new(MDI, Value::Register(A), Value::Literal(0x04)).words();
    for (i, &word) in words.iter().enumerate() {
        machine.memory[i as u16] = word;
    }
    machine.set_signed_register(A, -0x0007);
    machine.tick();
    assert_eq!(machine.get_signed_register(A), -0x0003);
    assert_eq!(machine.get_register(PC), 0x0001);
    assert_eq!(machine.get_register(EX), 0x0000);
    assert_eq!(machine.cycle_wait, 2);
}

// Bitwise
#[test]
fn and_register_with_literal() {
    let mut machine = Processor::new();
    let words = Instruction::new(AND, Value::Register(A), Value::Literal(0x03)).words();
    for (i, &word) in words.iter().enumerate() {
        machine.memory[i as u16] = word;
    }
    machine.set_register(A, 0x0006);
    machine.tick();
    assert_eq!(machine.get_register(A), 0x0002);
    assert_eq!(machine.get_register(PC), 0x0001);
    assert_eq!(machine.get_register(EX), 0x0000);
    assert_eq!(machine.cycle_wait, 0);
}

#[test]
fn bor_register_with_literal() {
    let mut machine = Processor::new();
    let words = Instruction::new(BOR, Value::Register(A), Value::Literal(0x03)).words();
    for (i, &word) in words.iter().enumerate() {
        machine.memory[i as u16] = word;
    }
    machine.set_register(A, 0x0006);
    machine.tick();
    assert_eq!(machine.get_register(A), 0x0007);
    assert_eq!(machine.get_register(PC), 0x0001);
    assert_eq!(machine.get_register(EX), 0x0000);
    assert_eq!(machine.cycle_wait, 0);
}

#[test]
fn xor_register_with_literal() {
    let mut machine = Processor::new();
    let words = Instruction::new(XOR, Value::Register(A), Value::Literal(0x03)).words();
    for (i, &word) in words.iter().enumerate() {
        machine.memory[i as u16] = word;
    }
    machine.set_register(A, 0x0006);
    machine.tick();
    assert_eq!(machine.get_register(A), 0x0005);
    assert_eq!(machine.get_register(PC), 0x0001);
    assert_eq!(machine.get_register(EX), 0x0000);
    assert_eq!(machine.cycle_wait, 0);
}

// Shifting

#[test]
fn shr_register_with_literal() {
    let mut machine = Processor::new();
    let words = Instruction::new(SHR, Value::Register(A), Value::Literal(0x02)).words();
    for (i, &word) in words.iter().enumerate() {
        machine.memory[i as u16] = word;
    }
    machine.set_register(A, 0xF0AA);
    machine.tick();
    assert_eq!(machine.get_register(A), 0x3C2A);
    assert_eq!(machine.get_register(PC), 0x0001);
    assert_eq!(machine.get_register(EX), 0x8000);
    assert_eq!(machine.cycle_wait, 0);
}

#[test]
fn asr_register_with_literal() {
    let mut machine = Processor::new();
    let words = Instruction::new(ASR, Value::Register(A), Value::Literal(0x02)).words();
    for (i, &word) in words.iter().enumerate() {
        machine.memory[i as u16] = word;
    }
    machine.set_register(A, 0xF0AA);
    machine.tick();
    assert_eq!(machine.get_register(A), 0xFC2A);
    assert_eq!(machine.get_register(PC), 0x0001);
    assert_eq!(machine.get_register(EX), 0x8000);
    assert_eq!(machine.cycle_wait, 0);
}

#[test]
fn shl_register_with_literal() {
    let mut machine = Processor::new();
    let words = Instruction::new(SHL, Value::Register(A), Value::Literal(0x02)).words();
    for (i, &word) in words.iter().enumerate() {
        machine.memory[i as u16] = word;
    }
    machine.set_register(A, 0xF0AA);
    machine.tick();
    assert_eq!(machine.get_register(A), 0xC2A8);
    assert_eq!(machine.get_register(PC), 0x0001);
    assert_eq!(machine.get_register(EX), 0x0003);
    assert_eq!(machine.cycle_wait, 0);
}

// Conditionals
#[test]
fn ifb_register_with_literal_when_true() {
    let mut machine = Processor::new();
    let mut program = Program::new();
    program.add(SET, Value::Register(A), Value::Literal(0x03));
    program.add(IFB, Value::Register(A), Value::Literal(0x02));
    program.add(SET, Value::Register(C), Value::NextWord);
    program.add_word(0xBEEF);
    program.add(SET, Value::Register(X), Value::Literal(0x0C));
    machine.memory.load_program(0x0000, &program);

    // Set A
    machine.tick();
    // Run conditional - IFB
    assert_eq!(machine.get_register(PC), 0x0001);
    machine.tick();
    assert_eq!(machine.cycle(), 2);
    machine.tick();
    assert_eq!(machine.cycle(), 3);

    assert_eq!(machine.get_register(PC), 0x0002);

    // Run next instruction - SET
    machine.tick();
    machine.tick();
    assert_eq!(machine.cycle(), 5);
    assert_eq!(machine.get_register(PC), 0x0004);

    assert_eq!(machine.get_register(A), 0x0003);
    assert_eq!(machine.get_register(C), 0xBEEF);
    assert_eq!(machine.get_register(X), 0x0000);
    assert_eq!(machine.get_register(EX), 0x0000);
}

#[test]
fn ifb_register_with_literal_when_false() {
    let mut machine = Processor::new();
    let mut program = Program::new();
    program.add(SET, Value::Register(A), Value::Literal(0x03));
    program.add(IFB, Value::Register(A), Value::Literal(0x08));
    program.add(SET, Value::Register(C), Value::NextWord);
    program.add_word(0xBEEF);
    program.add(SET, Value::Register(X), Value::Literal(0x0C));
    machine.memory.load_program(0x0000, &program);

    // Set A
    machine.tick();
    // Run conditional - IFB
    assert_eq!(machine.get_register(PC), 0x0001);
    machine.tick();
    machine.tick();
    machine.tick();

    assert_eq!(machine.get_register(PC), 0x0004);

    // Run next instruction - SET
    machine.tick();
    assert_eq!(machine.get_register(PC), 0x0005);

    assert_eq!(machine.get_register(A), 0x0003);
    assert_eq!(machine.get_register(C), 0x0000);
    assert_eq!(machine.get_register(X), 0x000C);
    assert_eq!(machine.get_register(EX), 0x0000);
}

#[test]
fn ifc_register_with_literal_when_true() {
    let mut machine = Processor::new();
    let mut program = Program::new();
    program.add(SET, Value::Register(A), Value::Literal(0x03));
    program.add(IFC, Value::Register(A), Value::Literal(0x08));
    program.add(SET, Value::Register(C), Value::NextWord);
    program.add_word(0xBEEF);
    program.add(SET, Value::Register(X), Value::Literal(0x0C));
    machine.memory.load_program(0x0000, &program);

    // Set A
    machine.tick();
    // Run conditional - IFC
    assert_eq!(machine.get_register(PC), 0x0001);
    machine.tick();
    assert_eq!(machine.cycle(), 2);
    machine.tick();
    assert_eq!(machine.cycle(), 3);

    assert_eq!(machine.get_register(PC), 0x0002);

    // Run next instruction - SET
    machine.tick();
    machine.tick();
    assert_eq!(machine.cycle(), 5);
    assert_eq!(machine.get_register(PC), 0x0004);

    assert_eq!(machine.get_register(A), 0x0003);
    assert_eq!(machine.get_register(C), 0xBEEF);
    assert_eq!(machine.get_register(X), 0x0000);
    assert_eq!(machine.get_register(EX), 0x0000);
}

#[test]
fn ifc_register_with_literal_when_false() {
    let mut machine = Processor::new();
    let mut program = Program::new();
    program.add(SET, Value::Register(A), Value::Literal(0x03));
    program.add(IFC, Value::Register(A), Value::Literal(0x02));
    program.add(SET, Value::Register(C), Value::NextWord);
    program.add_word(0xBEEF);
    program.add(SET, Value::Register(X), Value::Literal(0x0C));
    machine.memory.load_program(0x0000, &program);

    // Set A
    machine.tick();
    // Run conditional - IFC
    assert_eq!(machine.get_register(PC), 0x0001);
    machine.tick();
    machine.tick();
    machine.tick();

    assert_eq!(machine.get_register(PC), 0x0004);

    // Run next instruction - SET
    machine.tick();
    assert_eq!(machine.get_register(PC), 0x0005);

    assert_eq!(machine.get_register(A), 0x0003);
    assert_eq!(machine.get_register(C), 0x0000);
    assert_eq!(machine.get_register(X), 0x000C);
    assert_eq!(machine.get_register(EX), 0x0000);
}

#[test]
fn ife_register_with_literal_when_true() {
    let mut machine = Processor::new();
    let mut program = Program::new();
    program.add(SET, Value::Register(A), Value::Literal(0x03));
    program.add(IFE, Value::Register(A), Value::Literal(0x03));
    program.add(SET, Value::Register(C), Value::NextWord);
    program.add_word(0xBEEF);
    program.add(SET, Value::Register(X), Value::Literal(0x0C));
    machine.memory.load_program(0x0000, &program);

    // Set A
    machine.tick();
    // Run conditional - IFE
    assert_eq!(machine.get_register(PC), 0x0001);
    machine.tick();
    assert_eq!(machine.cycle(), 2);
    machine.tick();
    assert_eq!(machine.cycle(), 3);

    assert_eq!(machine.get_register(PC), 0x0002);

    // Run next instruction - SET
    machine.tick();
    machine.tick();
    assert_eq!(machine.cycle(), 5);
    assert_eq!(machine.get_register(PC), 0x0004);

    assert_eq!(machine.get_register(A), 0x0003);
    assert_eq!(machine.get_register(C), 0xBEEF);
    assert_eq!(machine.get_register(X), 0x0000);
    assert_eq!(machine.get_register(EX), 0x0000);
}

#[test]
fn ife_register_with_literal_when_false() {
    let mut machine = Processor::new();
    let mut program = Program::new();
    program.add(SET, Value::Register(A), Value::Literal(0x03));
    program.add(IFE, Value::Register(A), Value::Literal(0x02));
    program.add(SET, Value::Register(C), Value::NextWord);
    program.add_word(0xBEEF);
    program.add(SET, Value::Register(X), Value::Literal(0x0C));
    machine.memory.load_program(0x0000, &program);

    // Set A
    machine.tick();
    // Run conditional - IFE
    assert_eq!(machine.get_register(PC), 0x0001);
    machine.tick();
    machine.tick();
    machine.tick();

    assert_eq!(machine.get_register(PC), 0x0004);

    // Run next instruction - SET
    machine.tick();
    assert_eq!(machine.get_register(PC), 0x0005);

    assert_eq!(machine.get_register(A), 0x0003);
    assert_eq!(machine.get_register(C), 0x0000);
    assert_eq!(machine.get_register(X), 0x000C);
    assert_eq!(machine.get_register(EX), 0x0000);
}

#[test]
fn ifn_register_with_literal_when_true() {
    let mut machine = Processor::new();
    let mut program = Program::new();
    program.add(SET, Value::Register(A), Value::Literal(0x03));
    program.add(IFN, Value::Register(A), Value::Literal(0x02));
    program.add(SET, Value::Register(C), Value::NextWord);
    program.add_word(0xBEEF);
    program.add(SET, Value::Register(X), Value::Literal(0x0C));
    machine.memory.load_program(0x0000, &program);

    // Set A
    machine.tick();
    // Run conditional - IFN
    assert_eq!(machine.get_register(PC), 0x0001);
    machine.tick();
    assert_eq!(machine.cycle(), 2);
    machine.tick();
    assert_eq!(machine.cycle(), 3);

    assert_eq!(machine.get_register(PC), 0x0002);

    // Run next instruction - SET
    machine.tick();
    machine.tick();
    assert_eq!(machine.cycle(), 5);
    assert_eq!(machine.get_register(PC), 0x0004);

    assert_eq!(machine.get_register(A), 0x0003);
    assert_eq!(machine.get_register(C), 0xBEEF);
    assert_eq!(machine.get_register(X), 0x0000);
    assert_eq!(machine.get_register(EX), 0x0000);
}

#[test]
fn ifn_register_with_literal_when_false() {
    let mut machine = Processor::new();
    let mut program = Program::new();
    program.add(SET, Value::Register(A), Value::Literal(0x03));
    program.add(IFN, Value::Register(A), Value::Literal(0x03));
    program.add(SET, Value::Register(C), Value::NextWord);
    program.add_word(0xBEEF);
    program.add(SET, Value::Register(X), Value::Literal(0x0C));
    machine.memory.load_program(0x0000, &program);

    // Set A
    machine.tick();
    // Run conditional - IFN
    assert_eq!(machine.get_register(PC), 0x0001);
    machine.tick();
    machine.tick();
    machine.tick();

    assert_eq!(machine.get_register(PC), 0x0004);

    // Run next instruction - SET
    machine.tick();
    assert_eq!(machine.get_register(PC), 0x0005);

    assert_eq!(machine.get_register(A), 0x0003);
    assert_eq!(machine.get_register(C), 0x0000);
    assert_eq!(machine.get_register(X), 0x000C);
    assert_eq!(machine.get_register(EX), 0x0000);
}

#[test]
fn ifg_register_with_literal_when_true() {
    let mut machine = Processor::new();
    let mut program = Program::new();
    program.add(SET, Value::Register(A), Value::Literal(0x03));
    program.add(IFG, Value::Register(A), Value::Literal(0x02));
    program.add(SET, Value::Register(C), Value::NextWord);
    program.add_word(0xBEEF);
    program.add(SET, Value::Register(X), Value::Literal(0x0C));
    machine.memory.load_program(0x0000, &program);

    // Set A
    machine.tick();
    // Run conditional - IFG
    assert_eq!(machine.get_register(PC), 0x0001);
    machine.tick();
    assert_eq!(machine.cycle(), 2);
    machine.tick();
    assert_eq!(machine.cycle(), 3);

    assert_eq!(machine.get_register(PC), 0x0002);

    // Run next instruction - SET
    machine.tick();
    machine.tick();
    assert_eq!(machine.cycle(), 5);
    assert_eq!(machine.get_register(PC), 0x0004);

    assert_eq!(machine.get_register(A), 0x0003);
    assert_eq!(machine.get_register(C), 0xBEEF);
    assert_eq!(machine.get_register(X), 0x0000);
    assert_eq!(machine.get_register(EX), 0x0000);
}

#[test]
fn ifg_register_with_literal_when_false() {
    let mut machine = Processor::new();
    let mut program = Program::new();
    program.add(SET, Value::Register(A), Value::Literal(0x03));
    program.add(IFG, Value::Register(A), Value::Literal(0x04));
    program.add(SET, Value::Register(C), Value::NextWord);
    program.add_word(0xBEEF);
    program.add(SET, Value::Register(X), Value::Literal(0x0C));
    machine.memory.load_program(0x0000, &program);

    // Set A
    machine.tick();
    // Run conditional - IFG
    assert_eq!(machine.get_register(PC), 0x0001);
    machine.tick();
    machine.tick();
    machine.tick();

    assert_eq!(machine.get_register(PC), 0x0004);

    // Run next instruction - SET
    machine.tick();
    assert_eq!(machine.get_register(PC), 0x0005);

    assert_eq!(machine.get_register(A), 0x0003);
    assert_eq!(machine.get_register(C), 0x0000);
    assert_eq!(machine.get_register(X), 0x000C);
    assert_eq!(machine.get_register(EX), 0x0000);
}

#[test]
fn ifa_register_with_literal_when_true() {
    let mut machine = Processor::new();
    let mut program = Program::new();
    program.add(SET, Value::Register(A), Value::NextWord);
    program.add_word(to_unsigned(-0x02));
    program.add(IFA, Value::Register(A), Value::NextWord);
    program.add_word(to_unsigned(-0x03));
    program.add(SET, Value::Register(C), Value::NextWord);
    program.add_word(0xBEEF);
    program.add(SET, Value::Register(X), Value::Literal(0x0C));
    machine.memory.load_program(0x0000, &program);

    // Set A
    machine.tick();
    machine.tick();
    // Run conditional - IFA
    assert_eq!(machine.get_register(PC), 0x0002);
    machine.tick();
    machine.tick();
    machine.tick();

    assert_eq!(machine.get_register(PC), 0x0004);

    // Run next instruction - SET
    machine.tick();
    machine.tick();
    assert_eq!(machine.get_register(PC), 0x0006);

    assert_eq!(machine.get_signed_register(A), -0x0002);
    assert_eq!(machine.get_register(C), 0xBEEF);
    assert_eq!(machine.get_register(X), 0x0000);
    assert_eq!(machine.get_register(EX), 0x0000);
}

#[test]
fn ifa_register_with_literal_when_false() {
    let mut machine = Processor::new();
    let mut program = Program::new();
    program.add(SET, Value::Register(A), Value::NextWord);
    program.add_word(to_unsigned(-0x02));
    program.add(IFA, Value::Register(A), Value::NextWord);
    program.add_word(to_unsigned(0x03));
    program.add(SET, Value::Register(C), Value::NextWord);
    program.add_word(0xBEEF);
    program.add(SET, Value::Register(X), Value::Literal(0x0C));
    machine.memory.load_program(0x0000, &program);

    // Set A
    machine.tick();
    machine.tick();
    // Run conditional - IFA
    assert_eq!(machine.get_register(PC), 0x0002);
    machine.tick();
    machine.tick();
    machine.tick();

    assert_eq!(machine.get_register(PC), 0x0006);

    // Run next instruction - SET
    machine.tick();
    machine.tick();
    assert_eq!(machine.get_register(PC), 0x0007);

    assert_eq!(machine.get_signed_register(A), -0x0002);
    assert_eq!(machine.get_register(C), 0x0000);
    assert_eq!(machine.get_register(X), 0x000C);
    assert_eq!(machine.get_register(EX), 0x0000);
}

#[test]
fn ifl_register_with_literal_when_true() {
    let mut machine = Processor::new();
    let mut program = Program::new();
    program.add(SET, Value::Register(A), Value::Literal(0x02));
    program.add(IFL, Value::Register(A), Value::Literal(0x03));
    program.add(SET, Value::Register(C), Value::NextWord);
    program.add_word(0xBEEF);
    program.add(SET, Value::Register(X), Value::Literal(0x0C));
    machine.memory.load_program(0x0000, &program);

    // Set A
    machine.tick();
    // Run conditional - IFL
    assert_eq!(machine.get_register(PC), 0x0001);
    machine.tick();
    assert_eq!(machine.cycle(), 2);
    machine.tick();
    assert_eq!(machine.cycle(), 3);

    assert_eq!(machine.get_register(PC), 0x0002);

    // Run next instruction - SET
    machine.tick();
    machine.tick();
    assert_eq!(machine.cycle(), 5);
    assert_eq!(machine.get_register(PC), 0x0004);

    assert_eq!(machine.get_register(A), 0x0002);
    assert_eq!(machine.get_register(C), 0xBEEF);
    assert_eq!(machine.get_register(X), 0x0000);
    assert_eq!(machine.get_register(EX), 0x0000);
}

#[test]
fn ifl_register_with_literal_when_false() {
    let mut machine = Processor::new();
    let mut program = Program::new();
    program.add(SET, Value::Register(A), Value::Literal(0x04));
    program.add(IFL, Value::Register(A), Value::Literal(0x03));
    program.add(SET, Value::Register(C), Value::NextWord);
    program.add_word(0xBEEF);
    program.add(SET, Value::Register(X), Value::Literal(0x0C));
    machine.memory.load_program(0x0000, &program);

    // Set A
    machine.tick();
    // Run conditional - IFL
    assert_eq!(machine.get_register(PC), 0x0001);
    machine.tick();
    machine.tick();
    machine.tick();

    assert_eq!(machine.get_register(PC), 0x0004);

    // Run next instruction - SET
    machine.tick();
    assert_eq!(machine.get_register(PC), 0x0005);

    assert_eq!(machine.get_register(A), 0x0004);
    assert_eq!(machine.get_register(C), 0x0000);
    assert_eq!(machine.get_register(X), 0x000C);
    assert_eq!(machine.get_register(EX), 0x0000);
}

#[test]
fn ifu_register_with_literal_when_true() {
    let mut machine = Processor::new();
    let mut program = Program::new();
    program.add(SET, Value::Register(A), Value::NextWord);
    program.add_word(to_unsigned(-0x03));
    program.add(IFU, Value::Register(A), Value::NextWord);
    program.add_word(to_unsigned(-0x02));
    program.add(SET, Value::Register(C), Value::NextWord);
    program.add_word(0xBEEF);
    program.add(SET, Value::Register(X), Value::Literal(0x0C));
    machine.memory.load_program(0x0000, &program);

    // Set A
    machine.tick();
    machine.tick();
    // Run conditional - IFU
    assert_eq!(machine.get_register(PC), 0x0002);
    machine.tick();
    machine.tick();
    machine.tick();

    assert_eq!(machine.get_register(PC), 0x0004);

    // Run next instruction - SET
    machine.tick();
    machine.tick();
    assert_eq!(machine.get_register(PC), 0x0006);

    assert_eq!(machine.get_signed_register(A), -0x0003);
    assert_eq!(machine.get_register(C), 0xBEEF);
    assert_eq!(machine.get_register(X), 0x0000);
    assert_eq!(machine.get_register(EX), 0x0000);
}

#[test]
fn ifu_register_with_literal_when_false() {
    let mut machine = Processor::new();
    let mut program = Program::new();
    program.add(SET, Value::Register(A), Value::NextWord);
    program.add_word(to_unsigned(0x02));
    program.add(IFU, Value::Register(A), Value::NextWord);
    program.add_word(to_unsigned(-0x03));
    program.add(SET, Value::Register(C), Value::NextWord);
    program.add_word(0xBEEF);
    program.add(SET, Value::Register(X), Value::Literal(0x0C));
    machine.memory.load_program(0x0000, &program);

    // Set A
    machine.tick();
    machine.tick();
    // Run conditional - IFU
    assert_eq!(machine.get_register(PC), 0x0002);
    machine.tick();
    machine.tick();
    machine.tick();

    assert_eq!(machine.get_register(PC), 0x0006);

    // Run next instruction - SET
    machine.tick();
    machine.tick();
    assert_eq!(machine.get_register(PC), 0x0007);

    assert_eq!(machine.get_signed_register(A), 0x0002);
    assert_eq!(machine.get_register(C), 0x0000);
    assert_eq!(machine.get_register(X), 0x000C);
    assert_eq!(machine.get_register(EX), 0x0000);
}

#[test]
fn adx_register_with_literal() {
    let mut machine = Processor::new();
    let mut program = Program::new();

    // Set A,B to 0x0F0A2C2A
    program.add(SET, Value::Register(A), Value::NextWord);
    program.add_word(0x0F0A);
    program.add(SET, Value::Register(B), Value::NextWord);
    program.add_word(0x2C2A);

    // Set X,Y to 0x000186A0
    program.add(SET, Value::Register(X), Value::NextWord);
    program.add_word(0xFF03);
    program.add(SET, Value::Register(Y), Value::NextWord);
    program.add_word(0x86A0);

    // AB + XY = 0xBB2CA

    // Add AB to XY
    program.add(ADD, Value::Register(B), Value::Register(Y));
    program.add(ADX, Value::Register(A), Value::Register(X));

    machine.memory.load_program(0x0000, &program);

    // Set A
    machine.tick();
    machine.tick();
    // Set B
    machine.tick();
    machine.tick();

    // Set X
    machine.tick();
    machine.tick();
    // Set Y
    machine.tick();
    machine.tick();

    // Add lows
    machine.tick();
    machine.tick();

    // ADX highs
    machine.tick();
    machine.tick();
    machine.tick();

    assert_eq!(machine.get_register(A), 0x0E0D);
    assert_eq!(machine.get_register(B), 0xB2CA);
    assert_eq!(machine.get_register(X), 0xFF03);
    assert_eq!(machine.get_register(Y), 0x86A0);
    assert_eq!(machine.get_register(PC), 0x000A);
    assert_eq!(machine.get_register(EX), 0x0001);
}

#[test]
fn sbx_register_with_literal() {
    let mut machine = Processor::new();
    let mut program = Program::new();

    // Set A,B to 0x0F0A2C2A
    program.add(SET, Value::Register(A), Value::NextWord);
    program.add_word(0x0F0A);
    program.add(SET, Value::Register(B), Value::NextWord);
    program.add_word(0x2C2A);

    // Set X,Y to 0x000186A0
    program.add(SET, Value::Register(X), Value::NextWord);
    program.add_word(0xFF03);
    program.add(SET, Value::Register(Y), Value::NextWord);
    program.add_word(0x86A0);

    // AB + XY = 0xBB2CA

    // Add AB to XY
    program.add(ADD, Value::Register(B), Value::Register(Y));
    program.add(SBX, Value::Register(A), Value::Register(X));

    machine.memory.load_program(0x0000, &program);

    // Set A
    machine.tick();
    machine.tick();
    // Set B
    machine.tick();
    machine.tick();

    // Set X
    machine.tick();
    machine.tick();
    // Set Y
    machine.tick();
    machine.tick();

    // Add lows
    machine.tick();
    machine.tick();

    // SBX highs
    machine.tick();
    machine.tick();
    machine.tick();

    assert_eq!(machine.get_register(A), 0x1007);
    assert_eq!(machine.get_register(B), 0xB2CA);
    assert_eq!(machine.get_register(X), 0xFF03);
    assert_eq!(machine.get_register(Y), 0x86A0);
    assert_eq!(machine.get_register(PC), 0x000A);
    assert_eq!(machine.get_register(EX), 0xFFFF);
}

#[test]
fn sti_register_with_literal() {
    let mut machine = Processor::new();
    let mut program = Program::new();
    program.add(SET, Value::Register(I), Value::Literal(0x0003));
    program.add(SET, Value::Register(J), Value::Literal(0x0005));
    program.add(STI, Value::Register(A), Value::Literal(0x0001));
    program.add(STI, Value::Register(A), Value::Literal(0x0004));
    program.add(STI, Value::Register(A), Value::Literal(0x000B));
    machine.memory.load_program(0x0000, &program);

    // Set I
    machine.tick();
    // Set J
    machine.tick();
    assert_eq!(machine.get_register(I), 0x0003);
    assert_eq!(machine.get_register(J), 0x0005);

    // STI 1
    machine.tick();
    machine.tick();
    assert_eq!(machine.get_register(A), 0x0001);
    assert_eq!(machine.get_register(I), 0x0004);
    assert_eq!(machine.get_register(J), 0x0006);
    assert_eq!(machine.get_register(PC), 0x0003);
    // STI 2
    machine.tick();
    machine.tick();
    assert_eq!(machine.get_register(A), 0x0004);
    assert_eq!(machine.get_register(I), 0x0005);
    assert_eq!(machine.get_register(J), 0x0007);
    assert_eq!(machine.get_register(PC), 0x0004);
    // STI 3
    machine.tick();
    machine.tick();
    assert_eq!(machine.get_register(A), 0x000B);
    assert_eq!(machine.get_register(I), 0x0006);
    assert_eq!(machine.get_register(J), 0x0008);

    assert_eq!(machine.get_register(PC), 0x0005);
    assert_eq!(machine.get_register(EX), 0x0000);
}

#[test]
fn std_register_with_literal() {
    let mut machine = Processor::new();
    let mut program = Program::new();
    program.add(SET, Value::Register(I), Value::Literal(0x0003));
    program.add(SET, Value::Register(J), Value::Literal(0x0005));
    program.add(STD, Value::Register(A), Value::Literal(0x0001));
    program.add(STD, Value::Register(A), Value::Literal(0x0004));
    program.add(STD, Value::Register(A), Value::Literal(0x000B));
    machine.memory.load_program(0x0000, &program);

    // Set I
    machine.tick();
    // Set J
    machine.tick();
    assert_eq!(machine.get_register(I), 0x0003);
    assert_eq!(machine.get_register(J), 0x0005);

    // STD 1
    machine.tick();
    machine.tick();
    assert_eq!(machine.get_register(A), 0x0001);
    assert_eq!(machine.get_register(I), 0x0002);
    assert_eq!(machine.get_register(J), 0x0004);
    assert_eq!(machine.get_register(PC), 0x0003);
    // STD 2
    machine.tick();
    machine.tick();
    assert_eq!(machine.get_register(A), 0x0004);
    assert_eq!(machine.get_register(I), 0x0001);
    assert_eq!(machine.get_register(J), 0x0003);
    assert_eq!(machine.get_register(PC), 0x0004);
    // STD 3
    machine.tick();
    machine.tick();
    assert_eq!(machine.get_register(A), 0x000B);
    assert_eq!(machine.get_register(I), 0x0000);
    assert_eq!(machine.get_register(J), 0x0002);

    assert_eq!(machine.get_register(PC), 0x0005);
    assert_eq!(machine.get_register(EX), 0x0000);
}

#[test]
fn jsr_with_literal() {
    let mut machine = Processor::new();
    let mut program = Program::new();
    program.add(SPL, Value::OpCode(JSR), Value::Literal(0x04));
    machine.memory.load_program(0x0000, &program);

    machine.tick();
    machine.tick();
    machine.tick();
    assert_eq!(machine.get_register(SP), 0xFFFF);
    assert_eq!(machine.get_register(PC), 0x04);
}

#[test]
fn iag_with_register() {
    let mut machine = Processor::new();
    let mut program = Program::new();
    machine.set_register(IA, 0x1234);
    program.add(SPL, Value::OpCode(IAG), Value::Register(A));
    machine.memory.load_program(0x0000, &program);

    machine.tick();
    assert_eq!(machine.get_register(A), 0x1234);
    assert_eq!(machine.get_register(IA), 0x1234);
    assert_eq!(machine.get_register(PC), 0x0001);
    assert_eq!(machine.get_register(EX), 0x0000);
}

#[test]
fn ias_with_register() {
    let mut machine = Processor::new();
    let mut program = Program::new();
    machine.set_register(A, 0x1234);
    program.add(SPL, Value::OpCode(IAS), Value::Register(A));
    machine.memory.load_program(0x0000, &program);

    machine.tick();
    assert_eq!(machine.get_register(A), 0x1234);
    assert_eq!(machine.get_register(IA), 0x1234);
    assert_eq!(machine.get_register(PC), 0x0001);
    assert_eq!(machine.get_register(EX), 0x0000);
}

// Interrupts
#[test]
fn trigger_a_software_interrupt() {
    let mut machine = Processor::new();

    // Small program that'll be called when an interrupt triggers
    let mut interrupt_handler = Program::new();
    interrupt_handler.add(SET, Value::Register(B), Value::Literal(0x06));
    interrupt_handler.add(ADD, Value::Register(B), Value::Register(A));
    interrupt_handler.add(SPL, Value::OpCode(RFI), Value::Literal(0x00));
    machine.memory.load_program(0x4000, &interrupt_handler);

    // Program to trigger the interrupt
    let mut program = Program::new();
    program.add(SET, Value::Register(A), Value::Literal(0x0A));
    program.add(SPL, Value::OpCode(IAS), Value::NextWord);
    program.add_word(0x4000); // Where the handler lives
    program.add(SPL, Value::OpCode(INT), Value::Literal(0x03));
    machine.memory.load_program(0x0000, &program);

    // Set A
    machine.tick();
    // Set IA
    machine.tick();
    assert_eq!(machine.get_register(IA), 0x4000);
    // Trigger interrupt
    machine.tick();
    machine.tick();
    machine.tick();
    machine.tick();
    assert_eq!(machine.get_register(PC), 0x4000);
    assert_eq!(machine.get_register(A), 0x03);

    // Set B
    machine.tick();
    // Add A (interrupt message)
    machine.tick();
    machine.tick();
    assert_eq!(machine.get_register(B), 0x09);
    // Return
    machine.tick();
    machine.tick();
    machine.tick();
    assert_eq!(machine.get_register(PC), 0x04);
    assert_eq!(machine.get_register(A), 0x0A);
}

// Stack
#[test]
fn push_and_pop_the_stack() {
    let mut machine = Processor::new();
    let mut program = Program::new();
    program.add(SET, Value::Register(A), Value::Literal(0x0A));
    program.add(SET, Value::Push, Value::Literal(0x05));
    program.add(SET, Value::Push, Value::Register(A));
    program.add(SET, Value::Register(B), Value::Pop);
    program.add(SET, Value::Register(C), Value::Pop);
    machine.memory.load_program(0x0000, &program);

    machine.tick(); // Set A
    assert_eq!(machine.get_register(SP), 0x0000);
    machine.tick(); // Push 0x05
    assert_eq!(machine.get_register(SP), 0xFFFF);
    machine.tick(); // Push A
    assert_eq!(machine.get_register(SP), 0xFFFE);
    machine.tick(); // Pop B
    assert_eq!(machine.get_register(SP), 0xFFFF);
    machine.tick(); // Pop C
    assert_eq!(machine.get_register(SP), 0x0000);

    assert_eq!(machine.get_register(A), 0x000A);
    assert_eq!(machine.get_register(B), 0x000A);
    assert_eq!(machine.get_register(C), 0x0005);
    assert_eq!(machine.get_register(PC), 0x0005);
    assert_eq!(machine.get_register(EX), 0x0000);
}

// Setters
#[test]
fn write_to_literal_memory_address() {
    let mut machine = Processor::new();
    let mut program = Program::new();
    program.add(SET, Value::NextWordPointer, Value::Literal(0x0A));
    program.add_word(0xBEEF);
    machine.memory.load_program(0x0000, &program);

    machine.tick();
    machine.tick();

    assert_eq!(machine.get_memory(0xBEEF), 0x000A);
    assert_eq!(machine.get_register(PC), 0x0002);
}

#[test]
fn write_to_register_relative_memory_address() {
    let mut machine = Processor::new();
    let mut program = Program::new();
    program.add(SET, Value::Register(A), Value::Literal(0x0A));
    program.add(SET, Value::RegisterPointerOffset(A), Value::Literal(0x04));
    program.add_word(0x05);
    machine.memory.load_program(0x0000, &program);

    machine.tick();
    machine.tick();
    machine.tick();

    assert_eq!(machine.get_memory(0x000F), 0x04);
    assert_eq!(machine.get_register(PC), 0x0003);
}

// Getters
#[test]
fn read_from_literal_memory_address() {
    let mut machine = Processor::new();
    let mut program = Program::new();
    program.add(SET, Value::Register(A), Value::NextWordPointer);
    program.add_word(0xBEEF);
    machine.memory.load_program(0x0000, &program);
    machine.set_memory(0xBEEF, 0x1234);

    machine.tick();
    machine.tick();

    assert_eq!(machine.get_register(A), 0x1234);
    assert_eq!(machine.get_register(PC), 0x0002);
}

#[test]
fn read_from_register_relative_memory_address() {
    let mut machine = Processor::new();
    let mut program = Program::new();
    program.add(SET, Value::Register(A), Value::Literal(0x0A));
    program.add(SET, Value::Register(B), Value::RegisterPointerOffset(A));
    program.add_word(0x05);
    machine.memory.load_program(0x0000, &program);
    machine.set_memory(0x000F, 0x1234);

    machine.tick();
    machine.tick();
    machine.tick();

    assert_eq!(machine.get_register(B), 0x1234);
    assert_eq!(machine.get_register(PC), 0x0003);
}

#[test]
fn copy_from_memory_to_memory_using_literals() {
    let mut machine = Processor::new();
    let mut program = Program::new();
    program.add(SET, Value::NextWordPointer, Value::NextWordPointer);
    program.add_word(0xDEAD);
    program.add_word(0xBEEF);
    machine.memory.load_program(0x0000, &program);
    machine.set_memory(0xDEAD, 0x5555);
    machine.set_memory(0xBEEF, 0x2222);

    machine.tick();
    machine.tick();

    assert_eq!(machine.get_memory(0xDEAD), 0x5555);
    assert_eq!(machine.get_memory(0xBEEF), 0x5555);
    assert_eq!(machine.get_register(PC), 0x0003);
}

#[test]
fn copy_from_memory_to_memory_using_registers() {
    let mut machine = Processor::new();
    let mut program = Program::new();
    program.add(SET, Value::Register(A), Value::NextWord);
    program.add_word(0xDEAD);
    program.add(SET, Value::Register(B), Value::NextWord);
    program.add_word(0xBEEF);
    program.add(SET, Value::RegisterPointer(B), Value::RegisterPointer(A));
    machine.memory.load_program(0x0000, &program);
    machine.set_memory(0xDEAD, 0x5555);
    machine.set_memory(0xBEEF, 0x2222);

    machine.tick();
    machine.tick();
    machine.tick();
    machine.tick();
    machine.tick();

    assert_eq!(machine.get_memory(0xDEAD), 0x5555);
    assert_eq!(machine.get_memory(0xBEEF), 0x5555);
    assert_eq!(machine.get_register(PC), 0x0005);
}

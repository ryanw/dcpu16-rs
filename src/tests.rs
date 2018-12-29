use super::*;

#[test]
fn set_register_to_next_word() {
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
    assert_eq!(machine.get_register(PC), 0x0002);
    assert_eq!(machine.cycle_wait, 1);
}

// Setting

#[test]
fn set_register_to_literal() {
    let mut machine = Processor::new();
    let words =
        Instruction::new(SET, Value::Register(A), Value::Literal(0x05)).words(&mut machine);
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
    let words =
        Instruction::new(SET, Value::Register(B), Value::Register(A)).words(&mut machine);
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
// TODO check overflows

#[test]
fn add_register_to_literal() {
    let mut machine = Processor::new();
    let words =
        Instruction::new(ADD, Value::Register(A), Value::Literal(0x05)).words(&mut machine);
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
    let words =
        Instruction::new(ADD, Value::Register(B), Value::Register(A)).words(&mut machine);
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
    let words =
        Instruction::new(SUB, Value::Register(A), Value::Literal(0x05)).words(&mut machine);
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
    let words =
        Instruction::new(SUB, Value::Register(B), Value::Register(A)).words(&mut machine);
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
    let words =
        Instruction::new(MUL, Value::Register(A), Value::Literal(0x02)).words(&mut machine);
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
    let words =
        Instruction::new(MUL, Value::Register(A), Value::Literal(0x03)).words(&mut machine);
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
    let words =
        Instruction::new(MUL, Value::Register(A), Value::Register(B)).words(&mut machine);
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
    let words =
        Instruction::new(MLI, Value::Register(A), Value::Literal(0x02)).words(&mut machine);
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
    let words =
        Instruction::new(MLI, Value::Register(A), Value::Literal(0x06)).words(&mut machine);
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
    let words =
        Instruction::new(MLI, Value::Register(A), Value::Register(B)).words(&mut machine);
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
    let words =
        Instruction::new(DIV, Value::Register(A), Value::Literal(0x05)).words(&mut machine);
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
    let words =
        Instruction::new(DIV, Value::Register(A), Value::Literal(0x04)).words(&mut machine);
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
    let words =
        Instruction::new(DIV, Value::Register(A), Value::Register(B)).words(&mut machine);
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
    let words =
        Instruction::new(DIV, Value::Register(A), Value::Literal(0x00)).words(&mut machine);
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
    let words =
        Instruction::new(DVI, Value::Register(A), Value::Literal(0x04)).words(&mut machine);
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
    let words =
        Instruction::new(DVI, Value::Register(A), Value::Register(B)).words(&mut machine);
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
    let words =
        Instruction::new(DVI, Value::Register(A), Value::Literal(0x00)).words(&mut machine);
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
    let words =
        Instruction::new(MOD, Value::Register(A), Value::Literal(0x04)).words(&mut machine);
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
    let words =
        Instruction::new(MDI, Value::Register(A), Value::Literal(0x04)).words(&mut machine);
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
    let words =
        Instruction::new(AND, Value::Register(A), Value::Literal(0x03)).words(&mut machine);
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
    let words =
        Instruction::new(BOR, Value::Register(A), Value::Literal(0x03)).words(&mut machine);
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
    let words =
        Instruction::new(XOR, Value::Register(A), Value::Literal(0x03)).words(&mut machine);
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
    let words =
        Instruction::new(SHR, Value::Register(A), Value::Literal(0x02)).words(&mut machine);
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
    let words =
        Instruction::new(ASR, Value::Register(A), Value::Literal(0x02)).words(&mut machine);
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
    let words =
        Instruction::new(SHL, Value::Register(A), Value::Literal(0x02)).words(&mut machine);
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


use dcpu16_rs::*;
use std::thread::sleep;
use std::time::Duration;

fn draw_screen(machine: &Processor) {
    machine.with_hardware(0, |monitor: &Monitor, machine| {
        print!("{}", monitor.render_24bit_ansi(machine));
    });
}

fn main() {
    print!("\x1b[2J");
    let prog = include_bytes!("../progs/nyan.bin");
    let mut machine = Processor::new();

    let mut word: u16 = 0;
    for (i, &nibble) in prog.iter().enumerate() {
        if i % 2 != 0 {
            word |= nibble as u16;
            machine.set_memory((i / 2) as u16, word);
        }
        else {
            word = (nibble as u16) << 8;
        }

    }

    machine.connect_hardware(Monitor::new());

    loop {
        for _ in 0..5000 {
            machine.tick();
        }

        draw_screen(&machine);
        sleep(Duration::from_micros(50000));
    }
}

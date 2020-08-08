extern crate chip8;
use chip8::emulator::{basics::{SCREEN_HEIGHT, SCREEN_WIDTH}, vm::VirtualMachine};
use std::{io::Read, fs::File};

const ROM_FILE: &str = "tests/emulator/test_opcode.ch8";

const EXPECTED_OUTPUT: &str = 
"@@@ @ @  @@@ @ @      @@@ @@@  @@@ @ @     @@@  @@ @@@ @ @      
 @@  @   @ @ @@       @ @ @@   @ @ @@      @@@  @  @ @ @@       
  @ @ @  @ @ @ @      @ @ @    @ @ @ @     @ @   @ @ @ @ @      
@@@ @ @  @@@ @ @      @@@ @@@  @@@ @ @     @@@  @  @@@ @ @      
                                                                
@ @ @ @  @@@ @ @      @@@ @@@  @@@ @ @     @@@ @@@ @@@ @ @      
@@@  @   @ @ @@       @@@ @ @  @ @ @@      @@@ @   @ @ @@       
  @ @ @  @ @ @ @      @ @ @ @  @ @ @ @     @ @ @@@ @ @ @ @      
  @ @ @  @@@ @ @      @@@ @@@  @@@ @ @     @@@ @@@ @@@ @ @      
                                                                
 @@ @ @  @@@ @ @      @@@ @@   @@@ @ @     @@@ @@@ @@@ @ @      
 @   @   @ @ @@       @@@  @   @ @ @@      @@@ @@  @ @ @@       
  @ @ @  @ @ @ @      @ @  @   @ @ @ @     @ @ @   @ @ @ @      
 @  @ @  @@@ @ @      @@@ @@@  @@@ @ @     @@@ @@@ @@@ @ @      
                                                                
@@@ @ @  @@@ @ @      @@@ @@@  @@@ @ @     @@@  @@ @@@ @ @      
  @  @   @ @ @@       @@@   @  @ @ @@      @    @  @ @ @@       
  @ @ @  @ @ @ @      @ @ @@   @ @ @ @     @@    @ @ @ @ @      
  @ @ @  @@@ @ @      @@@ @@@  @@@ @ @     @    @  @@@ @ @      
                                                                
@@@ @ @  @@@ @ @      @@@ @@@  @@@ @ @     @@@ @@@ @@@ @ @      
@@@  @   @ @ @@       @@@  @@  @ @ @@      @    @@ @ @ @@       
  @ @ @  @ @ @ @      @ @   @  @ @ @ @     @@    @ @ @ @ @      
@@@ @ @  @@@ @ @      @@@ @@@  @@@ @ @     @   @@@ @@@ @ @      
                                                                
 @  @ @  @@@ @ @      @@@ @ @  @@@ @ @     @@  @ @ @@@ @ @      
@ @  @   @ @ @@       @@@ @@@  @ @ @@       @   @  @ @ @@       
@@@ @ @  @ @ @ @      @ @   @  @ @ @ @      @  @ @ @ @ @ @      
@ @ @ @  @@@ @ @      @@@   @  @@@ @ @     @@@ @ @ @@@ @ @      
                                                                
                                                                
                                                                ";

fn expected_display() -> [[bool; SCREEN_HEIGHT as usize]; SCREEN_WIDTH as usize] {
    assert_eq!(SCREEN_HEIGHT as usize * SCREEN_WIDTH as usize, EXPECTED_OUTPUT.chars().filter(|c| *c != '\n').count());
    let mut result = [[false; SCREEN_HEIGHT as usize]; SCREEN_WIDTH as usize];
    for (i, row) in EXPECTED_OUTPUT.split("\n").enumerate() {
        for (j, chr) in row.chars().enumerate() {
            result[j][i] = chr == '@';
        }
    }
    result
}


fn load_rom() -> VirtualMachine {
    let mut file = File::open(ROM_FILE).unwrap();
    let mut raw_rom = Vec::new();
    file.read_to_end(&mut raw_rom).unwrap();
    VirtualMachine::new(&raw_rom)
}

fn run_until_loop(vm: &mut VirtualMachine) {
    loop {
        let pc = vm.program_counter;
        vm.step();
        if vm.program_counter == pc {
            break;
        }
    }
}

#[test]
fn test_opcode8() {
    let mut vm = load_rom();
    run_until_loop(&mut vm);
    let display = vm.interface.lock().unwrap().display;
    let expected = expected_display();
    for x in 0..SCREEN_WIDTH as usize {
        for y in 0..SCREEN_HEIGHT as usize {
            assert_eq!(display[x][y], expected[x][y], "mismatch at {:?}", (x, y));
        }
    }
}
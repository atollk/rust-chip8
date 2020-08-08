mod emulator;
mod visualizer;

use emulator::program::Instruction;
use emulator::basics::*;

fn main() {
    let mut vm = emulator::vm::VirtualMachine::new(&[]);
    vm.execute_instruction(&Instruction::Draw(Register(0), Register(0), Value(5)));
    visualizer::main(vm.interface.clone());
    println!("Hello, world!");
}

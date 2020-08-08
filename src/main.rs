mod emulator;
mod visualizer;

use emulator::basics::*;
use emulator::program::Instruction;

fn main() {
    let mut vm = emulator::vm::VirtualMachine::new(&[]);
    vm.execute_instruction(&Instruction::Draw(Register(0), Register(0), Value(5)));
    let mut vis = visualizer::Visualizer::new(vm.interface.clone());
    vis.wait_for_init();
    println!("Hello, world!");
}

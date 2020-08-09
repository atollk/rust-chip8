mod emulator;
mod visualizer;

use emulator::executor;
use emulator::vm::VirtualMachine;
use std::{
    fs::File,
    io::Read,
    sync::{Arc, Mutex},
};

fn load_rom(filename: &str) -> VirtualMachine {
    let mut file = File::open(filename).unwrap();
    let mut raw_rom = Vec::new();
    file.read_to_end(&mut raw_rom).unwrap();
    VirtualMachine::new(&raw_rom)
}

const ROM_FILE: &str = "roms/CONNECT4";

fn main() {
    let mut vm = load_rom(ROM_FILE);
    let mut vis = visualizer::Visualizer::new(vm.interface.clone());
    let stop_vm = Arc::new(Mutex::new(false));
    vis.wait_for_init();
    executor::run_concurrent_vm_until(vm, stop_vm.clone());
    vis.wait_for_close();
    *stop_vm.lock().unwrap() = true;
}

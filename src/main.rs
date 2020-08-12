mod emulator;
mod rom_config;
mod visualizer;

use rom_config::load_rom;
use std::sync::{Arc, Mutex};

fn main() {
    let (executor, vis) = load_rom("connect4");
    let stop_vm = Arc::new(Mutex::new(false));
    vis.wait_for_init();
    executor.run_concurrent_until(stop_vm.clone());
    vis.wait_for_close();
    *stop_vm.lock().unwrap() = true;
}

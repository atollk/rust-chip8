use super::vm::VirtualMachine;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

const INSTRUCTION_SLEEP: Duration = Duration::from_millis(2);
const TIMER_INTERVAL: Duration = Duration::from_micros(16667);

pub fn run_concurrent_vm_until(mut vm: VirtualMachine, stopper: Arc<Mutex<bool>>) {
    let interface = vm.interface.clone();
    let stopper2 = stopper.clone();
    thread::spawn(move || loop {
        if *stopper.lock().unwrap() {
            break;
        }
        {
            let mut guard = interface.lock().unwrap();
            if guard.delay_timer.0 > 0 {
                guard.delay_timer.0 -= 1;
            }
            if guard.sound_timer.0 > 0 {
                guard.sound_timer.0 -= 1;
            }
        }
        thread::sleep(TIMER_INTERVAL);
    });
    thread::spawn(move || loop {
        if *stopper2.lock().unwrap() {
            break;
        }
        vm.step();
        thread::sleep(INSTRUCTION_SLEEP);
    });
}

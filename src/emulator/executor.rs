use super::vm::VirtualMachine;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

pub struct Executor {
    instruction_sleep: Duration,
    timer_interval: Duration,
    vm: VirtualMachine,
}

impl Executor {
    pub fn new(
        instruction_sleep: Duration,
        timer_interval: Duration,
        vm: VirtualMachine,
    ) -> Executor {
        Executor {
            instruction_sleep,
            timer_interval,
            vm,
        }
    }

    pub fn run_concurrent_until(mut self, stopper: Arc<Mutex<bool>>) {
        let interface = self.vm.interface.clone();
        let stopper2 = stopper.clone();
        let timer_interval = self.timer_interval;
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
            thread::sleep(timer_interval);
        });
        thread::spawn(move || loop {
            if *stopper2.lock().unwrap() {
                break;
            }
            self.vm.step();
            thread::sleep(self.instruction_sleep);
        });
    }
}

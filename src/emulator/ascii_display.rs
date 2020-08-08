use super::{
    basics::{SCREEN_HEIGHT, SCREEN_WIDTH},
    vm::VirtualMachine,
};

pub fn draw_vm_display(vm: &VirtualMachine) {
    let display = &vm.interface.lock().unwrap().display;
    for y in 0..SCREEN_HEIGHT as usize {
        for x in 0..SCREEN_WIDTH as usize {
            if display[x][y] {
                print!("@");
            } else {
                print!(" ");
            }
        }
        println!();
    }
}

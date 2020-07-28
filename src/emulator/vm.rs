use super::basics::{
    Address, Register, Value, FONT_OFFSET, MEMORY_SIZE, SCREEN_HEIGHT, SCREEN_WIDTH, STACK_DEPTH,
};
use super::program::Instruction;
use rand::Rng;
use std::sync::Mutex;
use std::{thread, time::Duration};

/// Holds the logic of a virtual machine in action, including things like the
/// program counter and the memory.
pub struct VirtualMachine {
    program_counter: Address,
    stack: Vec<Address>,
    registers: [Value; 16],
    register_i: Address,
    delay_timer: Mutex<Value>,
    sound_timer: Mutex<Value>,
    memory: [Value; MEMORY_SIZE],
    key_down: Mutex<Option<u8>>,
    display: [[bool; SCREEN_HEIGHT as usize]; SCREEN_WIDTH as usize],
}

impl VirtualMachine {
    /// Creates a new VM instance with all registers and memory set accordingly.
    pub fn new() -> VirtualMachine {
        let mut stack = Vec::new();
        stack.reserve(STACK_DEPTH);

        let mut memory = [Value(0); MEMORY_SIZE];
        let font_sprites = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80,
            0xF0, 0xF0, 0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0,
            0x10, 0xF0, 0xF0, 0x80, 0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90,
            0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0, 0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0,
            0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80, 0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0,
            0xF0, 0x70, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
        ];
        for (mem_cell, font_byte) in memory
            .iter_mut()
            .skip(FONT_OFFSET as usize)
            .zip(font_sprites.iter())
        {
            *mem_cell = Value(*font_byte);
        }

        VirtualMachine {
            program_counter: Address(0),
            stack: stack,
            registers: [Value(0); 16],
            register_i: Address(0),
            delay_timer: Mutex::new(Value(0)),
            sound_timer: Mutex::new(Value(0)),
            memory: memory,
            key_down: Mutex::new(None),
            display: [[false; SCREEN_HEIGHT as usize]; SCREEN_WIDTH as usize],
        }
    }

    /// Clears the entire display of a running VM to black.
    fn clear_display(&mut self) {
        for x in 0..SCREEN_WIDTH as usize {
            for y in 0..SCREEN_HEIGHT as usize {
                self.display[x][y] = false;
            }
        }
    }

    /// Returns the control flow from a subroutine.
    fn return_subroutine(&mut self) {
        if let Some(addr) = self.stack.pop() {
            self.program_counter = addr;
        } else {
            panic!("Tried to return from empty stack.");
        }
    }

    /// Calls a subroutine. Panics if the stack depth exceeds.
    fn call_subroutine(&mut self, addr: &Address) {
        if self.stack.len() >= STACK_DEPTH {
            panic!("Maximal stack depth exceeded.");
        }
        self.stack.push(self.program_counter);
        self.program_counter = *addr;
    }

    /// Returns the value of one of the registers.
    fn register(&mut self, reg: &Register) -> &mut Value {
        assert!(reg.0 < 16);
        &mut self.registers[reg.0 as usize]
    }

    /// Sets the VF register to a given value.
    fn set_vf(&mut self, value: u8) {
        self.registers[15] = Value(value);
    }

    /// Draws a pixel at a given coordinate on the display.
    /// If the pixel is already active, it is deactivated and the VF register is
    /// set to 1.
    fn draw_pixel(&mut self, x: u8, y: u8) {
        let pixel = &mut self.display[x as usize][y as usize];
        *pixel = !*pixel;
        let pixel = *pixel;
        if pixel {
            self.set_vf(1);
        }
    }

    /// Executes a single instruction. The program counter is updated,
    /// meaning for most instructions it will increase by 1 and move
    /// arbitrarily for others.
    fn execute_instruction(&mut self, instruction: &Instruction) {
        self.program_counter.incr();
        match instruction {
            // Jumps
            Instruction::CallSubroutine(addr) => self.call_subroutine(&addr),
            Instruction::ReturnSubroutine => self.return_subroutine(),
            Instruction::Jump(addr) => self.program_counter = *addr,
            Instruction::JumpAdd(addr) => {
                let new_addr = addr.0 + self.register(&Register(0)).0 as u16;
                self.program_counter = Address(new_addr);
            }

            // Conditionals
            Instruction::IfNotEqualConst(vx, n) => {
                if *self.register(vx) == *n {
                    self.program_counter.incr();
                }
            }
            Instruction::IfEqualConst(vx, n) => {
                if *self.register(vx) != *n {
                    self.program_counter.incr();
                }
            }
            Instruction::IfNotEqual(vx, vy) => {
                let x = *self.register(vx);
                let y = *self.register(vx);
                if x == y {
                    self.program_counter.incr();
                }
            }
            Instruction::IfEqual(vx, vy) => {
                let x = *self.register(vx);
                let y = *self.register(vx);
                if x != y {
                    self.program_counter.incr();
                }
            }

            // Register Arithmetic
            Instruction::SetConst(vx, n) => *self.register(vx) = *n,
            Instruction::AddConst(vx, n) => {
                let value = Value(self.register(vx).0 + n.0);
                *self.register(vx) = value;
            }
            Instruction::Set(vx, vy) => *self.register(vx) = *self.register(vy),
            Instruction::Or(vx, vy) => {
                let value_vx = *self.register(vx);
                let value_vy = *self.register(vy);
                *self.register(&vx) = Value(value_vx.0 | value_vy.0);
            }
            Instruction::And(vx, vy) => {
                let value_vx = *self.register(vx);
                let value_vy = *self.register(vy);
                *self.register(&vx) = Value(value_vx.0 & value_vy.0);
            }
            Instruction::Xor(vx, vy) => {
                let value_vx = *self.register(vx);
                let value_vy = *self.register(vy);
                *self.register(&vx) = Value(value_vx.0 ^ value_vy.0);
            }
            Instruction::Add(vx, vy) => {
                let value_vx = *self.register(vx);
                let value_vy = *self.register(vy);
                *self.register(&vx) = Value(value_vx.0 + value_vy.0);
            }
            Instruction::Sub(vx, vy) => {
                let value_vx = *self.register(vx);
                let value_vy = *self.register(vy);
                *self.register(&vx) = Value(value_vx.0 - value_vy.0);
            }
            Instruction::RightShift(vx) => {
                let value_vx = *self.register(vx);
                *self.register(&vx) = Value(value_vx.0 >> 1);
            }
            Instruction::NegSub(vx, vy) => {
                let value_vx = *self.register(vx);
                let value_vy = *self.register(vy);
                *self.register(&vx) = Value(value_vy.0 - value_vx.0);
            }
            Instruction::LeftShift(vx) => {
                let value_vx = *self.register(vx);
                *self.register(&vx) = Value(value_vx.0 << 1);
            }

            // Key presses
            Instruction::IfNotKey(vx) => {
                let target_key = self.register(vx).0;
                let current_key = *self.key_down.lock().unwrap();
                if current_key.is_some() || current_key.unwrap() != target_key {
                    self.program_counter.incr();
                }
            }
            Instruction::IfKey(vx) => {
                let target_key = self.register(vx).0;
                let current_key = *self.key_down.lock().unwrap();
                if !current_key.is_some() && current_key.unwrap() == target_key {
                    self.program_counter.incr();
                }
            }
            Instruction::WaitKey(vx) => {
                let found_key;
                loop {
                    let key = self.key_down.lock().unwrap();
                    if let Some(k) = *key {
                        found_key = Value(k);
                        break;
                    } else {
                        thread::sleep(Duration::from_millis(1));
                    }
                }
                *self.register(vx) = found_key;
            }

            // Graphics
            Instruction::Draw(vx, vy, n) => {
                self.set_vf(0);
                let x0 = self.register(vx).0;
                let y0 = self.register(vy).0;
                for y_off in 0..=n.0 {
                    let index = self.register_i.0 as usize + y_off as usize;
                    let row = self.memory[index].0;
                    for x_off in 0..8 {
                        self.draw_pixel((x0 + x_off) % SCREEN_WIDTH, (y0 + y_off) % SCREEN_HEIGHT);
                    }
                }
            }
            Instruction::ClearDisplay => self.clear_display(),
            Instruction::SpriteAddr(vx) => {
                let digit = self.register(vx).0;
                self.register_i = Address(FONT_OFFSET + (digit as u16) * 5);
            }

            // Timers
            Instruction::GetDelayTimer(vx) => {
                let value = *self.delay_timer.lock().unwrap();
                *self.register(vx) = value;
            }
            Instruction::SetDelayTimer(vx) => {
                *self.delay_timer.lock().unwrap() = *self.register(vx)
            }
            Instruction::SetSoundTimer(vx) => {
                *self.sound_timer.lock().unwrap() = *self.register(vx)
            }

            // I register
            Instruction::SetI(addr) => self.register_i = *addr,
            Instruction::AddToI(vx) => self.register_i.0 += self.register(vx).0 as u16,
            Instruction::Decimal(vx) => {
                let index = self.register_i.0 as usize;
                let value = self.register(vx).0;
                self.memory[index] = Value(value / 100);
                self.memory[index + 1] = Value(value / 10 % 10);
                self.memory[index + 2] = Value(value % 100);
            }
            Instruction::StoreRegisters(vx) => {
                let index = self.register_i.0 as usize;
                for i in 0..=vx.0 {
                    self.memory[index + i as usize] = *self.register(&Register(i));
                }
            }
            Instruction::LoadRegisters(vx) => {
                let index = self.register_i.0 as usize;
                for i in 0..=vx.0 {
                    *self.register(&Register(i)) = self.memory[index + i as usize];
                }
            }

            // Misc
            Instruction::Noop => (),
            Instruction::Rand(vx, n) => {
                let rand = rand::thread_rng().gen_range(0, 255) as u8;
                *self.register(vx) = Value(rand & n.0);
            }
            Instruction::MachineCodeRoutine(_addr) => {
                panic!("Machine code routines are not implemented.")
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_vm_new() {
        let vm = VirtualMachine::new();
        assert_eq!(vm.program_counter, Address(0));
        assert!(vm.stack.is_empty());
        for r in vm.registers.iter() {
            assert_eq!(*r, Value(0));
        }
        assert_eq!(vm.register_i, Address(0));
        assert_eq!(*vm.delay_timer.lock().unwrap(), Value(0));
        assert_eq!(*vm.sound_timer.lock().unwrap(), Value(0));
        for x in vm.memory.iter().skip(FONT_OFFSET as usize).take(5 * 16) {
            assert_ne!(*x, Value(0));
        }
        for x in vm.memory.iter().skip(512) {
            assert_eq!(*x, Value(0));
        }
        assert_eq!(*vm.key_down.lock().unwrap(), None);
        for x in 0..SCREEN_WIDTH as usize {
            for y in 0..SCREEN_HEIGHT as usize {
                assert!(!vm.display[x][y]);
            }
        }
    }

    #[test]
    fn test_noop() {
        let mut vm = VirtualMachine::new();
        let noop = Instruction::Noop;
        assert_eq!(vm.program_counter, Address(0));
        vm.execute_instruction(&noop);
        assert_eq!(vm.program_counter, Address(1));
        vm.execute_instruction(&noop);
        assert_eq!(vm.program_counter, Address(2));
    }

    #[test]
    fn test_subroutines() {
        let mut vm = VirtualMachine::new();
        assert_eq!(vm.program_counter, Address(0));
        vm.execute_instruction(&Instruction::Noop);
        assert_eq!(vm.program_counter, Address(1));
        assert_eq!(vm.stack.len(), 0);
        vm.execute_instruction(&Instruction::CallSubroutine(Address(123)));
        assert_eq!(vm.program_counter, Address(123));
        assert_eq!(vm.stack.len(), 1);
        vm.execute_instruction(&Instruction::Noop);
        assert_eq!(vm.program_counter, Address(124));
        vm.execute_instruction(&Instruction::CallSubroutine(Address(456)));
        assert_eq!(vm.program_counter, Address(456));
        assert_eq!(vm.stack.len(), 2);
        vm.execute_instruction(&Instruction::ReturnSubroutine);
        assert_eq!(vm.program_counter, Address(125));
        assert_eq!(vm.stack.len(), 1);
        vm.execute_instruction(&Instruction::Noop);
        assert_eq!(vm.program_counter, Address(126));
        vm.execute_instruction(&Instruction::ReturnSubroutine);
        assert_eq!(vm.program_counter, Address(2));
        assert_eq!(vm.stack.len(), 0);
    }

    #[test]
    fn test_stack_no_overflow() {
        let mut vm = VirtualMachine::new();
        let call = Instruction::CallSubroutine(Address(0));
        for i in 0..STACK_DEPTH {
            vm.execute_instruction(&call);
        }
    }

    #[test]
    #[should_panic]
    fn test_stack_overflow() {
        let mut vm = VirtualMachine::new();
        let call = Instruction::CallSubroutine(Address(0));
        for i in 0..STACK_DEPTH {
            vm.execute_instruction(&call);
        }
        vm.execute_instruction(&call);
    }

    #[test]
    #[should_panic]
    fn test_stack_empty() {
        let mut vm = VirtualMachine::new();
        let call = Instruction::ReturnSubroutine;
        vm.execute_instruction(&call);
    }

    #[test]
    fn test_jumps() {
        // TODO
    }

    #[test]
    fn test_conditionals() {
        // TODO
    }

    #[test]
    fn test_arithmetic() {
        // TODO
    }
}

use super::basics::{
    Address, Register, Value, FONT_OFFSET, MEMORY_SIZE, SCREEN_HEIGHT, SCREEN_WIDTH, STACK_DEPTH,
};
use super::program::Instruction;
use rand::Rng;
use std::sync::{Arc, Mutex};

/// Holds the logic of a virtual machine in action, including things like the
/// program counter and the memory.
pub struct VirtualMachine {
    pub program_counter: Address,
    stack: Vec<Address>,
    registers: [Value; 16],
    register_i: Address,
    memory: [Value; MEMORY_SIZE],
    logical_display: [[bool; SCREEN_HEIGHT as usize]; SCREEN_WIDTH as usize],
    pub interface: Arc<Mutex<VMInterface>>,
}

/// The "Interface" contains those parts of the VM that are used to communicate
/// with the "outside".
pub struct VMInterface {
    pub delay_timer: Value,
    pub sound_timer: Value,
    pub key_down: Option<u8>,
    pub display: Box<dyn Display>,
}

/// A "display", which is called whenever a drawing instruction is executed.
pub trait Display: Send {
    fn clear(&mut self);
    fn draw_pixels(&mut self, pixels: &[(u8, u8)]);
    fn get(&self, x: u8, y: u8) -> u8;
    fn frame(&mut self);
}

struct SimpleDisplay {
    display: [[bool; SCREEN_HEIGHT as usize]; SCREEN_WIDTH as usize],
}

impl Display for SimpleDisplay {
    fn clear(&mut self) {
        for column in self.display.iter_mut() {
            for pixel in column.iter_mut() {
                *pixel = false;
            }
        }
    }

    fn draw_pixels(&mut self, pixels: &[(u8, u8)]) {
        for (x, y) in pixels {
            let pixel = &mut self.display[*x as usize][*y as usize];
            *pixel = !*pixel;
        }
    }

    fn get(&self, x: u8, y: u8) -> u8 {
        if self.display[x as usize][y as usize] {
            255
        } else {
            0
        }
    }

    fn frame(&mut self) {}
}

impl VirtualMachine {
    /// Creates a new VM instance with all registers and memory set accordingly.
    pub fn new(program: &[u8]) -> VirtualMachine {
        let interface = VMInterface {
            delay_timer: Value(0),
            sound_timer: Value(0),
            key_down: None,
            display: Box::new(SimpleDisplay {
                display: [[false; SCREEN_HEIGHT as usize]; SCREEN_WIDTH as usize],
            }),
        };

        VirtualMachine {
            program_counter: Address(0x200),
            stack: Vec::new(),
            registers: [Value(0); 16],
            register_i: Address(0),
            memory: VirtualMachine::setup_memory(program),
            logical_display: [[false; SCREEN_HEIGHT as usize]; SCREEN_WIDTH as usize],
            interface: Arc::new(Mutex::new(interface)),
        }
    }

    fn setup_memory(program: &[u8]) -> [Value; MEMORY_SIZE] {
        let mut memory = [Value(0); MEMORY_SIZE];
        let font_sprites = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80,
            0xF0, 0xF0, 0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0,
            0x10, 0xF0, 0xF0, 0x80, 0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90,
            0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0, 0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0,
            0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80, 0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0,
            0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
        ];
        for (mem_cell, font_byte) in memory
            .iter_mut()
            .skip(FONT_OFFSET as usize)
            .zip(font_sprites.iter())
        {
            *mem_cell = Value(*font_byte);
        }
        for (mem_cell, prog_byte) in memory.iter_mut().skip(0x200).zip(program.iter()) {
            *mem_cell = Value(*prog_byte);
        }
        memory
    }

    pub fn current_instruction(&self) -> Instruction {
        let a = self.memory[self.program_counter.0 as usize].0;
        let b = self.memory[self.program_counter.0 as usize + 1].0;
        Instruction::from_16bit(a, b)
    }

    /// Executes the next instruction of the VM, according to the program counter.
    pub fn step(&mut self) {
        self.execute_instruction(&self.current_instruction());
    }

    /// Clears the entire display of a running VM to black.
    fn clear_display(&mut self) {
        for x in 0..SCREEN_WIDTH as usize {
            for y in 0..SCREEN_HEIGHT as usize {
                self.logical_display[x][y] = false;
            }
        }
        self.interface.lock().unwrap().display.clear();
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

    fn draw_shape(&mut self, vx: &Register, vy: &Register, n: &Value) {
        self.set_vf(0);
        let mut pixels = Vec::new();
        let x0 = self.register(vx).0;
        let y0 = self.register(vy).0;
        for y_off in 0..n.0 {
            let index = self.register_i.0 as usize + y_off as usize;
            let row = self.memory[index].0;
            for x_off in 0..8 {
                if row & (128 >> x_off) > 0 {
                    let x = (x0 + x_off) % SCREEN_WIDTH;
                    let y = (y0 + y_off) % SCREEN_HEIGHT;
                    pixels.push((x, y));
                }
            }
        }
        self.draw_pixels(&pixels);
    }

    fn draw_pixels(&mut self, pixels: &[(u8, u8)]) {
        for (x, y) in pixels {
            self.draw_pixel(*x, *y);
        }
        self.interface.lock().unwrap().display.draw_pixels(pixels);
    }

    /// Draws a pixel at a given coordinate on the display.
    /// If the pixel is already active, it is deactivated and the VF register is
    /// set to 1.
    fn draw_pixel(&mut self, x: u8, y: u8) {
        let was_cleared = {
            let pixel = &mut self.logical_display[x as usize][y as usize];
            *pixel = !*pixel;
            !*pixel
        };
        if was_cleared {
            self.set_vf(1);
        }
    }

    /// Executes a single instruction. The program counter is updated,
    /// meaning for most instructions it will increase by 1 and move
    /// arbitrarily for others.
    pub fn execute_instruction(&mut self, instruction: &Instruction) {
        self.program_counter.0 += 2;
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
                    self.program_counter.0 += 2;
                }
            }
            Instruction::IfEqualConst(vx, n) => {
                if *self.register(vx) != *n {
                    self.program_counter.0 += 2;
                }
            }
            Instruction::IfNotEqual(vx, vy) => {
                let x = *self.register(vx);
                let y = *self.register(vy);
                if x == y {
                    self.program_counter.0 += 2;
                }
            }
            Instruction::IfEqual(vx, vy) => {
                let x = *self.register(vx);
                let y = *self.register(vy);
                if x != y {
                    self.program_counter.0 += 2;
                }
            }

            // Register Arithmetic
            Instruction::SetConst(vx, n) => *self.register(vx) = *n,
            Instruction::AddConst(vx, n) => {
                let value = Value(self.register(vx).0.wrapping_add(n.0));
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
                self.set_vf(value_vx.0.checked_add(value_vy.0).is_none() as u8);
                *self.register(&vx) = Value(value_vx.0.wrapping_add(value_vy.0));
            }
            Instruction::Sub(vx, vy) => {
                let value_vx = *self.register(vx);
                let value_vy = *self.register(vy);
                self.set_vf((value_vx.0 > value_vy.0) as u8);
                *self.register(&vx) = Value(value_vx.0.wrapping_sub(value_vy.0));
            }
            Instruction::NegSub(vx, vy) => {
                let value_vx = *self.register(vx);
                let value_vy = *self.register(vy);
                self.set_vf((value_vy.0 > value_vx.0) as u8);
                *self.register(&vx) = Value(value_vy.0.wrapping_sub(value_vx.0));
            }
            Instruction::RightShift(vx) => {
                let value_vx = *self.register(vx);
                self.set_vf((value_vx.0 & 1) as u8);
                *self.register(&vx) = Value(value_vx.0 >> 1);
            }
            Instruction::LeftShift(vx) => {
                let value_vx = *self.register(vx);
                self.set_vf((value_vx.0 & 128 > 0) as u8);
                *self.register(&vx) = Value(value_vx.0 << 1);
            }

            // Key presses
            Instruction::IfNotKey(vx) => {
                let target_key = self.register(vx).0;
                let current_key = self.interface.lock().unwrap().key_down;
                if current_key.is_some() && current_key.unwrap() == target_key {
                    self.program_counter.0 += 2;
                }
            }
            Instruction::IfKey(vx) => {
                let target_key = self.register(vx).0;
                let current_key = self.interface.lock().unwrap().key_down;
                if current_key.is_none() || current_key.unwrap() != target_key {
                    self.program_counter.0 += 2;
                }
            }
            Instruction::WaitKey(vx) => {
                let key_down = self.interface.lock().unwrap().key_down;
                if let Some(k) = key_down {
                    *self.register(vx) = Value(k);
                } else {
                    self.program_counter.0 -= 2;
                }
            }

            // Graphics
            Instruction::Draw(vx, vy, n) => self.draw_shape(vx, vy, n),
            Instruction::ClearDisplay => self.clear_display(),
            Instruction::SpriteAddr(vx) => {
                let digit = self.register(vx).0;
                self.register_i = Address(FONT_OFFSET + (digit as u16) * 5);
            }

            // Timers
            Instruction::GetDelayTimer(vx) => {
                let value = self.interface.lock().unwrap().delay_timer;
                *self.register(vx) = value;
            }
            Instruction::SetDelayTimer(vx) => {
                self.interface.lock().unwrap().delay_timer = *self.register(vx)
            }
            Instruction::SetSoundTimer(vx) => {
                self.interface.lock().unwrap().sound_timer = *self.register(vx)
            }

            // I register
            Instruction::SetI(addr) => self.register_i = *addr,
            Instruction::AddToI(vx) => self.register_i.0 += self.register(vx).0 as u16,
            Instruction::Decimal(vx) => {
                let index = self.register_i.0 as usize;
                let value = self.register(vx).0;
                self.memory[index] = Value(value / 100);
                self.memory[index + 1] = Value(value / 10 % 10);
                self.memory[index + 2] = Value(value % 10);
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
        let vm = VirtualMachine::new(&[]);
        assert_eq!(vm.program_counter, Address(0x200));
        assert!(vm.stack.is_empty());
        for r in vm.registers.iter() {
            assert_eq!(*r, Value(0));
        }
        assert_eq!(vm.register_i, Address(0));
        assert_eq!(vm.interface.lock().unwrap().delay_timer, Value(0));
        assert_eq!(vm.interface.lock().unwrap().sound_timer, Value(0));
        for x in vm.memory.iter().skip(FONT_OFFSET as usize).take(5 * 16) {
            assert_ne!(*x, Value(0));
        }
        for x in vm.memory.iter().skip(512) {
            assert_eq!(*x, Value(0));
        }
        assert_eq!(vm.interface.lock().unwrap().key_down, None);
        for x in 0..SCREEN_WIDTH as usize {
            for y in 0..SCREEN_HEIGHT as usize {
                assert!(!vm.logical_display[x][y]);
            }
        }
    }

    #[test]
    fn test_noop() {
        let mut vm = VirtualMachine::new(&[]);
        let noop = Instruction::Noop;
        assert_eq!(vm.program_counter, Address(0x200));
        vm.execute_instruction(&noop);
        assert_eq!(vm.program_counter, Address(0x202));
        vm.execute_instruction(&noop);
        assert_eq!(vm.program_counter, Address(0x204));
    }

    #[test]
    fn test_subroutines() {
        let mut vm = VirtualMachine::new(&[]);
        vm.program_counter = Address(0);
        assert_eq!(vm.program_counter, Address(0));
        vm.execute_instruction(&Instruction::Noop);
        assert_eq!(vm.program_counter, Address(2));
        assert_eq!(vm.stack.len(), 0);
        vm.execute_instruction(&Instruction::CallSubroutine(Address(123)));
        assert_eq!(vm.program_counter, Address(123));
        assert_eq!(vm.stack.len(), 1);
        vm.execute_instruction(&Instruction::Noop);
        assert_eq!(vm.program_counter, Address(125));
        vm.execute_instruction(&Instruction::CallSubroutine(Address(456)));
        assert_eq!(vm.program_counter, Address(456));
        assert_eq!(vm.stack.len(), 2);
        vm.execute_instruction(&Instruction::ReturnSubroutine);
        assert_eq!(vm.program_counter, Address(127));
        assert_eq!(vm.stack.len(), 1);
        vm.execute_instruction(&Instruction::Noop);
        assert_eq!(vm.program_counter, Address(129));
        vm.execute_instruction(&Instruction::ReturnSubroutine);
        assert_eq!(vm.program_counter, Address(4));
        assert_eq!(vm.stack.len(), 0);
    }

    #[test]
    fn test_stack_no_overflow() {
        let mut vm = VirtualMachine::new(&[]);
        let call = Instruction::CallSubroutine(Address(0));
        for _ in 0..STACK_DEPTH {
            vm.execute_instruction(&call);
        }
    }

    #[test]
    #[should_panic]
    fn test_stack_overflow() {
        let mut vm = VirtualMachine::new(&[]);
        let call = Instruction::CallSubroutine(Address(0));
        for _ in 0..STACK_DEPTH {
            vm.execute_instruction(&call);
        }
        vm.execute_instruction(&call);
    }

    #[test]
    #[should_panic]
    fn test_stack_empty() {
        let mut vm = VirtualMachine::new(&[]);
        let call = Instruction::ReturnSubroutine;
        vm.execute_instruction(&call);
    }

    #[test]
    fn test_jumps() {
        let mut vm = VirtualMachine::new(&[]);
        vm.program_counter = Address(0);
        assert_eq!(vm.program_counter, Address(0));
        vm.execute_instruction(&Instruction::Noop);
        assert_eq!(vm.program_counter, Address(2));
        vm.execute_instruction(&Instruction::Jump(Address(42)));
        assert_eq!(vm.program_counter, Address(42));
        assert_eq!(vm.registers[0], Value(0));
        vm.execute_instruction(&Instruction::JumpAdd(Address(100)));
        assert_eq!(vm.program_counter, Address(100));
        vm.registers[0] = Value(13);
        vm.execute_instruction(&Instruction::JumpAdd(Address(100)));
        assert_eq!(vm.program_counter, Address(113));
        vm.execute_instruction(&Instruction::Jump(Address(50)));
        assert_eq!(vm.program_counter, Address(50));
    }

    #[test]
    fn test_conditionals() {
        let mut vm = VirtualMachine::new(&[]);
        vm.program_counter = Address(0);
        vm.registers = [
            Value(0),
            Value(1),
            Value(2),
            Value(3),
            Value(4),
            Value(5),
            Value(6),
            Value(7),
            Value(8),
            Value(9),
            Value(10),
            Value(11),
            Value(12),
            Value(13),
            Value(14),
            Value(0),
        ];
        assert_eq!(vm.program_counter, Address(0));
        vm.execute_instruction(&Instruction::IfEqualConst(Register(0), Value(0)));
        assert_eq!(vm.program_counter, Address(2));
        vm.execute_instruction(&Instruction::IfEqualConst(Register(1), Value(2)));
        assert_eq!(vm.program_counter, Address(6));
        vm.execute_instruction(&Instruction::IfNotEqualConst(Register(1), Value(1)));
        assert_eq!(vm.program_counter, Address(10));
        vm.execute_instruction(&Instruction::IfNotEqualConst(Register(2), Value(0)));
        assert_eq!(vm.program_counter, Address(12));
        vm.execute_instruction(&Instruction::IfEqual(Register(4), Register(4)));
        assert_eq!(vm.program_counter, Address(14));
        vm.execute_instruction(&Instruction::IfEqual(Register(4), Register(5)));
        assert_eq!(vm.program_counter, Address(18));
        vm.execute_instruction(&Instruction::IfEqual(Register(0), Register(15)));
        assert_eq!(vm.program_counter, Address(20));
        vm.execute_instruction(&Instruction::IfNotEqual(Register(4), Register(4)));
        assert_eq!(vm.program_counter, Address(24));
        vm.execute_instruction(&Instruction::IfNotEqual(Register(4), Register(5)));
        assert_eq!(vm.program_counter, Address(26));
        vm.execute_instruction(&Instruction::IfNotEqual(Register(0), Register(15)));
        assert_eq!(vm.program_counter, Address(30));
    }

    #[test]
    fn test_arithmetic() {
        let mut vm = VirtualMachine::new(&[]);
        vm.program_counter = Address(0);
        vm.registers = [
            Value(0),
            Value(1),
            Value(2),
            Value(3),
            Value(4),
            Value(5),
            Value(6),
            Value(7),
            Value(8),
            Value(9),
            Value(10),
            Value(11),
            Value(12),
            Value(13),
            Value(14),
            Value(0),
        ];
        assert_eq!(vm.program_counter, Address(0));
        assert_eq!(vm.registers[0], Value(0));
        vm.execute_instruction(&Instruction::SetConst(Register(0), Value(5)));
        assert_eq!(vm.program_counter, Address(2));
        assert_eq!(vm.registers[0], Value(5));
        vm.execute_instruction(&Instruction::AddConst(Register(1), Value(2)));
        assert_eq!(vm.program_counter, Address(4));
        assert_eq!(vm.registers[1], Value(3));
        vm.execute_instruction(&Instruction::Set(Register(0), Register(2)));
        assert_eq!(vm.program_counter, Address(6));
        assert_eq!(vm.registers[0], Value(2));
        assert_eq!(vm.registers[2], Value(2));
        vm.execute_instruction(&Instruction::Or(Register(4), Register(1)));
        assert_eq!(vm.program_counter, Address(8));
        assert_eq!(vm.registers[4], Value(7));
        assert_eq!(vm.registers[1], Value(3));
        vm.execute_instruction(&Instruction::And(Register(0), Register(1)));
        assert_eq!(vm.program_counter, Address(10));
        assert_eq!(vm.registers[0], Value(2));
        assert_eq!(vm.registers[1], Value(3));
        vm.execute_instruction(&Instruction::Xor(Register(14), Register(4)));
        assert_eq!(vm.program_counter, Address(12));
        assert_eq!(vm.registers[14], Value(9));
        assert_eq!(vm.registers[4], Value(7));
        vm.execute_instruction(&Instruction::Add(Register(6), Register(7)));
        assert_eq!(vm.program_counter, Address(14));
        assert_eq!(vm.registers[6], Value(13));
        assert_eq!(vm.registers[7], Value(7));
        vm.execute_instruction(&Instruction::Sub(Register(6), Register(5)));
        assert_eq!(vm.program_counter, Address(16));
        assert_eq!(vm.registers[6], Value(8));
        assert_eq!(vm.registers[5], Value(5));
        vm.execute_instruction(&Instruction::NegSub(Register(1), Register(4)));
        assert_eq!(vm.program_counter, Address(18));
        assert_eq!(vm.registers[1], Value(4));
        assert_eq!(vm.registers[4], Value(7));
        vm.execute_instruction(&Instruction::LeftShift(Register(0)));
        assert_eq!(vm.program_counter, Address(20));
        assert_eq!(vm.registers[0], Value(4));
        vm.execute_instruction(&Instruction::RightShift(Register(7)));
        assert_eq!(vm.program_counter, Address(22));
        assert_eq!(vm.registers[7], Value(3));
    }

    #[test]
    fn test_arithmetic_overflow() {
        let mut vm = VirtualMachine::new(&[]);
        vm.program_counter = Address(0);
        vm.registers = [
            Value(100),
            Value(100),
            Value(60),
            Value(40),
            Value(100),
            Value(0),
            Value(8),
            Value(9),
            Value(0),
            Value(65),
            Value(129),
            Value(0),
            Value(0),
            Value(0),
            Value(0),
            Value(0),
        ];
        assert_eq!(vm.program_counter, Address(0));
        vm.execute_instruction(&Instruction::Add(Register(0), Register(1)));
        assert_eq!(vm.program_counter, Address(2));
        assert_eq!(vm.registers[0], Value(200));
        assert_eq!(vm.registers[15], Value(0));
        vm.execute_instruction(&Instruction::Add(Register(0), Register(1)));
        assert_eq!(vm.program_counter, Address(4));
        assert_eq!(vm.registers[0], Value(44));
        assert_eq!(vm.registers[15], Value(1));
        vm.execute_instruction(&Instruction::Sub(Register(1), Register(2)));
        assert_eq!(vm.program_counter, Address(6));
        assert_eq!(vm.registers[1], Value(40));
        assert_eq!(vm.registers[15], Value(1));
        vm.execute_instruction(&Instruction::Sub(Register(1), Register(2)));
        assert_eq!(vm.program_counter, Address(8));
        assert_eq!(vm.registers[1], Value(236));
        assert_eq!(vm.registers[15], Value(0));
        vm.execute_instruction(&Instruction::NegSub(Register(2), Register(3)));
        assert_eq!(vm.program_counter, Address(10));
        assert_eq!(vm.registers[2], Value(236));
        assert_eq!(vm.registers[15], Value(0));
        vm.execute_instruction(&Instruction::NegSub(Register(3), Register(4)));
        assert_eq!(vm.program_counter, Address(12));
        assert_eq!(vm.registers[3], Value(60));
        assert_eq!(vm.registers[15], Value(1));
        vm.execute_instruction(&Instruction::RightShift(Register(6)));
        assert_eq!(vm.program_counter, Address(14));
        assert_eq!(vm.registers[6], Value(4));
        assert_eq!(vm.registers[15], Value(0));
        vm.execute_instruction(&Instruction::RightShift(Register(7)));
        assert_eq!(vm.program_counter, Address(16));
        assert_eq!(vm.registers[7], Value(4));
        assert_eq!(vm.registers[15], Value(1));
        vm.execute_instruction(&Instruction::LeftShift(Register(9)));
        assert_eq!(vm.program_counter, Address(18));
        assert_eq!(vm.registers[9], Value(130));
        assert_eq!(vm.registers[15], Value(0));
        vm.execute_instruction(&Instruction::LeftShift(Register(10)));
        assert_eq!(vm.program_counter, Address(20));
        assert_eq!(vm.registers[10], Value(2));
        assert_eq!(vm.registers[15], Value(1));
    }

    #[test]
    fn test_key_conditionals() {
        let mut vm = VirtualMachine::new(&[]);
        vm.program_counter = Address(0);
        assert_eq!(vm.interface.lock().unwrap().key_down, None);
        vm.registers[0] = Value(0);

        assert_eq!(vm.program_counter, Address(0));
        vm.execute_instruction(&Instruction::IfKey(Register(0)));
        assert_eq!(vm.program_counter, Address(4));
        vm.execute_instruction(&Instruction::IfNotKey(Register(0)));
        assert_eq!(vm.program_counter, Address(6));
        vm.interface.lock().unwrap().key_down = Some(1);
        vm.execute_instruction(&Instruction::IfKey(Register(0)));
        assert_eq!(vm.program_counter, Address(10));
        vm.execute_instruction(&Instruction::IfNotKey(Register(0)));
        assert_eq!(vm.program_counter, Address(12));
        vm.registers[0] = Value(1);
        vm.execute_instruction(&Instruction::IfKey(Register(0)));
        assert_eq!(vm.program_counter, Address(14));
        vm.execute_instruction(&Instruction::IfNotKey(Register(0)));
        assert_eq!(vm.program_counter, Address(18));
    }

    #[test]
    fn test_key_wait() {
        let mut vm = VirtualMachine::new(&[]);
        let interface = vm.interface.clone();
        assert!(vm.interface.lock().unwrap().key_down.is_none());
        assert_eq!(vm.program_counter, Address(0x200));
        vm.execute_instruction(&Instruction::WaitKey(Register(0)));
        assert_eq!(vm.program_counter, Address(0x200));
        vm.interface.lock().unwrap().key_down = Some(4);
        vm.execute_instruction(&Instruction::WaitKey(Register(0)));
        assert_eq!(vm.program_counter, Address(0x202));
        assert_eq!(vm.registers[0], Value(4));
    }

    #[test]
    fn test_graphics_draw_simple() {
        let mut vm = VirtualMachine::new(&[]);
        vm.registers = [
            Value(0),
            Value(1),
            Value(2),
            Value(3),
            Value(4),
            Value(5),
            Value(6),
            Value(7),
            Value(8),
            Value(9),
            Value(10),
            Value(11),
            Value(12),
            Value(13),
            Value(14),
            Value(0),
        ];
        vm.register_i = Address(0x200);

        assert!(!vm.logical_display[0][0]);
        vm.draw_pixel(0, 0);
        assert!(vm.logical_display[0][0]);

        vm.execute_instruction(&Instruction::Draw(Register(0), Register(1), Value(1)));
        assert!(!vm.logical_display[0][1]);
        assert!(!vm.logical_display[1][1]);
        assert!(!vm.logical_display[2][1]);
        assert!(!vm.logical_display[3][1]);
        assert!(!vm.logical_display[4][1]);
        assert!(!vm.logical_display[5][1]);
        assert!(!vm.logical_display[6][1]);
        assert!(!vm.logical_display[7][1]);
        assert_eq!(vm.registers[15], Value(0));

        vm.memory[vm.register_i.0 as usize] = Value(0b01010101);
        vm.execute_instruction(&Instruction::Draw(Register(0), Register(1), Value(1)));
        assert!(!vm.logical_display[0][1]);
        assert!(vm.logical_display[1][1]);
        assert!(!vm.logical_display[2][1]);
        assert!(vm.logical_display[3][1]);
        assert!(!vm.logical_display[4][1]);
        assert!(vm.logical_display[5][1]);
        assert!(!vm.logical_display[6][1]);
        assert!(vm.logical_display[7][1]);
        assert_eq!(vm.registers[15], Value(0));

        vm.execute_instruction(&Instruction::ClearDisplay);
        assert!(!vm.logical_display[0][0]);
        assert!(!vm.logical_display[0][1]);
        assert!(!vm.logical_display[1][1]);
        assert!(!vm.logical_display[2][1]);
        assert!(!vm.logical_display[3][1]);
        assert!(!vm.logical_display[4][1]);
        assert!(!vm.logical_display[5][1]);
        assert!(!vm.logical_display[6][1]);
        assert!(!vm.logical_display[7][1]);
        assert_eq!(vm.registers[15], Value(0));
    }

    #[test]
    fn test_graphics_draw_collision() {
        let mut vm = VirtualMachine::new(&[]);
        assert_eq!(vm.registers[15], Value(0));
        // Sprite 1:
        /*
        10101
        01010
        10101
        01010
        */
        vm.memory[0x200] = Value(0b10101000);
        vm.memory[0x201] = Value(0b01010000);
        vm.memory[0x202] = Value(0b10101000);
        vm.memory[0x203] = Value(0b01010000);
        vm.register_i = Address(0x200);
        vm.execute_instruction(&Instruction::Draw(Register(0), Register(0), Value(4)));
        assert_eq!(vm.registers[15], Value(0));
        // Sprite 2:
        /*
        11111
        10001
        10001
        11111
        */
        vm.memory[0x204] = Value(0b11111000);
        vm.memory[0x205] = Value(0b10001000);
        vm.memory[0x206] = Value(0b10001000);
        vm.memory[0x207] = Value(0b11111000);
        vm.register_i = Address(0x204);
        vm.execute_instruction(&Instruction::Draw(Register(0), Register(0), Value(4)));
        assert_eq!(vm.registers[15], Value(1));
        // Target Sprite:
        /*
        01010
        11011
        00100
        10101
        */
        assert!(!vm.logical_display[0][0]);
        assert!(vm.logical_display[1][0]);
        assert!(!vm.logical_display[2][0]);
        assert!(vm.logical_display[3][0]);
        assert!(!vm.logical_display[4][0]);
        assert!(vm.logical_display[0][1]);
        assert!(vm.logical_display[1][1]);
        assert!(!vm.logical_display[2][1]);
        assert!(vm.logical_display[3][1]);
        assert!(vm.logical_display[4][1]);
        assert!(!vm.logical_display[0][2]);
        assert!(!vm.logical_display[1][2]);
        assert!(vm.logical_display[2][2]);
        assert!(!vm.logical_display[3][2]);
        assert!(!vm.logical_display[4][2]);
        assert!(vm.logical_display[0][3]);
        assert!(!vm.logical_display[1][3]);
        assert!(vm.logical_display[2][3]);
        assert!(!vm.logical_display[3][3]);
        assert!(vm.logical_display[4][3]);
    }

    #[test]
    fn test_graphics_sprite_addr() {
        let mut vm = VirtualMachine::new(&[]);
        vm.register_i = Address(0x200);
        vm.registers[0] = Value(5);
        vm.execute_instruction(&Instruction::SpriteAddr(Register(0)));
        vm.execute_instruction(&Instruction::Draw(Register(1), Register(1), Value(5)));
        assert!(vm.logical_display[0][0]);
        assert!(vm.logical_display[1][0]);
        assert!(vm.logical_display[2][0]);
        assert!(vm.logical_display[3][0]);
        assert!(vm.logical_display[0][1]);
        assert!(!vm.logical_display[1][1]);
        assert!(!vm.logical_display[2][1]);
        assert!(!vm.logical_display[3][1]);
        assert!(vm.logical_display[0][2]);
        assert!(vm.logical_display[1][2]);
        assert!(vm.logical_display[2][2]);
        assert!(vm.logical_display[3][2]);
        assert!(!vm.logical_display[0][3]);
        assert!(!vm.logical_display[1][3]);
        assert!(!vm.logical_display[2][3]);
        assert!(vm.logical_display[3][3]);
        assert!(vm.logical_display[0][4]);
        assert!(vm.logical_display[1][4]);
        assert!(vm.logical_display[2][4]);
        assert!(vm.logical_display[3][4]);
    }

    #[test]
    fn test_timers() {
        let mut vm = VirtualMachine::new(&[]);
        vm.program_counter = Address(0);
        vm.registers[0] = Value(42);
        assert_eq!(vm.program_counter, Address(0));
        vm.execute_instruction(&Instruction::SetDelayTimer(Register(0)));
        assert_eq!(vm.program_counter, Address(2));
        assert_eq!(vm.interface.lock().unwrap().delay_timer, Value(42));
        vm.registers[0] = Value(130);
        vm.execute_instruction(&Instruction::SetSoundTimer(Register(0)));
        assert_eq!(vm.program_counter, Address(4));
        assert_eq!(vm.interface.lock().unwrap().sound_timer, Value(130));
        vm.execute_instruction(&Instruction::GetDelayTimer(Register(0)));
        assert_eq!(vm.program_counter, Address(6));
        assert_eq!(vm.registers[0], Value(42));
    }

    #[test]
    fn test_i_register() {
        let mut vm = VirtualMachine::new(&[]);
        vm.registers = [
            Value(0),
            Value(1),
            Value(11),
            Value(0),
            Value(213),
            Value(0),
            Value(0),
            Value(50),
            Value(43),
            Value(212),
            Value(0),
            Value(0),
            Value(0),
            Value(0),
            Value(0),
            Value(0),
        ];

        assert_eq!(vm.register_i, Address(0));
        vm.execute_instruction(&Instruction::SetI(Address(1247)));
        assert_eq!(vm.register_i, Address(1247));
        vm.execute_instruction(&Instruction::AddToI(Register(2)));
        assert_eq!(vm.register_i, Address(1258));

        vm.memory[1263] = Value(99);
        vm.execute_instruction(&Instruction::StoreRegisters(Register(4)));
        assert_eq!(vm.register_i, Address(1258));
        assert_eq!(vm.memory[1258], Value(0));
        assert_eq!(vm.memory[1259], Value(1));
        assert_eq!(vm.memory[1260], Value(11));
        assert_eq!(vm.memory[1261], Value(0));
        assert_eq!(vm.memory[1262], Value(213));
        assert_eq!(vm.memory[1263], Value(99));

        vm.execute_instruction(&Instruction::Decimal(Register(4)));
        assert_eq!(vm.register_i, Address(1258));
        assert_eq!(vm.memory[1258], Value(2));
        assert_eq!(vm.memory[1259], Value(1));
        assert_eq!(vm.memory[1260], Value(3));

        vm.memory[1261] = Value(4);
        vm.memory[1262] = Value(5);
        vm.execute_instruction(&Instruction::LoadRegisters(Register(3)));
        assert_eq!(vm.registers[0], Value(2));
        assert_eq!(vm.registers[1], Value(1));
        assert_eq!(vm.registers[2], Value(3));
        assert_eq!(vm.registers[3], Value(4));
        assert_eq!(vm.registers[4], Value(213));
    }

    #[test]
    fn test_rand() {
        // TODO
    }
}

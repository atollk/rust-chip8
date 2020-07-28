pub const MEMORY_SIZE: usize = 4096;
pub const SCREEN_WIDTH: u8 = 64;
pub const SCREEN_HEIGHT: u8 = 32;
pub const FONT_OFFSET: u16 = 0;
pub const STACK_DEPTH: usize = 16;

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Address(pub u16);

impl Address {
    pub fn incr(&mut self) {
        self.0 += 1;
    }
}
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Register(pub u8);

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Value(pub u8);

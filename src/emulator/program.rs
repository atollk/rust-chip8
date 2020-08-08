use super::basics::{Address, Register, Value};

pub enum Instruction {
    Noop,
    MachineCodeRoutine(Address),
    ClearDisplay,
    ReturnSubroutine,
    Jump(Address),
    CallSubroutine(Address),
    IfNotEqualConst(Register, Value),
    IfEqualConst(Register, Value),
    IfNotEqual(Register, Register),
    SetConst(Register, Value),
    AddConst(Register, Value),
    Set(Register, Register),
    Or(Register, Register),
    And(Register, Register),
    Xor(Register, Register),
    Add(Register, Register),
    Sub(Register, Register),
    RightShift(Register),
    NegSub(Register, Register),
    LeftShift(Register),
    IfEqual(Register, Register),
    SetI(Address),
    JumpAdd(Address),
    Rand(Register, Value),
    Draw(Register, Register, Value),
    IfNotKey(Register),
    IfKey(Register),
    GetDelayTimer(Register),
    WaitKey(Register),
    SetDelayTimer(Register),
    SetSoundTimer(Register),
    AddToI(Register),
    SpriteAddr(Register),
    Decimal(Register),
    StoreRegisters(Register),
    LoadRegisters(Register),
}

macro_rules! NNN {
    ($x:expr) => {
        Address(($x.1 as u16) * 256 + ($x.2 as u16) * 16 + ($x.3 as u16))
    };
}

macro_rules! NN {
    ($x:expr) => {
        Value($x.2 * 16 + $x.3)
    };
}

macro_rules! N {
    ($x:expr) => {
        Value($x.3)
    };
}

macro_rules! X {
    ($x:expr) => {
        Register($x.1)
    };
}

macro_rules! Y {
    ($x:expr) => {
        Register($x.2)
    };
}

impl Instruction {
    pub fn from_16bit(a: u8, b: u8) -> Instruction {
        let bytes = (a >> 4 & 0x0F, a & 0x0F, b >> 4 & 0x0F, b & 0x0F);
        match bytes {
            (0, 0, 0, 0) => Instruction::Noop,
            (0, 0, 14, 0) => Instruction::ClearDisplay,
            (0, 0, 14, 14) => Instruction::ReturnSubroutine,
            (0, 2..=15, _, _) => Instruction::MachineCodeRoutine(NNN!(bytes)),
            (1, _, _, _) => Instruction::Jump(NNN!(bytes)),
            (2, _, _, _) => Instruction::CallSubroutine(NNN!(bytes)),
            (3, _, _, _) => Instruction::IfNotEqualConst(X!(bytes), NN!(bytes)),
            (4, _, _, _) => Instruction::IfEqualConst(X!(bytes), NN!(bytes)),
            (5, _, _, 0) => Instruction::IfNotEqual(X!(bytes), Y!(bytes)),
            (6, _, _, _) => Instruction::SetConst(X!(bytes), NN!(bytes)),
            (7, _, _, _) => Instruction::AddConst(X!(bytes), NN!(bytes)),
            (8, _, _, 0) => Instruction::Set(X!(bytes), Y!(bytes)),
            (8, _, _, 1) => Instruction::Or(X!(bytes), Y!(bytes)),
            (8, _, _, 2) => Instruction::And(X!(bytes), Y!(bytes)),
            (8, _, _, 3) => Instruction::Xor(X!(bytes), Y!(bytes)),
            (8, _, _, 4) => Instruction::Add(X!(bytes), Y!(bytes)),
            (8, _, _, 5) => Instruction::Sub(X!(bytes), Y!(bytes)),
            (8, _, _, 6) => Instruction::RightShift(X!(bytes)),
            (8, _, _, 7) => Instruction::NegSub(X!(bytes), Y!(bytes)),
            (8, _, _, 14) => Instruction::LeftShift(X!(bytes)),
            (9, _, _, 0) => Instruction::IfEqual(X!(bytes), Y!(bytes)),
            (10, _, _, _) => Instruction::SetI(NNN!(bytes)),
            (11, _, _, _) => Instruction::JumpAdd(NNN!(bytes)),
            (12, _, _, _) => Instruction::Rand(X!(bytes), NN!(bytes)),
            (13, _, _, _) => Instruction::Draw(X!(bytes), Y!(bytes), N!(bytes)),
            (14, _, 9, 14) => Instruction::IfNotKey(X!(bytes)),
            (14, _, 10, 1) => Instruction::IfKey(X!(bytes)),
            (15, _, 0, 7) => Instruction::GetDelayTimer(X!(bytes)),
            (15, _, 0, 10) => Instruction::WaitKey(X!(bytes)),
            (15, _, 1, 5) => Instruction::SetDelayTimer(X!(bytes)),
            (15, _, 1, 8) => Instruction::SetSoundTimer(X!(bytes)),
            (15, _, 1, 14) => Instruction::AddToI(X!(bytes)),
            (15, _, 2, 9) => Instruction::SpriteAddr(X!(bytes)),
            (15, _, 3, 3) => Instruction::Decimal(X!(bytes)),
            (15, _, 5, 5) => Instruction::StoreRegisters(X!(bytes)),
            (15, _, 6, 5) => Instruction::LoadRegisters(X!(bytes)),
            _ => panic!("Invalid rawop: {:?}", bytes),
        }
    }
}

#[cfg(test)]
mod tests {
    // TODO
}

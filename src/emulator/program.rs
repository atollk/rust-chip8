use super::basics::{Address, Register, Value};

pub struct Program {
    instructions: Vec<Instruction>,
}

#[derive(std::fmt::Debug)]
pub struct RawOp(u8, u8, u8, u8);

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

impl Program {
    pub fn from_rawops(bytes: Vec<RawOp>) -> Program {
        Program {
            instructions: bytes.into_iter().map(Instruction::from_rawop).collect(),
        }
    }
}

macro_rules! NNN {
    ($op:expr) => {
        Address(($op.1 as u16) * 256 + ($op.2 as u16) * 16 + ($op.3 as u16))
    };
}

macro_rules! NN {
    ($op:expr) => {
        Value($op.2 * 16 + $op.3)
    };
}

macro_rules! N {
    ($op:expr) => {
        Value($op.3)
    };
}

macro_rules! X {
    ($op:expr) => {
        Register($op.1)
    };
}

macro_rules! Y {
    ($op:expr) => {
        Register($op.2)
    };
}

impl Instruction {
    fn from_rawop(op: RawOp) -> Instruction {
        match (op.0, op.1, op.2, op.3) {
            (0, 0, 0, 0) => Instruction::Noop,
            (0, 0, 14, 0) => Instruction::ClearDisplay,
            (0, 0, 14, 14) => Instruction::ReturnSubroutine,
            (0, 2..=15, _, _) => Instruction::MachineCodeRoutine(NNN!(op)),
            (1, _, _, _) => Instruction::Jump(NNN!(op)),
            (2, _, _, _) => Instruction::CallSubroutine(NNN!(op)),
            (3, _, _, _) => Instruction::IfNotEqualConst(X!(op), NN!(op)),
            (4, _, _, _) => Instruction::IfEqualConst(X!(op), NN!(op)),
            (5, _, _, 0) => Instruction::IfNotEqual(X!(op), Y!(op)),
            (6, _, _, _) => Instruction::SetConst(X!(op), NN!(op)),
            (7, _, _, _) => Instruction::AddConst(X!(op), NN!(op)),
            (8, _, _, 0) => Instruction::Set(X!(op), Y!(op)),
            (8, _, _, 1) => Instruction::Or(X!(op), Y!(op)),
            (8, _, _, 2) => Instruction::And(X!(op), Y!(op)),
            (8, _, _, 3) => Instruction::Xor(X!(op), Y!(op)),
            (8, _, _, 4) => Instruction::Add(X!(op), Y!(op)),
            (8, _, _, 5) => Instruction::Sub(X!(op), Y!(op)),
            (8, _, _, 6) => Instruction::RightShift(X!(op)),
            (8, _, _, 7) => Instruction::NegSub(X!(op), Y!(op)),
            (8, _, _, 14) => Instruction::LeftShift(X!(op)),
            (9, _, _, 0) => Instruction::IfEqual(X!(op), Y!(op)),
            (10, _, _, _) => Instruction::SetI(NNN!(op)),
            (11, _, _, _) => Instruction::JumpAdd(NNN!(op)),
            (12, _, _, _) => Instruction::Rand(X!(op), NN!(op)),
            (13, _, _, _) => Instruction::Draw(X!(op), Y!(op), N!(op)),
            (14, _, 9, 14) => Instruction::IfNotKey(X!(op)),
            (14, _, 10, 1) => Instruction::IfKey(X!(op)),
            (15, _, 0, 7) => Instruction::GetDelayTimer(X!(op)),
            (15, _, 0, 10) => Instruction::WaitKey(X!(op)),
            (15, _, 1, 5) => Instruction::SetDelayTimer(X!(op)),
            (15, _, 1, 8) => Instruction::SetSoundTimer(X!(op)),
            (15, _, 1, 14) => Instruction::AddToI(X!(op)),
            (15, _, 2, 9) => Instruction::SpriteAddr(X!(op)),
            (15, _, 3, 3) => Instruction::Decimal(X!(op)),
            (15, _, 5, 5) => Instruction::StoreRegisters(X!(op)),
            (15, _, 6, 5) => Instruction::LoadRegisters(X!(op)),
            _ => panic!("Invalid rawop: {:?}", op),
        }
    }
}

#[cfg(test)]
mod tests {
    // TODO
}

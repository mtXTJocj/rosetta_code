use std::fmt;

#[derive(Debug)]
pub enum InstructionKind {
    Fetch(u32),
    Store(u32),
    Push(i32),
    Jump(i32),
    Jz(i32),
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Ne,
    And,
    Or,
    Neg,
    Not,
    Prtc,
    Prti,
    Prts,
    Halt,
}

#[derive(Debug)]
pub struct Instruction {
    pub(crate) kind: InstructionKind,
    pub(crate) address: u32,
}

impl Instruction {
    pub fn new(kind: InstructionKind, address: u32) -> Self {
        Instruction { kind, address }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            InstructionKind::Fetch(val) => write!(f, "{} fetch [{}]", self.address, val),
            InstructionKind::Store(val) => write!(f, "{} store [{}]", self.address, val),
            InstructionKind::Push(val) => write!(f, "{} push {}", self.address, val),
            InstructionKind::Jump(val) => write!(
                f,
                "{} jmp ({}) {}",
                self.address,
                val,
                (self.address + 1).wrapping_add(val as u32)
            ),
            InstructionKind::Jz(val) => write!(
                f,
                "{} jz ({}) {}",
                self.address,
                val,
                (self.address + 1).wrapping_add(val as u32)
            ),
            InstructionKind::Add => write!(f, "{} add", self.address),
            InstructionKind::Sub => write!(f, "{} sub", self.address),
            InstructionKind::Mul => write!(f, "{} mul", self.address),
            InstructionKind::Div => write!(f, "{} div", self.address),
            InstructionKind::Mod => write!(f, "{} mod", self.address),
            InstructionKind::Lt => write!(f, "{} lt", self.address),
            InstructionKind::Gt => write!(f, "{} gt", self.address),
            InstructionKind::Le => write!(f, "{} le", self.address),
            InstructionKind::Ge => write!(f, "{} ge", self.address),
            InstructionKind::Eq => write!(f, "{} eq", self.address),
            InstructionKind::Ne => write!(f, "{} ne", self.address),
            InstructionKind::And => write!(f, "{} and", self.address),
            InstructionKind::Or => write!(f, "{} or", self.address),
            InstructionKind::Neg => write!(f, "{} neg", self.address),
            InstructionKind::Not => write!(f, "{} not", self.address),
            InstructionKind::Prtc => write!(f, "{} prtc", self.address),
            InstructionKind::Prti => write!(f, "{} prti", self.address),
            InstructionKind::Prts => write!(f, "{} prts", self.address),
            InstructionKind::Halt => write!(f, "{} halt", self.address),
        }
    }
}

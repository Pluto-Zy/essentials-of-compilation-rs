use std::fmt::Display;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
#[rustfmt::skip]
pub enum Reg {
    RSP, RBP, RAX, RBX, RCX, RDX, RSI, RDI,
    R8, R9, R10, R11, R12, R13, R14, R15,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum VarArg {
    Imm(i64),
    Reg(Reg),
    Deref(Reg, i64),
    Variable(String),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Instruction<Arg> {
    // Note that for `addq a, b`, b is `lhs` and a is `rhs`.
    Addq { lhs: Arg, rhs: Arg },
    // Note that for `subq a, b`, b is `lhs` and a is `rhs`, since it represents b - a
    Subq { lhs: Arg, rhs: Arg },
    Negq { operand: Arg },
    // Note that for `movq a, b`, a is `from` and b is `to`.
    Movq { from: Arg, to: Arg },
    Pushq { operand: Arg },
    Popq { operand: Arg },
    Callq { callee: String },
    Retq,
    Jmp { target: String },
}

pub type VarInstr = Instruction<VarArg>;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Block<ArgType> {
    pub label: String,
    pub instructions: Vec<Instruction<ArgType>>,
}

pub type VarBlock = Block<VarArg>;

impl<ArgType> Block<ArgType> {
    pub fn new(label: String) -> Self {
        Self {
            label,
            instructions: Vec::new(),
        }
    }

    pub fn add_instr(&mut self, instr: Instruction<ArgType>) {
        self.instructions.push(instr);
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Program<ArgType> {
    pub body: Vec<Block<ArgType>>,
}

pub type VarProgram = Program<VarArg>;

impl<ArgType> Program<ArgType> {
    pub fn new() -> Self {
        Self { body: Vec::new() }
    }
}

impl From<Reg> for VarArg {
    fn from(value: Reg) -> Self {
        VarArg::Reg(value)
    }
}

impl Display for Reg {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Reg::*;

        write!(
            f,
            "{}",
            match self {
                RSP => "%rsp", RBP => "%rbp", RAX => "%rax", RBX => "%rbx",
                RCX => "%rcx", RDX => "%rdx", RSI => "%rsi", RDI => "%rdi",
                R8 => "%r8", R9 => "%r9", R10 => "%r10", R11 => "%r11",
                R12 => "%r12", R13 => "%r13", R14 => "%r14", R15 => "%r15",
            }
        )
    }
}

impl Display for VarArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use VarArg::*;
        match self {
            Imm(value) => write!(f, "$0x{:x}", value),
            Reg(reg) => write!(f, "{}", reg),
            Deref(reg, offset) => write!(f, "{:x}({})", offset, reg),
            Variable(name) => write!(f, "{}", name),
        }
    }
}

impl<ArgType: Display> Display for Instruction<ArgType> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Instruction::*;
        match self {
            Addq { lhs, rhs } => write!(f, "addq    {}, {}", rhs, lhs),
            Subq { lhs, rhs } => write!(f, "subq    {}, {}", rhs, lhs),
            Negq { operand } => write!(f, "negq    {}", operand),
            Movq { from, to } => write!(f, "movq    {}, {}", from, to),
            Pushq { operand } => write!(f, "pushq   {}", operand),
            Popq { operand } => write!(f, "popq    {}", operand),
            Callq { callee } => write!(f, "callq   {}", callee),
            Retq => write!(f, "retq"),
            Jmp { target } => write!(f, "jmp     {}", target),
        }
    }
}

impl<ArgType: Display> Display for Block<ArgType> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}:", self.label)?;
        self.instructions
            .iter()
            .try_for_each(|instr| writeln!(f, "    {}", instr))
    }
}

impl<ArgType: Display> Display for Program<ArgType> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.body
            .iter()
            .try_for_each(|block| write!(f, "{}", block))
    }
}

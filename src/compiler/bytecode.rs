/// Bytecode instructions for the Aether VM.
#[derive(Debug, Clone)]
pub enum OpCode {
    // Stack operations
    Push(Constant),
    Pop,
    Dup,

    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Neg,

    // Comparison
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,

    // Logical
    And,
    Or,
    Not,

    // Variables
    LoadLocal(usize),
    StoreLocal(usize),
    LoadGlobal(String),
    StoreGlobal(String),

    // Functions
    Call(usize), // arg count
    Return,

    // Jumps
    Jump(usize),
    JumpIfFalse(usize),
    JumpIfTrue(usize),
    Loop(usize),

    // Objects
    GetField(String),
    SetField(String),
    CreateInstance(String),

    // Collections
    CreateList(usize),
    CreateMap(usize),
    Index,
    SetIndex,

    // Constants
    LoadConst(usize),

    // String interpolation
    BuildString(usize),

    // Print (built-in)
    Print(usize),

    // Nil
    PushNil,
    PushTrue,
    PushFalse,

    // Halt
    Halt,
}

/// Constant values in the constant pool.
#[derive(Debug, Clone)]
pub enum Constant {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Nil,
}

/// A compiled function chunk.
#[derive(Debug, Clone)]
pub struct Chunk {
    pub name: String,
    pub code: Vec<OpCode>,
    pub constants: Vec<Constant>,
    pub local_count: usize,
}

impl Chunk {
    pub fn new(name: String) -> Self {
        Self {
            name,
            code: Vec::new(),
            constants: Vec::new(),
            local_count: 0,
        }
    }

    pub fn emit(&mut self, op: OpCode) -> usize {
        let idx = self.code.len();
        self.code.push(op);
        idx
    }

    pub fn add_constant(&mut self, val: Constant) -> usize {
        self.constants.push(val);
        self.constants.len() - 1
    }

    pub fn patch_jump(&mut self, idx: usize) {
        let target = self.code.len();
        match &mut self.code[idx] {
            OpCode::Jump(ref mut t) | OpCode::JumpIfFalse(ref mut t) | OpCode::JumpIfTrue(ref mut t) => {
                *t = target;
            }
            _ => {}
        }
    }
}

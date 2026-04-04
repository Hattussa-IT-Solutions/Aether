use std::collections::HashMap;
use crate::compiler::bytecode::*;

/// VM runtime value.
#[derive(Debug, Clone)]
pub enum VMValue {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Nil,
    List(Vec<VMValue>),
    /// Function reference: (chunk_index, param_count).
    Function(usize, usize),
}

impl std::fmt::Display for VMValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VMValue::Int(n) => write!(f, "{}", n),
            VMValue::Float(n) => {
                if n.fract() == 0.0 && n.is_finite() {
                    write!(f, "{:.1}", n)
                } else {
                    write!(f, "{}", n)
                }
            }
            VMValue::Str(s) => write!(f, "{}", s),
            VMValue::Bool(b) => write!(f, "{}", b),
            VMValue::Nil => write!(f, "nil"),
            VMValue::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            VMValue::Function(idx, _) => write!(f, "<function@{}>", idx),
        }
    }
}

/// A call frame on the VM call stack.
struct CallFrame {
    /// Which chunk (function) is executing.
    chunk_idx: usize,
    /// Instruction pointer within the chunk.
    ip: usize,
    /// Stack base: where this frame's locals start on the stack.
    stack_base: usize,
}

/// Stack-based virtual machine with call frames.
pub struct VM {
    stack: Vec<VMValue>,
    globals: HashMap<String, VMValue>,
    frames: Vec<CallFrame>,
    chunks: Vec<Chunk>,
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

impl VM {
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(256),
            globals: HashMap::new(),
            frames: Vec::with_capacity(64),
            chunks: Vec::new(),
        }
    }

    /// Execute a set of compiled chunks. chunks[0] is main.
    pub fn execute(&mut self, chunk: &Chunk) -> Result<(), String> {
        // For backward compatibility — single chunk execution
        self.chunks = vec![chunk.clone()];
        self.run_from_chunk(0)
    }

    /// Execute with all chunks (functions).
    pub fn execute_all(&mut self, chunks: &[Chunk]) -> Result<(), String> {
        self.chunks = chunks.to_vec();
        self.run_from_chunk(0)
    }

    fn run_from_chunk(&mut self, chunk_idx: usize) -> Result<(), String> {
        // Push initial frame for main
        let local_count = self.chunks[chunk_idx].local_count;
        let stack_base = self.stack.len();
        // Pre-allocate locals
        for _ in 0..local_count {
            self.stack.push(VMValue::Nil);
        }
        self.frames.push(CallFrame {
            chunk_idx,
            ip: 0,
            stack_base,
        });

        self.run()
    }

    fn run(&mut self) -> Result<(), String> {
        loop {
            let frame = self.frames.last_mut().unwrap();
            let chunk_idx = frame.chunk_idx;
            let ip = frame.ip;

            if ip >= self.chunks[chunk_idx].code.len() {
                break;
            }

            let op = self.chunks[chunk_idx].code[ip].clone();
            self.frames.last_mut().unwrap().ip += 1;

            match op {
                OpCode::LoadConst(idx) => {
                    let val = match &self.chunks[chunk_idx].constants[idx] {
                        Constant::Int(n) => VMValue::Int(*n),
                        Constant::Float(n) => VMValue::Float(*n),
                        Constant::Str(s) => VMValue::Str(s.clone()),
                        Constant::Bool(b) => VMValue::Bool(*b),
                        Constant::Nil => VMValue::Nil,
                        Constant::Function(ci, pc) => VMValue::Function(*ci, *pc),
                    };
                    self.stack.push(val);
                }
                OpCode::Push(c) => {
                    let val = match c {
                        Constant::Int(n) => VMValue::Int(n),
                        Constant::Float(n) => VMValue::Float(n),
                        Constant::Str(s) => VMValue::Str(s),
                        Constant::Bool(b) => VMValue::Bool(b),
                        Constant::Nil => VMValue::Nil,
                        Constant::Function(ci, pc) => VMValue::Function(ci, pc),
                    };
                    self.stack.push(val);
                }
                OpCode::Pop => { self.stack.pop(); }
                OpCode::Dup => {
                    if let Some(top) = self.stack.last().cloned() {
                        self.stack.push(top);
                    }
                }
                OpCode::PushNil => self.stack.push(VMValue::Nil),
                OpCode::PushTrue => self.stack.push(VMValue::Bool(true)),
                OpCode::PushFalse => self.stack.push(VMValue::Bool(false)),

                // Arithmetic
                OpCode::Add => {
                    let b = self.pop();
                    let a = self.pop();
                    self.stack.push(self.binary_add(a, b)?);
                }
                OpCode::Sub => {
                    let b = self.pop();
                    let a = self.pop();
                    self.stack.push(self.binary_num(a, b, |x, y| x - y, |x, y| x - y)?);
                }
                OpCode::Mul => {
                    let b = self.pop();
                    let a = self.pop();
                    self.stack.push(self.binary_num(a, b, |x, y| x * y, |x, y| x * y)?);
                }
                OpCode::Div => {
                    let b = self.pop();
                    let a = self.pop();
                    // Division by zero check
                    match (&a, &b) {
                        (_, VMValue::Int(0)) => return Err("division by zero".into()),
                        (_, VMValue::Float(f)) if *f == 0.0 => return Err("division by zero".into()),
                        _ => {}
                    }
                    self.stack.push(self.binary_num(a, b, |x, y| x / y, |x, y| x / y)?);
                }
                OpCode::Mod => {
                    let b = self.pop();
                    let a = self.pop();
                    self.stack.push(self.binary_num(a, b, |x, y| x % y, |x, y| x % y)?);
                }
                OpCode::Pow => {
                    let b = self.pop();
                    let a = self.pop();
                    match (&a, &b) {
                        (VMValue::Int(x), VMValue::Int(y)) if *y >= 0 => {
                            self.stack.push(VMValue::Int(x.pow(*y as u32)));
                        }
                        _ => {
                            let x = self.as_f64(&a);
                            let y = self.as_f64(&b);
                            self.stack.push(VMValue::Float(x.powf(y)));
                        }
                    }
                }
                OpCode::Neg => {
                    let a = self.pop();
                    match a {
                        VMValue::Int(n) => self.stack.push(VMValue::Int(-n)),
                        VMValue::Float(f) => self.stack.push(VMValue::Float(-f)),
                        _ => return Err("cannot negate".into()),
                    }
                }

                // Comparison
                OpCode::Eq => { let b = self.pop(); let a = self.pop(); self.stack.push(VMValue::Bool(self.equal(&a, &b))); }
                OpCode::Ne => { let b = self.pop(); let a = self.pop(); self.stack.push(VMValue::Bool(!self.equal(&a, &b))); }
                OpCode::Lt => { let b = self.pop(); let a = self.pop(); self.stack.push(VMValue::Bool(self.compare_lt(&a, &b))); }
                OpCode::Gt => { let b = self.pop(); let a = self.pop(); self.stack.push(VMValue::Bool(self.compare_lt(&b, &a))); }
                OpCode::Le => { let b = self.pop(); let a = self.pop(); self.stack.push(VMValue::Bool(!self.compare_lt(&b, &a))); }
                OpCode::Ge => { let b = self.pop(); let a = self.pop(); self.stack.push(VMValue::Bool(!self.compare_lt(&a, &b))); }

                // Logical
                OpCode::And => { let b = self.pop(); let a = self.pop(); self.stack.push(VMValue::Bool(self.truthy(&a) && self.truthy(&b))); }
                OpCode::Or => { let b = self.pop(); let a = self.pop(); self.stack.push(VMValue::Bool(self.truthy(&a) || self.truthy(&b))); }
                OpCode::Not => { let a = self.pop(); self.stack.push(VMValue::Bool(!self.truthy(&a))); }

                // Variables
                OpCode::LoadGlobal(ref name) => {
                    let val = self.globals.get(name).cloned().unwrap_or(VMValue::Nil);
                    self.stack.push(val);
                }
                OpCode::StoreGlobal(ref name) => {
                    let val = self.pop();
                    self.globals.insert(name.clone(), val);
                }
                OpCode::LoadLocal(idx) => {
                    let base = self.frames.last().unwrap().stack_base;
                    let val = self.stack.get(base + idx).cloned().unwrap_or(VMValue::Nil);
                    self.stack.push(val);
                }
                OpCode::StoreLocal(idx) => {
                    let val = self.pop();
                    let base = self.frames.last().unwrap().stack_base;
                    let slot = base + idx;
                    // Ensure stack is large enough
                    while self.stack.len() <= slot {
                        self.stack.push(VMValue::Nil);
                    }
                    self.stack[slot] = val;
                }

                // Jump
                OpCode::Jump(target) => {
                    self.frames.last_mut().unwrap().ip = target;
                }
                OpCode::JumpIfFalse(target) => {
                    let val = self.pop();
                    if !self.truthy(&val) {
                        self.frames.last_mut().unwrap().ip = target;
                    }
                }
                OpCode::JumpIfTrue(target) => {
                    let val = self.pop();
                    if self.truthy(&val) {
                        self.frames.last_mut().unwrap().ip = target;
                    }
                }
                OpCode::Loop(target) => {
                    self.frames.last_mut().unwrap().ip = target;
                }

                // Function call
                OpCode::Call(arg_count) => {
                    // The function value is below the arguments on the stack
                    let func_pos = self.stack.len() - arg_count - 1;
                    let func_val = self.stack[func_pos].clone();

                    if let VMValue::Function(target_chunk, _param_count) = func_val {
                        let local_count = self.chunks[target_chunk].local_count;
                        // Stack layout: [... func_val, arg0, arg1, ...]
                        // New frame's stack_base points to where locals start
                        let stack_base = func_pos + 1; // args start here

                        // Pad locals beyond params
                        let args_on_stack = arg_count;
                        for _ in args_on_stack..local_count {
                            self.stack.push(VMValue::Nil);
                        }

                        // Remove the function value from the stack by overwriting
                        // Actually, shift args down by 1 to overwrite the function value
                        let new_base = func_pos;
                        for i in 0..arg_count {
                            self.stack[new_base + i] = self.stack[new_base + 1 + i].clone();
                        }
                        // Remove the extra slot
                        self.stack.remove(new_base + arg_count);

                        // Pad for remaining locals
                        let current_locals = arg_count;
                        for _ in current_locals..local_count {
                            // Insert at the right position
                            self.stack.insert(new_base + arg_count, VMValue::Nil);
                        }

                        self.frames.push(CallFrame {
                            chunk_idx: target_chunk,
                            ip: 0,
                            stack_base: new_base,
                        });

                        // Check for stack overflow
                        if self.frames.len() > 1000 {
                            return Err("Stack overflow: maximum recursion depth (1000) exceeded".into());
                        }
                    } else {
                        return Err(format!("cannot call {}", func_val));
                    }
                }

                OpCode::Return => {
                    let return_val = self.pop();
                    let frame = self.frames.pop().unwrap();

                    if self.frames.is_empty() {
                        // Returning from main
                        self.stack.truncate(frame.stack_base);
                        self.stack.push(return_val);
                        break;
                    }

                    // Discard the frame's locals and args
                    self.stack.truncate(frame.stack_base);
                    self.stack.push(return_val);
                }

                // Collections
                OpCode::CreateList(count) => {
                    let mut items = Vec::new();
                    for _ in 0..count {
                        items.push(self.pop());
                    }
                    items.reverse();
                    self.stack.push(VMValue::List(items));
                }

                // Print
                OpCode::Print(count) => {
                    let mut vals = Vec::new();
                    for _ in 0..count {
                        vals.push(self.pop());
                    }
                    vals.reverse();
                    let output: Vec<String> = vals.iter().map(|v| v.to_string()).collect();
                    println!("{}", output.join(" "));
                }

                OpCode::BuildString(count) => {
                    let mut parts = Vec::new();
                    for _ in 0..count {
                        parts.push(self.pop());
                    }
                    parts.reverse();
                    let s: String = parts.iter().map(|v| v.to_string()).collect();
                    self.stack.push(VMValue::Str(s));
                }

                OpCode::GetField(_) | OpCode::SetField(_)
                | OpCode::CreateInstance(_) | OpCode::CreateMap(_) | OpCode::Index
                | OpCode::SetIndex => {
                    // Complex operations — not yet implemented in VM
                }

                OpCode::Halt => break,
            }
        }
        Ok(())
    }

    fn pop(&mut self) -> VMValue {
        self.stack.pop().unwrap_or(VMValue::Nil)
    }

    fn as_f64(&self, val: &VMValue) -> f64 {
        match val {
            VMValue::Int(n) => *n as f64,
            VMValue::Float(f) => *f,
            _ => 0.0,
        }
    }

    fn truthy(&self, val: &VMValue) -> bool {
        match val {
            VMValue::Bool(b) => *b,
            VMValue::Nil => false,
            VMValue::Int(0) => false,
            _ => true,
        }
    }

    fn compare_lt(&self, a: &VMValue, b: &VMValue) -> bool {
        match (a, b) {
            (VMValue::Int(x), VMValue::Int(y)) => x < y,
            (VMValue::Float(x), VMValue::Float(y)) => x < y,
            (VMValue::Int(x), VMValue::Float(y)) => (*x as f64) < *y,
            (VMValue::Float(x), VMValue::Int(y)) => *x < (*y as f64),
            _ => false,
        }
    }

    fn equal(&self, a: &VMValue, b: &VMValue) -> bool {
        match (a, b) {
            (VMValue::Int(x), VMValue::Int(y)) => x == y,
            (VMValue::Float(x), VMValue::Float(y)) => x == y,
            (VMValue::Int(x), VMValue::Float(y)) => (*x as f64) == *y,
            (VMValue::Float(x), VMValue::Int(y)) => *x == (*y as f64),
            (VMValue::Str(x), VMValue::Str(y)) => x == y,
            (VMValue::Bool(x), VMValue::Bool(y)) => x == y,
            (VMValue::Nil, VMValue::Nil) => true,
            _ => false,
        }
    }

    fn binary_add(&self, a: VMValue, b: VMValue) -> Result<VMValue, String> {
        match (&a, &b) {
            (VMValue::Int(x), VMValue::Int(y)) => Ok(VMValue::Int(x + y)),
            (VMValue::Float(x), VMValue::Float(y)) => Ok(VMValue::Float(x + y)),
            (VMValue::Int(x), VMValue::Float(y)) => Ok(VMValue::Float(*x as f64 + y)),
            (VMValue::Float(x), VMValue::Int(y)) => Ok(VMValue::Float(x + *y as f64)),
            (VMValue::Str(x), VMValue::Str(y)) => Ok(VMValue::Str(format!("{}{}", x, y))),
            _ => Err(format!("cannot add {} and {}", a, b)),
        }
    }

    fn binary_num(&self, a: VMValue, b: VMValue, int_fn: impl Fn(i64, i64) -> i64, float_fn: impl Fn(f64, f64) -> f64) -> Result<VMValue, String> {
        match (&a, &b) {
            (VMValue::Int(x), VMValue::Int(y)) => Ok(VMValue::Int(int_fn(*x, *y))),
            (VMValue::Float(x), VMValue::Float(y)) => Ok(VMValue::Float(float_fn(*x, *y))),
            (VMValue::Int(x), VMValue::Float(y)) => Ok(VMValue::Float(float_fn(*x as f64, *y))),
            (VMValue::Float(x), VMValue::Int(y)) => Ok(VMValue::Float(float_fn(*x, *y as f64))),
            _ => Err(format!("cannot perform arithmetic on {} and {}", a, b)),
        }
    }
}

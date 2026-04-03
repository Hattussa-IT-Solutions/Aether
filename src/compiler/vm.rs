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
}

impl std::fmt::Display for VMValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VMValue::Int(n) => write!(f, "{}", n),
            VMValue::Float(n) => write!(f, "{}", n),
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
        }
    }
}

/// Stack-based virtual machine.
pub struct VM {
    stack: Vec<VMValue>,
    globals: HashMap<String, VMValue>,
    ip: usize,
}

impl VM {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            globals: HashMap::new(),
            ip: 0,
        }
    }

    /// Execute a compiled chunk.
    pub fn execute(&mut self, chunk: &Chunk) -> Result<(), String> {
        self.ip = 0;
        while self.ip < chunk.code.len() {
            let op = chunk.code[self.ip].clone();
            self.ip += 1;

            match op {
                OpCode::LoadConst(idx) => {
                    let val = match &chunk.constants[idx] {
                        Constant::Int(n) => VMValue::Int(*n),
                        Constant::Float(n) => VMValue::Float(*n),
                        Constant::Str(s) => VMValue::Str(s.clone()),
                        Constant::Bool(b) => VMValue::Bool(*b),
                        Constant::Nil => VMValue::Nil,
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
                        (VMValue::Int(x), VMValue::Int(y)) => self.stack.push(VMValue::Int(x.pow(*y as u32))),
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
                OpCode::Lt => { let b = self.pop(); let a = self.pop(); self.stack.push(VMValue::Bool(self.as_f64(&a) < self.as_f64(&b))); }
                OpCode::Gt => { let b = self.pop(); let a = self.pop(); self.stack.push(VMValue::Bool(self.as_f64(&a) > self.as_f64(&b))); }
                OpCode::Le => { let b = self.pop(); let a = self.pop(); self.stack.push(VMValue::Bool(self.as_f64(&a) <= self.as_f64(&b))); }
                OpCode::Ge => { let b = self.pop(); let a = self.pop(); self.stack.push(VMValue::Bool(self.as_f64(&a) >= self.as_f64(&b))); }

                // Logical
                OpCode::And => { let b = self.pop(); let a = self.pop(); self.stack.push(VMValue::Bool(self.truthy(&a) && self.truthy(&b))); }
                OpCode::Or => { let b = self.pop(); let a = self.pop(); self.stack.push(VMValue::Bool(self.truthy(&a) || self.truthy(&b))); }
                OpCode::Not => { let a = self.pop(); self.stack.push(VMValue::Bool(!self.truthy(&a))); }

                // Variables
                OpCode::LoadGlobal(name) => {
                    let val = self.globals.get(&name).cloned().unwrap_or(VMValue::Nil);
                    self.stack.push(val);
                }
                OpCode::StoreGlobal(name) => {
                    let val = self.pop();
                    self.globals.insert(name, val);
                }
                OpCode::LoadLocal(idx) => {
                    let val = self.stack.get(idx).cloned().unwrap_or(VMValue::Nil);
                    self.stack.push(val);
                }
                OpCode::StoreLocal(idx) => {
                    let val = self.stack.last().cloned().unwrap_or(VMValue::Nil);
                    if idx < self.stack.len() {
                        self.stack[idx] = val;
                    }
                }

                // Jump
                OpCode::Jump(target) => { self.ip = target; }
                OpCode::JumpIfFalse(target) => {
                    let val = self.pop();
                    if !self.truthy(&val) { self.ip = target; }
                }
                OpCode::JumpIfTrue(target) => {
                    let val = self.pop();
                    if self.truthy(&val) { self.ip = target; }
                }
                OpCode::Loop(target) => { self.ip = target; }

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

                OpCode::Call(_) | OpCode::Return | OpCode::GetField(_) | OpCode::SetField(_)
                | OpCode::CreateInstance(_) | OpCode::CreateMap(_) | OpCode::Index
                | OpCode::SetIndex => {
                    // Complex operations — fall back to interpreter for now
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

    fn equal(&self, a: &VMValue, b: &VMValue) -> bool {
        match (a, b) {
            (VMValue::Int(x), VMValue::Int(y)) => x == y,
            (VMValue::Float(x), VMValue::Float(y)) => x == y,
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

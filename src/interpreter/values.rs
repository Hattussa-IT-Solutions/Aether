use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::sync::Mutex;

use crate::parser::ast::*;

// ═══════════════════════════════════════════════════════════════
// Global extension method registry
// ═══════════════════════════════════════════════════════════════

/// Global registry mapping type name -> list of extension FuncDefs.
/// Accessed from exec.rs (registration) and eval.rs (lookup).
static EXTENSIONS: Mutex<Option<HashMap<String, Vec<FuncDef>>>> = Mutex::new(None);

/// Register extension methods for a type.
pub fn register_extension(type_name: &str, methods: Vec<FuncDef>) {
    let mut guard = EXTENSIONS.lock().unwrap();
    let registry = guard.get_or_insert_with(HashMap::new);
    let entry = registry.entry(type_name.to_string()).or_insert_with(Vec::new);
    entry.extend(methods);
}

/// Look up extension methods for a type name.
/// Returns a clone of the method list (or empty).
pub fn get_extensions(type_name: &str) -> Vec<FuncDef> {
    let guard = EXTENSIONS.lock().unwrap();
    guard.as_ref()
        .and_then(|m| m.get(type_name))
        .cloned()
        .unwrap_or_default()
}

// ═══════════════════════════════════════════════════════════════
// Global weave registry
// ═══════════════════════════════════════════════════════════════

/// Global registry mapping weave name -> WeaveDef.
static WEAVES: Mutex<Option<HashMap<String, WeaveDef>>> = Mutex::new(None);

/// Register a weave definition.
pub fn register_weave(name: &str, weave: WeaveDef) {
    let mut guard = WEAVES.lock().unwrap();
    let registry = guard.get_or_insert_with(HashMap::new);
    registry.insert(name.to_string(), weave);
}

/// Look up a weave definition.
pub fn get_weave(name: &str) -> Option<WeaveDef> {
    let guard = WEAVES.lock().unwrap();
    guard.as_ref().and_then(|m| m.get(name)).cloned()
}

/// Runtime value for the Aether interpreter.
#[derive(Clone, Debug)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Char(char),
    Nil,

    /// List (mutable array).
    List(Rc<RefCell<Vec<Value>>>),

    /// Map (string-keyed dictionary).
    Map(Rc<RefCell<HashMap<String, Value>>>),

    /// Set (unique values).
    Set(Rc<RefCell<Vec<Value>>>),

    /// Tuple (fixed-size, immutable).
    Tuple(Vec<Value>),

    /// Range object.
    Range {
        start: i64,
        end: i64,
        inclusive: bool,
        step: i64,
    },

    /// User-defined function.
    Function(Rc<FunctionValue>),

    /// Native/built-in function.
    NativeFunction(Rc<NativeFunctionValue>),

    /// Class definition (the class itself, not an instance).
    Class(Rc<ClassValue>),

    /// Instance of a class.
    Instance(Rc<RefCell<InstanceValue>>),

    /// Struct definition.
    StructDef(Rc<StructDefValue>),

    /// Instance of a struct.
    StructInstance(Rc<RefCell<InstanceValue>>),

    /// Enum definition.
    EnumDef(Rc<EnumDefValue>),

    /// Enum variant value.
    EnumVariant {
        enum_name: String,
        variant: String,
        fields: Vec<Value>,
    },

    /// Result Ok/Err.
    Ok(Box<Value>),
    Err(Box<Value>),

    /// Wrapped Python object (via pyo3).
    PythonObject(crate::bridge::python::PythonObjectWrapper),
}

/// A user-defined function captured at runtime.
#[derive(Debug, Clone)]
pub struct FunctionValue {
    pub name: String,
    pub params: Vec<Param>,
    pub body: FuncBody,
    pub closure_env: Option<Rc<RefCell<Environment>>>,
    pub is_method: bool,
    /// Cached slot names: index i holds the variable name for slot i.
    /// Resolved once on first call, shared via Rc (zero-cost clone).
    pub slot_names: std::cell::RefCell<Option<Rc<Vec<String>>>>,
}

/// A native (built-in) function.
pub struct NativeFunctionValue {
    pub name: String,
    pub arity: Option<usize>, // None = variadic
    pub func: Box<dyn Fn(Vec<Value>) -> Result<Value, String>>,
}

impl fmt::Debug for NativeFunctionValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<native fn {}>", self.name)
    }
}

/// A class definition stored at runtime.
#[derive(Debug, Clone)]
pub struct ClassValue {
    pub name: String,
    pub parent: Option<Rc<ClassValue>>,
    pub fields: Vec<FieldDef>,
    pub methods: HashMap<String, Rc<FunctionValue>>,
    pub static_methods: HashMap<String, Rc<FunctionValue>>,
    pub operators: HashMap<String, crate::parser::ast::OperatorDef>,
    pub computed_props: HashMap<String, crate::parser::ast::Expr>,
    pub init: Option<Rc<FunctionValue>>,
    /// Genetic class support.
    pub is_genetic: bool,
    pub chromosomes: Vec<crate::parser::ast::ChromosomeDef>,
    pub fitness_fn: Option<Rc<FunctionValue>>,
    /// Reactive properties: recomputed on every access (like computed_props).
    pub reactive_props: HashMap<String, crate::parser::ast::Expr>,
    /// Temporal property names, ring buffer size, and optional default expr.
    pub temporal_props: HashMap<String, (usize, Option<crate::parser::ast::Expr>)>,
    /// Mutation-tracked property names.
    pub mutation_tracked: Vec<String>,
    /// Mutation-undoable property names with history depth and optional default.
    pub mutation_undoable: HashMap<String, (usize, Option<crate::parser::ast::Expr>)>,
    /// Face definitions: face name -> visible field list.
    pub faces: HashMap<String, Vec<String>>,
    /// Delegate field names.
    pub delegates: Vec<String>,
    /// Weave names attached to this class.
    pub weaves: Vec<String>,
    /// Lazy properties: name -> initializer expression (evaluated on first access).
    pub lazy_props: HashMap<String, crate::parser::ast::Expr>,
    /// Observed properties: name -> did_change body statements.
    pub observed_props: HashMap<String, Vec<crate::parser::ast::Stmt>>,
    /// Interface names this class claims to implement.
    pub interfaces: Vec<String>,
    /// Morph methods: name -> list of (condition_expr, body_stmts, params).
    pub morph_methods: HashMap<String, crate::parser::ast::MorphDef>,
    /// Bond definitions: field_name -> (target_type_name, via_field_name).
    pub bonds: HashMap<String, (String, String)>,
    /// Capabilities declared for this class.
    pub capabilities: Vec<String>,
}

/// An instance of a class or struct.
#[derive(Debug, Clone)]
pub struct InstanceValue {
    pub class_name: String,
    pub fields: HashMap<String, Value>,
    pub class: Option<Rc<ClassValue>>,
}

/// A struct definition.
#[derive(Debug, Clone)]
pub struct StructDefValue {
    pub name: String,
    pub fields: Vec<FieldDef>,
    pub methods: HashMap<String, Rc<FunctionValue>>,
}

/// An enum definition.
#[derive(Debug, Clone)]
pub struct EnumDefValue {
    pub name: String,
    pub variants: Vec<EnumVariantDef>,
    pub methods: HashMap<String, Rc<FunctionValue>>,
}

// Re-export Environment here so FunctionValue can reference it
use crate::interpreter::environment::Environment;

// ═══════════════════════════════════════════════════════════════
// Value display and comparison
// ═══════════════════════════════════════════════════════════════

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(n) => {
                if n.fract() == 0.0 { write!(f, "{:.1}", n) }
                else { write!(f, "{}", n) }
            }
            Value::Bool(b) => write!(f, "{}", b),
            Value::String(s) => write!(f, "{}", s),
            Value::Char(c) => write!(f, "{}", c),
            Value::Nil => write!(f, "nil"),
            Value::List(items) => {
                let items = items.borrow();
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    match item {
                        Value::String(s) => write!(f, "\"{}\"", s)?,
                        _ => write!(f, "{}", item)?,
                    }
                }
                write!(f, "]")
            }
            Value::Map(map) => {
                let map = map.borrow();
                write!(f, "{{")?;
                for (i, (k, v)) in map.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "\"{}\": {}", k, v)?;
                }
                write!(f, "}}")
            }
            Value::Set(items) => {
                let items = items.borrow();
                write!(f, "{{")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", item)?;
                }
                write!(f, "}}")
            }
            Value::Tuple(items) => {
                write!(f, "(")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", item)?;
                }
                write!(f, ")")
            }
            Value::Range { start, end, inclusive, step } => {
                if *inclusive { write!(f, "{}..={}", start, end)? }
                else { write!(f, "{}..{}", start, end)? }
                if *step != 1 { write!(f, " step {}", step)?; }
                Ok(())
            }
            Value::Function(func) => write!(f, "<fn {}>", func.name),
            Value::NativeFunction(func) => write!(f, "<native fn {}>", func.name),
            Value::Class(cls) => write!(f, "<class {}>", cls.name),
            Value::Instance(inst) => write!(f, "<{} instance>", inst.borrow().class_name),
            Value::StructDef(sd) => write!(f, "<struct {}>", sd.name),
            Value::StructInstance(inst) => write!(f, "<{} instance>", inst.borrow().class_name),
            Value::EnumDef(ed) => write!(f, "<enum {}>", ed.name),
            Value::EnumVariant { enum_name, variant, fields } => {
                write!(f, "{}.{}", enum_name, variant)?;
                if !fields.is_empty() {
                    write!(f, "(")?;
                    for (i, field) in fields.iter().enumerate() {
                        if i > 0 { write!(f, ", ")?; }
                        write!(f, "{}", field)?;
                    }
                    write!(f, ")")?;
                }
                Ok(())
            }
            Value::Ok(val) => write!(f, "Ok({})", val),
            Value::Err(val) => write!(f, "Err({})", val),
            Value::PythonObject(wrapper) => write!(f, "{}", wrapper),
        }
    }
}

impl Value {
    /// Check if a value is truthy.
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Nil => false,
            Value::Int(0) => false,
            Value::Float(f) if *f == 0.0 => false,
            Value::String(s) if s.is_empty() => false,
            _ => true,
        }
    }

    /// Check equality between two values.
    pub fn equals(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Int(a), Value::Float(b)) => (*a as f64) == *b,
            (Value::Float(a), Value::Int(b)) => *a == (*b as f64),
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Char(a), Value::Char(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            _ => false,
        }
    }

    /// Get a string key for map operations.
    pub fn as_map_key(&self) -> Option<String> {
        match self {
            Value::String(s) => Some(s.clone()),
            Value::Int(n) => Some(n.to_string()),
            Value::Bool(b) => Some(b.to_string()),
            _ => None,
        }
    }

    /// Try to get as integer.
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(n) => Some(*n),
            Value::Float(f) => Some(*f as i64),
            _ => None,
        }
    }

    /// Try to get as float.
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            Value::Int(n) => Some(*n as f64),
            _ => None,
        }
    }
}

/// Control flow signals for break/next/return.
#[derive(Debug, Clone)]
pub enum Signal {
    Break(Option<String>),
    Next(Option<String>),
    Return(Value),
    Throw(Value),
}

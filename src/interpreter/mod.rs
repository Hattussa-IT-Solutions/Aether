pub mod environment;
pub mod values;
pub mod eval;
pub mod exec;
pub mod parallel;
pub mod resolver;

use std::cell::RefCell;
use std::rc::Rc;

use crate::interpreter::environment::Environment;
use crate::interpreter::values::*;
use crate::parser::ast::Program;

/// Run a parsed Aether program.
pub fn interpret(program: &Program, env: &mut Environment) -> Result<(), String> {
    // Run the resolver to assign slot indices for fast variable access
    let slot_map = resolver::resolve(program);
    eval::set_slot_map(slot_map);

    match exec::exec_block(&program.statements, env) {
        Ok(()) => Ok(()),
        Err(Signal::Throw(val)) => Err(format!("Unhandled error: {}", val)),
        Err(Signal::Return(val)) => {
            if !matches!(val, Value::Nil) {
                println!("{}", val);
            }
            Ok(())
        }
        Err(Signal::Break(_)) => Err("'break' outside of loop".into()),
        Err(Signal::Next(_)) => Err("'next' outside of loop".into()),
    }
}

/// Register all built-in functions into the environment.
pub fn register_builtins(env: &mut Environment) {
    // print(...) — variadic print
    env.define("print", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "print".into(),
        arity: None,
        func: Box::new(|args| {
            let output: Vec<String> = args.iter().map(|a| a.to_string()).collect();
            println!("{}", output.join(" "));
            Ok(Value::Nil)
        }),
    })));

    // input(prompt) — read line from stdin
    env.define("input", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "input".into(),
        arity: Some(1),
        func: Box::new(|args| {
            if let Some(Value::String(prompt)) = args.first() {
                eprint!("{}", prompt);
            }
            let mut line = String::new();
            std::io::stdin().read_line(&mut line).map_err(|e| e.to_string())?;
            Ok(Value::String(line.trim_end().to_string()))
        }),
    })));

    // len(val) — generic length
    env.define("len", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "len".into(),
        arity: Some(1),
        func: Box::new(|args| {
            match args.first() {
                Some(Value::String(s)) => Ok(Value::Int(s.len() as i64)),
                Some(Value::List(l)) => Ok(Value::Int(l.borrow().len() as i64)),
                Some(Value::Map(m)) => Ok(Value::Int(m.borrow().len() as i64)),
                Some(Value::Set(s)) => Ok(Value::Int(s.borrow().len() as i64)),
                Some(Value::Tuple(t)) => Ok(Value::Int(t.len() as i64)),
                _ => Err("len() requires a collection".into()),
            }
        }),
    })));

    // type(val) — return type name as string
    env.define("type", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "type".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let name = match args.first() {
                Some(Value::Int(_)) => "Int",
                Some(Value::Float(_)) => "Float",
                Some(Value::Bool(_)) => "Bool",
                Some(Value::String(_)) => "Str",
                Some(Value::Char(_)) => "Char",
                Some(Value::Nil) => "Nil",
                Some(Value::List(_)) => "List",
                Some(Value::Map(_)) => "Map",
                Some(Value::Set(_)) => "Set",
                Some(Value::Tuple(_)) => "Tuple",
                Some(Value::Function(_)) => "Function",
                Some(Value::NativeFunction(_)) => "Function",
                Some(Value::Class(c)) => return Ok(Value::String(c.name.clone())),
                Some(Value::Instance(i)) => return Ok(Value::String(i.borrow().class_name.clone())),
                _ => "Unknown",
            };
            Ok(Value::String(name.to_string()))
        }),
    })));

    // str(val) — convert to string
    env.define("str", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "str".into(),
        arity: Some(1),
        func: Box::new(|args| {
            Ok(Value::String(args.first().map(|v| v.to_string()).unwrap_or_default()))
        }),
    })));

    // int(val) — convert to int
    env.define("int", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "int".into(),
        arity: Some(1),
        func: Box::new(|args| {
            match args.first() {
                Some(Value::Int(n)) => Ok(Value::Int(*n)),
                Some(Value::Float(f)) => Ok(Value::Int(*f as i64)),
                Some(Value::String(s)) => s.parse::<i64>().map(Value::Int).map_err(|e| e.to_string()),
                Some(Value::Bool(b)) => Ok(Value::Int(if *b { 1 } else { 0 })),
                _ => Err("cannot convert to Int".into()),
            }
        }),
    })));

    // float(val) — convert to float
    env.define("float", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "float".into(),
        arity: Some(1),
        func: Box::new(|args| {
            match args.first() {
                Some(Value::Float(f)) => Ok(Value::Float(*f)),
                Some(Value::Int(n)) => Ok(Value::Float(*n as f64)),
                Some(Value::String(s)) => s.parse::<f64>().map(Value::Float).map_err(|e| e.to_string()),
                _ => Err("cannot convert to Float".into()),
            }
        }),
    })));

    // Math functions
    register_math_builtins(env);

    // range(start, end) — create range
    env.define("range", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "range".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let start = args.first().and_then(|a| a.as_int()).unwrap_or(0);
            let end = args.get(1).and_then(|a| a.as_int()).unwrap_or(0);
            Ok(Value::Range { start, end, inclusive: false, step: 1 })
        }),
    })));

    // crossover(a, b) — produce a child from two genetic instances
    env.define("crossover", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "crossover".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let parent_a = args.first().cloned().unwrap_or(Value::Nil);
            let parent_b = args.get(1).cloned().unwrap_or(Value::Nil);
            let mut call_env = environment::Environment::new();
            eval::eval_crossover(&parent_a, &parent_b, 0.0, &mut call_env)
                .map_err(|sig| match sig {
                    Signal::Throw(v) => v.to_string(),
                    _ => "crossover error".to_string(),
                })
        }),
    })));

    // breed(a, b, mutation_rate) — crossover with mutation
    env.define("breed", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "breed".into(),
        arity: None,
        func: Box::new(|args| {
            let parent_a = args.first().cloned().unwrap_or(Value::Nil);
            let parent_b = args.get(1).cloned().unwrap_or(Value::Nil);
            let mutation_rate = args.get(2).and_then(|a| a.as_float()).unwrap_or(0.05);
            let mut call_env = environment::Environment::new();
            eval::eval_crossover(&parent_a, &parent_b, mutation_rate, &mut call_env)
                .map_err(|sig| match sig {
                    Signal::Throw(v) => v.to_string(),
                    _ => "breed error".to_string(),
                })
        }),
    })));
}

fn register_math_builtins(env: &mut Environment) {
    let math_fns: Vec<(&str, Box<dyn Fn(Vec<Value>) -> Result<Value, String>>)> = vec![
        ("sqrt", Box::new(|a| Ok(Value::Float(a[0].as_float().unwrap_or(0.0).sqrt())))),
        ("abs", Box::new(|a| match &a[0] {
            Value::Int(n) => Ok(Value::Int(n.abs())),
            Value::Float(f) => Ok(Value::Float(f.abs())),
            _ => Err("abs requires a number".into()),
        })),
        ("floor", Box::new(|a| Ok(Value::Int(a[0].as_float().unwrap_or(0.0).floor() as i64)))),
        ("ceil", Box::new(|a| Ok(Value::Int(a[0].as_float().unwrap_or(0.0).ceil() as i64)))),
        ("round", Box::new(|a| Ok(Value::Int(a[0].as_float().unwrap_or(0.0).round() as i64)))),
        ("sin", Box::new(|a| Ok(Value::Float(a[0].as_float().unwrap_or(0.0).sin())))),
        ("cos", Box::new(|a| Ok(Value::Float(a[0].as_float().unwrap_or(0.0).cos())))),
        ("tan", Box::new(|a| Ok(Value::Float(a[0].as_float().unwrap_or(0.0).tan())))),
        ("log", Box::new(|a| Ok(Value::Float(a[0].as_float().unwrap_or(0.0).ln())))),
        ("log2", Box::new(|a| Ok(Value::Float(a[0].as_float().unwrap_or(0.0).log2())))),
        ("log10", Box::new(|a| Ok(Value::Float(a[0].as_float().unwrap_or(0.0).log10())))),
        ("pow", Box::new(|a| {
            let base = a[0].as_float().unwrap_or(0.0);
            let exp = a.get(1).and_then(|v| v.as_float()).unwrap_or(1.0);
            Ok(Value::Float(base.powf(exp)))
        })),
        ("min", Box::new(|a| {
            let x = a[0].as_float().unwrap_or(f64::MAX);
            let y = a.get(1).and_then(|v| v.as_float()).unwrap_or(f64::MAX);
            Ok(Value::Float(x.min(y)))
        })),
        ("max", Box::new(|a| {
            let x = a[0].as_float().unwrap_or(f64::MIN);
            let y = a.get(1).and_then(|v| v.as_float()).unwrap_or(f64::MIN);
            Ok(Value::Float(x.max(y)))
        })),
    ];

    for (name, func) in math_fns {
        env.define(name, Value::NativeFunction(Rc::new(NativeFunctionValue {
            name: name.to_string(),
            arity: None,
            func,
        })));
    }

    // Math constants
    env.define("PI", Value::Float(std::f64::consts::PI));
    env.define("E", Value::Float(std::f64::consts::E));
    env.define("TAU", Value::Float(std::f64::consts::TAU));
    env.define("INF", Value::Float(f64::INFINITY));

    // Extended stdlib
    crate::stdlib::register_all(env);
}

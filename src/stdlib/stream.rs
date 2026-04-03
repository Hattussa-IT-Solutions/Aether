/// Lazy file streaming utilities for Aether.
///
/// v1 implements batch processing (read-all, filter, return) because the
/// interpreter does not yet have an iterator protocol.  The API mirrors a
/// future lazy implementation so user code will not change.
use std::cell::RefCell;
use std::fs;
use std::rc::Rc;
use crate::interpreter::values::*;

pub fn register(env: &mut crate::interpreter::environment::Environment) {
    // stream_lines(path) -> List[String]
    // Reads the file at `path` and returns every line as a list of strings.
    env.define("stream_lines", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "stream_lines".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let path = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("stream_lines: expected a file path string".into()),
            };
            let content = fs::read_to_string(&path)
                .map_err(|e| format!("stream_lines: {}", e))?;
            let lines: Vec<Value> = content
                .lines()
                .map(|l| Value::String(l.to_string()))
                .collect();
            Ok(Value::List(Rc::new(RefCell::new(lines))))
        }),
    })));

    // stream_count(path, filter_fn) -> Int
    // Count lines in `path` for which filter_fn(line) returns truthy.
    env.define("stream_count", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "stream_count".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let path = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("stream_count: first arg must be a file path string".into()),
            };
            let filter = match args.get(1) {
                Some(v @ Value::Function(_)) | Some(v @ Value::NativeFunction(_)) => v.clone(),
                _ => return Err("stream_count: second arg must be a filter function".into()),
            };
            let content = fs::read_to_string(&path)
                .map_err(|e| format!("stream_count: {}", e))?;
            let mut count: i64 = 0;
            for line in content.lines() {
                let result = call_fn_1(&filter, Value::String(line.to_string()))?;
                if result.is_truthy() {
                    count += 1;
                }
            }
            Ok(Value::Int(count))
        }),
    })));

    // stream_collect(path, filter_fn) -> List[String]
    // Return all lines in `path` for which filter_fn(line) returns truthy.
    env.define("stream_collect", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "stream_collect".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let path = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("stream_collect: first arg must be a file path string".into()),
            };
            let filter = match args.get(1) {
                Some(v @ Value::Function(_)) | Some(v @ Value::NativeFunction(_)) => v.clone(),
                _ => return Err("stream_collect: second arg must be a filter function".into()),
            };
            let content = fs::read_to_string(&path)
                .map_err(|e| format!("stream_collect: {}", e))?;
            let mut matched: Vec<Value> = Vec::new();
            for line in content.lines() {
                let result = call_fn_1(&filter, Value::String(line.to_string()))?;
                if result.is_truthy() {
                    matched.push(Value::String(line.to_string()));
                }
            }
            Ok(Value::List(Rc::new(RefCell::new(matched))))
        }),
    })));
}

/// Call a Value::Function or Value::NativeFunction with one argument.
fn call_fn_1(func: &Value, arg: Value) -> Result<Value, String> {
    match func {
        Value::NativeFunction(nf) => (nf.func)(vec![arg]),
        Value::Function(_) => {
            crate::interpreter::eval::call_function(
                func,
                vec![arg],
                &[],
                &mut crate::interpreter::environment::Environment::new(),
            )
            .map_err(|e| match e {
                Signal::Throw(v)  => v.to_string(),
                Signal::Return(v) => v.to_string(),
                _ => "function error".to_string(),
            })
        }
        _ => Err("expected a function".into()),
    }
}

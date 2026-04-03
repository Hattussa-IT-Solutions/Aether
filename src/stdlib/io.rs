use std::rc::Rc;
use crate::interpreter::values::*;

/// Register I/O functions.
pub fn register_io(env: &mut crate::interpreter::environment::Environment) {
    // fs.read(path) -> Str
    env.define("fs_read", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "fs_read".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let path = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("fs_read requires a string path".into()),
            };
            std::fs::read_to_string(&path).map(Value::String).map_err(|e| e.to_string())
        }),
    })));

    // fs.write(path, content)
    env.define("fs_write", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "fs_write".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let path = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("fs_write requires a string path".into()),
            };
            let content = match args.get(1) {
                Some(Value::String(s)) => s.clone(),
                Some(v) => v.to_string(),
                _ => return Err("fs_write requires content".into()),
            };
            std::fs::write(&path, &content).map(|_| Value::Nil).map_err(|e| e.to_string())
        }),
    })));

    // fs.exists(path) -> Bool
    env.define("fs_exists", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "fs_exists".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let path = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("fs_exists requires a string path".into()),
            };
            Ok(Value::Bool(std::path::Path::new(&path).exists()))
        }),
    })));

    // fs.lines(path) -> List<Str>
    env.define("fs_lines", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "fs_lines".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let path = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("fs_lines requires a string path".into()),
            };
            let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let lines: Vec<Value> = content.lines().map(|l| Value::String(l.to_string())).collect();
            Ok(Value::List(std::rc::Rc::new(std::cell::RefCell::new(lines))))
        }),
    })));
}

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use crate::interpreter::values::*;

pub fn register(env: &mut crate::interpreter::environment::Environment) {
    // process_exec(command) -> Map {stdout, stderr, code, success}
    // The command string is passed to the system shell.
    env.define("process_exec", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "process_exec".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let command = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("process_exec: argument must be a string command".into()),
            };

            #[cfg(target_os = "windows")]
            let output = std::process::Command::new("cmd")
                .args(["/C", &command])
                .output();

            #[cfg(not(target_os = "windows"))]
            let output = std::process::Command::new("sh")
                .args(["-c", &command])
                .output();

            match output {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
                    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
                    let code = out.status.code().unwrap_or(-1) as i64;
                    let success = out.status.success();

                    let mut map = HashMap::new();
                    map.insert("stdout".to_string(), Value::String(stdout));
                    map.insert("stderr".to_string(), Value::String(stderr));
                    map.insert("code".to_string(), Value::Int(code));
                    map.insert("success".to_string(), Value::Bool(success));
                    Ok(Value::Map(Rc::new(RefCell::new(map))))
                }
                Err(e) => Err(format!("process_exec: failed to run command: {}", e)),
            }
        }),
    })));

    // sys_env(key) -> Str | Nil
    env.define("sys_env", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "sys_env".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let key = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("sys_env: argument must be a string key".into()),
            };
            match std::env::var(&key) {
                Ok(val) => Ok(Value::String(val)),
                Err(_) => Ok(Value::Nil),
            }
        }),
    })));

    // sys_env_or(key, default) -> Str
    env.define("sys_env_or", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "sys_env_or".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let key = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("sys_env_or: first argument must be a string key".into()),
            };
            let default = match args.get(1) {
                Some(Value::String(s)) => s.clone(),
                Some(v) => v.to_string(),
                None => return Err("sys_env_or requires 2 arguments".into()),
            };
            Ok(Value::String(std::env::var(&key).unwrap_or(default)))
        }),
    })));

    // sys_set_env(key, value)
    env.define("sys_set_env", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "sys_set_env".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let key = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("sys_set_env: first argument must be a string key".into()),
            };
            let value = match args.get(1) {
                Some(Value::String(s)) => s.clone(),
                Some(v) => v.to_string(),
                None => return Err("sys_set_env requires 2 arguments".into()),
            };
            std::env::set_var(&key, &value);
            Ok(Value::Nil)
        }),
    })));

    // sys_platform() -> Str
    env.define("sys_platform", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "sys_platform".into(),
        arity: Some(0),
        func: Box::new(|_| {
            Ok(Value::String(std::env::consts::OS.to_string()))
        }),
    })));

    // sys_arch() -> Str
    env.define("sys_arch", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "sys_arch".into(),
        arity: Some(0),
        func: Box::new(|_| {
            Ok(Value::String(std::env::consts::ARCH.to_string()))
        }),
    })));

    // sys_cpu_count() -> Int
    env.define("sys_cpu_count", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "sys_cpu_count".into(),
        arity: Some(0),
        func: Box::new(|_| {
            // std::thread::available_parallelism is stable since Rust 1.59
            let count = std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1);
            Ok(Value::Int(count as i64))
        }),
    })));

    // sys_args() -> List<Str>
    env.define("sys_args", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "sys_args".into(),
        arity: Some(0),
        func: Box::new(|_| {
            let args: Vec<Value> = std::env::args()
                .map(Value::String)
                .collect();
            Ok(Value::List(Rc::new(RefCell::new(args))))
        }),
    })));
}

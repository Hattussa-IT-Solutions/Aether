use std::rc::Rc;
use std::sync::Mutex;
use crate::interpreter::values::*;

/// Log levels as integers for easy comparison.
/// 0 = debug, 1 = info, 2 = warn, 3 = error, 4 = off
static LOG_LEVEL: Mutex<u8> = Mutex::new(0); // default: debug (show everything)

/// Parse a level name to its integer code.
fn parse_level(s: &str) -> Option<u8> {
    match s.to_lowercase().as_str() {
        "debug" => Some(0),
        "info"  => Some(1),
        "warn" | "warning" => Some(2),
        "error" => Some(3),
        "off" | "none" => Some(4),
        _ => None,
    }
}

fn level_name(level: u8) -> &'static str {
    match level {
        0 => "DEBUG",
        1 => "INFO",
        2 => "WARN",
        3 => "ERROR",
        _ => "OFF",
    }
}

/// ANSI color codes
fn level_color(level: u8) -> &'static str {
    match level {
        0 => "\x1b[90m",    // dark gray (debug)
        1 => "\x1b[32m",    // green (info)
        2 => "\x1b[33m",    // yellow (warn)
        3 => "\x1b[31m",    // red (error)
        _ => "\x1b[0m",
    }
}

const RESET: &str = "\x1b[0m";

fn emit_log(level: u8, msg: &str) {
    let current = *LOG_LEVEL.lock().unwrap();
    if level < current {
        return; // filtered out
    }
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let color = level_color(level);
    let name = level_name(level);
    eprintln!("{}[{} {}]{} {}", color, timestamp, name, RESET, msg);
}

pub fn register(env: &mut crate::interpreter::environment::Environment) {
    // log_debug(msg)
    env.define("log_debug", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "log_debug".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let msg = args.first().map(|v| v.to_string()).unwrap_or_default();
            emit_log(0, &msg);
            Ok(Value::Nil)
        }),
    })));

    // log_info(msg)
    env.define("log_info", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "log_info".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let msg = args.first().map(|v| v.to_string()).unwrap_or_default();
            emit_log(1, &msg);
            Ok(Value::Nil)
        }),
    })));

    // log_warn(msg)
    env.define("log_warn", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "log_warn".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let msg = args.first().map(|v| v.to_string()).unwrap_or_default();
            emit_log(2, &msg);
            Ok(Value::Nil)
        }),
    })));

    // log_error(msg)
    env.define("log_error", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "log_error".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let msg = args.first().map(|v| v.to_string()).unwrap_or_default();
            emit_log(3, &msg);
            Ok(Value::Nil)
        }),
    })));

    // log_set_level(level)  — accepts "debug", "info", "warn", "error", "off"
    env.define("log_set_level", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "log_set_level".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let level_str = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("log_set_level: argument must be a string (debug/info/warn/error/off)".into()),
            };
            match parse_level(&level_str) {
                Some(lvl) => {
                    *LOG_LEVEL.lock().unwrap() = lvl;
                    Ok(Value::Nil)
                }
                None => Err(format!(
                    "log_set_level: unknown level '{}'. Use: debug, info, warn, error, off",
                    level_str
                )),
            }
        }),
    })));
}

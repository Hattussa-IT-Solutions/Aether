use std::rc::Rc;
use crate::interpreter::values::*;

pub fn register(env: &mut crate::interpreter::environment::Environment) {
    // term_color(text, color) -> Str
    env.define("term_color", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "term_color".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let text = match args.first() {
                Some(Value::String(s)) => s.clone(),
                Some(v) => v.to_string(),
                None => return Err("term_color requires text as first arg".into()),
            };
            let color = match args.get(1) {
                Some(Value::String(s)) => s.as_str().to_lowercase(),
                _ => return Err("term_color requires a color name as second arg".into()),
            };
            let code = fg_code(&color);
            Ok(Value::String(format!("\x1b[{}m{}\x1b[0m", code, text)))
        }),
    })));

    // term_bold(text) -> Str
    env.define("term_bold", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "term_bold".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let text = match args.first() {
                Some(Value::String(s)) => s.clone(),
                Some(v) => v.to_string(),
                None => return Err("term_bold requires text".into()),
            };
            Ok(Value::String(format!("\x1b[1m{}\x1b[0m", text)))
        }),
    })));

    // term_dim(text) -> Str
    env.define("term_dim", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "term_dim".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let text = match args.first() {
                Some(Value::String(s)) => s.clone(),
                Some(v) => v.to_string(),
                None => return Err("term_dim requires text".into()),
            };
            Ok(Value::String(format!("\x1b[2m{}\x1b[0m", text)))
        }),
    })));

    // term_underline(text) -> Str
    env.define("term_underline", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "term_underline".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let text = match args.first() {
                Some(Value::String(s)) => s.clone(),
                Some(v) => v.to_string(),
                None => return Err("term_underline requires text".into()),
            };
            Ok(Value::String(format!("\x1b[4m{}\x1b[0m", text)))
        }),
    })));

    // term_bg(text, color) -> Str
    env.define("term_bg", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "term_bg".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let text = match args.first() {
                Some(Value::String(s)) => s.clone(),
                Some(v) => v.to_string(),
                None => return Err("term_bg requires text as first arg".into()),
            };
            let color = match args.get(1) {
                Some(Value::String(s)) => s.as_str().to_lowercase(),
                _ => return Err("term_bg requires a color name as second arg".into()),
            };
            let code = bg_code(&color);
            Ok(Value::String(format!("\x1b[{}m{}\x1b[0m", code, text)))
        }),
    })));

    // term_clear() — clears the screen
    env.define("term_clear", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "term_clear".into(),
        arity: Some(0),
        func: Box::new(|_| {
            print!("\x1b[2J\x1b[H");
            Ok(Value::Nil)
        }),
    })));

    // term_width() -> Int — terminal width (default 80)
    env.define("term_width", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "term_width".into(),
        arity: Some(0),
        func: Box::new(|_| {
            // Try to get width via COLUMNS env var or ioctl fallback
            let width = std::env::var("COLUMNS")
                .ok()
                .and_then(|v| v.parse::<i64>().ok())
                .unwrap_or(80);
            Ok(Value::Int(width))
        }),
    })));

    // term_progress(current, total, label) — prints a progress bar
    env.define("term_progress", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "term_progress".into(),
        arity: Some(3),
        func: Box::new(|args| {
            let current = args.first().and_then(|v| v.as_int()).unwrap_or(0);
            let total = args.get(1).and_then(|v| v.as_int()).unwrap_or(100).max(1);
            let label = match args.get(2) {
                Some(Value::String(s)) => s.clone(),
                Some(v) => v.to_string(),
                None => String::new(),
            };

            let bar_width: i64 = 40;
            let filled = ((current as f64 / total as f64) * bar_width as f64).round() as i64;
            let filled = filled.max(0).min(bar_width);
            let empty = bar_width - filled;

            let bar = format!(
                "[{}{}] {}/{} {}",
                "#".repeat(filled as usize),
                "-".repeat(empty as usize),
                current,
                total,
                label
            );
            print!("\r{}", bar);
            use std::io::Write;
            let _ = std::io::stdout().flush();
            Ok(Value::Nil)
        }),
    })));
}

fn fg_code(color: &str) -> &'static str {
    match color {
        "red" => "31",
        "green" => "32",
        "yellow" => "33",
        "blue" => "34",
        "magenta" => "35",
        "cyan" => "36",
        "white" => "37",
        "gray" | "grey" => "90",
        _ => "37",
    }
}

fn bg_code(color: &str) -> &'static str {
    match color {
        "red" => "41",
        "green" => "42",
        "yellow" => "43",
        "blue" => "44",
        "magenta" => "45",
        "cyan" => "46",
        "white" => "47",
        "gray" | "grey" => "100",
        _ => "47",
    }
}

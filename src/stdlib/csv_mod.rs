use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::interpreter::values::*;

pub fn register(env: &mut crate::interpreter::environment::Environment) {
    // csv_parse(text) -> List of Maps
    env.define("csv_parse", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "csv_parse".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let text = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("csv_parse requires a string".into()),
            };
            Ok(parse_csv(&text))
        }),
    })));

    // csv_encode(rows) -> Str
    env.define("csv_encode", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "csv_encode".into(),
        arity: Some(1),
        func: Box::new(|args| {
            match args.first() {
                Some(Value::List(rows)) => {
                    let rows = rows.borrow();
                    Ok(Value::String(encode_csv(&rows)))
                }
                _ => Err("csv_encode requires a list of maps".into()),
            }
        }),
    })));

    // csv_read(path) -> List of Maps
    env.define("csv_read", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "csv_read".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let path = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("csv_read requires a file path string".into()),
            };
            match std::fs::read_to_string(&path) {
                Ok(text) => Ok(parse_csv(&text)),
                Err(e) => Ok(Value::Err(Box::new(Value::String(e.to_string())))),
            }
        }),
    })));

    // csv_write(path, rows)
    env.define("csv_write", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "csv_write".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let path = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("csv_write requires a file path as first arg".into()),
            };
            match args.get(1) {
                Some(Value::List(rows)) => {
                    let rows = rows.borrow();
                    let csv_text = encode_csv(&rows);
                    match std::fs::write(&path, csv_text) {
                        Ok(_) => Ok(Value::Nil),
                        Err(e) => Ok(Value::Err(Box::new(Value::String(e.to_string())))),
                    }
                }
                _ => Err("csv_write requires a list of maps as second arg".into()),
            }
        }),
    })));
}

/// Parse a CSV string into a list of maps, using the first row as headers.
fn parse_csv(text: &str) -> Value {
    let mut lines = text.lines();
    let header_line = match lines.next() {
        Some(l) => l,
        None => return Value::List(Rc::new(RefCell::new(vec![]))),
    };
    let headers: Vec<String> = parse_csv_row(header_line);

    let mut result = Vec::new();
    for line in lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let fields = parse_csv_row(line);
        let mut map = HashMap::new();
        for (i, header) in headers.iter().enumerate() {
            let val = fields.get(i).cloned().unwrap_or_default();
            map.insert(header.clone(), Value::String(val));
        }
        result.push(Value::Map(Rc::new(RefCell::new(map))));
    }
    Value::List(Rc::new(RefCell::new(result)))
}

/// Parse a single CSV row, handling quoted fields.
fn parse_csv_row(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '"' => {
                if in_quotes {
                    // Check for escaped double quote
                    if chars.peek() == Some(&'"') {
                        chars.next();
                        current.push('"');
                    } else {
                        in_quotes = false;
                    }
                } else {
                    in_quotes = true;
                }
            }
            ',' if !in_quotes => {
                fields.push(current.trim().to_string());
                current = String::new();
            }
            _ => current.push(c),
        }
    }
    fields.push(current.trim().to_string());
    fields
}

/// Encode a list of maps to CSV text.
fn encode_csv(rows: &[Value]) -> String {
    if rows.is_empty() {
        return String::new();
    }

    // Collect all keys from all rows for a consistent header
    let mut headers: Vec<String> = Vec::new();
    for row in rows {
        if let Value::Map(map) = row {
            for key in map.borrow().keys() {
                if !headers.contains(key) {
                    headers.push(key.clone());
                }
            }
        }
    }
    headers.sort();

    let mut lines = Vec::new();

    // Header row
    let header_row: Vec<String> = headers.iter().map(|h| csv_escape(h)).collect();
    lines.push(header_row.join(","));

    // Data rows
    for row in rows {
        if let Value::Map(map) = row {
            let map = map.borrow();
            let fields: Vec<String> = headers.iter().map(|h| {
                let val_str = match map.get(h) {
                    Some(Value::String(s)) => s.clone(),
                    Some(v) => v.to_string(),
                    None => String::new(),
                };
                csv_escape(&val_str)
            }).collect();
            lines.push(fields.join(","));
        }
    }

    lines.join("\n")
}

/// Escape a CSV field value, wrapping in quotes if needed.
fn csv_escape(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

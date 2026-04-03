use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::interpreter::values::*;

pub fn register_json(env: &mut crate::interpreter::environment::Environment) {
    // json_encode(val) -> Str
    env.define("json_encode", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "json_encode".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let val = args.first().unwrap_or(&Value::Nil);
            Ok(Value::String(value_to_json(val)))
        }),
    })));

    // json_decode(str) -> Value
    env.define("json_decode", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "json_decode".into(),
        arity: Some(1),
        func: Box::new(|args| {
            match args.first() {
                Some(Value::String(s)) => {
                    let parsed: serde_json::Value = serde_json::from_str(s)
                        .map_err(|e| e.to_string())?;
                    Ok(json_to_value(&parsed))
                }
                _ => Err("json_decode requires a string".into()),
            }
        }),
    })));
}

fn value_to_json(val: &Value) -> String {
    match val {
        Value::Int(n) => n.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
        Value::Nil => "null".to_string(),
        Value::List(items) => {
            let parts: Vec<String> = items.borrow().iter().map(|v| value_to_json(v)).collect();
            format!("[{}]", parts.join(","))
        }
        Value::Map(map) => {
            let parts: Vec<String> = map.borrow().iter()
                .map(|(k, v)| format!("\"{}\":{}", k, value_to_json(v)))
                .collect();
            format!("{{{}}}", parts.join(","))
        }
        _ => "null".to_string(),
    }
}

fn json_to_value(json: &serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Nil,
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() { Value::Int(i) }
            else { Value::Float(n.as_f64().unwrap_or(0.0)) }
        }
        serde_json::Value::String(s) => Value::String(s.clone()),
        serde_json::Value::Array(arr) => {
            let items: Vec<Value> = arr.iter().map(|v| json_to_value(v)).collect();
            Value::List(Rc::new(RefCell::new(items)))
        }
        serde_json::Value::Object(obj) => {
            let mut map = HashMap::new();
            for (k, v) in obj {
                map.insert(k.clone(), json_to_value(v));
            }
            Value::Map(Rc::new(RefCell::new(map)))
        }
    }
}

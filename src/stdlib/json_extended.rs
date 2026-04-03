use std::rc::Rc;
use crate::interpreter::values::*;

pub fn register(env: &mut crate::interpreter::environment::Environment) {
    env.define("json_pretty", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "json_pretty".into(), arity: Some(1),
        func: Box::new(|args| {
            let val = args.first().unwrap_or(&Value::Nil);
            let json_str = value_to_json(val);
            // Parse and re-serialize with pretty printing
            match serde_json::from_str::<serde_json::Value>(&json_str) {
                Ok(parsed) => Ok(Value::String(serde_json::to_string_pretty(&parsed).unwrap_or(json_str))),
                Err(_) => Ok(Value::String(json_str)),
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
            format!("[{}]", parts.join(", "))
        }
        Value::Map(map) => {
            let parts: Vec<String> = map.borrow().iter()
                .map(|(k, v)| format!("\"{}\": {}", k, value_to_json(v)))
                .collect();
            format!("{{{}}}", parts.join(", "))
        }
        _ => "null".to_string(),
    }
}

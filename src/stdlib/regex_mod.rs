use std::rc::Rc;
use std::cell::RefCell;
use crate::interpreter::values::*;

pub fn register(env: &mut crate::interpreter::environment::Environment) {
    // regex_match(pattern, text) -> Bool
    // Returns true only if the *entire* text matches the pattern.
    env.define("regex_match", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "regex_match".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let pattern = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("regex_match: first arg must be a string pattern".into()),
            };
            let text = match args.get(1) {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("regex_match: second arg must be a string".into()),
            };
            let re = regex::Regex::new(&pattern)
                .map_err(|e| format!("regex_match: invalid pattern: {}", e))?;
            Ok(Value::Bool(re.is_match(&text) && re.find(&text).map(|m| m.start() == 0 && m.end() == text.len()).unwrap_or(false)))
        }),
    })));

    // regex_contains(pattern, text) -> Bool
    // Returns true if text contains at least one match.
    env.define("regex_contains", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "regex_contains".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let pattern = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("regex_contains: first arg must be a string pattern".into()),
            };
            let text = match args.get(1) {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("regex_contains: second arg must be a string".into()),
            };
            let re = regex::Regex::new(&pattern)
                .map_err(|e| format!("regex_contains: invalid pattern: {}", e))?;
            Ok(Value::Bool(re.is_match(&text)))
        }),
    })));

    // regex_find_all(pattern, text) -> List<Str>
    env.define("regex_find_all", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "regex_find_all".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let pattern = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("regex_find_all: first arg must be a string pattern".into()),
            };
            let text = match args.get(1) {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("regex_find_all: second arg must be a string".into()),
            };
            let re = regex::Regex::new(&pattern)
                .map_err(|e| format!("regex_find_all: invalid pattern: {}", e))?;
            let matches: Vec<Value> = re.find_iter(&text)
                .map(|m| Value::String(m.as_str().to_string()))
                .collect();
            Ok(Value::List(Rc::new(RefCell::new(matches))))
        }),
    })));

    // regex_replace(pattern, text, replacement) -> Str
    // Replaces the FIRST match.
    env.define("regex_replace", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "regex_replace".into(),
        arity: Some(3),
        func: Box::new(|args| {
            let pattern = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("regex_replace: first arg must be a string pattern".into()),
            };
            let text = match args.get(1) {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("regex_replace: second arg must be a string".into()),
            };
            let replacement = match args.get(2) {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("regex_replace: third arg must be a string".into()),
            };
            let re = regex::Regex::new(&pattern)
                .map_err(|e| format!("regex_replace: invalid pattern: {}", e))?;
            Ok(Value::String(re.replacen(&text, 1, replacement.as_str()).into_owned()))
        }),
    })));

    // regex_replace_all(pattern, text, replacement) -> Str
    env.define("regex_replace_all", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "regex_replace_all".into(),
        arity: Some(3),
        func: Box::new(|args| {
            let pattern = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("regex_replace_all: first arg must be a string pattern".into()),
            };
            let text = match args.get(1) {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("regex_replace_all: second arg must be a string".into()),
            };
            let replacement = match args.get(2) {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("regex_replace_all: third arg must be a string".into()),
            };
            let re = regex::Regex::new(&pattern)
                .map_err(|e| format!("regex_replace_all: invalid pattern: {}", e))?;
            Ok(Value::String(re.replace_all(&text, replacement.as_str()).into_owned()))
        }),
    })));

    // regex_split(pattern, text) -> List<Str>
    env.define("regex_split", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "regex_split".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let pattern = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("regex_split: first arg must be a string pattern".into()),
            };
            let text = match args.get(1) {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("regex_split: second arg must be a string".into()),
            };
            let re = regex::Regex::new(&pattern)
                .map_err(|e| format!("regex_split: invalid pattern: {}", e))?;
            let parts: Vec<Value> = re.split(&text)
                .map(|s| Value::String(s.to_string()))
                .collect();
            Ok(Value::List(Rc::new(RefCell::new(parts))))
        }),
    })));
}

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::interpreter::values::*;

pub fn register(env: &mut crate::interpreter::environment::Environment) {
    // http_get(url) -> Map {status, body, headers}
    env.define("http_get", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "http_get".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let url = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("http_get requires a URL string".into()),
            };
            match ureq::get(&url).call() {
                Ok(response) => Ok(build_response(response)),
                Err(e) => Ok(Value::Err(Box::new(Value::String(e.to_string())))),
            }
        }),
    })));

    // http_post(url, body) -> Map {status, body, headers}
    env.define("http_post", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "http_post".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let url = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("http_post requires a URL string as first arg".into()),
            };
            let body = match args.get(1) {
                Some(Value::String(s)) => s.clone(),
                Some(other) => other.to_string(),
                None => String::new(),
            };
            match ureq::post(&url).set("Content-Type", "application/json").send_string(&body) {
                Ok(response) => Ok(build_response(response)),
                Err(e) => Ok(Value::Err(Box::new(Value::String(e.to_string())))),
            }
        }),
    })));

    // http_put(url, body) -> Map
    env.define("http_put", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "http_put".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let url = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("http_put requires a URL string as first arg".into()),
            };
            let body = match args.get(1) {
                Some(Value::String(s)) => s.clone(),
                Some(other) => other.to_string(),
                None => String::new(),
            };
            match ureq::put(&url).set("Content-Type", "application/json").send_string(&body) {
                Ok(response) => Ok(build_response(response)),
                Err(e) => Ok(Value::Err(Box::new(Value::String(e.to_string())))),
            }
        }),
    })));

    // http_delete(url) -> Map
    env.define("http_delete", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "http_delete".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let url = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("http_delete requires a URL string".into()),
            };
            match ureq::delete(&url).call() {
                Ok(response) => Ok(build_response(response)),
                Err(e) => Ok(Value::Err(Box::new(Value::String(e.to_string())))),
            }
        }),
    })));
}

fn build_response(response: ureq::Response) -> Value {
    let status = response.status() as i64;

    // Collect headers before consuming the response
    let header_names = vec![
        "content-type", "content-length", "content-encoding",
        "cache-control", "server", "date", "location",
        "x-request-id", "etag", "last-modified", "vary",
        "access-control-allow-origin",
    ];
    let mut headers_map = HashMap::new();
    for h in &header_names {
        if let Some(v) = response.header(h) {
            headers_map.insert(h.to_string(), Value::String(v.to_string()));
        }
    }

    let body = response.into_string().unwrap_or_default();

    let mut map = HashMap::new();
    map.insert("status".to_string(), Value::Int(status));
    map.insert("body".to_string(), Value::String(body));
    map.insert("headers".to_string(), Value::Map(Rc::new(RefCell::new(headers_map))));
    Value::Map(Rc::new(RefCell::new(map)))
}

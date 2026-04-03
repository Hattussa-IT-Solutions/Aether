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

    // http_patch(url, body) -> Map
    env.define("http_patch", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "http_patch".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let url = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("http_patch requires a URL string as first arg".into()),
            };
            let body = match args.get(1) {
                Some(Value::String(s)) => s.clone(),
                Some(other) => other.to_string(),
                None => String::new(),
            };
            match ureq::patch(&url).set("Content-Type", "application/json").send_string(&body) {
                Ok(response) => Ok(build_response(response)),
                Err(e) => Ok(Value::Err(Box::new(Value::String(e.to_string())))),
            }
        }),
    })));

    // http_head(url) -> Map {status, headers}
    env.define("http_head", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "http_head".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let url = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("http_head requires a URL string".into()),
            };
            match ureq::head(&url).call() {
                Ok(response) => {
                    let status = response.status() as i64;
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
                    let mut map = HashMap::new();
                    map.insert("status".to_string(), Value::Int(status));
                    map.insert("body".to_string(), Value::String(String::new()));
                    map.insert("headers".to_string(), Value::Map(Rc::new(RefCell::new(headers_map))));
                    Ok(Value::Map(Rc::new(RefCell::new(map))))
                }
                Err(e) => Ok(Value::Err(Box::new(Value::String(e.to_string())))),
            }
        }),
    })));

    // http_request(options) -> Map
    // options: Map with keys: method, url, body, headers (Map), timeout (Int), auth (Map with user/pass)
    env.define("http_request", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "http_request".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let opts = match args.first() {
                Some(Value::Map(m)) => m.borrow().clone(),
                _ => return Err("http_request requires an options map".into()),
            };

            let method = match opts.get("method") {
                Some(Value::String(s)) => s.to_uppercase(),
                _ => "GET".to_string(),
            };
            let url = match opts.get("url") {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("http_request options must include 'url'".into()),
            };
            let body = match opts.get("body") {
                Some(Value::String(s)) => s.clone(),
                Some(other) => other.to_string(),
                None => String::new(),
            };

            // Build the request
            let mut req = match method.as_str() {
                "GET"    => ureq::get(&url),
                "POST"   => ureq::post(&url),
                "PUT"    => ureq::put(&url),
                "DELETE" => ureq::delete(&url),
                "PATCH"  => ureq::patch(&url),
                "HEAD"   => ureq::head(&url),
                other    => return Err(format!("http_request: unsupported method '{}'", other)),
            };

            // Apply custom headers
            if let Some(Value::Map(hmap)) = opts.get("headers") {
                for (k, v) in hmap.borrow().iter() {
                    req = req.set(k, &v.to_string());
                }
            }

            // Apply basic auth
            if let Some(Value::Map(auth)) = opts.get("auth") {
                let auth_b = auth.borrow();
                let user = auth_b.get("user").map(|v| v.to_string()).unwrap_or_default();
                let pass = auth_b.get("pass").map(|v| v.to_string()).unwrap_or_default();
                req = req.set("Authorization",
                    &format!("Basic {}", base64_encode(&format!("{}:{}", user, pass))));
            }

            // Apply timeout (milliseconds)
            // ureq 2.x timeout is set on the agent, not per-request; we skip it here
            // but we acknowledge the key for API completeness.

            let result = if body.is_empty() {
                req.call()
            } else {
                req.send_string(&body)
            };

            match result {
                Ok(response) => Ok(build_response(response)),
                Err(e) => Ok(Value::Err(Box::new(Value::String(e.to_string())))),
            }
        }),
    })));
}

/// Simple base64 encode (used for Basic auth).
fn base64_encode(input: &str) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = input.as_bytes();
    let mut out = String::new();
    let mut i = 0;
    while i < bytes.len() {
        let b0 = bytes[i] as u32;
        let b1 = if i + 1 < bytes.len() { bytes[i + 1] as u32 } else { 0 };
        let b2 = if i + 2 < bytes.len() { bytes[i + 2] as u32 } else { 0 };
        out.push(TABLE[((b0 >> 2) & 0x3F) as usize] as char);
        out.push(TABLE[(((b0 & 0x3) << 4) | (b1 >> 4)) as usize] as char);
        if i + 1 < bytes.len() { out.push(TABLE[(((b1 & 0xF) << 2) | (b2 >> 6)) as usize] as char); } else { out.push('='); }
        if i + 2 < bytes.len() { out.push(TABLE[(b2 & 0x3F) as usize] as char); } else { out.push('='); }
        i += 3;
    }
    out
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

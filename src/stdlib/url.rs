/// URL parsing and encoding utilities for Aether (pure Rust, no external crate).
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::interpreter::values::*;

pub fn register(env: &mut crate::interpreter::environment::Environment) {
    // url_parse(raw) -> Map { scheme, host, port, path, query, fragment }
    env.define("url_parse", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "url_parse".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let raw = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("url_parse: expected a URL string".into()),
            };
            let parsed = parse_url(&raw);
            Ok(Value::Map(Rc::new(RefCell::new(parsed))))
        }),
    })));

    // url_encode(text) -> percent-encoded string
    env.define("url_encode", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "url_encode".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let text = match args.first() {
                Some(Value::String(s)) => s.clone(),
                Some(other) => other.to_string(),
                None => return Err("url_encode: expected a string argument".into()),
            };
            Ok(Value::String(percent_encode(&text)))
        }),
    })));

    // url_decode(text) -> decoded string
    env.define("url_decode", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "url_decode".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let text = match args.first() {
                Some(Value::String(s)) => s.clone(),
                Some(other) => other.to_string(),
                None => return Err("url_decode: expected a string argument".into()),
            };
            Ok(Value::String(percent_decode(&text)))
        }),
    })));
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Parse a URL string into its components.
fn parse_url(raw: &str) -> HashMap<String, Value> {
    let mut map = HashMap::new();

    let mut rest = raw;

    // Fragment
    let fragment = if let Some(idx) = rest.rfind('#') {
        let frag = rest[idx + 1..].to_string();
        rest = &rest[..idx];
        frag
    } else {
        String::new()
    };

    // Query
    let query = if let Some(idx) = rest.find('?') {
        let q = rest[idx + 1..].to_string();
        rest = &rest[..idx];
        q
    } else {
        String::new()
    };

    // Scheme
    let scheme = if let Some(idx) = rest.find("://") {
        let s = rest[..idx].to_string();
        rest = &rest[idx + 3..];
        s
    } else {
        String::new()
    };

    // Authority (host[:port]) vs path
    let (authority, path) = if let Some(idx) = rest.find('/') {
        (&rest[..idx], rest[idx..].to_string())
    } else {
        (rest, String::new())
    };

    // Host and port
    let (host, port) = if let Some(idx) = authority.rfind(':') {
        let port_str = &authority[idx + 1..];
        if port_str.chars().all(|c| c.is_ascii_digit()) {
            let p = port_str.parse::<i64>().unwrap_or(0);
            (authority[..idx].to_string(), Value::Int(p))
        } else {
            (authority.to_string(), Value::Nil)
        }
    } else {
        (authority.to_string(), Value::Nil)
    };

    map.insert("scheme".to_string(),   Value::String(scheme));
    map.insert("host".to_string(),     Value::String(host));
    map.insert("port".to_string(),     port);
    map.insert("path".to_string(),     Value::String(if path.is_empty() { "/".to_string() } else { path }));
    map.insert("query".to_string(),    Value::String(query));
    map.insert("fragment".to_string(), Value::String(fragment));
    map
}

/// Percent-encode a string (encodes everything except unreserved characters).
fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for byte in s.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            out.push(byte as char);
        } else {
            out.push('%');
            out.push(hex_digit(byte >> 4));
            out.push(hex_digit(byte & 0xF));
        }
    }
    out
}

/// Percent-decode a string.
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (from_hex(bytes[i + 1]), from_hex(bytes[i + 2])) {
                out.push((h << 4) | l);
                i += 3;
                continue;
            }
        } else if bytes[i] == b'+' {
            out.push(b' ');
            i += 1;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

fn hex_digit(n: u8) -> char {
    (if n < 10 { b'0' + n } else { b'A' + n - 10 }) as char
}

fn from_hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

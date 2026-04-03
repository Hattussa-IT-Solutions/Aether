use std::rc::Rc;
use base64::Engine as _;
use crate::interpreter::values::*;

pub fn register(env: &mut crate::interpreter::environment::Environment) {
    // base64_encode(input) -> Str
    env.define("base64_encode", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "base64_encode".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let input = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("base64_encode: argument must be a string".into()),
            };
            Ok(Value::String(base64::engine::general_purpose::STANDARD.encode(input.as_bytes())))
        }),
    })));

    // base64_decode(input) -> Ok(Str) | Err(Str)
    env.define("base64_decode", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "base64_decode".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let input = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("base64_decode: argument must be a string".into()),
            };
            match base64::engine::general_purpose::STANDARD.decode(input.as_bytes()) {
                Ok(bytes) => match String::from_utf8(bytes) {
                    Ok(s) => Ok(Value::Ok(Box::new(Value::String(s)))),
                    Err(e) => Ok(Value::Err(Box::new(Value::String(format!("UTF-8 error: {}", e))))),
                },
                Err(e) => Ok(Value::Err(Box::new(Value::String(format!("base64 decode error: {}", e))))),
            }
        }),
    })));

    // hex_encode(input) -> Str
    env.define("hex_encode", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "hex_encode".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let input = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("hex_encode: argument must be a string".into()),
            };
            let hex: String = input.as_bytes().iter().map(|b| format!("{:02x}", b)).collect();
            Ok(Value::String(hex))
        }),
    })));

    // hex_decode(input) -> Ok(Str) | Err(Str)
    env.define("hex_decode", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "hex_decode".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let input = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("hex_decode: argument must be a string".into()),
            };
            // Strip optional "0x" prefix
            let hex_str = input.trim_start_matches("0x");
            if hex_str.len() % 2 != 0 {
                return Ok(Value::Err(Box::new(Value::String(
                    "hex_decode: odd-length hex string".to_string()
                ))));
            }
            let mut bytes = Vec::with_capacity(hex_str.len() / 2);
            for chunk in hex_str.as_bytes().chunks(2) {
                let hi = hex_nibble(chunk[0]);
                let lo = hex_nibble(chunk[1]);
                match (hi, lo) {
                    (Some(h), Some(l)) => bytes.push((h << 4) | l),
                    _ => return Ok(Value::Err(Box::new(Value::String(
                        format!("hex_decode: invalid hex character near '{}{}'",
                            chunk[0] as char, chunk[1] as char)
                    )))),
                }
            }
            match String::from_utf8(bytes) {
                Ok(s) => Ok(Value::Ok(Box::new(Value::String(s)))),
                Err(e) => Ok(Value::Err(Box::new(Value::String(format!("UTF-8 error: {}", e))))),
            }
        }),
    })));

    // url_encode(input) -> Str  (percent-encoding)
    env.define("url_encode", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "url_encode".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let input = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("url_encode: argument must be a string".into()),
            };
            let encoded: String = input.bytes().flat_map(|b| {
                // Unreserved chars per RFC 3986: ALPHA / DIGIT / "-" / "." / "_" / "~"
                if b.is_ascii_alphanumeric() || b == b'-' || b == b'.' || b == b'_' || b == b'~' {
                    vec![b as char]
                } else {
                    format!("%{:02X}", b).chars().collect::<Vec<_>>()
                }
            }).collect();
            Ok(Value::String(encoded))
        }),
    })));

    // url_decode(input) -> Str  (percent-decoding; invalid sequences left as-is)
    env.define("url_decode", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "url_decode".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let input = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("url_decode: argument must be a string".into()),
            };
            let mut bytes: Vec<u8> = Vec::with_capacity(input.len());
            let input_bytes = input.as_bytes();
            let mut i = 0;
            while i < input_bytes.len() {
                if input_bytes[i] == b'%' && i + 2 < input_bytes.len() {
                    if let (Some(h), Some(l)) = (hex_nibble(input_bytes[i+1]), hex_nibble(input_bytes[i+2])) {
                        bytes.push((h << 4) | l);
                        i += 3;
                        continue;
                    }
                }
                if input_bytes[i] == b'+' {
                    bytes.push(b' ');
                } else {
                    bytes.push(input_bytes[i]);
                }
                i += 1;
            }
            let decoded = String::from_utf8_lossy(&bytes).into_owned();
            Ok(Value::String(decoded))
        }),
    })));
}

/// Convert an ASCII hex character to its nibble value (0-15).
fn hex_nibble(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

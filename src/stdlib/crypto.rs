use std::rc::Rc;
use sha2::Digest;
use crate::interpreter::values::*;

/// Encode a byte slice as a lowercase hex string.
fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn register(env: &mut crate::interpreter::environment::Environment) {
    // crypto_sha256(input) -> Str
    env.define("crypto_sha256", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "crypto_sha256".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let input = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("crypto_sha256: argument must be a string".into()),
            };
            let mut hasher = sha2::Sha256::new();
            hasher.update(input.as_bytes());
            Ok(Value::String(to_hex(&hasher.finalize())))
        }),
    })));

    // crypto_sha512(input) -> Str
    env.define("crypto_sha512", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "crypto_sha512".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let input = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("crypto_sha512: argument must be a string".into()),
            };
            let mut hasher = sha2::Sha512::new();
            hasher.update(input.as_bytes());
            Ok(Value::String(to_hex(&hasher.finalize())))
        }),
    })));

    // crypto_md5(input) -> Str
    env.define("crypto_md5", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "crypto_md5".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let input = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("crypto_md5: argument must be a string".into()),
            };
            let mut hasher = md5::Md5::new();
            hasher.update(input.as_bytes());
            Ok(Value::String(to_hex(&hasher.finalize())))
        }),
    })));
}

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Mutex;
use crate::interpreter::values::*;

/// Xorshift64 PRNG state. Seeded from system time on first use.
static SEED: Mutex<u64> = Mutex::new(0);

fn xorshift64_next() -> u64 {
    let mut guard = SEED.lock().unwrap();
    if *guard == 0 {
        // Initialize from system time
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos() as u64;
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        *guard = (secs ^ (nanos << 32)).wrapping_add(0x9e3779b97f4a7c15);
        if *guard == 0 {
            *guard = 0xdeadbeef12345678;
        }
    }
    let mut x = *guard;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *guard = x;
    x
}

/// Return a float in [0.0, 1.0)
fn rand_float() -> f64 {
    (xorshift64_next() >> 11) as f64 / (1u64 << 53) as f64
}

pub fn register(env: &mut crate::interpreter::environment::Environment) {
    // random_int(min, max) -> Int  (inclusive on both ends)
    env.define("random_int", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "random_int".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let min = match args.first() {
                Some(v) => v.as_int().ok_or_else(|| "random_int: first arg must be int".to_string())?,
                None => return Err("random_int requires 2 args".into()),
            };
            let max = match args.get(1) {
                Some(v) => v.as_int().ok_or_else(|| "random_int: second arg must be int".to_string())?,
                None => return Err("random_int requires 2 args".into()),
            };
            if min > max {
                return Err(format!("random_int: min ({}) > max ({})", min, max));
            }
            let range = (max - min + 1) as u64;
            let r = (xorshift64_next() % range) as i64 + min;
            Ok(Value::Int(r))
        }),
    })));

    // random_float() -> Float in [0.0, 1.0)
    env.define("random_float", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "random_float".into(),
        arity: Some(0),
        func: Box::new(|_| Ok(Value::Float(rand_float()))),
    })));

    // random_float_range(min, max) -> Float
    env.define("random_float_range", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "random_float_range".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let min = args.first().and_then(|v| v.as_float())
                .ok_or_else(|| "random_float_range: first arg must be numeric".to_string())?;
            let max = args.get(1).and_then(|v| v.as_float())
                .ok_or_else(|| "random_float_range: second arg must be numeric".to_string())?;
            Ok(Value::Float(min + rand_float() * (max - min)))
        }),
    })));

    // random_bool() -> Bool
    env.define("random_bool", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "random_bool".into(),
        arity: Some(0),
        func: Box::new(|_| Ok(Value::Bool(xorshift64_next() & 1 == 1))),
    })));

    // random_choice(list) -> Value
    env.define("random_choice", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "random_choice".into(),
        arity: Some(1),
        func: Box::new(|args| {
            match args.first() {
                Some(Value::List(lst)) => {
                    let lst = lst.borrow();
                    if lst.is_empty() {
                        return Err("random_choice: list is empty".into());
                    }
                    let idx = (xorshift64_next() as usize) % lst.len();
                    Ok(lst[idx].clone())
                }
                _ => Err("random_choice: argument must be a list".into()),
            }
        }),
    })));

    // random_shuffle(list) -> List (returns a new shuffled list)
    env.define("random_shuffle", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "random_shuffle".into(),
        arity: Some(1),
        func: Box::new(|args| {
            match args.first() {
                Some(Value::List(lst)) => {
                    let mut items: Vec<Value> = lst.borrow().clone();
                    // Fisher-Yates shuffle
                    let n = items.len();
                    for i in (1..n).rev() {
                        let j = (xorshift64_next() as usize) % (i + 1);
                        items.swap(i, j);
                    }
                    Ok(Value::List(Rc::new(RefCell::new(items))))
                }
                _ => Err("random_shuffle: argument must be a list".into()),
            }
        }),
    })));

    // random_sample(list, n) -> List (n unique random elements)
    env.define("random_sample", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "random_sample".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let lst = match args.first() {
                Some(Value::List(l)) => l.borrow().clone(),
                _ => return Err("random_sample: first arg must be a list".into()),
            };
            let n = match args.get(1) {
                Some(v) => v.as_int().ok_or_else(|| "random_sample: second arg must be int".to_string())? as usize,
                None => return Err("random_sample requires 2 args".into()),
            };
            if n > lst.len() {
                return Err(format!("random_sample: n ({}) > list length ({})", n, lst.len()));
            }
            let mut indices: Vec<usize> = (0..lst.len()).collect();
            // Partial Fisher-Yates to get n elements
            for i in 0..n {
                let j = i + (xorshift64_next() as usize) % (lst.len() - i);
                indices.swap(i, j);
            }
            let sample: Vec<Value> = indices[..n].iter().map(|&i| lst[i].clone()).collect();
            Ok(Value::List(Rc::new(RefCell::new(sample))))
        }),
    })));

    // random_hex(length) -> Str
    env.define("random_hex", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "random_hex".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let len = match args.first() {
                Some(v) => v.as_int().ok_or_else(|| "random_hex: arg must be int".to_string())? as usize,
                None => return Err("random_hex requires 1 arg".into()),
            };
            let mut result = String::with_capacity(len);
            let hex_chars = b"0123456789abcdef";
            let mut remaining = len;
            while remaining > 0 {
                let r = xorshift64_next();
                for byte_idx in 0..8 {
                    if remaining == 0 { break; }
                    let nibble = ((r >> (byte_idx * 8)) & 0xf) as usize;
                    result.push(hex_chars[nibble] as char);
                    remaining -= 1;
                }
            }
            Ok(Value::String(result))
        }),
    })));

    // random_uuid() -> Str (UUID v4 via uuid crate)
    env.define("random_uuid", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "random_uuid".into(),
        arity: Some(0),
        func: Box::new(|_| {
            Ok(Value::String(uuid::Uuid::new_v4().to_string()))
        }),
    })));

    // random_seed(n) — set the PRNG seed
    env.define("random_seed", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "random_seed".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let n = match args.first() {
                Some(v) => v.as_int().ok_or_else(|| "random_seed: arg must be int".to_string())? as u64,
                None => return Err("random_seed requires 1 arg".into()),
            };
            let seed = if n == 0 { 0xdeadbeef12345678 } else { n };
            *SEED.lock().unwrap() = seed;
            Ok(Value::Nil)
        }),
    })));

    // Also expose a map of all functions under "random" namespace for convenience
    let _ = HashMap::<String, Value>::new(); // suppress unused import warning
}

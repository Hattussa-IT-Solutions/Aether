use std::rc::Rc;
use crate::interpreter::values::*;

/// Register extended math functions.
pub fn register_math(env: &mut crate::interpreter::environment::Environment) {
    let fns: Vec<(&str, Box<dyn Fn(Vec<Value>) -> Result<Value, String>>)> = vec![
        ("asin", Box::new(|a| Ok(Value::Float(a[0].as_float().unwrap_or(0.0).asin())))),
        ("acos", Box::new(|a| Ok(Value::Float(a[0].as_float().unwrap_or(0.0).acos())))),
        ("atan", Box::new(|a| Ok(Value::Float(a[0].as_float().unwrap_or(0.0).atan())))),
        ("atan2", Box::new(|a| {
            let y = a[0].as_float().unwrap_or(0.0);
            let x = a.get(1).and_then(|v| v.as_float()).unwrap_or(1.0);
            Ok(Value::Float(y.atan2(x)))
        })),
        ("cbrt", Box::new(|a| Ok(Value::Float(a[0].as_float().unwrap_or(0.0).cbrt())))),
        ("exp", Box::new(|a| Ok(Value::Float(a[0].as_float().unwrap_or(0.0).exp())))),
        ("clamp", Box::new(|a| {
            let val = a[0].as_float().unwrap_or(0.0);
            let lo = a.get(1).and_then(|v| v.as_float()).unwrap_or(f64::MIN);
            let hi = a.get(2).and_then(|v| v.as_float()).unwrap_or(f64::MAX);
            Ok(Value::Float(val.max(lo).min(hi)))
        })),
        ("trunc", Box::new(|a| Ok(Value::Int(a[0].as_float().unwrap_or(0.0).trunc() as i64)))),
        ("random", Box::new(|a| {
            // Simple LCG random — not cryptographic
            use std::time::SystemTime;
            let seed = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default().subsec_nanos() as f64;
            if a.is_empty() {
                Ok(Value::Float((seed / 4294967296.0).fract()))
            } else {
                let lo = a[0].as_float().unwrap_or(0.0);
                let hi = a.get(1).and_then(|v| v.as_float()).unwrap_or(1.0);
                let r = (seed / 4294967296.0).fract();
                Ok(Value::Float(lo + r * (hi - lo)))
            }
        })),
    ];

    for (name, func) in fns {
        env.define(name, Value::NativeFunction(Rc::new(NativeFunctionValue {
            name: name.to_string(),
            arity: None,
            func,
        })));
    }
}

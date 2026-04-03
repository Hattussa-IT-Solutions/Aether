use std::rc::Rc;
use crate::interpreter::values::*;

pub fn register(env: &mut crate::interpreter::environment::Environment) {
    let fns: Vec<(&str, Box<dyn Fn(Vec<Value>) -> Result<Value, String>>)> = vec![
        ("math_gcd", Box::new(|a| {
            let mut x = a[0].as_int().unwrap_or(0).abs();
            let mut y = a.get(1).and_then(|v| v.as_int()).unwrap_or(0).abs();
            while y != 0 { let t = y; y = x % y; x = t; }
            Ok(Value::Int(x))
        })),
        ("math_lcm", Box::new(|a| {
            let x = a[0].as_int().unwrap_or(0).abs();
            let y = a.get(1).and_then(|v| v.as_int()).unwrap_or(0).abs();
            if x == 0 || y == 0 { return Ok(Value::Int(0)); }
            let mut gx = x; let mut gy = y;
            while gy != 0 { let t = gy; gy = gx % gy; gx = t; }
            Ok(Value::Int(x / gx * y))
        })),
        ("math_factorial", Box::new(|a| {
            let n = a[0].as_int().unwrap_or(0);
            if n < 0 { return Err("factorial of negative".into()); }
            let mut result: i64 = 1;
            for i in 2..=n { result = result.saturating_mul(i); }
            Ok(Value::Int(result))
        })),
        ("math_fib", Box::new(|a| {
            let n = a[0].as_int().unwrap_or(0);
            if n <= 0 { return Ok(Value::Int(0)); }
            if n == 1 { return Ok(Value::Int(1)); }
            let (mut a, mut b) = (0i64, 1i64);
            for _ in 2..=n { let t = a + b; a = b; b = t; }
            Ok(Value::Int(b))
        })),
        ("math_is_prime", Box::new(|a| {
            let n = a[0].as_int().unwrap_or(0);
            if n < 2 { return Ok(Value::Bool(false)); }
            if n < 4 { return Ok(Value::Bool(true)); }
            if n % 2 == 0 || n % 3 == 0 { return Ok(Value::Bool(false)); }
            let mut i = 5;
            while i * i <= n { if n % i == 0 || n % (i+2) == 0 { return Ok(Value::Bool(false)); } i += 6; }
            Ok(Value::Bool(true))
        })),
        ("math_sign", Box::new(|a| {
            let x = a[0].as_float().unwrap_or(0.0);
            Ok(Value::Int(if x > 0.0 { 1 } else if x < 0.0 { -1 } else { 0 }))
        })),
        ("math_lerp", Box::new(|a| {
            let s = a[0].as_float().unwrap_or(0.0);
            let e = a.get(1).and_then(|v| v.as_float()).unwrap_or(1.0);
            let t = a.get(2).and_then(|v| v.as_float()).unwrap_or(0.5);
            Ok(Value::Float(s + (e - s) * t))
        })),
        ("math_deg_to_rad", Box::new(|a| Ok(Value::Float(a[0].as_float().unwrap_or(0.0).to_radians())))),
        ("math_rad_to_deg", Box::new(|a| Ok(Value::Float(a[0].as_float().unwrap_or(0.0).to_degrees())))),
        ("math_is_nan", Box::new(|a| Ok(Value::Bool(a[0].as_float().unwrap_or(0.0).is_nan())))),
        ("math_is_inf", Box::new(|a| Ok(Value::Bool(a[0].as_float().unwrap_or(0.0).is_infinite())))),
        ("math_round_to", Box::new(|a| {
            let x = a[0].as_float().unwrap_or(0.0);
            let d = a.get(1).and_then(|v| v.as_int()).unwrap_or(0) as i32;
            let factor = 10f64.powi(d);
            Ok(Value::Float((x * factor).round() / factor))
        })),
        ("math_sinh", Box::new(|a| Ok(Value::Float(a[0].as_float().unwrap_or(0.0).sinh())))),
        ("math_cosh", Box::new(|a| Ok(Value::Float(a[0].as_float().unwrap_or(0.0).cosh())))),
        ("math_tanh", Box::new(|a| Ok(Value::Float(a[0].as_float().unwrap_or(0.0).tanh())))),
    ];
    for (name, func) in fns {
        env.define(name, Value::NativeFunction(Rc::new(NativeFunctionValue {
            name: name.to_string(), arity: None, func,
        })));
    }
}

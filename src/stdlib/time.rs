use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH, Instant};
use crate::interpreter::values::*;

/// Register time functions.
pub fn register_time(env: &mut crate::interpreter::environment::Environment) {
    // time_now() -> Float (seconds since epoch)
    env.define("time_now", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "time_now".into(),
        arity: Some(0),
        func: Box::new(|_| {
            let dur = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
            Ok(Value::Float(dur.as_secs_f64()))
        }),
    })));

    // time_sleep(seconds: Float)
    env.define("time_sleep", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "time_sleep".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let secs = args.first().and_then(|v| v.as_float()).unwrap_or(0.0);
            std::thread::sleep(std::time::Duration::from_secs_f64(secs));
            Ok(Value::Nil)
        }),
    })));

    // time_millis() -> Int (milliseconds since epoch)
    env.define("time_millis", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "time_millis".into(),
        arity: Some(0),
        func: Box::new(|_| {
            let dur = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
            Ok(Value::Int(dur.as_millis() as i64))
        }),
    })));
}

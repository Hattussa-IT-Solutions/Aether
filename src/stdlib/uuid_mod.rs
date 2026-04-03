use std::rc::Rc;
use crate::interpreter::values::*;

pub fn register(env: &mut crate::interpreter::environment::Environment) {
    // uuid_v4() -> Str
    env.define("uuid_v4", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "uuid_v4".into(),
        arity: Some(0),
        func: Box::new(|_| {
            Ok(Value::String(uuid::Uuid::new_v4().to_string()))
        }),
    })));

    // uuid_is_valid(str) -> Bool
    env.define("uuid_is_valid", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "uuid_is_valid".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let s = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("uuid_is_valid: argument must be a string".into()),
            };
            Ok(Value::Bool(uuid::Uuid::parse_str(&s).is_ok()))
        }),
    })));
}

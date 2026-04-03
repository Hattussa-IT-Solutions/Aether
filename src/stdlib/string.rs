use std::cell::RefCell;
use std::rc::Rc;
use crate::interpreter::values::*;

/// Register all Str methods as native functions.
pub fn register_string_methods(env: &mut crate::interpreter::environment::Environment) {
    env.define("Str", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "Str".into(),
        arity: Some(1),
        func: Box::new(|args| {
            Ok(Value::String(args.first().map(|v| v.to_string()).unwrap_or_default()))
        }),
    })));

    env.define("join", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "join".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let sep = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("join requires a string separator".into()),
            };
            let list = match args.get(1) {
                Some(Value::List(l)) => l.borrow().iter().map(|v| v.to_string()).collect::<Vec<_>>(),
                _ => return Err("join requires a list".into()),
            };
            Ok(Value::String(list.join(&sep)))
        }),
    })));
}

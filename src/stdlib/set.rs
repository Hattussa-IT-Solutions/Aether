// Set operations are handled as built-in methods in eval.rs
// This module provides additional set utility functions.

use std::cell::RefCell;
use std::rc::Rc;
use crate::interpreter::values::*;

pub fn register_set_methods(env: &mut crate::interpreter::environment::Environment) {
    // set_from(list) -> Set
    env.define("set_from", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "set_from".into(),
        arity: Some(1),
        func: Box::new(|args| {
            match args.first() {
                Some(Value::List(items)) => {
                    let items = items.borrow();
                    let mut set = Vec::new();
                    for item in items.iter() {
                        if !set.iter().any(|v: &Value| v.equals(item)) {
                            set.push(item.clone());
                        }
                    }
                    Ok(Value::Set(Rc::new(RefCell::new(set))))
                }
                _ => Err("set_from requires a list".into()),
            }
        }),
    })));
}

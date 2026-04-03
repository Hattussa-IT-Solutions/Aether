// List operations are handled as built-in methods in eval.rs
// This module provides additional list utility functions.

use std::cell::RefCell;
use std::rc::Rc;
use crate::interpreter::values::*;

pub fn register_list_methods(env: &mut crate::interpreter::environment::Environment) {
    // list_from(iterable) -> List
    env.define("list_from", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "list_from".into(),
        arity: Some(1),
        func: Box::new(|args| {
            match args.first() {
                Some(Value::Range { start, end, inclusive, step }) => {
                    let mut items = Vec::new();
                    let mut i = *start;
                    let s = *step;
                    if s > 0 {
                        while if *inclusive { i <= *end } else { i < *end } {
                            items.push(Value::Int(i));
                            i += s;
                        }
                    }
                    Ok(Value::List(Rc::new(RefCell::new(items))))
                }
                Some(Value::Set(s)) => Ok(Value::List(Rc::new(RefCell::new(s.borrow().clone())))),
                Some(Value::List(l)) => Ok(Value::List(Rc::new(RefCell::new(l.borrow().clone())))),
                _ => Err("list_from requires an iterable".into()),
            }
        }),
    })));
}

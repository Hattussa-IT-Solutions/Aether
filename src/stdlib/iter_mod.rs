use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::interpreter::values::*;

/// Helper: call a Value::Function or Value::NativeFunction with one argument.
fn call_fn_1(func: &Value, arg: Value) -> Result<Value, String> {
    match func {
        Value::NativeFunction(nf) => (nf.func)(vec![arg]),
        Value::Function(_) => {
            // Call via the interpreter's call_function
            crate::interpreter::eval::call_function(
                func,
                vec![arg],
                &[],
                &mut crate::interpreter::environment::Environment::new(),
            )
            .map_err(|e| match e {
                crate::interpreter::values::Signal::Throw(v) => v.to_string(),
                crate::interpreter::values::Signal::Return(v) => v.to_string(),
                _ => "function error".to_string(),
            })
        }
        _ => Err("expected a function".into()),
    }
}

pub fn register(env: &mut crate::interpreter::environment::Environment) {
    // iter_chain(list1, list2) -> List
    env.define("iter_chain", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "iter_chain".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let list1 = match args.first() {
                Some(Value::List(l)) => l.borrow().clone(),
                _ => return Err("iter_chain requires list as first arg".into()),
            };
            let list2 = match args.get(1) {
                Some(Value::List(l)) => l.borrow().clone(),
                _ => return Err("iter_chain requires list as second arg".into()),
            };
            let mut result = list1;
            result.extend(list2);
            Ok(Value::List(Rc::new(RefCell::new(result))))
        }),
    })));

    // iter_zip_longest(list1, list2, fill) -> List of Tuples
    env.define("iter_zip_longest", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "iter_zip_longest".into(),
        arity: Some(3),
        func: Box::new(|args| {
            let list1 = match args.first() {
                Some(Value::List(l)) => l.borrow().clone(),
                _ => return Err("iter_zip_longest requires list as first arg".into()),
            };
            let list2 = match args.get(1) {
                Some(Value::List(l)) => l.borrow().clone(),
                _ => return Err("iter_zip_longest requires list as second arg".into()),
            };
            let fill = args.get(2).cloned().unwrap_or(Value::Nil);
            let len = list1.len().max(list2.len());
            let result: Vec<Value> = (0..len).map(|i| {
                let a = list1.get(i).cloned().unwrap_or_else(|| fill.clone());
                let b = list2.get(i).cloned().unwrap_or_else(|| fill.clone());
                Value::Tuple(vec![a, b])
            }).collect();
            Ok(Value::List(Rc::new(RefCell::new(result))))
        }),
    })));

    // iter_window(list, size) -> List of Lists (sliding window)
    env.define("iter_window", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "iter_window".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let list = match args.first() {
                Some(Value::List(l)) => l.borrow().clone(),
                _ => return Err("iter_window requires a list as first arg".into()),
            };
            let size = args.get(1).and_then(|v| v.as_int()).unwrap_or(2) as usize;
            if size == 0 {
                return Ok(Value::List(Rc::new(RefCell::new(vec![]))));
            }
            let result: Vec<Value> = list.windows(size)
                .map(|w| Value::List(Rc::new(RefCell::new(w.to_vec()))))
                .collect();
            Ok(Value::List(Rc::new(RefCell::new(result))))
        }),
    })));

    // iter_flatten(nested_list) -> List
    env.define("iter_flatten", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "iter_flatten".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let list = match args.first() {
                Some(Value::List(l)) => l.borrow().clone(),
                _ => return Err("iter_flatten requires a list".into()),
            };
            let mut result = Vec::new();
            for item in list {
                match item {
                    Value::List(inner) => result.extend(inner.borrow().clone()),
                    other => result.push(other),
                }
            }
            Ok(Value::List(Rc::new(RefCell::new(result))))
        }),
    })));

    // iter_repeat(value, n) -> List
    env.define("iter_repeat", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "iter_repeat".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let value = args.first().cloned().unwrap_or(Value::Nil);
            let n = args.get(1).and_then(|v| v.as_int()).unwrap_or(0).max(0) as usize;
            let result: Vec<Value> = (0..n).map(|_| value.clone()).collect();
            Ok(Value::List(Rc::new(RefCell::new(result))))
        }),
    })));

    // iter_product(list1, list2) -> List of Tuples (cartesian product)
    env.define("iter_product", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "iter_product".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let list1 = match args.first() {
                Some(Value::List(l)) => l.borrow().clone(),
                _ => return Err("iter_product requires list as first arg".into()),
            };
            let list2 = match args.get(1) {
                Some(Value::List(l)) => l.borrow().clone(),
                _ => return Err("iter_product requires list as second arg".into()),
            };
            let mut result = Vec::new();
            for a in &list1 {
                for b in &list2 {
                    result.push(Value::Tuple(vec![a.clone(), b.clone()]));
                }
            }
            Ok(Value::List(Rc::new(RefCell::new(result))))
        }),
    })));

    // iter_take_while(list, fn) -> List
    env.define("iter_take_while", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "iter_take_while".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let list = match args.first() {
                Some(Value::List(l)) => l.borrow().clone(),
                _ => return Err("iter_take_while requires a list as first arg".into()),
            };
            let func = match args.get(1) {
                Some(f @ Value::Function(_)) | Some(f @ Value::NativeFunction(_)) => f.clone(),
                _ => return Err("iter_take_while requires a function as second arg".into()),
            };
            let mut result = Vec::new();
            for item in list {
                match call_fn_1(&func, item.clone()) {
                    Ok(v) if v.is_truthy() => result.push(item),
                    Ok(_) => break,
                    Err(e) => return Err(e),
                }
            }
            Ok(Value::List(Rc::new(RefCell::new(result))))
        }),
    })));

    // iter_drop_while(list, fn) -> List
    env.define("iter_drop_while", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "iter_drop_while".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let list = match args.first() {
                Some(Value::List(l)) => l.borrow().clone(),
                _ => return Err("iter_drop_while requires a list as first arg".into()),
            };
            let func = match args.get(1) {
                Some(f @ Value::Function(_)) | Some(f @ Value::NativeFunction(_)) => f.clone(),
                _ => return Err("iter_drop_while requires a function as second arg".into()),
            };
            let mut dropping = true;
            let mut result = Vec::new();
            for item in list {
                if dropping {
                    match call_fn_1(&func, item.clone()) {
                        Ok(v) if v.is_truthy() => {} // still dropping
                        Ok(_) => {
                            dropping = false;
                            result.push(item);
                        }
                        Err(e) => return Err(e),
                    }
                } else {
                    result.push(item);
                }
            }
            Ok(Value::List(Rc::new(RefCell::new(result))))
        }),
    })));

    // iter_group_by(list, fn) -> Map (key -> list)
    env.define("iter_group_by", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "iter_group_by".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let list = match args.first() {
                Some(Value::List(l)) => l.borrow().clone(),
                _ => return Err("iter_group_by requires a list as first arg".into()),
            };
            let func = match args.get(1) {
                Some(f @ Value::Function(_)) | Some(f @ Value::NativeFunction(_)) => f.clone(),
                _ => return Err("iter_group_by requires a function as second arg".into()),
            };

            let mut groups: HashMap<String, Vec<Value>> = HashMap::new();
            let mut key_order: Vec<String> = Vec::new();

            for item in list {
                let key_val = call_fn_1(&func, item.clone())?;
                let key_str = match &key_val {
                    Value::String(s) => s.clone(),
                    Value::Int(n) => n.to_string(),
                    Value::Float(f) => f.to_string(),
                    Value::Bool(b) => b.to_string(),
                    _ => key_val.to_string(),
                };
                if !groups.contains_key(&key_str) {
                    key_order.push(key_str.clone());
                }
                groups.entry(key_str).or_default().push(item);
            }

            let mut result_map = HashMap::new();
            for key in key_order {
                if let Some(group) = groups.remove(&key) {
                    result_map.insert(key, Value::List(Rc::new(RefCell::new(group))));
                }
            }

            Ok(Value::Map(Rc::new(RefCell::new(result_map))))
        }),
    })));
}

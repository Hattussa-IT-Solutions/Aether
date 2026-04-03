use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::interpreter::values::*;

pub fn register(env: &mut crate::interpreter::environment::Environment) {
    // counter(list) -> Map — count occurrences
    env.define("counter", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "counter".into(),
        arity: Some(1),
        func: Box::new(|args| {
            match args.first() {
                Some(Value::List(list)) => {
                    let mut counts: HashMap<String, Value> = HashMap::new();
                    for item in list.borrow().iter() {
                        let key = match item {
                            Value::String(s) => s.clone(),
                            Value::Int(n) => n.to_string(),
                            Value::Float(f) => f.to_string(),
                            Value::Bool(b) => b.to_string(),
                            _ => item.to_string(),
                        };
                        let entry = counts.entry(key).or_insert(Value::Int(0));
                        if let Value::Int(n) = entry {
                            *n += 1;
                        }
                    }
                    Ok(Value::Map(Rc::new(RefCell::new(counts))))
                }
                _ => Err("counter requires a list".into()),
            }
        }),
    })));

    // counter_most_common(counter_map, n) -> List of [key, count] tuples
    env.define("counter_most_common", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "counter_most_common".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let map = match args.first() {
                Some(Value::Map(m)) => m.clone(),
                _ => return Err("counter_most_common requires a map as first arg".into()),
            };
            let n = args.get(1).and_then(|v| v.as_int()).unwrap_or(i64::MAX);

            let mut pairs: Vec<(String, i64)> = map.borrow().iter()
                .filter_map(|(k, v)| {
                    if let Value::Int(count) = v {
                        Some((k.clone(), *count))
                    } else {
                        None
                    }
                })
                .collect();

            // Sort by count descending, then by key ascending for ties
            pairs.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

            let result: Vec<Value> = pairs.into_iter()
                .take(n.max(0) as usize)
                .map(|(k, count)| {
                    Value::Tuple(vec![Value::String(k), Value::Int(count)])
                })
                .collect();

            Ok(Value::List(Rc::new(RefCell::new(result))))
        }),
    })));

    // deque_new() -> Map with __type: "Deque", items: List
    env.define("deque_new", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "deque_new".into(),
        arity: Some(0),
        func: Box::new(|_| {
            let mut map = HashMap::new();
            map.insert("__type".to_string(), Value::String("Deque".to_string()));
            map.insert("items".to_string(), Value::List(Rc::new(RefCell::new(vec![]))));
            Ok(Value::Map(Rc::new(RefCell::new(map))))
        }),
    })));

    // deque_push_back(deque, value) — append to end
    env.define("deque_push_back", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "deque_push_back".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let deque = match args.first() {
                Some(Value::Map(m)) => m.clone(),
                _ => return Err("deque_push_back requires a deque map as first arg".into()),
            };
            let value = args.get(1).cloned().unwrap_or(Value::Nil);
            let items = deque.borrow().get("items").cloned();
            if let Some(Value::List(list)) = items {
                list.borrow_mut().push(value);
                Ok(Value::Nil)
            } else {
                Err("deque has no items list".into())
            }
        }),
    })));

    // deque_push_front(deque, value) — prepend to front
    env.define("deque_push_front", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "deque_push_front".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let deque = match args.first() {
                Some(Value::Map(m)) => m.clone(),
                _ => return Err("deque_push_front requires a deque map as first arg".into()),
            };
            let value = args.get(1).cloned().unwrap_or(Value::Nil);
            let items = deque.borrow().get("items").cloned();
            if let Some(Value::List(list)) = items {
                list.borrow_mut().insert(0, value);
                Ok(Value::Nil)
            } else {
                Err("deque has no items list".into())
            }
        }),
    })));

    // deque_pop_back(deque) -> value
    env.define("deque_pop_back", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "deque_pop_back".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let deque = match args.first() {
                Some(Value::Map(m)) => m.clone(),
                _ => return Err("deque_pop_back requires a deque map".into()),
            };
            let items = deque.borrow().get("items").cloned();
            if let Some(Value::List(list)) = items {
                Ok(list.borrow_mut().pop().unwrap_or(Value::Nil))
            } else {
                Err("deque has no items list".into())
            }
        }),
    })));

    // deque_pop_front(deque) -> value
    env.define("deque_pop_front", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "deque_pop_front".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let deque = match args.first() {
                Some(Value::Map(m)) => m.clone(),
                _ => return Err("deque_pop_front requires a deque map".into()),
            };
            let items = deque.borrow().get("items").cloned();
            if let Some(Value::List(list)) = items {
                let mut borrowed = list.borrow_mut();
                if borrowed.is_empty() {
                    Ok(Value::Nil)
                } else {
                    Ok(borrowed.remove(0))
                }
            } else {
                Err("deque has no items list".into())
            }
        }),
    })));

    // deque_len(deque) -> Int
    env.define("deque_len", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "deque_len".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let deque = match args.first() {
                Some(Value::Map(m)) => m.clone(),
                _ => return Err("deque_len requires a deque map".into()),
            };
            let items = deque.borrow().get("items").cloned();
            if let Some(Value::List(list)) = items {
                Ok(Value::Int(list.borrow().len() as i64))
            } else {
                Ok(Value::Int(0))
            }
        }),
    })));

    // stack_new() -> Map with __type: "Stack", items: List
    env.define("stack_new", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "stack_new".into(),
        arity: Some(0),
        func: Box::new(|_| {
            let mut map = HashMap::new();
            map.insert("__type".to_string(), Value::String("Stack".to_string()));
            map.insert("items".to_string(), Value::List(Rc::new(RefCell::new(vec![]))));
            Ok(Value::Map(Rc::new(RefCell::new(map))))
        }),
    })));

    // stack_push(stack, value)
    env.define("stack_push", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "stack_push".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let stack = match args.first() {
                Some(Value::Map(m)) => m.clone(),
                _ => return Err("stack_push requires a stack map as first arg".into()),
            };
            let value = args.get(1).cloned().unwrap_or(Value::Nil);
            let items = stack.borrow().get("items").cloned();
            if let Some(Value::List(list)) = items {
                list.borrow_mut().push(value);
                Ok(Value::Nil)
            } else {
                Err("stack has no items list".into())
            }
        }),
    })));

    // stack_pop(stack) -> value
    env.define("stack_pop", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "stack_pop".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let stack = match args.first() {
                Some(Value::Map(m)) => m.clone(),
                _ => return Err("stack_pop requires a stack map".into()),
            };
            let items = stack.borrow().get("items").cloned();
            if let Some(Value::List(list)) = items {
                Ok(list.borrow_mut().pop().unwrap_or(Value::Nil))
            } else {
                Err("stack has no items list".into())
            }
        }),
    })));

    // stack_peek(stack) -> value (top without removing)
    env.define("stack_peek", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "stack_peek".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let stack = match args.first() {
                Some(Value::Map(m)) => m.clone(),
                _ => return Err("stack_peek requires a stack map".into()),
            };
            let items = stack.borrow().get("items").cloned();
            if let Some(Value::List(list)) = items {
                Ok(list.borrow().last().cloned().unwrap_or(Value::Nil))
            } else {
                Err("stack has no items list".into())
            }
        }),
    })));

    // stack_len(stack) -> Int
    env.define("stack_len", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "stack_len".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let stack = match args.first() {
                Some(Value::Map(m)) => m.clone(),
                _ => return Err("stack_len requires a stack map".into()),
            };
            let items = stack.borrow().get("items").cloned();
            if let Some(Value::List(list)) = items {
                Ok(Value::Int(list.borrow().len() as i64))
            } else {
                Ok(Value::Int(0))
            }
        }),
    })));
}

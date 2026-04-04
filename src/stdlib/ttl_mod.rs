use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use crate::interpreter::values::*;

// ── Static registry ───────────────────────────────────────────────
// Each entry: (serialized_value: String, created_at: Instant, ttl: Duration)
// Value contains Rc<> which is !Send, so we serialize to String for static storage.

#[allow(clippy::type_complexity)]
static TTL_STORE: Mutex<Option<HashMap<String, (String, Instant, Duration)>>> = Mutex::new(None);

// ── MAP_IDS registry for cleanup tracking ────────────────────────

static MAP_IDS: Mutex<Option<Vec<String>>> = Mutex::new(None);

fn new_map_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    format!("ttlmap-{}", COUNTER.fetch_add(1, Ordering::SeqCst))
}

// ── Serialization helpers ─────────────────────────────────────────

fn value_to_str(val: &Value) -> String {
    match val {
        Value::Int(n) => format!("i:{}", n),
        Value::Float(f) => format!("f:{}", f),
        Value::Bool(b) => format!("b:{}", b),
        Value::String(s) => {
            let escaped = s.replace('\\', "\\\\").replace('\n', "\\n");
            format!("s:{}", escaped)
        }
        Value::Nil => "n:".to_string(),
        _ => format!("s:{}", val),
    }
}

fn str_to_value(s: &str) -> Value {
    if let Some(rest) = s.strip_prefix("i:") {
        if let Ok(n) = rest.parse::<i64>() { return Value::Int(n); }
    } else if let Some(rest) = s.strip_prefix("f:") {
        if let Ok(f) = rest.parse::<f64>() { return Value::Float(f); }
    } else if let Some(rest) = s.strip_prefix("b:") {
        match rest {
            "true" => return Value::Bool(true),
            "false" => return Value::Bool(false),
            _ => {}
        }
    } else if s == "n:" {
        return Value::Nil;
    } else if let Some(rest) = s.strip_prefix("s:") {
        return Value::String(rest.replace("\\n", "\n").replace("\\\\", "\\"));
    }
    Value::Nil
}

// ── Thread-scoped key prefix ──────────────────────────────────────
// Scope all top-level TTL keys to the current thread so parallel tests
// using the same logical key names don't interfere with each other.

fn thread_key(name: &str) -> String {
    format!("{:?}::{}", std::thread::current().id(), name)
}

// ── TTL map entry key ─────────────────────────────────────────────

fn ttl_map_entry_key(map_id: &str, key: &str) -> String {
    format!("{}||{}", map_id, key)
}

// ── register ──────────────────────────────────────────────────────

pub fn register(env: &mut crate::interpreter::environment::Environment) {

    // ── ttl_set(name: Str, value: Value, seconds: Float) ─────────
    env.define("ttl_set", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "ttl_set".into(),
        arity: Some(3),
        func: Box::new(|args| {
            let name = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("ttl_set: first arg must be a string name".into()),
            };
            let val = args.get(1).cloned().unwrap_or(Value::Nil);
            let secs = args.get(2).and_then(|v| v.as_float()).unwrap_or(0.0);
            let dur = Duration::from_secs_f64(secs.max(0.0));
            let serialized = value_to_str(&val);
            let key = thread_key(&name);
            let mut guard = TTL_STORE.lock().unwrap();
            let reg = guard.get_or_insert_with(HashMap::new);
            reg.insert(key, (serialized, Instant::now(), dur));
            Ok(Value::Nil)
        }),
    })));

    // ── ttl_get(name: Str) -> Value ──────────────────────────────
    env.define("ttl_get", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "ttl_get".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let name = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("ttl_get: first arg must be a string name".into()),
            };
            let key = thread_key(&name);
            let guard = TTL_STORE.lock().unwrap();
            if let Some(reg) = guard.as_ref() {
                if let Some((stored, created, dur)) = reg.get(&key) {
                    if created.elapsed() < *dur {
                        return Ok(str_to_value(stored));
                    }
                }
            }
            Ok(Value::Nil)
        }),
    })));

    // ── ttl_alive(name: Str) -> Bool ─────────────────────────────
    env.define("ttl_alive", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "ttl_alive".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let name = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("ttl_alive: first arg must be a string name".into()),
            };
            let key = thread_key(&name);
            let guard = TTL_STORE.lock().unwrap();
            if let Some(reg) = guard.as_ref() {
                if let Some((_, created, dur)) = reg.get(&key) {
                    return Ok(Value::Bool(created.elapsed() < *dur));
                }
            }
            Ok(Value::Bool(false))
        }),
    })));

    // ── ttl_remaining(name: Str) -> Float ────────────────────────
    env.define("ttl_remaining", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "ttl_remaining".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let name = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("ttl_remaining: first arg must be a string name".into()),
            };
            let key = thread_key(&name);
            let guard = TTL_STORE.lock().unwrap();
            if let Some(reg) = guard.as_ref() {
                if let Some((_, created, dur)) = reg.get(&key) {
                    let elapsed = created.elapsed();
                    if elapsed < *dur {
                        let remaining = dur.as_secs_f64() - elapsed.as_secs_f64();
                        return Ok(Value::Float(remaining));
                    }
                }
            }
            Ok(Value::Float(0.0))
        }),
    })));

    // ── ttl_refresh(name: Str) ───────────────────────────────────
    env.define("ttl_refresh", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "ttl_refresh".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let name = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("ttl_refresh: first arg must be a string name".into()),
            };
            let key = thread_key(&name);
            let mut guard = TTL_STORE.lock().unwrap();
            if let Some(reg) = guard.as_mut() {
                if let Some(entry) = reg.get_mut(&key) {
                    entry.1 = Instant::now();
                }
            }
            Ok(Value::Nil)
        }),
    })));

    // ── ttl_extend(name: Str, seconds: Float) ───────────────────
    env.define("ttl_extend", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "ttl_extend".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let name = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("ttl_extend: first arg must be a string name".into()),
            };
            let extra = args.get(1).and_then(|v| v.as_float()).unwrap_or(0.0);
            let key = thread_key(&name);
            let mut guard = TTL_STORE.lock().unwrap();
            if let Some(reg) = guard.as_mut() {
                if let Some(entry) = reg.get_mut(&key) {
                    entry.2 += Duration::from_secs_f64(extra.max(0.0));
                }
            }
            Ok(Value::Nil)
        }),
    })));

    // ── ttl_map_new() -> Map ─────────────────────────────────────
    env.define("ttl_map_new", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "ttl_map_new".into(),
        arity: Some(0),
        func: Box::new(|_| {
            let id = new_map_id();
            {
                let mut guard = MAP_IDS.lock().unwrap();
                guard.get_or_insert_with(Vec::new).push(id.clone());
            }
            let mut map = HashMap::new();
            map.insert("__type".to_string(), Value::String("TtlMap".to_string()));
            map.insert("__id".to_string(), Value::String(id));
            Ok(Value::Map(Rc::new(RefCell::new(map))))
        }),
    })));

    // ── ttl_map_set(m: Map, key: Str, value: Value, seconds: Float) ─
    env.define("ttl_map_set", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "ttl_map_set".into(),
        arity: Some(4),
        func: Box::new(|args| {
            let map_id = extract_map_id(&args, "ttl_map_set")?;
            let key = match args.get(1) {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("ttl_map_set: second arg must be a string key".into()),
            };
            let val = args.get(2).cloned().unwrap_or(Value::Nil);
            let secs = args.get(3).and_then(|v| v.as_float()).unwrap_or(0.0);
            let entry_key = ttl_map_entry_key(&map_id, &key);
            let serialized = value_to_str(&val);
            let dur = Duration::from_secs_f64(secs.max(0.0));
            let mut guard = TTL_STORE.lock().unwrap();
            let reg = guard.get_or_insert_with(HashMap::new);
            reg.insert(entry_key, (serialized, Instant::now(), dur));
            Ok(Value::Nil)
        }),
    })));

    // ── ttl_map_get(m: Map, key: Str) -> Value ───────────────────
    env.define("ttl_map_get", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "ttl_map_get".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let map_id = extract_map_id(&args, "ttl_map_get")?;
            let key = match args.get(1) {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("ttl_map_get: second arg must be a string key".into()),
            };
            let entry_key = ttl_map_entry_key(&map_id, &key);
            let guard = TTL_STORE.lock().unwrap();
            if let Some(reg) = guard.as_ref() {
                if let Some((stored, created, dur)) = reg.get(&entry_key) {
                    if created.elapsed() < *dur {
                        return Ok(str_to_value(stored));
                    }
                }
            }
            Ok(Value::Nil)
        }),
    })));

    // ── ttl_map_len(m: Map) -> Int ───────────────────────────────
    env.define("ttl_map_len", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "ttl_map_len".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let map_id = extract_map_id(&args, "ttl_map_len")?;
            let prefix = format!("{}||", map_id);
            let guard = TTL_STORE.lock().unwrap();
            let count = guard.as_ref().map(|reg| {
                reg.iter()
                    .filter(|(k, (_, created, dur))| {
                        k.starts_with(&prefix) && created.elapsed() < *dur
                    })
                    .count()
            }).unwrap_or(0);
            Ok(Value::Int(count as i64))
        }),
    })));

    // ── ttl_map_cleanup(m: Map) ──────────────────────────────────
    env.define("ttl_map_cleanup", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "ttl_map_cleanup".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let map_id = extract_map_id(&args, "ttl_map_cleanup")?;
            let prefix = format!("{}||", map_id);
            let mut guard = TTL_STORE.lock().unwrap();
            if let Some(reg) = guard.as_mut() {
                reg.retain(|k, (_, created, dur)| {
                    if k.starts_with(&prefix) {
                        created.elapsed() < *dur
                    } else {
                        true
                    }
                });
            }
            Ok(Value::Nil)
        }),
    })));
}

// ── Helper ────────────────────────────────────────────────────────

fn extract_map_id(args: &[Value], fn_name: &str) -> Result<String, String> {
    match args.first() {
        Some(Value::Map(m)) => {
            let borrow = m.borrow();
            match borrow.get("__id") {
                Some(Value::String(id)) => Ok(id.clone()),
                _ => Err(format!("{}: map has no __id field", fn_name)),
            }
        }
        _ => Err(format!("{}: expected a Map argument", fn_name)),
    }
}

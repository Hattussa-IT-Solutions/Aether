use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::mpsc;
use crate::interpreter::values::*;

// ── Static registries ────────────────────────────────────────────

static ATOMICS: Mutex<Option<HashMap<String, Arc<AtomicI64>>>> = Mutex::new(None);

#[allow(clippy::type_complexity)]
static CHANNELS: Mutex<Option<HashMap<String, (Option<mpsc::SyncSender<String>>, Arc<Mutex<mpsc::Receiver<String>>>)>>> = Mutex::new(None);

static MUTEXES: Mutex<Option<HashMap<String, Arc<Mutex<String>>>>> = Mutex::new(None);

// ── Helpers ──────────────────────────────────────────────────────

fn unique_id(prefix: &str) -> String {
    use std::sync::atomic::{AtomicU64, Ordering as AO};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    format!("{}-{}", prefix, COUNTER.fetch_add(1, AO::SeqCst))
}

/// Serialize a Value to a JSON string for channel transport.
fn value_to_json_string(val: &Value) -> String {
    match val {
        Value::Int(n) => format!("{{\"t\":\"i\",\"v\":{}}}", n),
        Value::Float(f) => format!("{{\"t\":\"f\",\"v\":{}}}", f),
        Value::Bool(b) => format!("{{\"t\":\"b\",\"v\":{}}}", b),
        Value::String(s) => {
            let escaped = s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r");
            format!("{{\"t\":\"s\",\"v\":\"{}\"}}", escaped)
        }
        Value::Nil => "{\"t\":\"n\"}".to_string(),
        Value::List(items) => {
            let parts: Vec<String> = items.borrow().iter().map(value_to_json_string).collect();
            format!("{{\"t\":\"l\",\"v\":[{}]}}", parts.join(","))
        }
        Value::Map(map) => {
            let parts: Vec<String> = map.borrow().iter()
                .map(|(k, v)| {
                    let kesc = k.replace('\\', "\\\\").replace('"', "\\\"");
                    format!("\"{}\":{}", kesc, value_to_json_string(v))
                })
                .collect();
            format!("{{\"t\":\"m\",\"v\":{{{}}}}}", parts.join(","))
        }
        _ => "{\"t\":\"n\"}".to_string(),
    }
}

/// Deserialize a JSON string back to a Value.
fn json_string_to_value(s: &str) -> Value {
    match serde_json::from_str::<serde_json::Value>(s) {
        Ok(j) => decode_json_value(&j),
        Err(_) => Value::Nil,
    }
}

fn decode_json_value(j: &serde_json::Value) -> Value {
    if let Some(obj) = j.as_object() {
        match obj.get("t").and_then(|t| t.as_str()) {
            Some("i") => {
                if let Some(n) = obj.get("v").and_then(|v| v.as_i64()) {
                    return Value::Int(n);
                }
            }
            Some("f") => {
                if let Some(f) = obj.get("v").and_then(|v| v.as_f64()) {
                    return Value::Float(f);
                }
            }
            Some("b") => {
                if let Some(b) = obj.get("v").and_then(|v| v.as_bool()) {
                    return Value::Bool(b);
                }
            }
            Some("s") => {
                if let Some(s) = obj.get("v").and_then(|v| v.as_str()) {
                    return Value::String(s.to_string());
                }
            }
            Some("n") => return Value::Nil,
            Some("l") => {
                if let Some(arr) = obj.get("v").and_then(|v| v.as_array()) {
                    let items: Vec<Value> = arr.iter().map(decode_json_value).collect();
                    return Value::List(Rc::new(RefCell::new(items)));
                }
            }
            Some("m") => {
                if let Some(map_obj) = obj.get("v").and_then(|v| v.as_object()) {
                    let mut map = HashMap::new();
                    for (k, v) in map_obj {
                        map.insert(k.clone(), decode_json_value(v));
                    }
                    return Value::Map(Rc::new(RefCell::new(map)));
                }
            }
            _ => {}
        }
    }
    Value::Nil
}

/// Serialize a Value to a string for mutex storage.
fn value_to_storage_string(val: &Value) -> String {
    value_to_json_string(val)
}

fn storage_string_to_value(s: &str) -> Value {
    json_string_to_value(s)
}

// ── register ─────────────────────────────────────────────────────

pub fn register(env: &mut crate::interpreter::environment::Environment) {

    // ── atomic_new(initial: Int) -> Map ──────────────────────────
    env.define("atomic_new", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "atomic_new".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let initial = args.first().and_then(|v| v.as_int()).unwrap_or(0);
            let id = unique_id("atom");
            let atom = Arc::new(AtomicI64::new(initial));
            {
                let mut guard = ATOMICS.lock().unwrap();
                let reg = guard.get_or_insert_with(HashMap::new);
                reg.insert(id.clone(), atom);
            }
            let mut map = HashMap::new();
            map.insert("__type".to_string(), Value::String("Atomic".to_string()));
            map.insert("__id".to_string(), Value::String(id));
            map.insert("value".to_string(), Value::Int(initial));
            Ok(Value::Map(Rc::new(RefCell::new(map))))
        }),
    })));

    // ── atomic_get(atom: Map) -> Int ─────────────────────────────
    env.define("atomic_get", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "atomic_get".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let id = extract_id(&args, "atomic_get")?;
            let guard = ATOMICS.lock().unwrap();
            let reg = guard.as_ref().ok_or("atomic registry not initialized")?;
            let atom = reg.get(&id).ok_or_else(|| format!("atomic_get: unknown id {}", id))?;
            Ok(Value::Int(atom.load(Ordering::SeqCst)))
        }),
    })));

    // ── atomic_add(atom: Map, n: Int) -> Int ─────────────────────
    env.define("atomic_add", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "atomic_add".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let id = extract_id(&args, "atomic_add")?;
            let n = args.get(1).and_then(|v| v.as_int()).unwrap_or(0);
            let guard = ATOMICS.lock().unwrap();
            let reg = guard.as_ref().ok_or("atomic registry not initialized")?;
            let atom = reg.get(&id).ok_or_else(|| format!("atomic_add: unknown id {}", id))?;
            let prev = atom.fetch_add(n, Ordering::SeqCst);
            Ok(Value::Int(prev))
        }),
    })));

    // ── atomic_set(atom: Map, n: Int) ────────────────────────────
    env.define("atomic_set", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "atomic_set".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let id = extract_id(&args, "atomic_set")?;
            let n = args.get(1).and_then(|v| v.as_int()).unwrap_or(0);
            let guard = ATOMICS.lock().unwrap();
            let reg = guard.as_ref().ok_or("atomic registry not initialized")?;
            let atom = reg.get(&id).ok_or_else(|| format!("atomic_set: unknown id {}", id))?;
            atom.store(n, Ordering::SeqCst);
            Ok(Value::Nil)
        }),
    })));

    // ── channel_new(buffer: Int) -> Map ─────────────────────────
    env.define("channel_new", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "channel_new".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let buffer = args.first().and_then(|v| v.as_int()).unwrap_or(0).max(0) as usize;
            let id = unique_id("chan");
            let (tx, rx) = mpsc::sync_channel::<String>(buffer);
            {
                let mut guard = CHANNELS.lock().unwrap();
                let reg = guard.get_or_insert_with(HashMap::new);
                reg.insert(id.clone(), (Some(tx), Arc::new(Mutex::new(rx))));
            }
            let mut map = HashMap::new();
            map.insert("__type".to_string(), Value::String("Channel".to_string()));
            map.insert("__id".to_string(), Value::String(id));
            Ok(Value::Map(Rc::new(RefCell::new(map))))
        }),
    })));

    // ── channel_send(ch: Map, value: Value) ─────────────────────
    env.define("channel_send", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "channel_send".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let id = extract_id(&args, "channel_send")?;
            let val = args.get(1).cloned().unwrap_or(Value::Nil);
            let serialized = value_to_json_string(&val);
            let guard = CHANNELS.lock().unwrap();
            let reg = guard.as_ref().ok_or("channel registry not initialized")?;
            let entry = reg.get(&id).ok_or_else(|| format!("channel_send: unknown id {}", id))?;
            match &entry.0 {
                Some(tx) => tx.send(serialized).map_err(|e| e.to_string())?,
                None => return Err("channel_send: channel is closed".into()),
            }
            Ok(Value::Nil)
        }),
    })));

    // ── channel_receive(ch: Map) -> Value ────────────────────────
    env.define("channel_receive", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "channel_receive".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let id = extract_id(&args, "channel_receive")?;
            let rx_arc = {
                let guard = CHANNELS.lock().unwrap();
                let reg = guard.as_ref().ok_or("channel registry not initialized")?;
                let entry = reg.get(&id).ok_or_else(|| format!("channel_receive: unknown id {}", id))?;
                Arc::clone(&entry.1)
            };
            let rx = rx_arc.lock().unwrap();
            match rx.recv() {
                Ok(s) => Ok(json_string_to_value(&s)),
                Err(_) => Ok(Value::Nil),
            }
        }),
    })));

    // ── channel_close(ch: Map) ──────────────────────────────────
    env.define("channel_close", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "channel_close".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let id = extract_id(&args, "channel_close")?;
            let mut guard = CHANNELS.lock().unwrap();
            let reg = guard.get_or_insert_with(HashMap::new);
            if let Some(entry) = reg.get_mut(&id) {
                entry.0 = None; // drop the sender
            }
            Ok(Value::Nil)
        }),
    })));

    // ── mutex_new(value: Value) -> Map ───────────────────────────
    env.define("mutex_new", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "mutex_new".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let val = args.first().cloned().unwrap_or(Value::Nil);
            let id = unique_id("mtx");
            let stored = value_to_storage_string(&val);
            let mx = Arc::new(Mutex::new(stored));
            {
                let mut guard = MUTEXES.lock().unwrap();
                let reg = guard.get_or_insert_with(HashMap::new);
                reg.insert(id.clone(), mx);
            }
            let mut map = HashMap::new();
            map.insert("__type".to_string(), Value::String("Mutex".to_string()));
            map.insert("__id".to_string(), Value::String(id));
            Ok(Value::Map(Rc::new(RefCell::new(map))))
        }),
    })));

    // ── mutex_lock_get(m: Map) -> Value ─────────────────────────
    env.define("mutex_lock_get", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "mutex_lock_get".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let id = extract_id(&args, "mutex_lock_get")?;
            let guard = MUTEXES.lock().unwrap();
            let reg = guard.as_ref().ok_or("mutex registry not initialized")?;
            let mx = reg.get(&id).ok_or_else(|| format!("mutex_lock_get: unknown id {}", id))?;
            let inner = mx.lock().unwrap();
            Ok(storage_string_to_value(&inner))
        }),
    })));

    // ── mutex_lock_set(m: Map, value: Value) ─────────────────────
    env.define("mutex_lock_set", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "mutex_lock_set".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let id = extract_id(&args, "mutex_lock_set")?;
            let val = args.get(1).cloned().unwrap_or(Value::Nil);
            let stored = value_to_storage_string(&val);
            let guard = MUTEXES.lock().unwrap();
            let reg = guard.as_ref().ok_or("mutex registry not initialized")?;
            let mx = reg.get(&id).ok_or_else(|| format!("mutex_lock_set: unknown id {}", id))?;
            let mut inner = mx.lock().unwrap();
            *inner = stored;
            Ok(Value::Nil)
        }),
    })));
}

// ── Helper to pull __id from a Map argument ──────────────────────

fn extract_id(args: &[Value], fn_name: &str) -> Result<String, String> {
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

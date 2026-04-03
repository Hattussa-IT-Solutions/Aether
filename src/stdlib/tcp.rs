/// TCP client/server module for Aether.
///
/// TcpStream and TcpListener values cannot be placed into Value directly
/// (they are not Clone), so we keep them in two global registries keyed
/// by a UUID-style string handle.
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::rc::Rc;
use std::sync::Mutex;

use crate::interpreter::values::*;

// ── Global registries ─────────────────────────────────────────────────────────

static STREAMS: Mutex<Option<HashMap<String, TcpStream>>> = Mutex::new(None);
static LISTENERS: Mutex<Option<HashMap<String, TcpListener>>> = Mutex::new(None);

fn next_handle(prefix: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{}-{}", prefix, t)
}

fn insert_stream(handle: String, stream: TcpStream) {
    let mut g = STREAMS.lock().unwrap();
    g.get_or_insert_with(HashMap::new).insert(handle, stream);
}

fn with_stream_mut<F, T>(handle: &str, f: F) -> Result<T, String>
where
    F: FnOnce(&mut TcpStream) -> Result<T, String>,
{
    let mut g = STREAMS.lock().unwrap();
    match g.as_mut().and_then(|m| m.get_mut(handle)) {
        Some(s) => f(s),
        None => Err(format!("tcp: unknown connection handle '{}'", handle)),
    }
}

fn remove_stream(handle: &str) {
    let mut g = STREAMS.lock().unwrap();
    if let Some(m) = g.as_mut() {
        m.remove(handle);
    }
}

fn insert_listener(handle: String, listener: TcpListener) {
    let mut g = LISTENERS.lock().unwrap();
    g.get_or_insert_with(HashMap::new).insert(handle, listener);
}

fn accept_on_listener(handle: &str) -> Result<TcpStream, String> {
    let g = LISTENERS.lock().unwrap();
    match g.as_ref().and_then(|m| m.get(handle)) {
        Some(l) => l.accept().map(|(s, _)| s).map_err(|e| e.to_string()),
        None => Err(format!("tcp: unknown listener handle '{}'", handle)),
    }
}

// ── Registration ──────────────────────────────────────────────────────────────

pub fn register(env: &mut crate::interpreter::environment::Environment) {
    // tcp_connect(addr) -> Map { handle, remote }
    env.define("tcp_connect", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "tcp_connect".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let addr = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("tcp_connect: expected address string".into()),
            };
            let stream = TcpStream::connect(&addr)
                .map_err(|e| format!("tcp_connect: {}", e))?;
            let remote = stream.peer_addr().map(|a| a.to_string()).unwrap_or_default();
            let handle = next_handle("conn");
            insert_stream(handle.clone(), stream);
            let mut map = HashMap::new();
            map.insert("handle".to_string(), Value::String(handle));
            map.insert("remote".to_string(), Value::String(remote));
            Ok(Value::Map(Rc::new(RefCell::new(map))))
        }),
    })));

    // tcp_write(conn, data) -> Int (bytes written)
    env.define("tcp_write", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "tcp_write".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let handle = match args.first() {
                Some(Value::String(s)) => s.clone(),
                Some(Value::Map(m)) => match m.borrow().get("handle") {
                    Some(Value::String(s)) => s.clone(),
                    _ => return Err("tcp_write: conn map missing 'handle'".into()),
                },
                _ => return Err("tcp_write: first arg must be a handle string or conn map".into()),
            };
            let data = match args.get(1) {
                Some(Value::String(s)) => s.as_bytes().to_vec(),
                Some(other) => other.to_string().into_bytes(),
                None => return Err("tcp_write: missing data argument".into()),
            };
            let n = with_stream_mut(&handle, |s| {
                s.write_all(&data).map_err(|e| e.to_string())?;
                Ok(data.len())
            })?;
            Ok(Value::Int(n as i64))
        }),
    })));

    // tcp_read(conn, max) -> String
    env.define("tcp_read", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "tcp_read".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let handle = match args.first() {
                Some(Value::String(s)) => s.clone(),
                Some(Value::Map(m)) => match m.borrow().get("handle") {
                    Some(Value::String(s)) => s.clone(),
                    _ => return Err("tcp_read: conn map missing 'handle'".into()),
                },
                _ => return Err("tcp_read: first arg must be a handle string or conn map".into()),
            };
            let max = match args.get(1) {
                Some(Value::Int(n)) => *n as usize,
                Some(Value::Float(f)) => *f as usize,
                _ => 4096,
            };
            let data = with_stream_mut(&handle, |s| {
                let mut buf = vec![0u8; max];
                let n = s.read(&mut buf).map_err(|e| e.to_string())?;
                Ok(String::from_utf8_lossy(&buf[..n]).to_string())
            })?;
            Ok(Value::String(data))
        }),
    })));

    // tcp_listen(addr) -> Map { handle, local }
    env.define("tcp_listen", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "tcp_listen".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let addr = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("tcp_listen: expected address string".into()),
            };
            let listener = TcpListener::bind(&addr)
                .map_err(|e| format!("tcp_listen: {}", e))?;
            let local = listener.local_addr().map(|a| a.to_string()).unwrap_or_default();
            let handle = next_handle("listener");
            insert_listener(handle.clone(), listener);
            let mut map = HashMap::new();
            map.insert("handle".to_string(), Value::String(handle));
            map.insert("local".to_string(), Value::String(local));
            Ok(Value::Map(Rc::new(RefCell::new(map))))
        }),
    })));

    // tcp_accept(listener) -> Map { handle, remote }
    env.define("tcp_accept", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "tcp_accept".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let handle = match args.first() {
                Some(Value::String(s)) => s.clone(),
                Some(Value::Map(m)) => match m.borrow().get("handle") {
                    Some(Value::String(s)) => s.clone(),
                    _ => return Err("tcp_accept: listener map missing 'handle'".into()),
                },
                _ => return Err("tcp_accept: first arg must be a handle string or listener map".into()),
            };
            let stream = accept_on_listener(&handle)
                .map_err(|e| format!("tcp_accept: {}", e))?;
            let remote = stream.peer_addr().map(|a| a.to_string()).unwrap_or_default();
            let conn_handle = next_handle("conn");
            insert_stream(conn_handle.clone(), stream);
            let mut map = HashMap::new();
            map.insert("handle".to_string(), Value::String(conn_handle));
            map.insert("remote".to_string(), Value::String(remote));
            Ok(Value::Map(Rc::new(RefCell::new(map))))
        }),
    })));

    // tcp_close(conn)
    env.define("tcp_close", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "tcp_close".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let handle = match args.first() {
                Some(Value::String(s)) => s.clone(),
                Some(Value::Map(m)) => match m.borrow().get("handle") {
                    Some(Value::String(s)) => s.clone(),
                    _ => return Err("tcp_close: conn map missing 'handle'".into()),
                },
                _ => return Err("tcp_close: first arg must be a handle string or conn map".into()),
            };
            remove_stream(&handle);
            Ok(Value::Nil)
        }),
    })));
}

use aether::interpreter;
use aether::interpreter::environment::Environment;
use aether::interpreter::values::Value;
use aether::lexer::scanner::Scanner;
use aether::parser::parser::Parser;

fn run_get(source: &str, var: &str) -> Value {
    let mut scanner = Scanner::new(source, "test.ae".to_string());
    let tokens = scanner.scan_tokens();
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("parse failed");
    let mut env = Environment::new();
    interpreter::register_builtins(&mut env);
    interpreter::interpret(&program, &mut env).expect("interpret failed");
    env.get(var).unwrap_or(Value::Nil)
}

fn run_ok(source: &str) -> bool {
    let mut scanner = Scanner::new(source, "test.ae".to_string());
    let tokens = scanner.scan_tokens();
    let mut parser = Parser::new(tokens);
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(_) => return false,
    };
    let mut env = Environment::new();
    interpreter::register_builtins(&mut env);
    interpreter::interpret(&program, &mut env).is_ok()
}

fn assert_int(v: &Value, n: i64) {
    match v { Value::Int(x) => assert_eq!(*x, n), _ => panic!("expected Int({}), got {:?}", n, v) }
}

fn assert_bool(v: &Value, b: bool) {
    match v { Value::Bool(x) => assert_eq!(*x, b), _ => panic!("expected {}, got {:?}", b, v) }
}

// ═══════════════════════════════════════════════════════════════
// Atomic tests
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_atomic_new() {
    assert!(run_ok("counter = atomic_new(0)\nprint(atomic_get(counter))"));
}

#[test]
fn test_atomic_add() {
    assert_int(&run_get("c = atomic_new(0)\natomic_add(c, 5)\nx = atomic_get(c)", "x"), 5);
}

#[test]
fn test_atomic_set() {
    assert_int(&run_get("c = atomic_new(0)\natomic_set(c, 42)\nx = atomic_get(c)", "x"), 42);
}

#[test]
fn test_atomic_multiple_adds() {
    assert_int(&run_get("c = atomic_new(0)\natomic_add(c, 1)\natomic_add(c, 2)\natomic_add(c, 3)\nx = atomic_get(c)", "x"), 6);
}

// ═══════════════════════════════════════════════════════════════
// Channel tests
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_channel_send_receive() {
    let src = r#"
ch = channel_new(5)
channel_send(ch, 42)
x = channel_receive(ch)
"#;
    assert_int(&run_get(src, "x"), 42);
}

#[test]
fn test_channel_multiple() {
    let src = r#"
ch = channel_new(10)
channel_send(ch, 1)
channel_send(ch, 2)
channel_send(ch, 3)
a = channel_receive(ch)
b = channel_receive(ch)
c = channel_receive(ch)
x = a + b + c
"#;
    assert_int(&run_get(src, "x"), 6);
}

#[test]
fn test_channel_string() {
    let src = r#"
ch = channel_new(5)
channel_send(ch, "hello")
x = channel_receive(ch)
"#;
    match run_get(src, "x") {
        Value::String(s) => assert_eq!(s, "hello"),
        other => panic!("expected String, got {:?}", other),
    }
}

// ═══════════════════════════════════════════════════════════════
// Mutex tests
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_mutex_new() {
    assert!(run_ok("m = mutex_new(0)\nprint(mutex_lock_get(m))"));
}

#[test]
fn test_mutex_set_get() {
    assert_int(&run_get("m = mutex_new(0)\nmutex_lock_set(m, 42)\nx = mutex_lock_get(m)", "x"), 42);
}

// ═══════════════════════════════════════════════════════════════
// TTL tests
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_ttl_set_get() {
    let src = r#"
ttl_set("mykey", "myvalue", 10.0)
x = ttl_get("mykey")
"#;
    match run_get(src, "x") {
        Value::String(s) => assert_eq!(s, "myvalue"),
        other => panic!("expected String, got {:?}", other),
    }
}

#[test]
fn test_ttl_alive() {
    assert_bool(&run_get(r#"ttl_set("k", 1, 10.0); x = ttl_alive("k")"#, "x"), true);
}

#[test]
fn test_ttl_remaining() {
    let src = r#"
ttl_set("k", 1, 10.0)
x = ttl_remaining("k")
"#;
    let v = run_get(src, "x");
    if let Value::Float(f) = v { assert!(f > 8.0 && f <= 10.0, "remaining {} should be ~10", f); }
    else { panic!("expected Float, got {:?}", v); }
}

#[test]
fn test_ttl_expired() {
    let src = r#"
ttl_set("k", "val", 0.1)
time_sleep(0.2)
x = ttl_alive("k")
y = ttl_get("k")
"#;
    assert_bool(&run_get(src, "x"), false);
}

#[test]
fn test_ttl_refresh() {
    let src = r#"
ttl_set("k", "val", 5.0)
time_sleep(1.0)
ttl_refresh("k")
x = ttl_remaining("k")
"#;
    let v = run_get(src, "x");
    if let Value::Float(f) = v { assert!(f > 4.0, "after refresh remaining {} should be ~5", f); }
}

#[test]
fn test_ttl_map() {
    let src = r#"
tm = ttl_map_new()
ttl_map_set(tm, "a", 1, 10.0)
ttl_map_set(tm, "b", 2, 10.0)
x = ttl_map_len(tm)
"#;
    assert_int(&run_get(src, "x"), 2);
}

#[test]
fn test_ttl_map_get() {
    let src = r#"
tm = ttl_map_new()
ttl_map_set(tm, "key", 42, 10.0)
x = ttl_map_get(tm, "key")
"#;
    assert_int(&run_get(src, "x"), 42);
}

#[test]
fn test_ttl_map_expired() {
    let src = r#"
tm = ttl_map_new()
ttl_map_set(tm, "fast", "gone", 0.1)
ttl_map_set(tm, "slow", "here", 10.0)
time_sleep(0.2)
x = ttl_map_len(tm)
"#;
    assert_int(&run_get(src, "x"), 1);
}

// ═══════════════════════════════════════════════════════════════
// Memory tests
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_memory_used() {
    let v = run_get("x = memory_used()", "x");
    if let Value::Int(n) = v { assert!(n > 0, "memory_used should be > 0"); }
}

#[test]
fn test_memory_peak() {
    let v = run_get("x = memory_peak()", "x");
    if let Value::Int(n) = v { assert!(n > 0, "memory_peak should be > 0"); }
}

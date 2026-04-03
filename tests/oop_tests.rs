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

fn assert_int(val: &Value, expected: i64) {
    match val { Value::Int(n) => assert_eq!(*n, expected), _ => panic!("expected Int({}), got {:?}", expected, val) }
}
fn assert_float(val: &Value, expected: f64) {
    match val {
        Value::Float(f) => assert!((f - expected).abs() < 0.01, "expected {}, got {}", expected, f),
        Value::Int(n) => assert!((*n as f64 - expected).abs() < 0.01),
        _ => panic!("expected Float({}), got {:?}", expected, val),
    }
}
fn assert_str(val: &Value, expected: &str) {
    match val { Value::String(s) => assert_eq!(s, expected), _ => panic!("expected '{}', got {:?}", expected, val) }
}
fn assert_bool(val: &Value, expected: bool) {
    match val { Value::Bool(b) => assert_eq!(*b, expected), _ => panic!("expected Bool({}), got {:?}", expected, val) }
}

// ═══════════════════════════════════════════════════════════════
// Operator overloading
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_operator_overload_add() {
    let src = r#"
class Vec2 {
    x: Float
    y: Float
    init(x, y) { self.x = x; self.y = y }
    operator +(other: Vec2) -> Vec2 {
        return Vec2(self.x + other.x, self.y + other.y)
    }
}
a = Vec2(1.0, 2.0)
b = Vec2(3.0, 4.0)
c = a + b
x = c.x
"#;
    assert_float(&run_get(src, "x"), 4.0);
}

#[test]
fn test_operator_overload_eq() {
    let src = r#"
class Point {
    x: Float
    y: Float
    init(x, y) { self.x = x; self.y = y }
    operator ==(other: Point) -> Bool {
        return self.x == other.x and self.y == other.y
    }
}
a = Point(1.0, 2.0)
b = Point(1.0, 2.0)
c = Point(3.0, 4.0)
x = a == b
y = a == c
"#;
    assert_bool(&run_get(src, "x"), true);
    assert_bool(&run_get(src, "y"), false);
}

// ═══════════════════════════════════════════════════════════════
// Computed properties
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_computed_property() {
    let src = r#"
class Rect {
    w: Float
    h: Float
    area: Float => self.w * self.h
    init(w, h) { self.w = w; self.h = h }
}
r = Rect(3.0, 4.0)
x = r.area
"#;
    assert_float(&run_get(src, "x"), 12.0);
}

#[test]
fn test_computed_property_updates() {
    let src = r#"
class Rect {
    w: Float
    h: Float
    area: Float => self.w * self.h
    init(w, h) { self.w = w; self.h = h }
}
r = Rect(3.0, 4.0)
r.w = 10.0
x = r.area
"#;
    assert_float(&run_get(src, "x"), 40.0);
}

// ═══════════════════════════════════════════════════════════════
// Super calls
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_super_method_call() {
    let src = r#"
class Base {
    def greet() { return "base" }
}
class Child : Base {
    def greet() { return "child" }
    def parent_greet() { return super.greet() }
}
c = Child()
x = c.parent_greet()
"#;
    assert_str(&run_get(src, "x"), "base");
}

#[test]
fn test_super_with_override() {
    let src = r#"
class Animal {
    def sound() { return "..." }
}
class Dog : Animal {
    def sound() { return "Woof" }
}
d = Dog()
x = d.sound()
"#;
    assert_str(&run_get(src, "x"), "Woof");
}

// ═══════════════════════════════════════════════════════════════
// Struct copy semantics
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_struct_copy_on_assign() {
    let src = r#"
struct Point {
    x: Float
    y: Float
}
p1 = Point(1.0, 2.0)
p2 = p1
p2.x = 99.0
x = p1.x
"#;
    assert_float(&run_get(src, "x"), 1.0); // p1 should be unchanged
}

#[test]
fn test_class_reference_semantics() {
    let src = r#"
class Box {
    val: Int
    init(val) { self.val = val }
}
a = Box(1)
b = a
b.val = 99
x = a.val
"#;
    assert_int(&run_get(src, "x"), 99); // class instances share references
}

// ═══════════════════════════════════════════════════════════════
// Set operations
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_set_len() { assert_int(&run_get("x = {1,2,3}.len()", "x"), 3); }

#[test]
fn test_set_contains() {
    assert_bool(&run_get("x = {1,2,3}.contains(2)", "x"), true);
    assert_bool(&run_get("x = {1,2,3}.contains(9)", "x"), false);
}

#[test]
fn test_set_insert() {
    assert_int(&run_get("s = {1,2}\ns.insert(3)\nx = s.len()", "x"), 3);
}

#[test]
fn test_set_remove() {
    assert_int(&run_get("s = {1,2,3}\ns.remove(2)\nx = s.len()", "x"), 2);
}

#[test]
fn test_set_union() {
    assert_int(&run_get("x = {1,2}.union({2,3}).len()", "x"), 3);
}

#[test]
fn test_set_intersect() {
    assert_int(&run_get("x = {1,2,3}.intersect({2,3,4}).len()", "x"), 2);
}

#[test]
fn test_set_difference() {
    assert_int(&run_get("x = {1,2,3}.difference({2,3,4}).len()", "x"), 1);
}

// ═══════════════════════════════════════════════════════════════
// List methods (new)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_list_reduce() {
    assert_int(&run_get("x = [1,2,3,4,5].reduce(0, (acc, x) -> acc + x)", "x"), 15);
}

#[test]
fn test_list_reduce_product() {
    assert_int(&run_get("x = [1,2,3,4].reduce(1, (acc, x) -> acc * x)", "x"), 24);
}

#[test]
fn test_list_flat_map() {
    assert_int(&run_get("x = [[1,2],[3,4]].flat_map(x -> x).len()", "x"), 4);
}

#[test]
fn test_list_zip() {
    assert_int(&run_get("x = [1,2,3].zip([4,5,6]).len()", "x"), 3);
}

#[test]
fn test_list_chunks() {
    assert_int(&run_get("x = [1,2,3,4,5].chunks(2).len()", "x"), 3);
}

// ═══════════════════════════════════════════════════════════════
// Inheritance
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_inherited_fields() {
    let src = r#"
class Animal {
    name: Str
    init(name) { self.name = name }
}
class Dog : Animal {
    tricks: Int = 0
}
d = Dog("Rex")
x = d.name
"#;
    assert_str(&run_get(src, "x"), "Rex");
}

#[test]
fn test_method_override() {
    let src = r#"
class Base {
    def value() { return 1 }
}
class Mid : Base {
    def value() { return 2 }
}
class Leaf : Mid {
    def value() { return 3 }
}
x = Leaf().value()
"#;
    assert_int(&run_get(src, "x"), 3);
}

#[test]
fn test_inherited_method() {
    let src = r#"
class Base {
    def greet() { return "hello" }
}
class Child : Base { }
x = Child().greet()
"#;
    assert_str(&run_get(src, "x"), "hello");
}

// ═══════════════════════════════════════════════════════════════
// Enum methods
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_enum_variant_match() {
    let src = r#"
x = match .Circle(5.0) {
    .Circle(r) -> r * r * 3.14
    _ -> 0.0
}
"#;
    assert_float(&run_get(src, "x"), 78.5);
}

// ═══════════════════════════════════════════════════════════════
// String methods (comprehensive)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_str_join() {
    assert_str(&run_get("x = [\"a\",\"b\",\"c\"].join(\", \")", "x"), "a, b, c");
}

#[test]
fn test_str_slice() {
    assert_str(&run_get("x = \"hello world\".slice(0, 5)", "x"), "hello");
}

#[test]
fn test_str_parse_int() {
    let val = run_get("x = \"42\".parse_int()", "x");
    assert!(matches!(val, Value::Ok(_)));
}

#[test]
fn test_str_parse_int_fail() {
    let val = run_get("x = \"abc\".parse_int()", "x");
    assert!(matches!(val, Value::Err(_)));
}

// ═══════════════════════════════════════════════════════════════
// JSON
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_json_encode() {
    assert_str(&run_get("x = json_encode(42)", "x"), "42");
}

#[test]
fn test_json_decode() {
    assert_int(&run_get("x = json_decode(\"42\")", "x"), 42);
}

// ═══════════════════════════════════════════════════════════════
// File I/O
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_fs_write_and_read() {
    let src = r#"
fs_write("/tmp/aether_test.txt", "hello aether")
x = fs_read("/tmp/aether_test.txt")
"#;
    assert_str(&run_get(src, "x"), "hello aether");
}

#[test]
fn test_fs_exists() {
    assert_bool(&run_get("fs_write(\"/tmp/aether_exists.txt\", \"x\")\nx = fs_exists(\"/tmp/aether_exists.txt\")", "x"), true);
    assert_bool(&run_get("x = fs_exists(\"/tmp/nonexistent_aether_file\")", "x"), false);
}

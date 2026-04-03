use aether::interpreter;
use aether::interpreter::environment::Environment;
use aether::interpreter::values::Value;
use aether::lexer::scanner::Scanner;
use aether::parser::parser::Parser;

/// Run source code and return stdout output.
fn run(source: &str) -> String {
    // Capture stdout by using print to a buffer
    let mut scanner = Scanner::new(source, "test.ae".to_string());
    let tokens = scanner.scan_tokens();
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("parse failed");
    let mut env = Environment::new();
    interpreter::register_builtins(&mut env);

    // We can't easily capture stdout in tests, so test via environment state
    interpreter::interpret(&program, &mut env).expect("interpret failed");
    // Return a value from env if set
    env.get("__result").map(|v| v.to_string()).unwrap_or_default()
}

/// Run and get a named variable's value.
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
    match val {
        Value::Int(n) => assert_eq!(*n, expected, "expected {}, got {}", expected, n),
        _ => panic!("expected Int({}), got {:?}", expected, val),
    }
}

fn assert_float_approx(val: &Value, expected: f64) {
    match val {
        Value::Float(f) => assert!((f - expected).abs() < 1e-6, "expected {}, got {}", expected, f),
        Value::Int(n) => assert!((*n as f64 - expected).abs() < 1e-6),
        _ => panic!("expected Float({}), got {:?}", expected, val),
    }
}

fn assert_bool(val: &Value, expected: bool) {
    match val {
        Value::Bool(b) => assert_eq!(*b, expected),
        _ => panic!("expected Bool({}), got {:?}", expected, val),
    }
}

fn assert_str(val: &Value, expected: &str) {
    match val {
        Value::String(s) => assert_eq!(s, expected, "expected '{}', got '{}'", expected, s),
        _ => panic!("expected String('{}'), got {:?}", expected, val),
    }
}

// ═══════════════════════════════════════════════════════════════
// Variables and arithmetic
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_basic_assignment() {
    let val = run_get("x = 42", "x");
    assert_int(&val, 42);
}

#[test]
fn test_let_immutable() {
    let val = run_get("let x = 10", "x");
    assert_int(&val, 10);
}

#[test]
fn test_arithmetic() {
    assert_int(&run_get("x = 2 + 3", "x"), 5);
    assert_int(&run_get("x = 10 - 3", "x"), 7);
    assert_int(&run_get("x = 4 * 5", "x"), 20);
    assert_int(&run_get("x = 10 / 3", "x"), 3);
    assert_int(&run_get("x = 10 % 3", "x"), 1);
    assert_int(&run_get("x = 2 ** 10", "x"), 1024);
}

#[test]
fn test_float_arithmetic() {
    assert_float_approx(&run_get("x = 1.5 + 2.5", "x"), 4.0);
    assert_float_approx(&run_get("x = 10.0 / 3.0", "x"), 10.0/3.0);
}

#[test]
fn test_int_float_promotion() {
    assert_float_approx(&run_get("x = 1 + 2.5", "x"), 3.5);
    assert_float_approx(&run_get("x = 3.14 * 2", "x"), 6.28);
}

#[test]
fn test_string_concat() {
    assert_str(&run_get("x = \"hello\" + \" world\"", "x"), "hello world");
}

#[test]
fn test_string_interpolation() {
    assert_str(&run_get("name = \"World\"\nx = \"Hello, {name}!\"", "x"), "Hello, World!");
}

#[test]
fn test_compound_assignment() {
    assert_int(&run_get("x = 10\nx += 5", "x"), 15);
    assert_int(&run_get("x = 10\nx -= 3", "x"), 7);
    assert_int(&run_get("x = 10\nx *= 2", "x"), 20);
}

// ═══════════════════════════════════════════════════════════════
// Comparison and logical
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_comparison() {
    assert_bool(&run_get("x = 5 > 3", "x"), true);
    assert_bool(&run_get("x = 5 < 3", "x"), false);
    assert_bool(&run_get("x = 5 == 5", "x"), true);
    assert_bool(&run_get("x = 5 != 3", "x"), true);
    assert_bool(&run_get("x = 5 >= 5", "x"), true);
    assert_bool(&run_get("x = 5 <= 4", "x"), false);
}

#[test]
fn test_logical_operators() {
    assert_bool(&run_get("x = true and false", "x"), false);
    assert_bool(&run_get("x = true or false", "x"), true);
    assert_bool(&run_get("x = not true", "x"), false);
    assert_bool(&run_get("x = true && true", "x"), true);
    assert_bool(&run_get("x = false || true", "x"), true);
}

#[test]
fn test_short_circuit() {
    // `and` should short-circuit: false and (side-effect) should not eval right side
    let val = run_get("x = false and (1 / 0 == 0)", "x");
    assert_bool(&val, false);
}

// ═══════════════════════════════════════════════════════════════
// Control flow
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_if_else() {
    assert_int(&run_get("x = 0\nif true { x = 1 } else { x = 2 }", "x"), 1);
    assert_int(&run_get("x = 0\nif false { x = 1 } else { x = 2 }", "x"), 2);
}

#[test]
fn test_if_else_if() {
    let src = "x = 0\nn = 15\nif n > 20 { x = 1 } else if n > 10 { x = 2 } else { x = 3 }";
    assert_int(&run_get(src, "x"), 2);
}

#[test]
fn test_if_expression() {
    assert_str(&run_get("x = if true then \"yes\" else \"no\"", "x"), "yes");
    assert_str(&run_get("x = if false then \"yes\" else \"no\"", "x"), "no");
}

#[test]
fn test_for_loop_range() {
    assert_int(&run_get("x = 0\nfor i in 0..5 { x += i }", "x"), 10);
}

#[test]
fn test_for_loop_list() {
    assert_int(&run_get("x = 0\nfor item in [1, 2, 3] { x += item }", "x"), 6);
}

#[test]
fn test_for_enumerate() {
    assert_int(&run_get("x = 0\nfor i, val in [10, 20, 30] { x += i }", "x"), 3);
}

#[test]
fn test_for_step() {
    assert_int(&run_get("x = 0\nfor i in 0..10 step 3 { x += 1 }", "x"), 4);
}

#[test]
fn test_loop_times() {
    assert_int(&run_get("x = 0\nloop 5 times { x += 1 }", "x"), 5);
}

#[test]
fn test_loop_while() {
    assert_int(&run_get("x = 10\nloop while x > 0 { x -= 3 }", "x"), -2);
}

#[test]
fn test_loop_until() {
    assert_int(&run_get("x = 0\nloop { x += 7 } until x > 50", "x"), 56);
}

#[test]
fn test_break() {
    assert_int(&run_get("x = 0\nfor i in 0..100 { if i >= 5 { break }\nx += 1 }", "x"), 5);
}

#[test]
fn test_next_if() {
    // Skip even numbers
    assert_int(&run_get("x = 0\nfor i in 0..10 { next if i % 2 == 0\nx += 1 }", "x"), 5);
}

#[test]
fn test_labeled_break() {
    let src = "x = 0\nfor:outer a in 0..3 {\n  for b in 0..3 {\n    if a == 1 { break:outer }\n    x += 1\n  }\n}";
    assert_int(&run_get(src, "x"), 3);
}

// ═══════════════════════════════════════════════════════════════
// Functions
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_function_def_and_call() {
    assert_int(&run_get("def add(a, b) { return a + b }\nx = add(3, 4)", "x"), 7);
}

#[test]
fn test_function_implicit_return() {
    assert_int(&run_get("def double(n) { n * 2 }\nx = double(21)", "x"), 42);
}

#[test]
fn test_function_expression_body() {
    assert_int(&run_get("def triple(n) = n * 3\nx = triple(7)", "x"), 21);
}

#[test]
fn test_default_params() {
    assert_int(&run_get("def f(a, b = 10) { return a + b }\nx = f(5)", "x"), 15);
    assert_int(&run_get("def f(a, b = 10) { return a + b }\nx = f(5, 20)", "x"), 25);
}

#[test]
fn test_recursion() {
    assert_int(&run_get("def fact(n) { if n <= 1 { return 1 }\nreturn n * fact(n - 1) }\nx = fact(10)", "x"), 3628800);
}

#[test]
fn test_lambda() {
    assert_int(&run_get("f = x -> x * 2\nx = f(21)", "x"), 42);
}

#[test]
fn test_lambda_multi_param() {
    assert_int(&run_get("f = (a, b) -> a + b\nx = f(3, 4)", "x"), 7);
}

#[test]
fn test_closure() {
    assert_int(&run_get("def make_adder(n) { return x -> x + n }\nadd5 = make_adder(5)\nx = add5(10)", "x"), 15);
}

// ═══════════════════════════════════════════════════════════════
// Pattern matching
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_match_literal() {
    assert_str(&run_get("x = match 2 { 1 -> \"one\"\n2 -> \"two\"\n_ -> \"other\" }", "x"), "two");
}

#[test]
fn test_match_wildcard() {
    assert_str(&run_get("x = match 99 { 1 -> \"one\"\n_ -> \"other\" }", "x"), "other");
}

#[test]
fn test_match_range() {
    assert_str(&run_get("x = match 5 { 1..10 -> \"small\"\n_ -> \"big\" }", "x"), "small");
}

#[test]
fn test_match_guard() {
    assert_str(&run_get("x = match 4 { n if n % 2 == 0 -> \"even\"\n_ -> \"odd\" }", "x"), "even");
}

#[test]
fn test_match_destructure_ok() {
    assert_int(&run_get("r = Ok(42)\nx = match r { Ok(v) -> v\nErr(e) -> 0 }", "x"), 42);
}

#[test]
fn test_match_destructure_err() {
    assert_int(&run_get("r = Err(\"fail\")\nx = match r { Ok(v) -> v\nErr(e) -> -1 }", "x"), -1);
}

#[test]
fn test_match_enum_variant() {
    assert_float_approx(
        &run_get("x = match .Circle(5.0) { .Circle(r) -> 3.14 * r * r\n_ -> 0.0 }", "x"),
        78.5,
    );
}

#[test]
fn test_match_as_expression_in_function() {
    let src = "def classify(n) {\n  match n {\n    0 -> \"zero\"\n    _ -> \"nonzero\"\n  }\n}\nx = classify(0)";
    assert_str(&run_get(src, "x"), "zero");
}

// ═══════════════════════════════════════════════════════════════
// Error handling
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_ok_err() {
    let val = run_get("x = Ok(42)", "x");
    assert!(matches!(val, Value::Ok(_)));
    let val = run_get("x = Err(\"fail\")", "x");
    assert!(matches!(val, Value::Err(_)));
}

#[test]
fn test_error_propagation() {
    let src = "def safe(s) {\n  n = s.parse_int()?\n  return Ok(n * 2)\n}\nx = safe(\"21\")";
    let val = run_get(src, "x");
    if let Value::Ok(inner) = val { assert_int(&inner, 42); }
    else { panic!("expected Ok, got {:?}", val); }
}

#[test]
fn test_error_propagation_failure() {
    let src = "def safe(s) {\n  n = s.parse_int()?\n  return Ok(n * 2)\n}\nx = safe(\"bad\")";
    let val = run_get(src, "x");
    assert!(matches!(val, Value::Err(_)), "expected Err, got {:?}", val);
}

#[test]
fn test_try_catch() {
    let src = "x = 0\ntry {\n  throw \"error\"\n} catch any as e {\n  x = 1\n}";
    assert_int(&run_get(src, "x"), 1);
}

#[test]
fn test_try_finally() {
    let src = "x = 0\ntry {\n  x = 1\n} catch any as e {\n  x = 2\n} finally {\n  x += 10\n}";
    assert_int(&run_get(src, "x"), 11);
}

#[test]
fn test_try_catch_finally_with_throw() {
    let src = "x = 0\ntry {\n  throw \"err\"\n} catch any as e {\n  x = 1\n} finally {\n  x += 10\n}";
    assert_int(&run_get(src, "x"), 11);
}

#[test]
fn test_nil_coalescing() {
    assert_int(&run_get("x = nil ?? 42", "x"), 42);
    assert_int(&run_get("x = 10 ?? 42", "x"), 10);
}

#[test]
fn test_guard_with_value() {
    let src = "def f(val) {\n  guard let n = val else { return -1 }\n  return n * 2\n}\nx = f(5)";
    assert_int(&run_get(src, "x"), 10);
}

#[test]
fn test_guard_with_nil() {
    let src = "def f(val) {\n  guard let n = val else { return -1 }\n  return n * 2\n}\nx = f(nil)";
    assert_int(&run_get(src, "x"), -1);
}

// ═══════════════════════════════════════════════════════════════
// Collections
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_list_operations() {
    assert_int(&run_get("x = [1, 2, 3].len()", "x"), 3);
    assert_int(&run_get("l = [1, 2, 3]\nl.push(4)\nx = l.len()", "x"), 4);
    assert_int(&run_get("x = [10, 20, 30].sum()", "x"), 60);
}

#[test]
fn test_list_map_filter() {
    let val = run_get("x = [1, 2, 3, 4].filter(x -> x > 2).len()", "x");
    assert_int(&val, 2);
}

#[test]
fn test_map_operations() {
    assert_int(&run_get("m = {\"a\": 1, \"b\": 2}\nx = m.len()", "x"), 2);
    assert_bool(&run_get("m = {\"a\": 1}\nx = m.contains_key(\"a\")", "x"), true);
    assert_bool(&run_get("m = {\"a\": 1}\nx = m.contains_key(\"z\")", "x"), false);
}

#[test]
fn test_string_methods() {
    assert_int(&run_get("x = \"hello\".len()", "x"), 5);
    assert_str(&run_get("x = \"hello\".upper()", "x"), "HELLO");
    assert_str(&run_get("x = \"  hi  \".trim()", "x"), "hi");
    assert_bool(&run_get("x = \"hello\".contains(\"ell\")", "x"), true);
    assert_bool(&run_get("x = \"hello\".starts_with(\"hel\")", "x"), true);
}

#[test]
fn test_comprehension() {
    assert_int(&run_get("x = [i * i for i in 0..5].len()", "x"), 5);
    assert_int(&run_get("x = [i for i in 0..10 if i % 2 == 0].len()", "x"), 5);
}

#[test]
fn test_range_iteration() {
    assert_int(&run_get("x = 0\nfor i in 0..5 { x += i }", "x"), 10);
}

#[test]
fn test_index_access() {
    assert_int(&run_get("x = [10, 20, 30][1]", "x"), 20);
    assert_int(&run_get("m = {\"a\": 42}\nx = m[\"a\"]", "x"), 42);
}

#[test]
fn test_index_assignment() {
    assert_int(&run_get("l = [1, 2, 3]\nl[1] = 99\nx = l[1]", "x"), 99);
}

// ═══════════════════════════════════════════════════════════════
// Classes
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_class_basic() {
    let src = "class C { val: Int }\no = C(42)\nx = o.val";
    assert_int(&run_get(src, "x"), 42);
}

#[test]
fn test_class_with_init() {
    let src = "class C {\n  val: Int\n  init(v) { self.val = v * 2 }\n}\no = C(21)\nx = o.val";
    assert_int(&run_get(src, "x"), 42);
}

#[test]
fn test_class_methods() {
    let src = "class C {\n  n: Int\n  init(n) { self.n = n }\n  def double() { return self.n * 2 }\n}\no = C(21)\nx = o.double()";
    assert_int(&run_get(src, "x"), 42);
}

#[test]
fn test_class_field_mutation() {
    let src = "class C {\n  n: Int\n  init(n) { self.n = n }\n}\no = C(1)\no.n = 42\nx = o.n";
    assert_int(&run_get(src, "x"), 42);
}

#[test]
fn test_class_inheritance() {
    let src = r#"
class Animal {
    name: Str
    init(name) { self.name = name }
    def speak() { return "..." }
}
class Dog : Animal {
    def speak() { return "Woof" }
}
d = Dog("Rex")
x = d.speak()
"#;
    assert_str(&run_get(src, "x"), "Woof");
}

#[test]
fn test_class_inherited_method() {
    let src = r#"
class Base {
    def greet() { return "hello" }
}
class Child : Base {
}
c = Child()
x = c.greet()
"#;
    assert_str(&run_get(src, "x"), "hello");
}

// ═══════════════════════════════════════════════════════════════
// Pipeline and special operators
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_pipeline() {
    assert_int(&run_get("f = x -> x + 1\ng = x -> x * 2\nx = 5 |> f |> g", "x"), 12);
}

#[test]
fn test_nil_coalescing_non_nil() {
    assert_int(&run_get("x = 10 ?? 99", "x"), 10);
}

#[test]
fn test_optional_chaining_nil() {
    // optional chaining on nil returns nil
    assert!(matches!(run_get("x = nil?.foo", "x"), Value::Nil));
}

// ═══════════════════════════════════════════════════════════════
// Math builtins
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_math_sqrt() {
    assert_float_approx(&run_get("x = sqrt(16.0)", "x"), 4.0);
}

#[test]
fn test_math_abs() {
    assert_int(&run_get("x = abs(-42)", "x"), 42);
}

#[test]
fn test_math_constants() {
    assert_float_approx(&run_get("x = PI", "x"), std::f64::consts::PI);
}

// ═══════════════════════════════════════════════════════════════
// Truthiness
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_truthiness() {
    assert_int(&run_get("x = if 1 then 1 else 0", "x"), 1);
    assert_int(&run_get("x = if 0 then 1 else 0", "x"), 0);
    assert_int(&run_get("x = if \"\" then 1 else 0", "x"), 0);
    assert_int(&run_get("x = if \"hi\" then 1 else 0", "x"), 1);
    assert_int(&run_get("x = if nil then 1 else 0", "x"), 0);
}

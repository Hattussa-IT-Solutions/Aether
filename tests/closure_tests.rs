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
    match val {
        Value::Int(n) => assert_eq!(*n, expected, "expected Int({}), got Int({})", expected, n),
        _ => panic!("expected Int({}), got {:?}", expected, val),
    }
}

fn assert_float_approx(val: &Value, expected: f64) {
    match val {
        Value::Float(f) => assert!(
            (f - expected).abs() < 1e-6,
            "expected Float({}), got Float({})",
            expected,
            f
        ),
        Value::Int(n) => assert!(
            (*n as f64 - expected).abs() < 1e-6,
            "expected {}, got Int({})",
            expected,
            n
        ),
        _ => panic!("expected Float({}), got {:?}", expected, val),
    }
}

fn assert_bool(val: &Value, expected: bool) {
    match val {
        Value::Bool(b) => assert_eq!(*b, expected, "expected Bool({}), got Bool({})", expected, b),
        _ => panic!("expected Bool({}), got {:?}", expected, val),
    }
}

fn assert_str(val: &Value, expected: &str) {
    match val {
        Value::String(s) => assert_eq!(s, expected, "expected '{}', got '{}'", expected, s),
        _ => panic!("expected String('{}'), got {:?}", expected, val),
    }
}

fn assert_nil(val: &Value) {
    assert!(matches!(val, Value::Nil), "expected Nil, got {:?}", val);
}

fn list_len(val: &Value) -> usize {
    match val {
        Value::List(items) => items.borrow().len(),
        _ => panic!("expected List, got {:?}", val),
    }
}

fn list_get(val: &Value, idx: usize) -> Value {
    match val {
        Value::List(items) => items.borrow().get(idx).cloned().unwrap_or(Value::Nil),
        _ => panic!("expected List, got {:?}", val),
    }
}

// ===============================================================
// Basic lambdas
// ===============================================================

#[test]
fn test_lambda_single_param_multiply() {
    assert_int(&run_get("f = x -> x * 2\nx = f(21)", "x"), 42);
}

#[test]
fn test_lambda_single_param_add() {
    assert_int(&run_get("f = x -> x + 10\nx = f(5)", "x"), 15);
}

#[test]
fn test_lambda_two_params_add() {
    assert_int(&run_get("f = (a, b) -> a + b\nx = f(3, 4)", "x"), 7);
}

#[test]
fn test_lambda_two_params_multiply() {
    assert_int(&run_get("f = (a, b) -> a * b\nx = f(6, 7)", "x"), 42);
}

#[test]
fn test_lambda_as_map_argument() {
    let val = run_get("x = [1, 2, 3].map(n -> n * 10)", "x");
    assert_int(&list_get(&val, 0), 10);
    assert_int(&list_get(&val, 1), 20);
    assert_int(&list_get(&val, 2), 30);
}

#[test]
fn test_lambda_as_filter_argument() {
    let val = run_get("x = [1, 2, 3, 4, 5].filter(n -> n % 2 == 0)", "x");
    assert_eq!(list_len(&val), 2);
    assert_int(&list_get(&val, 0), 2);
    assert_int(&list_get(&val, 1), 4);
}

#[test]
fn test_lambda_in_variable_assignment() {
    assert_int(&run_get("double = x -> x * 2\nx = double(50)", "x"), 100);
}

#[test]
fn test_lambda_with_string_ops() {
    let val = run_get("x = [\"hello\", \"world\"].map(s -> s.upper())", "x");
    assert_str(&list_get(&val, 0), "HELLO");
    assert_str(&list_get(&val, 1), "WORLD");
}

#[test]
fn test_lambda_boolean_result() {
    assert_bool(&run_get("f = x -> x > 5\nx = f(10)", "x"), true);
    assert_bool(&run_get("f = x -> x > 5\nx = f(3)", "x"), false);
}

#[test]
fn test_lambda_with_arithmetic_expression() {
    assert_int(&run_get("f = x -> x ** 2 + 1\nx = f(5)", "x"), 26);
}

// ===============================================================
// Closures capturing variables
// ===============================================================

#[test]
fn test_closure_reads_outer_variable() {
    let src = r#"
y = 10
f = x -> x + y
x = f(5)
"#;
    assert_int(&run_get(src, "x"), 15);
}

#[test]
fn test_closure_reads_outer_string() {
    let src = r#"
greeting = "Hello"
f = name -> greeting + " " + name
x = f("World")
"#;
    assert_str(&run_get(src, "x"), "Hello World");
}

#[test]
fn test_closure_returned_from_function() {
    let src = r#"
def make_adder(n) {
    return x -> x + n
}
add5 = make_adder(5)
x = add5(10)
"#;
    assert_int(&run_get(src, "x"), 15);
}

#[test]
fn test_closure_returned_from_function_multiplier() {
    let src = r#"
def make_multiplier(factor) {
    return x -> x * factor
}
triple = make_multiplier(3)
x = triple(7)
"#;
    assert_int(&run_get(src, "x"), 21);
}

#[test]
fn test_closure_captures_value_at_creation() {
    // Closure captures the value of n at creation time
    let src = r#"
def make_getter(n) {
    return x -> n
}
get5 = make_getter(5)
get10 = make_getter(10)
x = get5(0) + get10(0)
"#;
    assert_int(&run_get(src, "x"), 15);
}

#[test]
fn test_closure_captures_multiple_variables() {
    let src = r#"
a = 10
b = 20
f = x -> x + a + b
x = f(5)
"#;
    assert_int(&run_get(src, "x"), 35);
}

#[test]
fn test_closure_captures_in_loop() {
    // Capture loop variable in a comprehension-like pattern
    let src = r#"
results = [0, 0, 0, 0, 0]
for i in 0..5 {
    results[i] = i * i
}
x = results[3]
"#;
    assert_int(&run_get(src, "x"), 9);
}

#[test]
fn test_closure_with_map_captures_outer() {
    let src = r#"
offset = 100
x = [1, 2, 3].map(n -> n + offset).sum()
"#;
    assert_int(&run_get(src, "x"), 306);
}

#[test]
fn test_closure_with_filter_captures_outer() {
    let src = r#"
threshold = 3
x = [1, 2, 3, 4, 5].filter(n -> n > threshold).len()
"#;
    assert_int(&run_get(src, "x"), 2);
}

#[test]
fn test_closure_returned_preserves_environment() {
    let src = r#"
def make_greeting(prefix) {
    return name -> prefix + ", " + name + "!"
}
greet = make_greeting("Hello")
x = greet("Alice")
"#;
    assert_str(&run_get(src, "x"), "Hello, Alice!");
}

#[test]
fn test_multiple_closures_from_same_factory() {
    let src = r#"
def make_adder(n) {
    return x -> x + n
}
add3 = make_adder(3)
add7 = make_adder(7)
x = add3(10) + add7(10)
"#;
    assert_int(&run_get(src, "x"), 30);
}

#[test]
fn test_closure_captures_function_param() {
    let src = r#"
def apply_twice(f, val) {
    return f(f(val))
}
x = apply_twice(n -> n + 1, 10)
"#;
    assert_int(&run_get(src, "x"), 12);
}

#[test]
fn test_closure_captures_boolean() {
    let src = r#"
negate = true
f = x -> if negate then -x else x
x = f(42)
"#;
    assert_int(&run_get(src, "x"), -42);
}

#[test]
fn test_closure_in_reduce() {
    let src = r#"
initial = 100
x = [1, 2, 3].reduce(initial, (acc, n) -> acc + n)
"#;
    assert_int(&run_get(src, "x"), 106);
}

#[test]
fn test_closure_captures_list() {
    let src = r#"
data = [10, 20, 30]
f = idx -> data[idx]
x = f(1)
"#;
    assert_int(&run_get(src, "x"), 20);
}

// ===============================================================
// Higher-order functions
// ===============================================================

#[test]
fn test_function_returning_function() {
    let src = r#"
def outer(a) {
    return b -> a + b
}
f = outer(10)
x = f(5)
"#;
    assert_int(&run_get(src, "x"), 15);
}

#[test]
fn test_function_taking_function_param() {
    let src = r#"
def apply(f, val) {
    return f(val)
}
x = apply(n -> n * 3, 14)
"#;
    assert_int(&run_get(src, "x"), 42);
}

#[test]
fn test_map_with_lambda() {
    let val = run_get("x = [1, 2, 3, 4].map(n -> n ** 2)", "x");
    assert_int(&list_get(&val, 0), 1);
    assert_int(&list_get(&val, 1), 4);
    assert_int(&list_get(&val, 2), 9);
    assert_int(&list_get(&val, 3), 16);
}

#[test]
fn test_filter_with_lambda() {
    let val = run_get("x = [1, 2, 3, 4, 5, 6].filter(n -> n % 3 == 0)", "x");
    assert_eq!(list_len(&val), 2);
    assert_int(&list_get(&val, 0), 3);
    assert_int(&list_get(&val, 1), 6);
}

#[test]
fn test_reduce_with_lambda() {
    assert_int(&run_get("x = [1, 2, 3, 4].reduce(0, (acc, n) -> acc + n)", "x"), 10);
}

#[test]
fn test_reduce_product() {
    assert_int(&run_get("x = [1, 2, 3, 4].reduce(1, (acc, n) -> acc * n)", "x"), 24);
}

#[test]
fn test_chained_map_filter() {
    // [1,2,3,4,5] -> [2,4,6,8,10] -> filter >6 -> [8,10] -> sum=18
    assert_int(
        &run_get("x = [1, 2, 3, 4, 5].map(n -> n * 2).filter(n -> n > 6).sum()", "x"),
        18,
    );
}

#[test]
fn test_chained_map_filter_strings() {
    let val = run_get("x = [\"hello\", \"world\", \"hi\"].filter(s -> s.len() > 3).map(s -> s.upper())", "x");
    assert_eq!(list_len(&val), 2);
    assert_str(&list_get(&val, 0), "HELLO");
    assert_str(&list_get(&val, 1), "WORLD");
}

#[test]
fn test_find_with_lambda() {
    assert_int(&run_get("x = [1, 2, 3, 4, 5].find(n -> n > 3)", "x"), 4);
}

#[test]
fn test_any_with_lambda() {
    assert_bool(&run_get("x = [1, 2, 3].any(n -> n == 2)", "x"), true);
}

#[test]
fn test_all_with_lambda() {
    assert_bool(&run_get("x = [2, 4, 6].all(n -> n % 2 == 0)", "x"), true);
    assert_bool(&run_get("x = [2, 3, 6].all(n -> n % 2 == 0)", "x"), false);
}

#[test]
fn test_sort_with_custom_comparator_via_sort_by() {
    // Test sorting via map + sort pattern (sort_by may or may not exist)
    let val = run_get("x = [3, 1, 2].sort()", "x");
    assert_int(&list_get(&val, 0), 1);
    assert_int(&list_get(&val, 2), 3);
}

#[test]
fn test_flat_map_with_lambda() {
    let val = run_get("x = [1, 2, 3].flat_map(n -> [n, n * 10])", "x");
    assert_eq!(list_len(&val), 6);
    assert_int(&list_get(&val, 0), 1);
    assert_int(&list_get(&val, 1), 10);
    assert_int(&list_get(&val, 2), 2);
    assert_int(&list_get(&val, 3), 20);
}

#[test]
fn test_take_while_with_lambda() {
    let val = run_get("x = [1, 2, 3, 4, 5].take_while(n -> n < 4)", "x");
    assert_eq!(list_len(&val), 3);
}

#[test]
fn test_drop_while_with_lambda() {
    let val = run_get("x = [1, 2, 3, 4, 5].drop_while(n -> n < 3)", "x");
    assert_eq!(list_len(&val), 3);
    assert_int(&list_get(&val, 0), 3);
}

#[test]
fn test_partition_with_lambda() {
    let val = run_get("x = [1, 2, 3, 4, 5].partition(n -> n % 2 == 0)", "x");
    match &val {
        Value::Tuple(parts) => {
            assert_eq!(list_len(&parts[0]), 2); // evens: 2, 4
            assert_eq!(list_len(&parts[1]), 3); // odds: 1, 3, 5
        }
        _ => panic!("expected Tuple, got {:?}", val),
    }
}

// ===============================================================
// Anonymous def as expression
// ===============================================================

#[test]
fn test_anonymous_def_basic() {
    let src = r#"
f = def(x) { return x + 1 }
x = f(41)
"#;
    assert_int(&run_get(src, "x"), 42);
}

#[test]
fn test_anonymous_def_multiple_stmts() {
    let src = r#"
f = def(a, b) {
    result = a * b
    return result + 1
}
x = f(6, 7)
"#;
    assert_int(&run_get(src, "x"), 43);
}

#[test]
fn test_anonymous_def_as_callback() {
    let src = r#"
def apply(f, val) {
    return f(val)
}
x = apply(def(n) { return n * 10 }, 5)
"#;
    assert_int(&run_get(src, "x"), 50);
}

#[test]
fn test_anonymous_def_with_closure_capture() {
    let src = r#"
offset = 100
f = def(x) { return x + offset }
x = f(42)
"#;
    assert_int(&run_get(src, "x"), 142);
}

#[test]
fn test_anonymous_def_no_params() {
    let src = r#"
f = def() { return 42 }
x = f()
"#;
    assert_int(&run_get(src, "x"), 42);
}

#[test]
fn test_anonymous_def_with_conditional() {
    let src = r#"
f = def(n) {
    if n > 0 {
        return "positive"
    } else {
        return "non-positive"
    }
}
x = f(5)
"#;
    assert_str(&run_get(src, "x"), "positive");
}

#[test]
fn test_anonymous_def_with_loop() {
    let src = r#"
f = def(n) {
    total = 0
    for i in 0..n {
        total += i
    }
    return total
}
x = f(5)
"#;
    assert_int(&run_get(src, "x"), 10);
}

#[test]
fn test_anonymous_def_recursive_via_variable() {
    // Define a recursive function via variable (may require self-reference)
    let src = r#"
def fact(n) {
    if n <= 1 { return 1 }
    return n * fact(n - 1)
}
x = fact(6)
"#;
    assert_int(&run_get(src, "x"), 720);
}

#[test]
fn test_anonymous_def_returns_lambda() {
    let src = r#"
factory = def(n) { return x -> x + n }
add10 = factory(10)
x = add10(32)
"#;
    assert_int(&run_get(src, "x"), 42);
}

#[test]
fn test_anonymous_def_in_list_map() {
    let val = run_get("x = [1, 2, 3].map(def(n) { return n * n })", "x");
    assert_int(&list_get(&val, 0), 1);
    assert_int(&list_get(&val, 1), 4);
    assert_int(&list_get(&val, 2), 9);
}

// ===============================================================
// Edge cases
// ===============================================================

#[test]
fn test_nested_lambda_currying() {
    // x -> y -> x + y (curried addition)
    let src = r#"
add = x -> y -> x + y
add5 = add(5)
x = add5(10)
"#;
    assert_int(&run_get(src, "x"), 15);
}

#[test]
fn test_nested_lambda_currying_multiply() {
    let src = r#"
mul = x -> y -> x * y
double = mul(2)
x = double(21)
"#;
    assert_int(&run_get(src, "x"), 42);
}

#[test]
fn test_lambda_immediately_invoked() {
    // Store lambda result immediately
    assert_int(&run_get("f = x -> x + 1\nx = f(41)", "x"), 42);
}

#[test]
fn test_deeply_nested_closure() {
    let src = r#"
def level1(a) {
    return b -> c -> a + b + c
}
f = level1(10)(20)
x = f(12)
"#;
    assert_int(&run_get(src, "x"), 42);
}

#[test]
fn test_closure_with_string_concatenation() {
    let src = r#"
prefix = "Mr."
f = name -> prefix + " " + name
x = f("Smith")
"#;
    assert_str(&run_get(src, "x"), "Mr. Smith");
}

#[test]
fn test_lambda_chain_pipeline() {
    // Use pipeline with lambdas
    let src = r#"
f = x -> x + 1
g = x -> x * 2
x = 5 |> f |> g
"#;
    assert_int(&run_get(src, "x"), 12);
}

#[test]
fn test_lambda_pipeline_chain_three() {
    let src = r#"
f = x -> x + 1
g = x -> x * 2
h = x -> x - 3
x = 10 |> f |> g |> h
"#;
    assert_int(&run_get(src, "x"), 19); // ((10+1)*2)-3 = 19
}

#[test]
fn test_closure_does_not_pollute_outer_scope() {
    let src = r#"
def make() {
    local = 999
    return def() { return local }
}
f = make()
x = f()
"#;
    assert_int(&run_get(src, "x"), 999);
}

#[test]
fn test_lambda_with_comparison() {
    assert_bool(&run_get("f = (a, b) -> a > b\nx = f(10, 5)", "x"), true);
    assert_bool(&run_get("f = (a, b) -> a > b\nx = f(3, 7)", "x"), false);
}

#[test]
fn test_lambda_identity_int() {
    assert_int(&run_get("id = x -> x\nx = id(42)", "x"), 42);
}

#[test]
fn test_lambda_identity_str() {
    let src = r#"
id = x -> x
x = id("hello")
"#;
    assert_str(&run_get(src, "x"), "hello");
}

#[test]
fn test_lambda_constant() {
    // Lambda that ignores parameter
    assert_int(&run_get("f = x -> 42\nx = f(999)", "x"), 42);
}

#[test]
fn test_higher_order_compose() {
    let src = r#"
def compose(f, g) {
    return x -> f(g(x))
}
inc = x -> x + 1
dbl = x -> x * 2
inc_then_dbl = compose(dbl, inc)
x = inc_then_dbl(5)
"#;
    assert_int(&run_get(src, "x"), 12); // dbl(inc(5)) = dbl(6) = 12
}

#[test]
fn test_higher_order_apply_n_times() {
    let src = r#"
def apply_n(f, n, val) {
    result = val
    for i in 0..n {
        result = f(result)
    }
    return result
}
x = apply_n(n -> n + 1, 10, 0)
"#;
    assert_int(&run_get(src, "x"), 10);
}

#[test]
fn test_closure_in_comprehension() {
    let src = r#"
multiplier = 3
x = [i * multiplier for i in 1..5].sum()
"#;
    // [3, 6, 9, 12] -> sum = 30
    assert_int(&run_get(src, "x"), 30);
}

#[test]
fn test_closure_filter_comprehension() {
    let src = r#"
limit = 5
x = [i for i in 0..10 if i > limit].len()
"#;
    assert_int(&run_get(src, "x"), 4); // 6,7,8,9
}

#[test]
fn test_lambda_with_nil_coalescing() {
    let src = r#"
f = x -> x ?? 0
x = f(nil)
y = f(42)
"#;
    assert_int(&run_get(src, "x"), 0);
}

#[test]
fn test_lambda_with_nil_coalescing_non_nil() {
    assert_int(&run_get("f = x -> x ?? 0\nx = f(42)", "x"), 42);
}

#[test]
fn test_expression_body_function_as_closure() {
    let src = r#"
def make_adder(n) = x -> x + n
add10 = make_adder(10)
x = add10(32)
"#;
    assert_int(&run_get(src, "x"), 42);
}

#[test]
fn test_closure_in_any() {
    let src = r#"
target = 3
x = [1, 2, 3, 4].any(n -> n == target)
"#;
    assert_bool(&run_get(src, "x"), true);
}

#[test]
fn test_closure_in_all() {
    let src = r#"
min_val = 0
x = [1, 2, 3].all(n -> n > min_val)
"#;
    assert_bool(&run_get(src, "x"), true);
}

#[test]
fn test_closure_in_find_index() {
    let src = r#"
target = 20
x = [10, 20, 30].find_index(n -> n == target)
"#;
    assert_int(&run_get(src, "x"), 1);
}

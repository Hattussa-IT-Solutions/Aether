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
        Value::Int(n) => assert_eq!(*n, expected, "expected {}, got {}", expected, n),
        _ => panic!("expected Int({}), got {:?}", expected, val),
    }
}

fn assert_float(val: &Value, expected: f64) {
    match val {
        Value::Float(f) => assert!((f - expected).abs() < 0.01, "expected {}, got {}", expected, f),
        Value::Int(n) => assert!((*n as f64 - expected).abs() < 0.01),
        _ => panic!("expected Float({}), got {:?}", expected, val),
    }
}

fn assert_str(val: &Value, expected: &str) {
    match val {
        Value::String(s) => assert_eq!(s, expected, "expected '{}', got '{}'", expected, s),
        _ => panic!("expected String('{}'), got {:?}", expected, val),
    }
}

fn assert_bool(val: &Value, expected: bool) {
    match val {
        Value::Bool(b) => assert_eq!(*b, expected),
        _ => panic!("expected Bool({}), got {:?}", expected, val),
    }
}

// ═══════════════════════════════════════════════════════════════
// If / Else (15 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_if_true() {
    assert_int(&run_get("x = 0\nif true { x = 1 }", "x"), 1);
}

#[test]
fn test_if_false() {
    assert_int(&run_get("x = 0\nif false { x = 1 }", "x"), 0);
}

#[test]
fn test_if_else_true_branch() {
    assert_int(&run_get("x = 0\nif true { x = 1 } else { x = 2 }", "x"), 1);
}

#[test]
fn test_if_else_false_branch() {
    assert_int(&run_get("x = 0\nif false { x = 1 } else { x = 2 }", "x"), 2);
}

#[test]
fn test_if_else_if_first() {
    let src = "x = 0\nn = 25\nif n > 20 { x = 1 } else if n > 10 { x = 2 } else { x = 3 }";
    assert_int(&run_get(src, "x"), 1);
}

#[test]
fn test_if_else_if_second() {
    let src = "x = 0\nn = 15\nif n > 20 { x = 1 } else if n > 10 { x = 2 } else { x = 3 }";
    assert_int(&run_get(src, "x"), 2);
}

#[test]
fn test_if_else_if_else() {
    let src = "x = 0\nn = 5\nif n > 20 { x = 1 } else if n > 10 { x = 2 } else { x = 3 }";
    assert_int(&run_get(src, "x"), 3);
}

#[test]
fn test_if_chain_four_branches() {
    let src = r#"
x = 0
n = 35
if n > 100 { x = 1 } else if n > 50 { x = 2 } else if n > 30 { x = 3 } else { x = 4 }
"#;
    assert_int(&run_get(src, "x"), 3);
}

#[test]
fn test_nested_if_two_levels() {
    let src = r#"
x = 0
if true {
    if true { x = 1 } else { x = 2 }
} else { x = 3 }
"#;
    assert_int(&run_get(src, "x"), 1);
}

#[test]
fn test_nested_if_five_levels() {
    let src = r#"
x = 0
if true {
  if true {
    if true {
      if true {
        if true { x = 5 }
      }
    }
  }
}
"#;
    assert_int(&run_get(src, "x"), 5);
}

#[test]
fn test_if_expression_true() {
    assert_str(&run_get("x = if true then \"pos\" else \"neg\"", "x"), "pos");
}

#[test]
fn test_if_expression_false() {
    assert_str(&run_get("x = if false then \"pos\" else \"neg\"", "x"), "neg");
}

#[test]
fn test_if_expression_with_comparison() {
    assert_str(&run_get("n = 10\nx = if n > 5 then \"big\" else \"small\"", "x"), "big");
}

#[test]
fn test_if_expression_numeric() {
    assert_int(&run_get("x = if 5 > 3 then 100 else 200", "x"), 100);
}

#[test]
fn test_if_with_and_condition() {
    assert_int(&run_get("x = 0\na = 5\nb = 10\nif a > 3 and b > 8 { x = 1 }", "x"), 1);
}

#[test]
fn test_if_with_or_condition() {
    assert_int(&run_get("x = 0\nif false or true { x = 1 }", "x"), 1);
}

#[test]
fn test_guard_let_with_value() {
    let src = r#"
def f(val) {
    guard let n = val else { return -1 }
    return n * 3
}
x = f(7)
"#;
    assert_int(&run_get(src, "x"), 21);
}

#[test]
fn test_guard_let_with_nil() {
    let src = r#"
def f(val) {
    guard let n = val else { return -1 }
    return n * 3
}
x = f(nil)
"#;
    assert_int(&run_get(src, "x"), -1);
}

// ═══════════════════════════════════════════════════════════════
// Match (15 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_match_int_literal() {
    assert_str(&run_get("x = match 1 { 1 -> \"one\"\n2 -> \"two\"\n_ -> \"other\" }", "x"), "one");
}

#[test]
fn test_match_string_literal() {
    assert_int(&run_get("x = match \"hi\" { \"hi\" -> 1\n\"bye\" -> 2\n_ -> 0 }", "x"), 1);
}

#[test]
fn test_match_bool_literal() {
    assert_str(&run_get("x = match true { true -> \"yes\"\nfalse -> \"no\" }", "x"), "yes");
}

#[test]
fn test_match_range_lower_bound() {
    assert_str(&run_get("x = match 1 { 1..10 -> \"small\"\n_ -> \"big\" }", "x"), "small");
}

#[test]
fn test_match_range_upper_bound() {
    assert_str(&run_get("x = match 9 { 1..10 -> \"small\"\n_ -> \"big\" }", "x"), "small");
}

#[test]
fn test_match_range_out_of_range() {
    assert_str(&run_get("x = match 15 { 1..10 -> \"small\"\n_ -> \"big\" }", "x"), "big");
}

#[test]
fn test_match_wildcard_default() {
    assert_str(&run_get("x = match 999 { 1 -> \"one\"\n_ -> \"default\" }", "x"), "default");
}

#[test]
fn test_match_destructure_ok_value() {
    assert_int(&run_get("r = Ok(100)\nx = match r { Ok(v) -> v\nErr(e) -> 0 }", "x"), 100);
}

#[test]
fn test_match_destructure_err_value() {
    assert_int(&run_get("r = Err(\"fail\")\nx = match r { Ok(v) -> v\nErr(e) -> -1 }", "x"), -1);
}

#[test]
fn test_match_as_expression() {
    let src = r#"
score = 85
grade = match score {
    90..100 -> "A"
    80..90 -> "B"
    70..80 -> "C"
    _ -> "F"
}
x = grade
"#;
    assert_str(&run_get(src, "x"), "B");
}

#[test]
fn test_match_with_guard() {
    assert_str(&run_get("x = match 6 { n if n % 2 == 0 -> \"even\"\n_ -> \"odd\" }", "x"), "even");
}

#[test]
fn test_match_guard_odd() {
    assert_str(&run_get("x = match 7 { n if n % 2 == 0 -> \"even\"\n_ -> \"odd\" }", "x"), "odd");
}

#[test]
fn test_match_enum_variant() {
    assert_float(&run_get("x = match .Circle(3.0) { .Circle(r) -> r * r\n_ -> 0.0 }", "x"), 9.0);
}

#[test]
fn test_match_returning_from_function() {
    let src = r#"
def classify(n) {
    match n {
        0 -> "zero"
        1 -> "one"
        _ -> "many"
    }
}
x = classify(1)
"#;
    assert_str(&run_get(src, "x"), "one");
}

#[test]
fn test_match_many_arms() {
    let src = r#"
x = match 7 {
    1 -> "one"
    2 -> "two"
    3 -> "three"
    4 -> "four"
    5 -> "five"
    6 -> "six"
    7 -> "seven"
    8 -> "eight"
    9 -> "nine"
    10 -> "ten"
    _ -> "other"
}
"#;
    assert_str(&run_get(src, "x"), "seven");
}

// ═══════════════════════════════════════════════════════════════
// Loops (20 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_for_range_exclusive() {
    assert_int(&run_get("x = 0\nfor i in 0..10 { x += 1 }", "x"), 10);
}

#[test]
fn test_for_range_exclusive_sum() {
    assert_int(&run_get("x = 0\nfor i in 0..5 { x += i }", "x"), 10);
}

#[test]
fn test_for_range_inclusive() {
    assert_int(&run_get("x = 0\nfor i in 0..=10 { x += 1 }", "x"), 11);
}

#[test]
fn test_for_range_inclusive_sum() {
    assert_int(&run_get("x = 0\nfor i in 1..=5 { x += i }", "x"), 15);
}

#[test]
fn test_for_item_in_list() {
    assert_int(&run_get("x = 0\nfor item in [10, 20, 30] { x += item }", "x"), 60);
}

#[test]
fn test_for_enumerate_indices() {
    assert_int(&run_get("x = 0\nfor i, val in [10, 20, 30] { x += i }", "x"), 3);
}

#[test]
fn test_for_enumerate_values() {
    assert_int(&run_get("x = 0\nfor i, val in [10, 20, 30] { x += val }", "x"), 60);
}

#[test]
fn test_for_map_iteration() {
    assert_int(&run_get("x = 0\nm = {\"a\": 1, \"b\": 2, \"c\": 3}\nfor (key, val) in m { x += val }", "x"), 6);
}

#[test]
fn test_loop_n_times() {
    assert_int(&run_get("x = 0\nloop 10 times { x += 1 }", "x"), 10);
}

#[test]
fn test_loop_zero_times() {
    assert_int(&run_get("x = 0\nloop 0 times { x += 1 }", "x"), 0);
}

#[test]
fn test_loop_while() {
    assert_int(&run_get("x = 0\nloop while x < 10 { x += 1 }", "x"), 10);
}

#[test]
fn test_loop_while_never_enters() {
    assert_int(&run_get("x = 100\nloop while x < 10 { x += 1 }", "x"), 100);
}

#[test]
fn test_loop_until() {
    assert_int(&run_get("x = 0\nloop { x += 1 } until x >= 5", "x"), 5);
}

#[test]
fn test_break_exits_loop() {
    assert_int(&run_get("x = 0\nfor i in 0..1000 { if i >= 3 { break }\nx += 1 }", "x"), 3);
}

#[test]
fn test_next_skips_iteration() {
    // Sum only odd numbers from 0..10
    assert_int(&run_get("x = 0\nfor i in 0..10 { next if i % 2 == 0\nx += i }", "x"), 25);
}

#[test]
fn test_next_if_compound() {
    // Skip multiples of 3
    assert_int(&run_get("x = 0\nfor i in 1..10 { next if i % 3 == 0\nx += 1 }", "x"), 6);
}

#[test]
fn test_nested_loops() {
    let src = r#"
x = 0
for i in 0..3 {
    for j in 0..4 {
        x += 1
    }
}
"#;
    assert_int(&run_get(src, "x"), 12);
}

#[test]
fn test_labeled_break_outer() {
    let src = r#"
x = 0
for:outer a in 0..5 {
    for b in 0..5 {
        if a == 2 and b == 1 { break:outer }
        x += 1
    }
}
"#;
    // a=0: 5 iterations, a=1: 5 iterations, a=2: b=0 then b=1 breaks -> 5+5+1=11
    assert_int(&run_get(src, "x"), 11);
}

#[test]
fn test_for_step_by_two() {
    assert_int(&run_get("x = 0\nfor i in 0..10 step 2 { x += 1 }", "x"), 5);
}

#[test]
fn test_for_step_by_three() {
    // 0, 3, 6, 9 -> 4 iterations
    assert_int(&run_get("x = 0\nfor i in 0..10 step 3 { x += 1 }", "x"), 4);
}

#[test]
fn test_for_step_sum() {
    // 0, 2, 4, 6, 8 -> sum = 20
    assert_int(&run_get("x = 0\nfor i in 0..10 step 2 { x += i }", "x"), 20);
}

#[test]
fn test_loop_with_complex_body() {
    let src = r#"
x = 0
for i in 0..10 {
    if i % 2 == 0 {
        x += i * 2
    } else {
        x += i
    }
}
"#;
    // evens: 0,2,4,6,8 -> doubled: 0+4+8+12+16=40
    // odds: 1,3,5,7,9 -> 25
    // total: 65
    assert_int(&run_get(src, "x"), 65);
}

// ═══════════════════════════════════════════════════════════════
// Comprehensions (10 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_comprehension_squares() {
    assert_int(&run_get("x = [i * i for i in 0..5].len()", "x"), 5);
}

#[test]
fn test_comprehension_squares_sum() {
    assert_int(&run_get("x = [i * i for i in 0..5].sum()", "x"), 30);
}

#[test]
fn test_comprehension_with_filter() {
    assert_int(&run_get("x = [i for i in 0..10 if i % 2 == 0].len()", "x"), 5);
}

#[test]
fn test_comprehension_filter_sum() {
    // Even numbers: 0, 2, 4, 6, 8 -> sum = 20
    assert_int(&run_get("x = [i for i in 0..10 if i % 2 == 0].sum()", "x"), 20);
}

#[test]
fn test_comprehension_with_expression() {
    // [2, 4, 6, 8, 10]
    assert_int(&run_get("x = [(i + 1) * 2 for i in 0..5].sum()", "x"), 30);
}

#[test]
fn test_comprehension_strings() {
    assert_int(&run_get("x = [\"x\" for i in 0..3].len()", "x"), 3);
}

#[test]
fn test_comprehension_filter_large() {
    // Numbers > 5 from 0..10: 6,7,8,9 -> 4 items
    assert_int(&run_get("x = [i for i in 0..10 if i > 5].len()", "x"), 4);
}

#[test]
fn test_comprehension_empty_filter() {
    assert_int(&run_get("x = [i for i in 0..5 if i > 100].len()", "x"), 0);
}

#[test]
fn test_comprehension_over_list() {
    assert_int(&run_get("x = [v * 10 for v in [1, 2, 3]].sum()", "x"), 60);
}

#[test]
fn test_comprehension_combined_ops() {
    // Square even numbers from 0..8: 0, 4, 16, 36 -> sum = 56
    assert_int(&run_get("x = [i * i for i in 0..8 if i % 2 == 0].sum()", "x"), 56);
}

// ═══════════════════════════════════════════════════════════════
// Error handling (20 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_try_catch_basic() {
    let src = "x = 0\ntry {\n  throw \"oops\"\n} catch any as e {\n  x = 1\n}";
    assert_int(&run_get(src, "x"), 1);
}

#[test]
fn test_try_no_error() {
    let src = "x = 0\ntry {\n  x = 42\n} catch any as e {\n  x = -1\n}";
    assert_int(&run_get(src, "x"), 42);
}

#[test]
fn test_try_catch_finally_no_error() {
    let src = "x = 0\ntry {\n  x = 10\n} catch any as e {\n  x = -1\n} finally {\n  x += 5\n}";
    assert_int(&run_get(src, "x"), 15);
}

#[test]
fn test_try_catch_finally_with_error() {
    let src = "x = 0\ntry {\n  throw \"err\"\n} catch any as e {\n  x = 10\n} finally {\n  x += 5\n}";
    assert_int(&run_get(src, "x"), 15);
}

#[test]
fn test_try_finally_always_runs() {
    let src = "x = 0\ntry {\n  x = 1\n} finally {\n  x += 100\n}";
    assert_int(&run_get(src, "x"), 101);
}

#[test]
fn test_nested_try_catch() {
    let src = r#"
x = 0
try {
    try {
        throw "inner"
    } catch any as e {
        x += 1
    }
    x += 10
} catch any as e {
    x += 100
}
"#;
    assert_int(&run_get(src, "x"), 11);
}

#[test]
fn test_nested_try_catch_outer() {
    let src = r#"
x = 0
try {
    try {
        x += 1
    } catch any as e {
        x += 10
    }
    throw "outer"
} catch any as e {
    x += 100
}
"#;
    assert_int(&run_get(src, "x"), 101);
}

#[test]
fn test_throw_custom_string() {
    let src = r#"
x = "none"
try {
    throw "custom error"
} catch any as e {
    x = "caught"
}
"#;
    assert_str(&run_get(src, "x"), "caught");
}

#[test]
fn test_error_propagation_ok() {
    let src = r#"
def safe_parse(s) {
    n = s.parse_int()?
    return Ok(n + 1)
}
x = safe_parse("10")
"#;
    let val = run_get(src, "x");
    if let Value::Ok(inner) = val {
        assert_int(&inner, 11);
    } else {
        panic!("expected Ok, got {:?}", val);
    }
}

#[test]
fn test_error_propagation_err() {
    let src = r#"
def safe_parse(s) {
    n = s.parse_int()?
    return Ok(n + 1)
}
x = safe_parse("abc")
"#;
    let val = run_get(src, "x");
    assert!(matches!(val, Value::Err(_)), "expected Err, got {:?}", val);
}

#[test]
fn test_optional_chaining_nil() {
    assert!(matches!(run_get("x = nil?.foo", "x"), Value::Nil));
}

#[test]
fn test_optional_chaining_non_nil() {
    let src = r#"
class Obj {
    val: Int
    init(v) { self.val = v }
}
o = Obj(42)
x = o?.val
"#;
    assert_int(&run_get(src, "x"), 42);
}

#[test]
fn test_nil_coalescing_nil_value() {
    assert_int(&run_get("x = nil ?? 99", "x"), 99);
}

#[test]
fn test_nil_coalescing_non_nil() {
    assert_int(&run_get("x = 42 ?? 99", "x"), 42);
}

#[test]
fn test_nil_coalescing_string() {
    assert_str(&run_get("x = nil ?? \"default\"", "x"), "default");
}

#[test]
fn test_nil_coalescing_chain() {
    assert_int(&run_get("x = nil ?? nil ?? 7", "x"), 7);
}

#[test]
fn test_ok_value() {
    let val = run_get("x = Ok(42)", "x");
    assert!(matches!(val, Value::Ok(_)));
}

#[test]
fn test_err_value() {
    let val = run_get("x = Err(\"fail\")", "x");
    assert!(matches!(val, Value::Err(_)));
}

#[test]
fn test_match_ok_err() {
    let src = r#"
r = Ok(5)
x = match r {
    Ok(v) -> v * 10
    Err(e) -> -1
}
"#;
    assert_int(&run_get(src, "x"), 50);
}

#[test]
fn test_match_err_case() {
    let src = r#"
r = Err("oops")
x = match r {
    Ok(v) -> v
    Err(e) -> -1
}
"#;
    assert_int(&run_get(src, "x"), -1);
}

// ═══════════════════════════════════════════════════════════════
// Functions with control flow (bonus)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_function_with_if_return() {
    let src = r#"
def abs_val(n) {
    if n < 0 { return -n }
    return n
}
x = abs_val(-7)
"#;
    assert_int(&run_get(src, "x"), 7);
}

#[test]
fn test_function_with_loop_break() {
    let src = r#"
def first_above(items, threshold) {
    for item in items {
        if item > threshold { return item }
    }
    return -1
}
x = first_above([1, 5, 10, 20], 8)
"#;
    assert_int(&run_get(src, "x"), 10);
}

#[test]
fn test_recursive_fibonacci() {
    let src = r#"
def fib(n) {
    if n <= 1 { return n }
    return fib(n - 1) + fib(n - 2)
}
x = fib(10)
"#;
    assert_int(&run_get(src, "x"), 55);
}

#[test]
fn test_recursive_sum_list() {
    let src = r#"
def sum_to(n) {
    if n <= 0 { return 0 }
    return n + sum_to(n - 1)
}
x = sum_to(10)
"#;
    assert_int(&run_get(src, "x"), 55);
}

#[test]
fn test_function_match_dispatch() {
    let src = r#"
def describe(val) {
    match val {
        0 -> "zero"
        1 -> "one"
        _ -> "many"
    }
}
a = describe(0)
b = describe(1)
c = describe(99)
x = a + "," + b + "," + c
"#;
    assert_str(&run_get(src, "x"), "zero,one,many");
}

#[test]
fn test_for_building_list() {
    let src = r#"
result = []
for i in 0..5 {
    result.push(i * i)
}
x = result.sum()
"#;
    assert_int(&run_get(src, "x"), 30);
}

#[test]
fn test_while_loop_factorial() {
    let src = r#"
n = 10
result = 1
loop while n > 1 {
    result *= n
    n -= 1
}
x = result
"#;
    assert_int(&run_get(src, "x"), 3628800);
}

#[test]
fn test_break_with_accumulator() {
    let src = r#"
x = 0
for i in 0..100 {
    x += i
    if x > 50 { break }
}
"#;
    // 0+1+2+...+10 = 55 > 50
    assert_int(&run_get(src, "x"), 55);
}

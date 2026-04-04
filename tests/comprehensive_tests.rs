use aether::interpreter;
use aether::interpreter::environment::Environment;
use aether::interpreter::values::Value;
use aether::lexer::scanner::Scanner;
use aether::parser::parser::Parser;

// ═══════════════════════════════════════════════════════════════
// Test helpers
// ═══════════════════════════════════════════════════════════════

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

fn run_expect_error(source: &str) -> bool {
    let mut scanner = Scanner::new(source, "test.ae".to_string());
    let tokens = scanner.scan_tokens();
    let mut parser = Parser::new(tokens);
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(_) => return true,
    };
    let mut env = Environment::new();
    interpreter::register_builtins(&mut env);
    interpreter::interpret(&program, &mut env).is_err()
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
            "expected ~{}, got Int({})",
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


// ═══════════════════════════════════════════════════════════════
// 1. String interpolation (20 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_interp_simple_var() {
    assert_str(&run_get("name = \"Alice\"\nx = \"hello {name}\"", "x"), "hello Alice");
}

#[test]
fn test_interp_int_var() {
    assert_str(&run_get("n = 42\nx = \"value is {n}\"", "x"), "value is 42");
}

#[test]
fn test_interp_float_var() {
    let val = run_get("f = 3.14\nx = \"pi is {f}\"", "x");
    if let Value::String(s) = &val {
        assert!(s.starts_with("pi is 3.14"), "got '{}'", s);
    } else {
        panic!("expected String, got {:?}", val);
    }
}

#[test]
fn test_interp_bool_var() {
    assert_str(&run_get("b = true\nx = \"flag: {b}\"", "x"), "flag: true");
}

#[test]
fn test_interp_expression_add() {
    assert_str(&run_get("a = 3\nb = 4\nx = \"sum: {a + b}\"", "x"), "sum: 7");
}

#[test]
fn test_interp_expression_multiply() {
    assert_str(&run_get("n = 6\nx = \"result: {n * 7}\"", "x"), "result: 42");
}

#[test]
fn test_interp_nested_string_concat() {
    assert_str(
        &run_get("first = \"hello\"\nlast = \"world\"\nx = \"{first} {last}\"", "x"),
        "hello world",
    );
}

#[test]
fn test_interp_multiple_vars() {
    assert_str(
        &run_get("a = 1\nb = 2\nc = 3\nx = \"{a},{b},{c}\"", "x"),
        "1,2,3",
    );
}

#[test]
fn test_interp_with_method_call_upper() {
    assert_str(
        &run_get("s = \"hello\"\nx = \"shout: {s.upper()}\"", "x"),
        "shout: HELLO",
    );
}

#[test]
fn test_interp_with_method_call_len() {
    assert_str(
        &run_get("s = \"hello\"\nx = \"length: {s.len()}\"", "x"),
        "length: 5",
    );
}

#[test]
fn test_interp_with_nil() {
    assert_str(&run_get("v = nil\nx = \"got: {v}\"", "x"), "got: nil");
}

#[test]
fn test_interp_empty_string_var() {
    assert_str(&run_get("s = \"\"\nx = \"empty:{s}:end\"", "x"), "empty::end");
}

#[test]
fn test_interp_adjacent_braces() {
    assert_str(&run_get("a = 1\nb = 2\nx = \"{a}{b}\"", "x"), "12");
}

#[test]
fn test_interp_in_concat() {
    assert_str(
        &run_get("name = \"X\"\nx = \"A\" + \"{name}\" + \"B\"", "x"),
        "AXB",
    );
}

#[test]
fn test_interp_with_comparison() {
    assert_str(&run_get("n = 5\nx = \"big: {n > 3}\"", "x"), "big: true");
}

#[test]
fn test_interp_with_ternary() {
    assert_str(
        &run_get("n = 10\nx = \"sign: {if n > 0 then \"pos\" else \"neg\"}\"", "x"),
        "sign: pos",
    );
}

#[test]
fn test_interp_list_var() {
    let val = run_get("l = [1, 2, 3]\nx = \"list: {l}\"", "x");
    if let Value::String(s) = &val {
        assert!(s.contains("1") && s.contains("2") && s.contains("3"), "got '{}'", s);
    } else {
        panic!("expected String, got {:?}", val);
    }
}

#[test]
fn test_interp_function_result() {
    assert_str(
        &run_get("def greet(n) = \"hi \" + n\nx = \"msg: {greet(\"Bob\")}\"", "x"),
        "msg: hi Bob",
    );
}

#[test]
fn test_interp_subtraction_expr() {
    assert_str(&run_get("x = \"diff: {10 - 3}\"", "x"), "diff: 7");
}

#[test]
fn test_interp_chained_method() {
    assert_str(
        &run_get("x = \"result: {\"  hello  \".trim().upper()}\"", "x"),
        "result: HELLO",
    );
}


// ═══════════════════════════════════════════════════════════════
// 2. Range operations (20 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_range_exclusive_sum() {
    assert_int(&run_get("x = 0\nfor i in 0..5 { x += i }", "x"), 10);
}

#[test]
fn test_range_exclusive_count() {
    assert_int(&run_get("x = 0\nfor i in 0..10 { x += 1 }", "x"), 10);
}

#[test]
fn test_range_inclusive_sum() {
    assert_int(&run_get("x = 0\nfor i in 0..=5 { x += i }", "x"), 15);
}

#[test]
fn test_range_inclusive_count() {
    assert_int(&run_get("x = 0\nfor i in 0..=10 { x += 1 }", "x"), 11);
}

#[test]
fn test_range_start_nonzero() {
    assert_int(&run_get("x = 0\nfor i in 5..8 { x += i }", "x"), 18);
}

#[test]
fn test_range_single_element_inclusive() {
    assert_int(&run_get("x = 0\nfor i in 3..=3 { x += i }", "x"), 3);
}

#[test]
fn test_range_empty() {
    assert_int(&run_get("x = 0\nfor i in 5..5 { x += 1 }", "x"), 0);
}

#[test]
fn test_range_step_by_2() {
    // 0, 2, 4, 6, 8 -> sum = 20
    assert_int(&run_get("x = 0\nfor i in 0..10 step 2 { x += i }", "x"), 20);
}

#[test]
fn test_range_step_by_3() {
    // 0, 3, 6, 9 -> count = 4
    assert_int(&run_get("x = 0\nfor i in 0..10 step 3 { x += 1 }", "x"), 4);
}

#[test]
fn test_range_step_by_5() {
    // 0, 5 -> sum = 5
    assert_int(&run_get("x = 0\nfor i in 0..10 step 5 { x += i }", "x"), 5);
}

#[test]
fn test_range_boundary_exclusive() {
    // Verify end is excluded: 0..3 gives 0,1,2
    assert_int(&run_get("x = 0\nfor i in 0..3 { x += 1 }", "x"), 3);
}

#[test]
fn test_range_boundary_inclusive() {
    // Verify end is included: 0..=3 gives 0,1,2,3
    assert_int(&run_get("x = 0\nfor i in 0..=3 { x += 1 }", "x"), 4);
}

#[test]
fn test_range_in_comprehension() {
    let val = run_get("x = [i for i in 0..5]", "x");
    assert_eq!(list_len(&val), 5);
    assert_int(&list_get(&val, 0), 0);
    assert_int(&list_get(&val, 4), 4);
}

#[test]
fn test_range_squared_comprehension() {
    let val = run_get("x = [i * i for i in 0..6]", "x");
    assert_eq!(list_len(&val), 6);
    assert_int(&list_get(&val, 0), 0);
    assert_int(&list_get(&val, 3), 9);
    assert_int(&list_get(&val, 5), 25);
}

#[test]
fn test_range_filtered_comprehension() {
    let val = run_get("x = [i for i in 0..10 if i % 3 == 0]", "x");
    // 0, 3, 6, 9
    assert_eq!(list_len(&val), 4);
    assert_int(&list_get(&val, 0), 0);
    assert_int(&list_get(&val, 3), 9);
}

#[test]
fn test_range_with_break() {
    assert_int(&run_get("x = 0\nfor i in 0..100 { if i >= 7 { break }\nx += 1 }", "x"), 7);
}

#[test]
fn test_range_with_next() {
    // Skip multiples of 3 in 0..9, count the rest
    assert_int(&run_get("x = 0\nfor i in 0..9 { next if i % 3 == 0\nx += 1 }", "x"), 6);
}

#[test]
fn test_range_large() {
    // Sum 0..1000 = 999*1000/2 = 499500
    assert_int(&run_get("x = 0\nfor i in 0..1000 { x += i }", "x"), 499500);
}

#[test]
fn test_range_negative_start() {
    // -3, -2, -1, 0, 1, 2 -> sum = -3
    assert_int(&run_get("x = 0\nfor i in -3..3 { x += i }", "x"), -3);
}

#[test]
fn test_range_inclusive_step() {
    // 1, 3, 5, 7, 9 -> sum = 25
    assert_int(&run_get("x = 0\nfor i in 1..=9 step 2 { x += i }", "x"), 25);
}


// ═══════════════════════════════════════════════════════════════
// 3. Pattern matching advanced (20 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_match_int_literal() {
    assert_str(&run_get("x = match 1 { 1 -> \"one\"\n2 -> \"two\"\n_ -> \"other\" }", "x"), "one");
}

#[test]
fn test_match_string_literal() {
    assert_str(
        &run_get("x = match \"hi\" { \"hi\" -> \"greeting\"\n_ -> \"unknown\" }", "x"),
        "greeting",
    );
}

#[test]
fn test_match_wildcard_fallthrough() {
    assert_str(&run_get("x = match 999 { 1 -> \"a\"\n2 -> \"b\"\n_ -> \"default\" }", "x"), "default");
}

#[test]
fn test_match_ok_destructure() {
    assert_int(&run_get("r = Ok(100)\nx = match r { Ok(v) -> v\nErr(e) -> 0 }", "x"), 100);
}

#[test]
fn test_match_err_destructure() {
    assert_str(
        &run_get("r = Err(\"boom\")\nx = match r { Ok(v) -> \"ok\"\nErr(e) -> e }", "x"),
        "boom",
    );
}

#[test]
fn test_match_ok_with_computation() {
    assert_int(
        &run_get("r = Ok(21)\nx = match r { Ok(v) -> v * 2\nErr(e) -> -1 }", "x"),
        42,
    );
}

#[test]
fn test_match_as_expression_assign() {
    assert_str(
        &run_get("n = 42\nx = match n { 0 -> \"zero\"\n_ -> \"nonzero\" }", "x"),
        "nonzero",
    );
}

#[test]
fn test_match_guard_even() {
    assert_str(
        &run_get("x = match 6 { n if n % 2 == 0 -> \"even\"\n_ -> \"odd\" }", "x"),
        "even",
    );
}

#[test]
fn test_match_guard_odd() {
    assert_str(
        &run_get("x = match 7 { n if n % 2 == 0 -> \"even\"\n_ -> \"odd\" }", "x"),
        "odd",
    );
}

#[test]
fn test_match_guard_with_range() {
    assert_str(
        &run_get("x = match 15 { n if n > 10 -> \"big\"\n_ -> \"small\" }", "x"),
        "big",
    );
}

#[test]
fn test_match_range_pattern() {
    assert_str(
        &run_get("x = match 50 { 1..10 -> \"low\"\n10..100 -> \"mid\"\n_ -> \"high\" }", "x"),
        "mid",
    );
}

#[test]
fn test_match_range_below() {
    assert_str(
        &run_get("x = match 3 { 1..10 -> \"low\"\n_ -> \"other\" }", "x"),
        "low",
    );
}

#[test]
fn test_match_enum_circle() {
    assert_float_approx(
        &run_get("x = match .Circle(3.0) { .Circle(r) -> 3.14 * r * r\n_ -> 0.0 }", "x"),
        28.26,
    );
}

#[test]
fn test_match_enum_rect() {
    assert_float_approx(
        &run_get("x = match .Rect(4.0, 5.0) { .Circle(r) -> 0.0\n.Rect(w, h) -> w * h\n_ -> 0.0 }", "x"),
        20.0,
    );
}

#[test]
fn test_match_bool_true() {
    assert_str(
        &run_get("x = match true { true -> \"yes\"\nfalse -> \"no\" }", "x"),
        "yes",
    );
}

#[test]
fn test_match_bool_false() {
    assert_str(
        &run_get("x = match false { true -> \"yes\"\nfalse -> \"no\" }", "x"),
        "no",
    );
}

#[test]
fn test_match_in_function() {
    let src = r#"
def describe(n) {
    match n {
        0 -> "zero"
        1 -> "one"
        _ -> "many"
    }
}
x = describe(1)
"#;
    assert_str(&run_get(src, "x"), "one");
}

#[test]
fn test_match_nested_function_call() {
    let src = r#"
def f(n) { return n * 10 }
x = match 3 {
    1 -> f(1)
    3 -> f(3)
    _ -> 0
}
"#;
    assert_int(&run_get(src, "x"), 30);
}

#[test]
fn test_match_nil_pattern() {
    assert_str(
        &run_get("v = nil\nx = match v { nil -> \"nothing\"\n_ -> \"something\" }", "x"),
        "nothing",
    );
}

#[test]
fn test_match_multiple_ranges() {
    let src = r#"
def grade(score) {
    match score {
        90..100 -> "A"
        80..90 -> "B"
        70..80 -> "C"
        _ -> "F"
    }
}
x = grade(85)
"#;
    assert_str(&run_get(src, "x"), "B");
}


// ═══════════════════════════════════════════════════════════════
// 4. Nil handling (20 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_nil_equals_nil() {
    assert_bool(&run_get("x = nil == nil", "x"), true);
}

#[test]
fn test_nil_not_equals_zero() {
    assert_bool(&run_get("x = nil != 0", "x"), true);
}

#[test]
fn test_nil_not_equals_false() {
    assert_bool(&run_get("x = nil != false", "x"), true);
}

#[test]
fn test_nil_not_equals_empty_string() {
    assert_bool(&run_get("x = nil != \"\"", "x"), true);
}

#[test]
fn test_nil_coalescing_with_nil() {
    assert_int(&run_get("x = nil ?? 42", "x"), 42);
}

#[test]
fn test_nil_coalescing_with_value() {
    assert_int(&run_get("x = 10 ?? 42", "x"), 10);
}

#[test]
fn test_nil_coalescing_string() {
    assert_str(&run_get("x = nil ?? \"default\"", "x"), "default");
}

#[test]
fn test_nil_coalescing_chained() {
    assert_int(&run_get("x = nil ?? nil ?? 99", "x"), 99);
}

#[test]
fn test_nil_coalescing_first_non_nil() {
    assert_int(&run_get("x = nil ?? 5 ?? 99", "x"), 5);
}

#[test]
fn test_nil_optional_chaining_on_nil() {
    assert_nil(&run_get("x = nil?.foo", "x"));
}

#[test]
fn test_nil_is_falsy() {
    assert_int(&run_get("x = if nil then 1 else 0", "x"), 0);
}

#[test]
fn test_nil_or_true() {
    assert_bool(&run_get("x = false or nil == nil", "x"), true);
}

#[test]
fn test_nil_assignment() {
    assert_nil(&run_get("x = nil", "x"));
}

#[test]
fn test_nil_in_list() {
    let val = run_get("x = [1, nil, 3]", "x");
    assert_eq!(list_len(&val), 3);
    assert_nil(&list_get(&val, 1));
}

#[test]
fn test_nil_from_map_missing_key() {
    assert_nil(&run_get("m = {\"a\": 1}\nx = m.get(\"z\")", "x"));
}

#[test]
fn test_nil_guard_returns_early() {
    let src = r#"
def f(val) {
    guard let n = val else { return -1 }
    return n
}
x = f(nil)
"#;
    assert_int(&run_get(src, "x"), -1);
}

#[test]
fn test_nil_guard_passes() {
    let src = r#"
def f(val) {
    guard let n = val else { return -1 }
    return n
}
x = f(42)
"#;
    assert_int(&run_get(src, "x"), 42);
}

#[test]
fn test_nil_coalescing_with_function() {
    let src = r#"
def get_val() { return nil }
x = get_val() ?? 100
"#;
    assert_int(&run_get(src, "x"), 100);
}

#[test]
fn test_nil_equality_symmetric() {
    assert_bool(&run_get("x = (nil == nil) and (nil == nil)", "x"), true);
}

#[test]
fn test_nil_not_equals_list() {
    assert_bool(&run_get("x = nil != []", "x"), true);
}


// ═══════════════════════════════════════════════════════════════
// 5. Type conversion functions (15 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_int_from_string() {
    assert_int(&run_get("x = int(\"42\")", "x"), 42);
}

#[test]
fn test_int_from_float() {
    assert_int(&run_get("x = int(3.9)", "x"), 3);
}

#[test]
fn test_int_from_bool_true() {
    assert_int(&run_get("x = int(true)", "x"), 1);
}

#[test]
fn test_int_from_bool_false() {
    assert_int(&run_get("x = int(false)", "x"), 0);
}

#[test]
fn test_int_from_negative_string() {
    assert_int(&run_get("x = int(\"-100\")", "x"), -100);
}

#[test]
fn test_float_from_string() {
    assert_float_approx(&run_get("x = float(\"3.14\")", "x"), 3.14);
}

#[test]
fn test_float_from_int() {
    assert_float_approx(&run_get("x = float(42)", "x"), 42.0);
}

#[test]
fn test_str_from_int() {
    assert_str(&run_get("x = str(42)", "x"), "42");
}

#[test]
fn test_str_from_float() {
    let val = run_get("x = str(3.14)", "x");
    if let Value::String(s) = &val {
        assert!(s.starts_with("3.14"), "got '{}'", s);
    } else {
        panic!("expected String, got {:?}", val);
    }
}

#[test]
fn test_str_from_bool() {
    assert_str(&run_get("x = str(true)", "x"), "true");
}

#[test]
fn test_str_from_nil() {
    assert_str(&run_get("x = str(nil)", "x"), "nil");
}

#[test]
fn test_type_of_int() {
    assert_str(&run_get("x = type(42)", "x"), "Int");
}

#[test]
fn test_type_of_string() {
    assert_str(&run_get("x = type(\"hello\")", "x"), "Str");
}

#[test]
fn test_type_of_list() {
    assert_str(&run_get("x = type([1,2,3])", "x"), "List");
}

#[test]
fn test_type_of_bool() {
    assert_str(&run_get("x = type(true)", "x"), "Bool");
}


// ═══════════════════════════════════════════════════════════════
// 6. Nested data structures (20 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_list_of_lists() {
    let val = run_get("x = [[1, 2], [3, 4]]", "x");
    assert_eq!(list_len(&val), 2);
    let inner = list_get(&val, 0);
    assert_eq!(list_len(&inner), 2);
}

#[test]
fn test_list_of_lists_deep_access() {
    assert_int(&run_get("x = [[1, 2], [3, 4]][0][1]", "x"), 2);
}

#[test]
fn test_list_of_lists_deep_access_second() {
    assert_int(&run_get("x = [[10, 20], [30, 40]][1][0]", "x"), 30);
}

#[test]
fn test_nested_map() {
    assert_int(
        &run_get("m = {\"a\": {\"b\": 42}}\nx = m[\"a\"][\"b\"]", "x"),
        42,
    );
}

#[test]
fn test_list_of_maps() {
    assert_int(
        &run_get("x = [{\"val\": 1}, {\"val\": 2}][1][\"val\"]", "x"),
        2,
    );
}

#[test]
fn test_map_of_lists() {
    let val = run_get("m = {\"nums\": [1, 2, 3]}\nx = m[\"nums\"]", "x");
    assert_eq!(list_len(&val), 3);
}

#[test]
fn test_map_of_lists_access() {
    assert_int(
        &run_get("m = {\"nums\": [10, 20, 30]}\nx = m[\"nums\"][2]", "x"),
        30,
    );
}

#[test]
fn test_nested_list_push() {
    let src = "grid = [[1], [2]]\ngrid[0].push(10)\nx = grid[0].len()";
    assert_int(&run_get(src, "x"), 2);
}

#[test]
fn test_three_deep_list() {
    assert_int(&run_get("x = [[[1, 2], [3, 4]], [[5, 6]]][0][1][0]", "x"), 3);
}

#[test]
fn test_list_of_strings() {
    let val = run_get("x = [\"a\", \"b\", \"c\"]", "x");
    assert_eq!(list_len(&val), 3);
    assert_str(&list_get(&val, 0), "a");
    assert_str(&list_get(&val, 2), "c");
}

#[test]
fn test_list_of_mixed_types() {
    let val = run_get("x = [1, \"two\", true, nil]", "x");
    assert_eq!(list_len(&val), 4);
    assert_int(&list_get(&val, 0), 1);
    assert_str(&list_get(&val, 1), "two");
    assert_bool(&list_get(&val, 2), true);
    assert_nil(&list_get(&val, 3));
}

#[test]
fn test_map_with_bool_values() {
    assert_bool(
        &run_get("m = {\"flag\": true}\nx = m[\"flag\"]", "x"),
        true,
    );
}

#[test]
fn test_empty_nested_list() {
    let val = run_get("x = [[], [], []]", "x");
    assert_eq!(list_len(&val), 3);
    assert_eq!(list_len(&list_get(&val, 0)), 0);
}

#[test]
fn test_build_nested_list() {
    let src = "l = []\nl.push([1, 2])\nl.push([3, 4])\nx = l[1][0]";
    assert_int(&run_get(src, "x"), 3);
}

#[test]
fn test_map_set_nested() {
    let src = "m = {}\nm.set(\"inner\", {\"x\": 99})\nx = m[\"inner\"][\"x\"]";
    assert_int(&run_get(src, "x"), 99);
}

#[test]
fn test_list_comprehension_of_lists() {
    let val = run_get("x = [[i, i*2] for i in 0..3]", "x");
    assert_eq!(list_len(&val), 3);
    let inner = list_get(&val, 2);
    assert_int(&list_get(&inner, 0), 2);
    assert_int(&list_get(&inner, 1), 4);
}

#[test]
fn test_nested_map_keys() {
    let src = r#"
m = {"x": 1, "y": 2, "z": 3}
x = m.keys().len()
"#;
    assert_int(&run_get(src, "x"), 3);
}

#[test]
fn test_map_values_sum() {
    // Extract values and sum them
    let src = r#"
m = {"a": 10, "b": 20, "c": 30}
x = m.values().sum()
"#;
    assert_int(&run_get(src, "x"), 60);
}

#[test]
fn test_nested_list_len_sum() {
    let src = "x = [[1,2,3], [4,5], [6]]\ny = x[0].len() + x[1].len() + x[2].len()";
    assert_int(&run_get(src, "y"), 6);
}

#[test]
fn test_list_index_assignment_nested() {
    let src = "l = [[0, 0], [0, 0]]\nl[1][0] = 42\nx = l[1][0]";
    assert_int(&run_get(src, "x"), 42);
}


// ═══════════════════════════════════════════════════════════════
// 7. Edge cases (20 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_empty_string_len() {
    assert_int(&run_get("x = \"\".len()", "x"), 0);
}

#[test]
fn test_empty_string_upper() {
    assert_str(&run_get("x = \"\".upper()", "x"), "");
}

#[test]
fn test_empty_string_contains() {
    assert_bool(&run_get("x = \"\".contains(\"\")", "x"), true);
}

#[test]
fn test_empty_list_len() {
    assert_int(&run_get("x = [].len()", "x"), 0);
}

#[test]
fn test_empty_list_sum() {
    assert_int(&run_get("x = [].sum()", "x"), 0);
}

#[test]
fn test_empty_list_is_empty() {
    assert_bool(&run_get("x = [].is_empty()", "x"), true);
}

#[test]
fn test_non_empty_list_is_empty() {
    assert_bool(&run_get("x = [1].is_empty()", "x"), false);
}

#[test]
fn test_empty_map_len() {
    assert_int(&run_get("x = {}.len()", "x"), 0);
}

#[test]
fn test_empty_map_is_empty() {
    assert_bool(&run_get("x = {}.is_empty()", "x"), true);
}

#[test]
fn test_large_number_power() {
    // 2^50 = 1125899906842624
    assert_int(&run_get("x = 2 ** 50", "x"), 1125899906842624);
}

#[test]
fn test_large_number_arithmetic() {
    assert_int(&run_get("x = 1000000 * 1000000", "x"), 1000000000000);
}

#[test]
fn test_negative_number_abs() {
    assert_int(&run_get("x = abs(-999)", "x"), 999);
}

#[test]
fn test_zero_times_anything() {
    assert_int(&run_get("x = 0 * 99999", "x"), 0);
}

#[test]
fn test_boolean_and_true_true() {
    assert_bool(&run_get("x = true and true", "x"), true);
}

#[test]
fn test_boolean_or_false_false() {
    assert_bool(&run_get("x = false or false", "x"), false);
}

#[test]
fn test_double_negation() {
    assert_bool(&run_get("x = not not true", "x"), true);
}

#[test]
fn test_precedence_multiply_add() {
    assert_int(&run_get("x = 2 + 3 * 4", "x"), 14);
}

#[test]
fn test_precedence_parens_override() {
    assert_int(&run_get("x = (2 + 3) * 4", "x"), 20);
}

#[test]
fn test_deeply_nested_arithmetic() {
    assert_int(&run_get("x = ((((1 + 2) * 3) + 4) * 5)", "x"), 65);
}

#[test]
fn test_chained_comparison_logic() {
    // 1 < 2 and 2 < 3 -> true
    assert_bool(&run_get("x = 1 < 2 and 2 < 3", "x"), true);
    assert_bool(&run_get("x = 1 < 2 and 2 > 3", "x"), false);
}


// ═══════════════════════════════════════════════════════════════
// 8. Error recovery (15 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_error_division_by_zero() {
    assert!(run_expect_error("x = 10 / 0"));
}

#[test]
fn test_error_nil_division() {
    assert!(run_expect_error("x = nil / 1"));
}

#[test]
fn test_error_undefined_variable() {
    assert!(run_expect_error("x = undefined_var + 1"));
}

#[test]
fn test_error_wrong_arg_count_too_few() {
    assert!(run_expect_error("def f(a, b) { return a + b }\nx = f(1)"));
}

#[test]
fn test_error_bool_minus_int() {
    assert!(run_expect_error("x = true - 5"));
}

#[test]
fn test_error_index_out_of_bounds() {
    assert!(run_expect_error("x = [1, 2, 3][10]"));
}

#[test]
fn test_error_nil_addition() {
    assert!(run_expect_error("x = nil + 1"));
}

#[test]
fn test_error_nil_subtraction() {
    assert!(run_expect_error("x = nil - 1"));
}

#[test]
fn test_error_string_minus_string() {
    assert!(run_expect_error("x = \"hello\" - \"world\""));
}

#[test]
fn test_error_call_non_function() {
    assert!(run_expect_error("x = 42\ny = x(1)"));
}

#[test]
fn test_error_int_parse_invalid() {
    assert!(run_expect_error("x = int(\"not_a_number\")"));
}

#[test]
fn test_error_float_parse_invalid() {
    assert!(run_expect_error("x = float(\"not_a_float\")"));
}

#[test]
fn test_try_catch_catches_throw() {
    let src = r#"
x = 0
try {
    throw "boom"
    x = 99
} catch any as e {
    x = 1
}
"#;
    assert_int(&run_get(src, "x"), 1);
}

#[test]
fn test_try_catch_error_value() {
    let src = r#"
x = ""
try {
    throw "crash"
} catch any as e {
    x = str(e)
}
"#;
    assert_str(&run_get(src, "x"), "crash");
}

#[test]
fn test_try_finally_always_runs() {
    let src = r#"
x = 0
try {
    x = 1
} catch any as e {
    x = 2
} finally {
    x = x + 100
}
"#;
    assert_int(&run_get(src, "x"), 101);
}


// ═══════════════════════════════════════════════════════════════
// 9. Pipeline operator (15 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_pipeline_basic() {
    assert_int(&run_get("f = x -> x + 1\nx = 5 |> f", "x"), 6);
}

#[test]
fn test_pipeline_double() {
    assert_int(&run_get("f = x -> x * 2\nx = 3 |> f", "x"), 6);
}

#[test]
fn test_pipeline_chained_two() {
    assert_int(
        &run_get("inc = x -> x + 1\ndbl = x -> x * 2\nx = 5 |> inc |> dbl", "x"),
        12,
    );
}

#[test]
fn test_pipeline_chained_three() {
    assert_int(
        &run_get("a = x -> x + 1\nb = x -> x * 2\nc = x -> x - 3\nx = 5 |> a |> b |> c", "x"),
        9,
    );
}

#[test]
fn test_pipeline_with_named_function() {
    let src = r#"
def square(n) { return n * n }
x = 4 |> square
"#;
    assert_int(&run_get(src, "x"), 16);
}

#[test]
fn test_pipeline_string_operation() {
    let src = r#"
def exclaim(s) { return s + "!" }
x = "hello" |> exclaim
"#;
    assert_str(&run_get(src, "x"), "hello!");
}

#[test]
fn test_pipeline_from_literal() {
    assert_int(&run_get("f = x -> x + 10\nx = 0 |> f", "x"), 10);
}

#[test]
fn test_pipeline_to_str() {
    assert_str(&run_get("x = 42 |> str", "x"), "42");
}

#[test]
fn test_pipeline_to_type() {
    assert_str(&run_get("x = 42 |> type", "x"), "Int");
}

#[test]
fn test_pipeline_identity() {
    assert_int(&run_get("id = x -> x\nx = 99 |> id", "x"), 99);
}

#[test]
fn test_pipeline_negate() {
    assert_int(&run_get("neg = x -> 0 - x\nx = 42 |> neg", "x"), -42);
}

#[test]
fn test_pipeline_bool_to_int() {
    assert_int(&run_get("x = true |> int", "x"), 1);
}

#[test]
fn test_pipeline_chain_with_abs() {
    let src = "neg = x -> 0 - x\nx = 5 |> neg |> abs";
    assert_int(&run_get(src, "x"), 5);
}

#[test]
fn test_pipeline_expression_body_func() {
    let src = "def triple(n) = n * 3\nx = 7 |> triple";
    assert_int(&run_get(src, "x"), 21);
}

#[test]
fn test_pipeline_from_variable() {
    let src = "n = 10\nf = x -> x * x\nx = n |> f";
    assert_int(&run_get(src, "x"), 100);
}


// ═══════════════════════════════════════════════════════════════
// 10. Anonymous functions and closures (15 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_anon_lambda_assign() {
    assert_int(&run_get("f = x -> x + 1\nx = f(10)", "x"), 11);
}

#[test]
fn test_anon_lambda_multiply() {
    assert_int(&run_get("f = (a, b) -> a * b\nx = f(6, 7)", "x"), 42);
}

#[test]
fn test_anon_lambda_in_map() {
    let val = run_get("x = [1, 2, 3].map(x -> x * 10)", "x");
    assert_eq!(list_len(&val), 3);
    assert_int(&list_get(&val, 0), 10);
    assert_int(&list_get(&val, 2), 30);
}

#[test]
fn test_anon_lambda_in_filter() {
    let val = run_get("x = [1, 2, 3, 4, 5].filter(x -> x > 3)", "x");
    assert_eq!(list_len(&val), 2);
    assert_int(&list_get(&val, 0), 4);
    assert_int(&list_get(&val, 1), 5);
}

#[test]
fn test_closure_captures_variable() {
    let src = r#"
def make_adder(base) {
    return x -> x + base
}
f = make_adder(100)
x = f(5)
"#;
    assert_int(&run_get(src, "x"), 105);
}

#[test]
fn test_closure_captures_outer_let() {
    let src = "let n = 100\nf = x -> x + n\nx = f(5)";
    assert_int(&run_get(src, "x"), 105);
}

#[test]
fn test_function_as_argument() {
    let src = r#"
def apply(f, val) { return f(val) }
double = x -> x * 2
x = apply(double, 21)
"#;
    assert_int(&run_get(src, "x"), 42);
}

#[test]
fn test_function_returns_function() {
    let src = r#"
def make_adder(n) { return x -> x + n }
add10 = make_adder(10)
x = add10(32)
"#;
    assert_int(&run_get(src, "x"), 42);
}

#[test]
fn test_lambda_no_params_usage() {
    // Zero-param lambda used via variable
    let src = r#"
def call_it(f) { return f(0) }
x = call_it(x -> 42)
"#;
    assert_int(&run_get(src, "x"), 42);
}

#[test]
fn test_lambda_string_transform() {
    assert_str(
        &run_get("f = s -> s + \"!\"\nx = f(\"hello\")", "x"),
        "hello!",
    );
}

#[test]
fn test_lambda_boolean_logic() {
    assert_bool(&run_get("is_pos = x -> x > 0\nx = is_pos(5)", "x"), true);
    assert_bool(&run_get("is_pos = x -> x > 0\nx = is_pos(-5)", "x"), false);
}

#[test]
fn test_multiple_closures_independent() {
    let src = r#"
def make_adder(n) { return x -> x + n }
add5 = make_adder(5)
add10 = make_adder(10)
x = add5(1) + add10(1)
"#;
    assert_int(&run_get(src, "x"), 17);
}

#[test]
fn test_closure_in_loop() {
    let src = r#"
fns = []
for i in 0..3 {
    fns.push(x -> x + i)
}
x = fns[0](10)
"#;
    // Closure captures the value of i at creation time
    let val = run_get(src, "x");
    // Just check it returns an integer (exact value depends on capture semantics)
    assert!(matches!(val, Value::Int(_)), "expected Int, got {:?}", val);
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
    assert_int(&run_get(src, "x"), 12);
}

#[test]
fn test_recursive_lambda_via_def() {
    let src = r#"
def fib(n) {
    if n <= 1 { return n }
    return fib(n - 1) + fib(n - 2)
}
x = fib(10)
"#;
    assert_int(&run_get(src, "x"), 55);
}


// ═══════════════════════════════════════════════════════════════
// 11. Miscellaneous — OOP, comprehensions, builtins (40 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_class_with_method_returning_self_field() {
    let src = r#"
class Point {
    x: Int
    y: Int
    init(x, y) { self.x = x; self.y = y }
    def sum() { return self.x + self.y }
}
p = Point(3, 4)
x = p.sum()
"#;
    assert_int(&run_get(src, "x"), 7);
}

#[test]
fn test_class_field_update_and_read() {
    let src = r#"
class Counter {
    n: Int
    init() { self.n = 0 }
    def inc() { self.n = self.n + 1 }
    def get() { return self.n }
}
c = Counter()
c.inc()
c.inc()
c.inc()
x = c.get()
"#;
    assert_int(&run_get(src, "x"), 3);
}

#[test]
fn test_class_two_instances() {
    let src = r#"
class Box {
    val: Int
    init(v) { self.val = v }
}
a = Box(10)
b = Box(20)
x = a.val + b.val
"#;
    assert_int(&run_get(src, "x"), 30);
}

#[test]
fn test_class_inheritance_field() {
    let src = r#"
class Animal {
    name: Str
    init(name) { self.name = name }
}
class Cat : Animal {
    def speak() { return "meow" }
}
c = Cat("Whiskers")
x = c.name
"#;
    assert_str(&run_get(src, "x"), "Whiskers");
}

#[test]
fn test_class_override_method() {
    let src = r#"
class Shape {
    def area() { return 0 }
}
class Square : Shape {
    side: Int
    init(s) { self.side = s }
    def area() { return self.side * self.side }
}
s = Square(5)
x = s.area()
"#;
    assert_int(&run_get(src, "x"), 25);
}

#[test]
fn test_comprehension_squares() {
    let val = run_get("x = [i * i for i in 1..=5]", "x");
    assert_eq!(list_len(&val), 5);
    assert_int(&list_get(&val, 0), 1);
    assert_int(&list_get(&val, 4), 25);
}

#[test]
fn test_comprehension_strings() {
    let val = run_get("x = [\"item\" for i in 0..3]", "x");
    assert_eq!(list_len(&val), 3);
    assert_str(&list_get(&val, 0), "item");
}

#[test]
fn test_comprehension_nested_expr() {
    let val = run_get("x = [i + 100 for i in 0..4]", "x");
    assert_eq!(list_len(&val), 4);
    assert_int(&list_get(&val, 0), 100);
    assert_int(&list_get(&val, 3), 103);
}

#[test]
fn test_comprehension_with_filter() {
    let val = run_get("x = [i for i in 0..20 if i % 5 == 0]", "x");
    // 0, 5, 10, 15
    assert_eq!(list_len(&val), 4);
}

#[test]
fn test_list_map_and_sum() {
    assert_int(&run_get("x = [1, 2, 3].map(x -> x * 10).sum()", "x"), 60);
}

#[test]
fn test_list_filter_and_len() {
    assert_int(&run_get("x = [1, 2, 3, 4, 5, 6].filter(x -> x % 2 == 0).len()", "x"), 3);
}

#[test]
fn test_list_contains() {
    assert_bool(&run_get("x = [1, 2, 3].contains(2)", "x"), true);
    assert_bool(&run_get("x = [1, 2, 3].contains(5)", "x"), false);
}

#[test]
fn test_list_first_last() {
    assert_int(&run_get("x = [10, 20, 30].first()", "x"), 10);
    assert_int(&run_get("x = [10, 20, 30].last()", "x"), 30);
}

#[test]
fn test_list_reverse() {
    let val = run_get("x = [1, 2, 3].reverse()", "x");
    assert_int(&list_get(&val, 0), 3);
    assert_int(&list_get(&val, 2), 1);
}

#[test]
fn test_list_join() {
    assert_str(&run_get("x = [\"a\", \"b\", \"c\"].join(\", \")", "x"), "a, b, c");
}

#[test]
fn test_list_min_max() {
    assert_int(&run_get("x = [5, 1, 9, 3].min()", "x"), 1);
    assert_int(&run_get("x = [5, 1, 9, 3].max()", "x"), 9);
}

#[test]
fn test_string_split() {
    let val = run_get("x = \"a,b,c\".split(\",\")", "x");
    assert_eq!(list_len(&val), 3);
    assert_str(&list_get(&val, 0), "a");
    assert_str(&list_get(&val, 2), "c");
}

#[test]
fn test_string_replace() {
    assert_str(&run_get("x = \"hello world\".replace(\"world\", \"earth\")", "x"), "hello earth");
}

#[test]
fn test_string_starts_with() {
    assert_bool(&run_get("x = \"hello\".starts_with(\"hel\")", "x"), true);
    assert_bool(&run_get("x = \"hello\".starts_with(\"xyz\")", "x"), false);
}

#[test]
fn test_string_ends_with() {
    assert_bool(&run_get("x = \"hello\".ends_with(\"llo\")", "x"), true);
    assert_bool(&run_get("x = \"hello\".ends_with(\"xyz\")", "x"), false);
}

#[test]
fn test_string_lower() {
    assert_str(&run_get("x = \"HELLO\".lower()", "x"), "hello");
}

#[test]
fn test_math_floor_ceil_round() {
    assert_int(&run_get("x = floor(3.7)", "x"), 3);
    assert_int(&run_get("x = ceil(3.2)", "x"), 4);
    assert_int(&run_get("x = round(3.5)", "x"), 4);
}

#[test]
fn test_math_trig_sin_zero() {
    assert_float_approx(&run_get("x = sin(0.0)", "x"), 0.0);
}

#[test]
fn test_math_trig_cos_zero() {
    assert_float_approx(&run_get("x = cos(0.0)", "x"), 1.0);
}

#[test]
fn test_math_sqrt_perfect() {
    assert_float_approx(&run_get("x = sqrt(25.0)", "x"), 5.0);
}

#[test]
fn test_math_log_e() {
    assert_float_approx(&run_get("x = log(E)", "x"), 1.0);
}

#[test]
fn test_math_constants_tau() {
    assert_float_approx(&run_get("x = TAU", "x"), std::f64::consts::TAU);
}

#[test]
fn test_loop_times_accumulate() {
    assert_int(&run_get("x = 0\nloop 10 times { x += 3 }", "x"), 30);
}

#[test]
fn test_loop_while_countdown() {
    // 100 - 7*8 = 44 (first value <= 50)
    assert_int(&run_get("x = 100\nloop while x > 50 { x -= 7 }", "x"), 44);
}

#[test]
fn test_loop_until() {
    assert_int(&run_get("x = 0\nloop { x += 11 } until x >= 100", "x"), 110);
}

#[test]
fn test_for_enumerate_sum_indices() {
    assert_int(&run_get("x = 0\nfor i, val in [10, 20, 30, 40] { x += i }", "x"), 6);
}

#[test]
fn test_for_enumerate_sum_values() {
    assert_int(&run_get("x = 0\nfor i, val in [10, 20, 30] { x += val }", "x"), 60);
}

#[test]
fn test_nested_for_product() {
    let src = "x = 0\nfor i in 0..3 {\n  for j in 0..3 {\n    x += 1\n  }\n}";
    assert_int(&run_get(src, "x"), 9);
}

#[test]
fn test_labeled_break_nested() {
    let src = r#"
x = 0
for:outer i in 0..10 {
    for j in 0..10 {
        if i * 10 + j >= 25 { break:outer }
        x += 1
    }
}
"#;
    assert_int(&run_get(src, "x"), 25);
}

#[test]
fn test_next_if_skip() {
    assert_int(&run_get("x = 0\nfor i in 0..10 { next if i < 5\nx += 1 }", "x"), 5);
}

#[test]
fn test_ok_wrapping() {
    let val = run_get("x = Ok(42)", "x");
    assert!(matches!(val, Value::Ok(_)));
}

#[test]
fn test_err_wrapping() {
    let val = run_get("x = Err(\"fail\")", "x");
    assert!(matches!(val, Value::Err(_)));
}

#[test]
fn test_error_propagation_ok_path() {
    let src = r#"
def safe_parse(s) {
    n = s.parse_int()?
    return Ok(n * 2)
}
x = safe_parse("21")
"#;
    let val = run_get(src, "x");
    if let Value::Ok(inner) = val {
        assert_int(&inner, 42);
    } else {
        panic!("expected Ok, got {:?}", val);
    }
}

#[test]
fn test_error_propagation_err_path() {
    let src = r#"
def safe_parse(s) {
    n = s.parse_int()?
    return Ok(n)
}
x = safe_parse("xyz")
"#;
    let val = run_get(src, "x");
    assert!(matches!(val, Value::Err(_)), "expected Err, got {:?}", val);
}

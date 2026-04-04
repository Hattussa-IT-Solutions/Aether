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

fn assert_ok_int(val: &Value, expected: i64) {
    match val {
        Value::Ok(inner) => assert_int(inner, expected),
        _ => panic!("expected Ok(Int({})), got {:?}", expected, val),
    }
}

fn assert_ok_float(val: &Value, expected: f64) {
    match val {
        Value::Ok(inner) => match inner.as_ref() {
            Value::Float(f) => assert!(
                (f - expected).abs() < 1e-6,
                "expected Ok(Float({})), got Ok(Float({}))",
                expected,
                f
            ),
            _ => panic!("expected Ok(Float({})), got {:?}", expected, inner),
        },
        _ => panic!("expected Ok(Float({})), got {:?}", expected, val),
    }
}

fn assert_err(val: &Value) {
    assert!(matches!(val, Value::Err(_)), "expected Err, got {:?}", val);
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
// Math: sqrt, abs, floor, ceil, round
// ===============================================================

#[test]
fn test_math_sqrt_perfect() {
    assert_float_approx(&run_get("x = sqrt(16.0)", "x"), 4.0);
}

#[test]
fn test_math_sqrt_non_perfect() {
    assert_float_approx(&run_get("x = sqrt(2.0)", "x"), std::f64::consts::SQRT_2);
}

#[test]
fn test_math_sqrt_zero() {
    assert_float_approx(&run_get("x = sqrt(0.0)", "x"), 0.0);
}

#[test]
fn test_math_abs_positive() {
    assert_int(&run_get("x = abs(42)", "x"), 42);
}

#[test]
fn test_math_abs_negative() {
    assert_int(&run_get("x = abs(-42)", "x"), 42);
}

#[test]
fn test_math_abs_zero() {
    assert_int(&run_get("x = abs(0)", "x"), 0);
}

#[test]
fn test_math_floor_positive() {
    assert_int(&run_get("x = floor(3.7)", "x"), 3);
}

#[test]
fn test_math_floor_negative() {
    assert_int(&run_get("x = floor(-1.2)", "x"), -2);
}

#[test]
fn test_math_ceil_positive() {
    assert_int(&run_get("x = ceil(3.2)", "x"), 4);
}

#[test]
fn test_math_ceil_negative() {
    assert_int(&run_get("x = ceil(-1.7)", "x"), -1);
}

#[test]
fn test_math_round_up() {
    assert_int(&run_get("x = round(3.6)", "x"), 4);
}

#[test]
fn test_math_round_down() {
    assert_int(&run_get("x = round(3.2)", "x"), 3);
}

#[test]
fn test_math_round_half() {
    assert_int(&run_get("x = round(2.5)", "x"), 3);
}

// ===============================================================
// Math: sin, cos, tan
// ===============================================================

#[test]
fn test_math_sin_zero() {
    assert_float_approx(&run_get("x = sin(0.0)", "x"), 0.0);
}

#[test]
fn test_math_sin_pi_half() {
    assert_float_approx(&run_get("x = sin(PI / 2.0)", "x"), 1.0);
}

#[test]
fn test_math_cos_zero() {
    assert_float_approx(&run_get("x = cos(0.0)", "x"), 1.0);
}

#[test]
fn test_math_cos_pi() {
    assert_float_approx(&run_get("x = cos(PI)", "x"), -1.0);
}

#[test]
fn test_math_tan_zero() {
    assert_float_approx(&run_get("x = tan(0.0)", "x"), 0.0);
}

// ===============================================================
// Math: log, log2, log10
// ===============================================================

#[test]
fn test_math_log_e() {
    assert_float_approx(&run_get("x = log(E)", "x"), 1.0);
}

#[test]
fn test_math_log_1() {
    assert_float_approx(&run_get("x = log(1.0)", "x"), 0.0);
}

#[test]
fn test_math_log2_8() {
    assert_float_approx(&run_get("x = log2(8.0)", "x"), 3.0);
}

#[test]
fn test_math_log2_1() {
    assert_float_approx(&run_get("x = log2(1.0)", "x"), 0.0);
}

#[test]
fn test_math_log10_1000() {
    assert_float_approx(&run_get("x = log10(1000.0)", "x"), 3.0);
}

#[test]
fn test_math_log10_1() {
    assert_float_approx(&run_get("x = log10(1.0)", "x"), 0.0);
}

// ===============================================================
// Math: pow, min, max, constants
// ===============================================================

#[test]
fn test_math_pow_int() {
    assert_float_approx(&run_get("x = pow(2.0, 10.0)", "x"), 1024.0);
}

#[test]
fn test_math_pow_fractional() {
    assert_float_approx(&run_get("x = pow(4.0, 0.5)", "x"), 2.0);
}

#[test]
fn test_math_min_two() {
    assert_float_approx(&run_get("x = min(3, 7)", "x"), 3.0);
}

#[test]
fn test_math_min_negative() {
    assert_float_approx(&run_get("x = min(-5, 5)", "x"), -5.0);
}

#[test]
fn test_math_max_two() {
    assert_float_approx(&run_get("x = max(3, 7)", "x"), 7.0);
}

#[test]
fn test_math_max_negative() {
    assert_float_approx(&run_get("x = max(-5, 5)", "x"), 5.0);
}

#[test]
fn test_math_pi_constant() {
    assert_float_approx(&run_get("x = PI", "x"), std::f64::consts::PI);
}

#[test]
fn test_math_e_constant() {
    assert_float_approx(&run_get("x = E", "x"), std::f64::consts::E);
}

#[test]
fn test_math_pi_in_expression() {
    assert_float_approx(&run_get("x = PI * 2.0", "x"), std::f64::consts::TAU);
}

// ===============================================================
// String methods: len, upper, lower, trim
// ===============================================================

#[test]
fn test_str_len_basic() {
    assert_int(&run_get("x = \"hello\".len()", "x"), 5);
}

#[test]
fn test_str_len_empty() {
    assert_int(&run_get("x = \"\".len()", "x"), 0);
}

#[test]
fn test_str_upper() {
    assert_str(&run_get("x = \"hello world\".upper()", "x"), "HELLO WORLD");
}

#[test]
fn test_str_upper_already_upper() {
    assert_str(&run_get("x = \"ABC\".upper()", "x"), "ABC");
}

#[test]
fn test_str_lower() {
    assert_str(&run_get("x = \"HELLO\".lower()", "x"), "hello");
}

#[test]
fn test_str_lower_already_lower() {
    assert_str(&run_get("x = \"abc\".lower()", "x"), "abc");
}

#[test]
fn test_str_trim() {
    assert_str(&run_get("x = \"  hello  \".trim()", "x"), "hello");
}

#[test]
fn test_str_trim_no_whitespace() {
    assert_str(&run_get("x = \"hello\".trim()", "x"), "hello");
}

#[test]
fn test_str_trim_tabs() {
    assert_str(&run_get("x = \"\\thello\\t\".trim()", "x"), "hello");
}

// ===============================================================
// String methods: contains, starts_with, ends_with
// ===============================================================

#[test]
fn test_str_contains_true() {
    assert_bool(&run_get("x = \"hello world\".contains(\"world\")", "x"), true);
}

#[test]
fn test_str_contains_false() {
    assert_bool(&run_get("x = \"hello\".contains(\"xyz\")", "x"), false);
}

#[test]
fn test_str_contains_empty_substr() {
    assert_bool(&run_get("x = \"hello\".contains(\"\")", "x"), true);
}

#[test]
fn test_str_starts_with_true() {
    assert_bool(&run_get("x = \"hello world\".starts_with(\"hello\")", "x"), true);
}

#[test]
fn test_str_starts_with_false() {
    assert_bool(&run_get("x = \"hello world\".starts_with(\"world\")", "x"), false);
}

#[test]
fn test_str_ends_with_true() {
    assert_bool(&run_get("x = \"hello world\".ends_with(\"world\")", "x"), true);
}

#[test]
fn test_str_ends_with_false() {
    assert_bool(&run_get("x = \"hello world\".ends_with(\"hello\")", "x"), false);
}

// ===============================================================
// String methods: split, join
// ===============================================================

#[test]
fn test_str_split_comma() {
    let val = run_get("x = \"a,b,c\".split(\",\")", "x");
    assert_eq!(list_len(&val), 3);
    assert_str(&list_get(&val, 0), "a");
    assert_str(&list_get(&val, 2), "c");
}

#[test]
fn test_str_split_space() {
    let val = run_get("x = \"hello world\".split(\" \")", "x");
    assert_eq!(list_len(&val), 2);
    assert_str(&list_get(&val, 0), "hello");
}

#[test]
fn test_str_split_no_match() {
    let val = run_get("x = \"hello\".split(\",\")", "x");
    assert_eq!(list_len(&val), 1);
    assert_str(&list_get(&val, 0), "hello");
}

#[test]
fn test_str_join_list() {
    assert_str(&run_get("x = [\"a\", \"b\", \"c\"].join(\", \")", "x"), "a, b, c");
}

#[test]
fn test_str_join_empty_sep() {
    assert_str(&run_get("x = [\"a\", \"b\"].join(\"\")", "x"), "ab");
}

#[test]
fn test_str_join_single_item() {
    assert_str(&run_get("x = [\"hello\"].join(\", \")", "x"), "hello");
}

// ===============================================================
// String methods: replace, find, rfind
// ===============================================================

#[test]
fn test_str_replace_basic() {
    assert_str(&run_get("x = \"hello world\".replace(\"world\", \"rust\")", "x"), "hello rust");
}

#[test]
fn test_str_replace_no_match() {
    assert_str(&run_get("x = \"hello\".replace(\"xyz\", \"abc\")", "x"), "hello");
}

#[test]
fn test_str_replace_multiple() {
    assert_str(&run_get("x = \"aaa\".replace(\"a\", \"b\")", "x"), "bbb");
}

#[test]
fn test_str_find_found() {
    assert_int(&run_get("x = \"hello world\".find(\"world\")", "x"), 6);
}

#[test]
fn test_str_find_not_found() {
    assert_nil(&run_get("x = \"hello\".find(\"xyz\")", "x"));
}

#[test]
fn test_str_find_beginning() {
    assert_int(&run_get("x = \"hello\".find(\"he\")", "x"), 0);
}

#[test]
fn test_str_rfind_basic() {
    assert_int(&run_get("x = \"hello hello\".rfind(\"hello\")", "x"), 6);
}

#[test]
fn test_str_rfind_not_found() {
    assert_nil(&run_get("x = \"hello\".rfind(\"xyz\")", "x"));
}

// ===============================================================
// String methods: slice, chars, repeat
// ===============================================================

#[test]
fn test_str_slice_basic() {
    assert_str(&run_get("x = \"hello world\".slice(0, 5)", "x"), "hello");
}

#[test]
fn test_str_slice_middle() {
    assert_str(&run_get("x = \"hello\".slice(1, 4)", "x"), "ell");
}

#[test]
fn test_str_chars_basic() {
    let val = run_get("x = \"hi\".chars()", "x");
    assert_eq!(list_len(&val), 2);
}

#[test]
fn test_str_chars_empty() {
    let val = run_get("x = \"\".chars()", "x");
    assert_eq!(list_len(&val), 0);
}

#[test]
fn test_str_repeat_three() {
    assert_str(&run_get("x = \"ab\".repeat(3)", "x"), "ababab");
}

#[test]
fn test_str_repeat_zero() {
    assert_str(&run_get("x = \"hello\".repeat(0)", "x"), "");
}

#[test]
fn test_str_repeat_one() {
    assert_str(&run_get("x = \"hello\".repeat(1)", "x"), "hello");
}

// ===============================================================
// String methods: is_empty, is_numeric, is_alpha
// ===============================================================

#[test]
fn test_str_is_empty_true() {
    assert_bool(&run_get("x = \"\".is_empty()", "x"), true);
}

#[test]
fn test_str_is_empty_false() {
    assert_bool(&run_get("x = \"hi\".is_empty()", "x"), false);
}

#[test]
fn test_str_is_numeric_digits() {
    assert_bool(&run_get("x = \"12345\".is_numeric()", "x"), true);
}

#[test]
fn test_str_is_numeric_mixed() {
    assert_bool(&run_get("x = \"12abc\".is_numeric()", "x"), false);
}

#[test]
fn test_str_is_numeric_empty() {
    assert_bool(&run_get("x = \"\".is_numeric()", "x"), false);
}

#[test]
fn test_str_is_alpha_letters() {
    assert_bool(&run_get("x = \"hello\".is_alpha()", "x"), true);
}

#[test]
fn test_str_is_alpha_with_numbers() {
    assert_bool(&run_get("x = \"hello1\".is_alpha()", "x"), false);
}

#[test]
fn test_str_is_alpha_empty() {
    assert_bool(&run_get("x = \"\".is_alpha()", "x"), false);
}

// ===============================================================
// String methods: pad_left, pad_right
// ===============================================================

#[test]
fn test_str_pad_left_spaces() {
    assert_str(&run_get("x = \"hi\".pad_left(5)", "x"), "   hi");
}

#[test]
fn test_str_pad_left_custom_char() {
    assert_str(&run_get("x = \"42\".pad_left(5, \"0\")", "x"), "00042");
}

#[test]
fn test_str_pad_left_already_long() {
    assert_str(&run_get("x = \"hello\".pad_left(3)", "x"), "hello");
}

#[test]
fn test_str_pad_right_spaces() {
    assert_str(&run_get("x = \"hi\".pad_right(5)", "x"), "hi   ");
}

#[test]
fn test_str_pad_right_custom_char() {
    assert_str(&run_get("x = \"42\".pad_right(5, \"0\")", "x"), "42000");
}

#[test]
fn test_str_pad_right_already_long() {
    assert_str(&run_get("x = \"hello\".pad_right(3)", "x"), "hello");
}

// ===============================================================
// String methods: reverse, capitalize, title
// ===============================================================

#[test]
fn test_str_reverse_basic() {
    assert_str(&run_get("x = \"hello\".reverse()", "x"), "olleh");
}

#[test]
fn test_str_reverse_palindrome() {
    assert_str(&run_get("x = \"racecar\".reverse()", "x"), "racecar");
}

#[test]
fn test_str_reverse_empty() {
    assert_str(&run_get("x = \"\".reverse()", "x"), "");
}

#[test]
fn test_str_capitalize_basic() {
    assert_str(&run_get("x = \"hello world\".capitalize()", "x"), "Hello world");
}

#[test]
fn test_str_capitalize_empty() {
    assert_str(&run_get("x = \"\".capitalize()", "x"), "");
}

#[test]
fn test_str_capitalize_single_char() {
    assert_str(&run_get("x = \"a\".capitalize()", "x"), "A");
}

#[test]
fn test_str_title_basic() {
    assert_str(&run_get("x = \"hello world\".title()", "x"), "Hello World");
}

#[test]
fn test_str_title_single_word() {
    assert_str(&run_get("x = \"hello\".title()", "x"), "Hello");
}

#[test]
fn test_str_title_already_titled() {
    assert_str(&run_get("x = \"Hello World\".title()", "x"), "Hello World");
}

// ===============================================================
// List methods: push, pop, shift, unshift
// ===============================================================

#[test]
fn test_list_push_basic() {
    assert_int(&run_get("l = [1, 2]\nl.push(3)\nx = l.len()", "x"), 3);
}

#[test]
fn test_list_push_access() {
    assert_int(&run_get("l = [1, 2]\nl.push(99)\nx = l[2]", "x"), 99);
}

#[test]
fn test_list_pop_basic() {
    assert_int(&run_get("l = [1, 2, 3]\nx = l.pop()", "x"), 3);
}

#[test]
fn test_list_pop_mutates() {
    assert_int(&run_get("l = [1, 2, 3]\nl.pop()\nx = l.len()", "x"), 2);
}

#[test]
fn test_list_pop_empty() {
    assert_nil(&run_get("x = [].pop()", "x"));
}

// ===============================================================
// List methods: map, filter, reduce, find
// ===============================================================

#[test]
fn test_list_map_squares() {
    let val = run_get("x = [1, 2, 3, 4].map(n -> n ** 2)", "x");
    assert_eq!(list_len(&val), 4);
    assert_int(&list_get(&val, 3), 16);
}

#[test]
fn test_list_map_strings() {
    let val = run_get("x = [\"a\", \"b\", \"c\"].map(s -> s.upper())", "x");
    assert_str(&list_get(&val, 0), "A");
    assert_str(&list_get(&val, 2), "C");
}

#[test]
fn test_list_map_empty() {
    let val = run_get("x = [].map(n -> n * 2)", "x");
    assert_eq!(list_len(&val), 0);
}

#[test]
fn test_list_filter_evens() {
    let val = run_get("x = [1, 2, 3, 4, 5, 6].filter(n -> n % 2 == 0)", "x");
    assert_eq!(list_len(&val), 3);
}

#[test]
fn test_list_filter_none_match() {
    let val = run_get("x = [1, 2, 3].filter(n -> n > 100)", "x");
    assert_eq!(list_len(&val), 0);
}

#[test]
fn test_list_filter_all_match() {
    let val = run_get("x = [1, 2, 3].filter(n -> n > 0)", "x");
    assert_eq!(list_len(&val), 3);
}

#[test]
fn test_list_reduce_sum() {
    assert_int(&run_get("x = [1, 2, 3, 4, 5].reduce(0, (acc, n) -> acc + n)", "x"), 15);
}

#[test]
fn test_list_reduce_product() {
    assert_int(&run_get("x = [1, 2, 3, 4].reduce(1, (acc, n) -> acc * n)", "x"), 24);
}

#[test]
fn test_list_reduce_max() {
    assert_int(&run_get("x = [3, 1, 4, 1, 5].reduce(0, (acc, n) -> if n > acc then n else acc)", "x"), 5);
}

#[test]
fn test_list_find_basic() {
    assert_int(&run_get("x = [10, 20, 30].find(n -> n > 15)", "x"), 20);
}

#[test]
fn test_list_find_not_found() {
    assert_nil(&run_get("x = [1, 2, 3].find(n -> n > 100)", "x"));
}

// ===============================================================
// List methods: sort, reverse, unique, flatten
// ===============================================================

#[test]
fn test_list_sort_basic() {
    let val = run_get("x = [3, 1, 4, 1, 5].sort()", "x");
    assert_int(&list_get(&val, 0), 1);
    assert_int(&list_get(&val, 4), 5);
}

#[test]
fn test_list_sort_empty() {
    let val = run_get("x = [].sort()", "x");
    assert_eq!(list_len(&val), 0);
}

#[test]
fn test_list_sort_already_sorted() {
    let val = run_get("x = [1, 2, 3].sort()", "x");
    assert_int(&list_get(&val, 0), 1);
    assert_int(&list_get(&val, 2), 3);
}

#[test]
fn test_list_reverse_basic() {
    let val = run_get("x = [1, 2, 3].reverse()", "x");
    assert_int(&list_get(&val, 0), 3);
    assert_int(&list_get(&val, 2), 1);
}

#[test]
fn test_list_reverse_empty() {
    let val = run_get("x = [].reverse()", "x");
    assert_eq!(list_len(&val), 0);
}

#[test]
fn test_list_unique_basic() {
    let val = run_get("x = [1, 2, 2, 3, 3, 3].unique()", "x");
    assert_eq!(list_len(&val), 3);
}

#[test]
fn test_list_unique_no_dups() {
    let val = run_get("x = [1, 2, 3].unique()", "x");
    assert_eq!(list_len(&val), 3);
}

#[test]
fn test_list_flatten_basic() {
    let val = run_get("x = [[1, 2], [3, 4], [5]].flatten()", "x");
    assert_eq!(list_len(&val), 5);
}

#[test]
fn test_list_flatten_empty() {
    let val = run_get("x = [].flatten()", "x");
    assert_eq!(list_len(&val), 0);
}

// ===============================================================
// List methods: any, all, sum, min, max
// ===============================================================

#[test]
fn test_list_any_true() {
    assert_bool(&run_get("x = [1, 2, 3].any(n -> n == 2)", "x"), true);
}

#[test]
fn test_list_any_false() {
    assert_bool(&run_get("x = [1, 2, 3].any(n -> n > 10)", "x"), false);
}

#[test]
fn test_list_all_true() {
    assert_bool(&run_get("x = [2, 4, 6].all(n -> n % 2 == 0)", "x"), true);
}

#[test]
fn test_list_all_false() {
    assert_bool(&run_get("x = [2, 3, 4].all(n -> n % 2 == 0)", "x"), false);
}

#[test]
fn test_list_sum_ints() {
    assert_int(&run_get("x = [1, 2, 3, 4, 5].sum()", "x"), 15);
}

#[test]
fn test_list_sum_empty() {
    assert_int(&run_get("x = [].sum()", "x"), 0);
}

#[test]
fn test_list_min_basic() {
    assert_int(&run_get("x = [5, 3, 8, 1, 4].min()", "x"), 1);
}

#[test]
fn test_list_min_empty() {
    assert_nil(&run_get("x = [].min()", "x"));
}

#[test]
fn test_list_max_basic() {
    assert_int(&run_get("x = [5, 3, 8, 1, 4].max()", "x"), 8);
}

#[test]
fn test_list_max_empty() {
    assert_nil(&run_get("x = [].max()", "x"));
}

// ===============================================================
// List methods: zip, join, includes/contains, index_of
// ===============================================================

#[test]
fn test_list_zip_basic() {
    let val = run_get("x = [1, 2, 3].zip([\"a\", \"b\", \"c\"])", "x");
    assert_eq!(list_len(&val), 3);
}

#[test]
fn test_list_zip_unequal() {
    let val = run_get("x = [1, 2, 3].zip([\"a\", \"b\"])", "x");
    assert_eq!(list_len(&val), 2);
}

#[test]
fn test_list_join_comma() {
    assert_str(&run_get("x = [\"x\", \"y\", \"z\"].join(\"-\")", "x"), "x-y-z");
}

#[test]
fn test_list_contains_true() {
    assert_bool(&run_get("x = [1, 2, 3].contains(2)", "x"), true);
}

#[test]
fn test_list_contains_false() {
    assert_bool(&run_get("x = [1, 2, 3].contains(99)", "x"), false);
}

#[test]
fn test_list_index_of_found() {
    assert_int(&run_get("x = [10, 20, 30].index_of(20)", "x"), 1);
}

#[test]
fn test_list_index_of_not_found() {
    assert_int(&run_get("x = [10, 20, 30].index_of(99)", "x"), -1);
}

// ===============================================================
// List methods: slice, insert, remove
// ===============================================================

#[test]
fn test_list_insert_at_position() {
    assert_int(&run_get("l = [1, 3]\nl.insert(1, 2)\nx = l[1]", "x"), 2);
}

#[test]
fn test_list_insert_grows() {
    assert_int(&run_get("l = [1, 2]\nl.insert(0, 0)\nx = l.len()", "x"), 3);
}

#[test]
fn test_list_remove_basic() {
    assert_int(&run_get("l = [1, 2, 3]\nx = l.remove(1)", "x"), 2);
}

#[test]
fn test_list_remove_shrinks() {
    assert_int(&run_get("l = [1, 2, 3]\nl.remove(0)\nx = l.len()", "x"), 2);
}

// ===============================================================
// List methods: first, last, len
// ===============================================================

#[test]
fn test_list_first_basic() {
    assert_int(&run_get("x = [10, 20, 30].first()", "x"), 10);
}

#[test]
fn test_list_first_empty() {
    assert_nil(&run_get("x = [].first()", "x"));
}

#[test]
fn test_list_last_basic() {
    assert_int(&run_get("x = [10, 20, 30].last()", "x"), 30);
}

#[test]
fn test_list_last_empty() {
    assert_nil(&run_get("x = [].last()", "x"));
}

#[test]
fn test_list_len_basic() {
    assert_int(&run_get("x = [1, 2, 3, 4, 5].len()", "x"), 5);
}

#[test]
fn test_list_len_empty() {
    assert_int(&run_get("x = [].len()", "x"), 0);
}

// ===============================================================
// Map methods: keys, values, entries
// ===============================================================

#[test]
fn test_map_keys_len() {
    assert_int(&run_get("x = {\"a\": 1, \"b\": 2, \"c\": 3}.keys().len()", "x"), 3);
}

#[test]
fn test_map_values_len() {
    assert_int(&run_get("x = {\"a\": 1, \"b\": 2}.values().len()", "x"), 2);
}

#[test]
fn test_map_entries_len() {
    assert_int(&run_get("x = {\"a\": 1, \"b\": 2}.entries().len()", "x"), 2);
}

// ===============================================================
// Map methods: has/contains_key, get, set
// ===============================================================

#[test]
fn test_map_contains_key_true() {
    assert_bool(&run_get("x = {\"a\": 1}.contains_key(\"a\")", "x"), true);
}

#[test]
fn test_map_contains_key_false() {
    assert_bool(&run_get("x = {\"a\": 1}.contains_key(\"z\")", "x"), false);
}

#[test]
fn test_map_get_existing() {
    assert_int(&run_get("x = {\"a\": 42}.get(\"a\")", "x"), 42);
}

#[test]
fn test_map_get_missing() {
    assert_nil(&run_get("x = {\"a\": 1}.get(\"z\")", "x"));
}

#[test]
fn test_map_set_new_key() {
    assert_int(&run_get("m = {\"a\": 1}\nm.set(\"b\", 2)\nx = m.len()", "x"), 2);
}

#[test]
fn test_map_set_overwrite() {
    assert_int(&run_get("m = {\"a\": 1}\nm.set(\"a\", 99)\nx = m.get(\"a\")", "x"), 99);
}

// ===============================================================
// Map methods: delete/remove, clear, merge, len
// ===============================================================

#[test]
fn test_map_remove_key() {
    assert_int(&run_get("m = {\"a\": 1, \"b\": 2}\nm.remove(\"a\")\nx = m.len()", "x"), 1);
}

#[test]
fn test_map_merge_basic() {
    assert_int(&run_get("x = {\"a\": 1}.merge({\"b\": 2}).len()", "x"), 2);
}

#[test]
fn test_map_merge_overwrite() {
    assert_int(&run_get("x = {\"a\": 1}.merge({\"a\": 99}).get(\"a\")", "x"), 99);
}

#[test]
fn test_map_len_basic() {
    assert_int(&run_get("x = {\"a\": 1, \"b\": 2, \"c\": 3}.len()", "x"), 3);
}

#[test]
fn test_map_len_empty() {
    assert_int(&run_get("x = {}.len()", "x"), 0);
}

#[test]
fn test_map_is_empty_true() {
    assert_bool(&run_get("x = {}.is_empty()", "x"), true);
}

#[test]
fn test_map_is_empty_false() {
    assert_bool(&run_get("x = {\"a\": 1}.is_empty()", "x"), false);
}

#[test]
fn test_map_get_or_default() {
    assert_int(&run_get("x = {\"a\": 1}.get_or(\"z\", 42)", "x"), 42);
}

#[test]
fn test_map_get_or_existing() {
    assert_int(&run_get("x = {\"a\": 1}.get_or(\"a\", 99)", "x"), 1);
}

#[test]
fn test_map_invert_basic() {
    assert_int(&run_get("x = {\"a\": \"x\", \"b\": \"y\"}.invert().len()", "x"), 2);
}

// ===============================================================
// Type conversions: int(), float(), str()
// ===============================================================

#[test]
fn test_conv_str_of_int() {
    assert_str(&run_get("x = str(42)", "x"), "42");
}

#[test]
fn test_conv_str_of_float() {
    let val = run_get("x = str(3.14)", "x");
    match &val {
        Value::String(s) => assert!(s.starts_with("3.14"), "expected '3.14...', got '{}'", s),
        _ => panic!("expected String, got {:?}", val),
    }
}

#[test]
fn test_conv_str_of_bool() {
    assert_str(&run_get("x = str(true)", "x"), "true");
}

#[test]
fn test_conv_str_of_string() {
    assert_str(&run_get("x = str(\"hello\")", "x"), "hello");
}

#[test]
fn test_conv_int_of_string() {
    assert_int(&run_get("x = int(\"42\")", "x"), 42);
}

#[test]
fn test_conv_int_of_float() {
    assert_int(&run_get("x = int(3.7)", "x"), 3);
}

#[test]
fn test_conv_int_of_bool_true() {
    assert_int(&run_get("x = int(true)", "x"), 1);
}

#[test]
fn test_conv_int_of_bool_false() {
    assert_int(&run_get("x = int(false)", "x"), 0);
}

#[test]
fn test_conv_float_of_string() {
    assert_float_approx(&run_get("x = float(\"3.14\")", "x"), 3.14);
}

#[test]
fn test_conv_float_of_int() {
    assert_float_approx(&run_get("x = float(42)", "x"), 42.0);
}

// ===============================================================
// String interpolation and method chaining
// ===============================================================

#[test]
fn test_str_interpolation_basic() {
    assert_str(&run_get("name = \"World\"\nx = \"Hello, {name}!\"", "x"), "Hello, World!");
}

#[test]
fn test_str_interpolation_expression() {
    assert_str(&run_get("a = 3\nb = 4\nx = \"Sum is {a + b}\"", "x"), "Sum is 7");
}

#[test]
fn test_str_method_chain_upper_len() {
    assert_int(&run_get("x = \"hello\".upper().len()", "x"), 5);
}

#[test]
fn test_str_method_chain_trim_lower() {
    assert_str(&run_get("x = \"  HELLO  \".trim().lower()", "x"), "hello");
}

#[test]
fn test_str_method_chain_replace_upper() {
    assert_str(
        &run_get("x = \"hello world\".replace(\"world\", \"rust\").upper()", "x"),
        "HELLO RUST",
    );
}

// ===============================================================
// List method chaining
// ===============================================================

#[test]
fn test_list_chain_filter_map_sum() {
    assert_int(
        &run_get("x = [1, 2, 3, 4, 5].filter(n -> n > 2).map(n -> n * 10).sum()", "x"),
        120, // (3+4+5)*10 = 120
    );
}

#[test]
fn test_list_chain_sort_first() {
    assert_int(&run_get("x = [5, 3, 1, 4, 2].sort().first()", "x"), 1);
}

#[test]
fn test_list_chain_sort_last() {
    assert_int(&run_get("x = [5, 3, 1, 4, 2].sort().last()", "x"), 5);
}

#[test]
fn test_list_chain_unique_sort() {
    let val = run_get("x = [3, 1, 2, 1, 3].unique().sort()", "x");
    assert_int(&list_get(&val, 0), 1);
    assert_int(&list_get(&val, 2), 3);
}

#[test]
fn test_list_chain_reverse_take() {
    let val = run_get("x = [1, 2, 3, 4, 5].reverse().take(3)", "x");
    assert_eq!(list_len(&val), 3);
    assert_int(&list_get(&val, 0), 5);
}

// ===============================================================
// Comprehensions with stdlib
// ===============================================================

#[test]
fn test_list_comprehension_basic() {
    assert_int(&run_get("x = [i * i for i in 0..5].sum()", "x"), 30); // 0+1+4+9+16
}

#[test]
fn test_list_comprehension_filtered() {
    assert_int(
        &run_get("x = [i for i in 0..10 if i % 2 == 0].len()", "x"),
        5,
    );
}

#[test]
fn test_list_comprehension_with_method() {
    assert_int(
        &run_get("x = [i * 2 for i in 0..5].sum()", "x"),
        20, // 0+2+4+6+8
    );
}

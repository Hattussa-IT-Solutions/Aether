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
        Value::Int(n) => assert_eq!(*n, expected, "expected Int({}), got {:?}", expected, val),
        _ => panic!("expected Int({}), got {:?}", expected, val),
    }
}

fn assert_bool(val: &Value, expected: bool) {
    match val {
        Value::Bool(b) => assert_eq!(*b, expected, "expected Bool({}), got {:?}", expected, val),
        _ => panic!("expected Bool({}), got {:?}", expected, val),
    }
}

fn assert_str(val: &Value, expected: &str) {
    match val {
        Value::String(s) => assert_eq!(s, expected, "expected '{}', got {:?}", expected, val),
        _ => panic!("expected String('{}'), got {:?}", expected, val),
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

// ═══════════════════════════════════════════════════════
// len
// ═══════════════════════════════════════════════════════

#[test]
fn test_len_basic() {
    assert_int(&run_get("x = [1, 2, 3].len()", "x"), 3);
}

#[test]
fn test_len_empty() {
    assert_int(&run_get("x = [].len()", "x"), 0);
}

// ═══════════════════════════════════════════════════════
// push / pop
// ═══════════════════════════════════════════════════════

#[test]
fn test_push_basic() {
    assert_int(&run_get("l = [1, 2]\nl.push(3)\nx = l.len()", "x"), 3);
}

#[test]
fn test_push_then_access() {
    assert_int(&run_get("l = [1, 2]\nl.push(99)\nx = l[2]", "x"), 99);
}

#[test]
fn test_pop_basic() {
    assert_int(&run_get("l = [1, 2, 3]\nx = l.pop()", "x"), 3);
}

#[test]
fn test_pop_empty() {
    // popping empty list returns Nil
    let val = run_get("x = [].pop()", "x");
    assert_nil(&val);
}

// ═══════════════════════════════════════════════════════
// first / last
// ═══════════════════════════════════════════════════════

#[test]
fn test_first_basic() {
    assert_int(&run_get("x = [10, 20, 30].first()", "x"), 10);
}

#[test]
fn test_first_empty() {
    assert_nil(&run_get("x = [].first()", "x"));
}

#[test]
fn test_first_single() {
    assert_int(&run_get("x = [42].first()", "x"), 42);
}

#[test]
fn test_last_basic() {
    assert_int(&run_get("x = [10, 20, 30].last()", "x"), 30);
}

#[test]
fn test_last_empty() {
    assert_nil(&run_get("x = [].last()", "x"));
}

#[test]
fn test_last_single() {
    assert_int(&run_get("x = [42].last()", "x"), 42);
}

// ═══════════════════════════════════════════════════════
// contains
// ═══════════════════════════════════════════════════════

#[test]
fn test_contains_true() {
    assert_bool(&run_get("x = [1, 2, 3].contains(2)", "x"), true);
}

#[test]
fn test_contains_false() {
    assert_bool(&run_get("x = [1, 2, 3].contains(99)", "x"), false);
}

// ═══════════════════════════════════════════════════════
// reverse
// ═══════════════════════════════════════════════════════

#[test]
fn test_reverse_basic() {
    let val = run_get("x = [1, 2, 3].reverse()", "x");
    assert_int(&list_get(&val, 0), 3);
    assert_int(&list_get(&val, 2), 1);
}

#[test]
fn test_reverse_empty() {
    let val = run_get("x = [].reverse()", "x");
    assert_eq!(list_len(&val), 0);
}

// ═══════════════════════════════════════════════════════
// join
// ═══════════════════════════════════════════════════════

#[test]
fn test_join_basic() {
    assert_str(&run_get("x = [\"a\", \"b\", \"c\"].join(\", \")", "x"), "a, b, c");
}

#[test]
fn test_join_empty_sep() {
    assert_str(&run_get("x = [\"a\", \"b\"].join(\"\")", "x"), "ab");
}

// ═══════════════════════════════════════════════════════
// map
// ═══════════════════════════════════════════════════════

#[test]
fn test_map_double() {
    let val = run_get("x = [1, 2, 3].map(n -> n * 2)", "x");
    assert_int(&list_get(&val, 0), 2);
    assert_int(&list_get(&val, 1), 4);
    assert_int(&list_get(&val, 2), 6);
}

#[test]
fn test_map_empty() {
    let val = run_get("x = [].map(n -> n * 2)", "x");
    assert_eq!(list_len(&val), 0);
}

// ═══════════════════════════════════════════════════════
// filter
// ═══════════════════════════════════════════════════════

#[test]
fn test_filter_basic() {
    let val = run_get("x = [1, 2, 3, 4, 5].filter(n -> n > 3)", "x");
    assert_eq!(list_len(&val), 2);
    assert_int(&list_get(&val, 0), 4);
}

#[test]
fn test_filter_empty_result() {
    let val = run_get("x = [1, 2, 3].filter(n -> n > 100)", "x");
    assert_eq!(list_len(&val), 0);
}

// ═══════════════════════════════════════════════════════
// sum
// ═══════════════════════════════════════════════════════

#[test]
fn test_sum_basic() {
    assert_int(&run_get("x = [1, 2, 3].sum()", "x"), 6);
}

#[test]
fn test_sum_empty() {
    assert_int(&run_get("x = [].sum()", "x"), 0);
}

#[test]
fn test_sum_single() {
    assert_int(&run_get("x = [42].sum()", "x"), 42);
}

#[test]
fn test_sum_large() {
    // sum of 0..1000 = 0+1+...+999 = 499500
    assert_int(
        &run_get("x = [i for i in 0..1000].sum()", "x"),
        499500,
    );
}

// ═══════════════════════════════════════════════════════
// min / max
// ═══════════════════════════════════════════════════════

#[test]
fn test_min_basic() {
    assert_int(&run_get("x = [3, 1, 2].min()", "x"), 1);
}

#[test]
fn test_min_empty() {
    assert_nil(&run_get("x = [].min()", "x"));
}

#[test]
fn test_max_basic() {
    assert_int(&run_get("x = [3, 1, 2].max()", "x"), 3);
}

#[test]
fn test_max_empty() {
    assert_nil(&run_get("x = [].max()", "x"));
}

// ═══════════════════════════════════════════════════════
// index_of
// ═══════════════════════════════════════════════════════

#[test]
fn test_index_of_found() {
    assert_int(&run_get("x = [10, 20, 30].index_of(20)", "x"), 1);
}

#[test]
fn test_index_of_not_found() {
    assert_int(&run_get("x = [10, 20, 30].index_of(99)", "x"), -1);
}

// ═══════════════════════════════════════════════════════
// remove
// ═══════════════════════════════════════════════════════

#[test]
fn test_remove_basic() {
    let val = run_get("l = [1, 2, 3]\nl.remove(1)\nx = l.len()", "x");
    assert_int(&val, 2);
}

#[test]
fn test_remove_returns_value() {
    assert_int(&run_get("l = [10, 20, 30]\nx = l.remove(1)", "x"), 20);
}

// ═══════════════════════════════════════════════════════
// insert
// ═══════════════════════════════════════════════════════

#[test]
fn test_insert_basic() {
    let val = run_get("l = [1, 3]\nl.insert(1, 2)\nx = l[1]", "x");
    assert_int(&val, 2);
}

#[test]
fn test_insert_length() {
    let val = run_get("l = [1, 2]\nl.insert(0, 0)\nx = l.len()", "x");
    assert_int(&val, 3);
}

// ═══════════════════════════════════════════════════════
// sort
// ═══════════════════════════════════════════════════════

#[test]
fn test_sort_basic() {
    let val = run_get("x = [3, 1, 2].sort()", "x");
    assert_int(&list_get(&val, 0), 1);
    assert_int(&list_get(&val, 1), 2);
    assert_int(&list_get(&val, 2), 3);
}

#[test]
fn test_sort_empty() {
    let val = run_get("x = [].sort()", "x");
    assert_eq!(list_len(&val), 0);
}

// ═══════════════════════════════════════════════════════
// any / all
// ═══════════════════════════════════════════════════════

#[test]
fn test_any_true() {
    assert_bool(&run_get("x = [1, 2, 3].any(n -> n > 2)", "x"), true);
}

#[test]
fn test_any_false() {
    assert_bool(&run_get("x = [1, 2, 3].any(n -> n > 10)", "x"), false);
}

#[test]
fn test_all_true() {
    assert_bool(&run_get("x = [2, 4, 6].all(n -> n % 2 == 0)", "x"), true);
}

#[test]
fn test_all_false() {
    assert_bool(&run_get("x = [2, 3, 6].all(n -> n % 2 == 0)", "x"), false);
}

// ═══════════════════════════════════════════════════════
// unique
// ═══════════════════════════════════════════════════════

#[test]
fn test_unique_basic() {
    let val = run_get("x = [1, 2, 2, 3, 3].unique()", "x");
    assert_eq!(list_len(&val), 3);
}

#[test]
fn test_unique_all_same() {
    let val = run_get("x = [5, 5, 5].unique()", "x");
    assert_eq!(list_len(&val), 1);
}

// ═══════════════════════════════════════════════════════
// reduce
// ═══════════════════════════════════════════════════════

#[test]
fn test_reduce_sum() {
    assert_int(&run_get("x = [1, 2, 3, 4].reduce(0, (acc, n) -> acc + n)", "x"), 10);
}

#[test]
fn test_reduce_empty() {
    // reduce with empty list returns the initial value
    assert_int(&run_get("x = [].reduce(99, (acc, n) -> acc + n)", "x"), 99);
}

// ═══════════════════════════════════════════════════════
// flat_map
// ═══════════════════════════════════════════════════════

#[test]
fn test_flat_map_basic() {
    let val = run_get("x = [1, 2, 3].flat_map(n -> [n, n * 10])", "x");
    assert_eq!(list_len(&val), 6);
    assert_int(&list_get(&val, 0), 1);
    assert_int(&list_get(&val, 1), 10);
}

#[test]
fn test_flat_map_empty() {
    let val = run_get("x = [].flat_map(n -> [n, n])", "x");
    assert_eq!(list_len(&val), 0);
}

// ═══════════════════════════════════════════════════════
// zip
// ═══════════════════════════════════════════════════════

#[test]
fn test_zip_basic() {
    let val = run_get("x = [1, 2, 3].zip([\"a\", \"b\", \"c\"])", "x");
    assert_eq!(list_len(&val), 3);
}

#[test]
fn test_zip_shorter() {
    // zip stops at the shorter list
    let val = run_get("x = [1, 2, 3].zip([10, 20])", "x");
    assert_eq!(list_len(&val), 2);
}

// ═══════════════════════════════════════════════════════
// chunks
// ═══════════════════════════════════════════════════════

#[test]
fn test_chunks_basic() {
    let val = run_get("x = [1, 2, 3, 4].chunks(2)", "x");
    assert_eq!(list_len(&val), 2);
}

#[test]
fn test_chunks_uneven() {
    // 5 elements, chunk size 2 => [[1,2],[3,4],[5]]
    let val = run_get("x = [1, 2, 3, 4, 5].chunks(2)", "x");
    assert_eq!(list_len(&val), 3);
}

// ═══════════════════════════════════════════════════════
// get (safe index)
// ═══════════════════════════════════════════════════════

#[test]
fn test_get_valid() {
    assert_int(&run_get("x = [10, 20, 30].get(1)", "x"), 20);
}

#[test]
fn test_get_out_of_bounds() {
    assert_nil(&run_get("x = [1, 2, 3].get(99)", "x"));
}

#[test]
fn test_get_negative() {
    assert_nil(&run_get("x = [1, 2, 3].get(-1)", "x"));
}

// ═══════════════════════════════════════════════════════
// sort_desc
// ═══════════════════════════════════════════════════════

#[test]
fn test_sort_desc_basic() {
    let val = run_get("x = [1, 3, 2].sort_desc()", "x");
    assert_int(&list_get(&val, 0), 3);
    assert_int(&list_get(&val, 1), 2);
    assert_int(&list_get(&val, 2), 1);
}

#[test]
fn test_sort_desc_empty() {
    let val = run_get("x = [].sort_desc()", "x");
    assert_eq!(list_len(&val), 0);
}

// ═══════════════════════════════════════════════════════
// find
// ═══════════════════════════════════════════════════════

#[test]
fn test_find_found() {
    assert_int(&run_get("x = [1, 2, 3, 4].find(n -> n > 2)", "x"), 3);
}

#[test]
fn test_find_not_found() {
    assert_nil(&run_get("x = [1, 2, 3].find(n -> n > 100)", "x"));
}

// ═══════════════════════════════════════════════════════
// find_index
// ═══════════════════════════════════════════════════════

#[test]
fn test_find_index_found() {
    assert_int(&run_get("x = [10, 20, 30].find_index(n -> n == 20)", "x"), 1);
}

#[test]
fn test_find_index_not_found() {
    assert_int(&run_get("x = [1, 2, 3].find_index(n -> n > 100)", "x"), -1);
}

// ═══════════════════════════════════════════════════════
// take
// ═══════════════════════════════════════════════════════

#[test]
fn test_take_basic() {
    let val = run_get("x = [1, 2, 3, 4, 5].take(3)", "x");
    assert_eq!(list_len(&val), 3);
    assert_int(&list_get(&val, 0), 1);
    assert_int(&list_get(&val, 2), 3);
}

#[test]
fn test_take_more_than_length() {
    let val = run_get("x = [1, 2].take(10)", "x");
    assert_eq!(list_len(&val), 2);
}

#[test]
fn test_take_zero() {
    let val = run_get("x = [1, 2, 3].take(0)", "x");
    assert_eq!(list_len(&val), 0);
}

// ═══════════════════════════════════════════════════════
// drop
// ═══════════════════════════════════════════════════════

#[test]
fn test_drop_basic() {
    let val = run_get("x = [1, 2, 3, 4, 5].drop(2)", "x");
    assert_eq!(list_len(&val), 3);
    assert_int(&list_get(&val, 0), 3);
}

#[test]
fn test_drop_all() {
    let val = run_get("x = [1, 2, 3].drop(10)", "x");
    assert_eq!(list_len(&val), 0);
}

#[test]
fn test_drop_zero() {
    let val = run_get("x = [1, 2, 3].drop(0)", "x");
    assert_eq!(list_len(&val), 3);
}

// ═══════════════════════════════════════════════════════
// take_while
// ═══════════════════════════════════════════════════════

#[test]
fn test_take_while_basic() {
    let val = run_get("x = [1, 2, 3, 4, 5].take_while(n -> n < 4)", "x");
    assert_eq!(list_len(&val), 3);
    assert_int(&list_get(&val, 2), 3);
}

#[test]
fn test_take_while_none() {
    let val = run_get("x = [5, 6, 7].take_while(n -> n < 1)", "x");
    assert_eq!(list_len(&val), 0);
}

// ═══════════════════════════════════════════════════════
// drop_while
// ═══════════════════════════════════════════════════════

#[test]
fn test_drop_while_basic() {
    let val = run_get("x = [1, 2, 3, 4, 5].drop_while(n -> n < 3)", "x");
    assert_eq!(list_len(&val), 3);
    assert_int(&list_get(&val, 0), 3);
}

#[test]
fn test_drop_while_all() {
    let val = run_get("x = [1, 2, 3].drop_while(n -> n < 100)", "x");
    assert_eq!(list_len(&val), 0);
}

// ═══════════════════════════════════════════════════════
// compact
// ═══════════════════════════════════════════════════════

#[test]
fn test_compact_removes_nil() {
    let val = run_get("x = [1, nil, 3, nil, 5].compact()", "x");
    assert_eq!(list_len(&val), 3);
    assert_int(&list_get(&val, 0), 1);
    assert_int(&list_get(&val, 1), 3);
}

#[test]
fn test_compact_no_nils() {
    let val = run_get("x = [1, 2, 3].compact()", "x");
    assert_eq!(list_len(&val), 3);
}

#[test]
fn test_compact_all_nil() {
    let val = run_get("x = [nil, nil].compact()", "x");
    assert_eq!(list_len(&val), 0);
}

// ═══════════════════════════════════════════════════════
// flatten
// ═══════════════════════════════════════════════════════

#[test]
fn test_flatten_basic() {
    let val = run_get("x = [[1, 2], [3, 4]].flatten()", "x");
    assert_eq!(list_len(&val), 4);
    assert_int(&list_get(&val, 0), 1);
    assert_int(&list_get(&val, 3), 4);
}

#[test]
fn test_flatten_mixed() {
    // non-list items pass through
    let val = run_get("x = [[1, 2], 3, [4]].flatten()", "x");
    assert_eq!(list_len(&val), 4);
}

#[test]
fn test_flatten_empty() {
    let val = run_get("x = [].flatten()", "x");
    assert_eq!(list_len(&val), 0);
}

// ═══════════════════════════════════════════════════════
// is_empty
// ═══════════════════════════════════════════════════════

#[test]
fn test_is_empty_true() {
    assert_bool(&run_get("x = [].is_empty()", "x"), true);
}

#[test]
fn test_is_empty_false() {
    assert_bool(&run_get("x = [1].is_empty()", "x"), false);
}

// ═══════════════════════════════════════════════════════
// is_sorted
// ═══════════════════════════════════════════════════════

#[test]
fn test_is_sorted_true() {
    assert_bool(&run_get("x = [1, 2, 3, 4].is_sorted()", "x"), true);
}

#[test]
fn test_is_sorted_false() {
    assert_bool(&run_get("x = [1, 3, 2].is_sorted()", "x"), false);
}

#[test]
fn test_is_sorted_empty() {
    assert_bool(&run_get("x = [].is_sorted()", "x"), true);
}

#[test]
fn test_is_sorted_single() {
    assert_bool(&run_get("x = [42].is_sorted()", "x"), true);
}

// ═══════════════════════════════════════════════════════
// rotate
// ═══════════════════════════════════════════════════════

#[test]
fn test_rotate_basic() {
    let val = run_get("x = [1, 2, 3, 4, 5].rotate(2)", "x");
    assert_int(&list_get(&val, 0), 3);
    assert_int(&list_get(&val, 4), 2);
}

#[test]
fn test_rotate_zero() {
    let val = run_get("x = [1, 2, 3].rotate(0)", "x");
    assert_int(&list_get(&val, 0), 1);
}

#[test]
fn test_rotate_empty() {
    let val = run_get("x = [].rotate(2)", "x");
    assert_eq!(list_len(&val), 0);
}

// ═══════════════════════════════════════════════════════
// interleave
// ═══════════════════════════════════════════════════════

#[test]
fn test_interleave_equal_lengths() {
    let val = run_get("x = [1, 2, 3].interleave([10, 20, 30])", "x");
    assert_eq!(list_len(&val), 6);
    assert_int(&list_get(&val, 0), 1);
    assert_int(&list_get(&val, 1), 10);
    assert_int(&list_get(&val, 2), 2);
}

#[test]
fn test_interleave_unequal() {
    let val = run_get("x = [1, 2].interleave([10, 20, 30])", "x");
    assert_eq!(list_len(&val), 5);
}

// ═══════════════════════════════════════════════════════
// mean
// ═══════════════════════════════════════════════════════

#[test]
fn test_mean_basic() {
    assert_float_approx(&run_get("x = [1, 2, 3, 4, 5].mean()", "x"), 3.0);
}

#[test]
fn test_mean_empty() {
    assert_nil(&run_get("x = [].mean()", "x"));
}

#[test]
fn test_mean_single() {
    assert_float_approx(&run_get("x = [10].mean()", "x"), 10.0);
}

// ═══════════════════════════════════════════════════════
// median
// ═══════════════════════════════════════════════════════

#[test]
fn test_median_odd() {
    assert_float_approx(&run_get("x = [3, 1, 2].median()", "x"), 2.0);
}

#[test]
fn test_median_even() {
    assert_float_approx(&run_get("x = [1, 2, 3, 4].median()", "x"), 2.5);
}

#[test]
fn test_median_empty() {
    assert_nil(&run_get("x = [].median()", "x"));
}

// ═══════════════════════════════════════════════════════
// to_set
// ═══════════════════════════════════════════════════════

#[test]
fn test_to_set_deduplicates() {
    let val = run_get("x = [1, 2, 2, 3].to_set()", "x");
    match &val {
        Value::Set(s) => assert_eq!(s.borrow().len(), 3),
        _ => panic!("expected Set, got {:?}", val),
    }
}

#[test]
fn test_to_set_empty() {
    let val = run_get("x = [].to_set()", "x");
    match &val {
        Value::Set(s) => assert_eq!(s.borrow().len(), 0),
        _ => panic!("expected Set, got {:?}", val),
    }
}

// ═══════════════════════════════════════════════════════
// frequencies
// ═══════════════════════════════════════════════════════

#[test]
fn test_frequencies_basic() {
    let val = run_get("x = [1, 2, 2, 3, 3, 3].frequencies()", "x");
    match &val {
        Value::Map(m) => {
            let m = m.borrow();
            match m.get("1") { Some(Value::Int(1)) => {}, other => panic!("expected Int(1) for key '1', got {:?}", other) }
            match m.get("2") { Some(Value::Int(2)) => {}, other => panic!("expected Int(2) for key '2', got {:?}", other) }
            match m.get("3") { Some(Value::Int(3)) => {}, other => panic!("expected Int(3) for key '3', got {:?}", other) }
        }
        _ => panic!("expected Map, got {:?}", val),
    }
}

#[test]
fn test_frequencies_empty() {
    let val = run_get("x = [].frequencies()", "x");
    match &val {
        Value::Map(m) => assert_eq!(m.borrow().len(), 0),
        _ => panic!("expected Map, got {:?}", val),
    }
}

// ═══════════════════════════════════════════════════════
// partition
// ═══════════════════════════════════════════════════════

#[test]
fn test_partition_basic() {
    let val = run_get("x = [1, 2, 3, 4, 5].partition(n -> n % 2 == 0)", "x");
    match &val {
        Value::Tuple(parts) => {
            assert_eq!(parts.len(), 2);
            let evens = &parts[0];
            let odds = &parts[1];
            assert_eq!(list_len(evens), 2); // 2, 4
            assert_eq!(list_len(odds), 3);  // 1, 3, 5
        }
        _ => panic!("expected Tuple, got {:?}", val),
    }
}

#[test]
fn test_partition_empty() {
    let val = run_get("x = [].partition(n -> n > 0)", "x");
    match &val {
        Value::Tuple(parts) => {
            assert_eq!(list_len(&parts[0]), 0);
            assert_eq!(list_len(&parts[1]), 0);
        }
        _ => panic!("expected Tuple, got {:?}", val),
    }
}

// ═══════════════════════════════════════════════════════
// window (sliding)
// ═══════════════════════════════════════════════════════

#[test]
fn test_window_basic() {
    let val = run_get("x = [1, 2, 3, 4].window(2)", "x");
    // windows of size 2: [1,2],[2,3],[3,4] => 3 windows
    assert_eq!(list_len(&val), 3);
}

#[test]
fn test_window_size_1() {
    let val = run_get("x = [1, 2, 3].window(1)", "x");
    assert_eq!(list_len(&val), 3);
}

#[test]
fn test_window_empty() {
    let val = run_get("x = [].window(2)", "x");
    assert_eq!(list_len(&val), 0);
}

// ═══════════════════════════════════════════════════════
// Large list tests
// ═══════════════════════════════════════════════════════

#[test]
fn test_large_list_len() {
    assert_int(&run_get("x = [i for i in 0..1000].len()", "x"), 1000);
}

#[test]
fn test_large_list_filter() {
    // even numbers in 0..1000: 0,2,4,...,998 = 500 elements
    assert_int(
        &run_get("x = [i for i in 0..1000].filter(n -> n % 2 == 0).len()", "x"),
        500,
    );
}

#[test]
fn test_large_list_map() {
    assert_int(
        &run_get("x = [i for i in 0..1000].map(n -> n * 2).len()", "x"),
        1000,
    );
}

#[test]
fn test_large_list_sort() {
    // sorting reversed large list
    let val = run_get(
        "x = [i for i in 0..100].reverse().sort()",
        "x",
    );
    assert_int(&list_get(&val, 0), 0);
    assert_int(&list_get(&val, 99), 99);
}

// ═══════════════════════════════════════════════════════
// Nil elements
// ═══════════════════════════════════════════════════════

#[test]
fn test_nil_in_list_compact() {
    let val = run_get("x = [1, nil, 3].compact()", "x");
    assert_eq!(list_len(&val), 2);
}

#[test]
fn test_nil_in_list_len() {
    // nil counts as an element
    assert_int(&run_get("x = [1, nil, 3].len()", "x"), 3);
}

#[test]
fn test_nil_in_list_contains() {
    assert_bool(&run_get("x = [1, nil, 3].contains(nil)", "x"), true);
}

// ═══════════════════════════════════════════════════════
// Nested lists
// ═══════════════════════════════════════════════════════

#[test]
fn test_nested_flatten() {
    let val = run_get("x = [[1, 2], [3, 4], [5]].flatten()", "x");
    assert_eq!(list_len(&val), 5);
}

#[test]
fn test_nested_flat_map() {
    let val = run_get("x = [[1, 2], [3]].flat_map(inner -> inner.map(n -> n * 10))", "x");
    assert_eq!(list_len(&val), 3);
}

// ═══════════════════════════════════════════════════════
// Method chaining
// ═══════════════════════════════════════════════════════

#[test]
fn test_chain_filter_map_sum() {
    assert_int(
        &run_get("x = [1, 2, 3, 4, 5].filter(n -> n % 2 == 0).map(n -> n * 10).sum()", "x"),
        60, // (2+4)*10
    );
}

#[test]
fn test_chain_sort_reverse() {
    let val = run_get("x = [3, 1, 2].sort().reverse()", "x");
    assert_int(&list_get(&val, 0), 3);
    assert_int(&list_get(&val, 2), 1);
}

#[test]
fn test_take_then_sum() {
    assert_int(&run_get("x = [1, 2, 3, 4, 5].take(3).sum()", "x"), 6);
}

#[test]
fn test_drop_then_first() {
    assert_int(&run_get("x = [1, 2, 3, 4].drop(2).first()", "x"), 3);
}

// ═══════════════════════════════════════════════════════
// Index bounds for regular indexing
// ═══════════════════════════════════════════════════════

#[test]
fn test_get_first_element() {
    assert_int(&run_get("x = [10, 20, 30].get(0)", "x"), 10);
}

#[test]
fn test_get_last_element() {
    assert_int(&run_get("x = [10, 20, 30].get(2)", "x"), 30);
}

#[test]
fn test_get_on_empty() {
    assert_nil(&run_get("x = [].get(0)", "x"));
}

// ═══════════════════════════════════════════════════════
// sort_desc with floats
// ═══════════════════════════════════════════════════════

#[test]
fn test_sort_desc_floats() {
    let val = run_get("x = [1.5, 3.2, 2.7].sort_desc()", "x");
    assert_float_approx(&list_get(&val, 0), 3.2);
}

// ═══════════════════════════════════════════════════════
// mean / median with floats
// ═══════════════════════════════════════════════════════

#[test]
fn test_mean_floats() {
    assert_float_approx(&run_get("x = [1.0, 2.0, 3.0].mean()", "x"), 2.0);
}

#[test]
fn test_median_single() {
    assert_float_approx(&run_get("x = [7].median()", "x"), 7.0);
}

// ═══════════════════════════════════════════════════════
// frequencies with strings
// ═══════════════════════════════════════════════════════

#[test]
fn test_frequencies_strings() {
    let val = run_get("x = [\"a\", \"b\", \"a\", \"c\", \"a\"].frequencies()", "x");
    match &val {
        Value::Map(m) => {
            let m = m.borrow();
            match m.get("a") { Some(Value::Int(3)) => {}, other => panic!("expected Int(3) for key 'a', got {:?}", other) }
            match m.get("b") { Some(Value::Int(1)) => {}, other => panic!("expected Int(1) for key 'b', got {:?}", other) }
        }
        _ => panic!("expected Map, got {:?}", val),
    }
}

// ═══════════════════════════════════════════════════════
// window content check
// ═══════════════════════════════════════════════════════

#[test]
fn test_window_content() {
    let val = run_get("x = [1, 2, 3].window(2)", "x");
    let first_window = list_get(&val, 0);
    assert_int(&list_get(&first_window, 0), 1);
    assert_int(&list_get(&first_window, 1), 2);
}

// ═══════════════════════════════════════════════════════
// rotate negative
// ═══════════════════════════════════════════════════════

#[test]
fn test_rotate_by_length() {
    // rotating by full length should give same list
    let val = run_get("x = [1, 2, 3].rotate(3)", "x");
    assert_int(&list_get(&val, 0), 1);
}

// ═══════════════════════════════════════════════════════
// is_sorted with equal elements
// ═══════════════════════════════════════════════════════

#[test]
fn test_is_sorted_equal() {
    assert_bool(&run_get("x = [5, 5, 5].is_sorted()", "x"), true);
}

// ═══════════════════════════════════════════════════════
// drop_while nothing to drop
// ═══════════════════════════════════════════════════════

#[test]
fn test_drop_while_nothing() {
    let val = run_get("x = [5, 6, 7].drop_while(n -> n < 1)", "x");
    assert_eq!(list_len(&val), 3);
}

// ═══════════════════════════════════════════════════════
// Partition: all match / none match
// ═══════════════════════════════════════════════════════

#[test]
fn test_partition_all_match() {
    let val = run_get("x = [2, 4, 6].partition(n -> n % 2 == 0)", "x");
    match &val {
        Value::Tuple(parts) => {
            assert_eq!(list_len(&parts[0]), 3);
            assert_eq!(list_len(&parts[1]), 0);
        }
        _ => panic!("expected Tuple"),
    }
}

#[test]
fn test_partition_none_match() {
    let val = run_get("x = [1, 3, 5].partition(n -> n % 2 == 0)", "x");
    match &val {
        Value::Tuple(parts) => {
            assert_eq!(list_len(&parts[0]), 0);
            assert_eq!(list_len(&parts[1]), 3);
        }
        _ => panic!("expected Tuple"),
    }
}

// ═══════════════════════════════════════════════════════
// compact preserves order
// ═══════════════════════════════════════════════════════

#[test]
fn test_compact_order() {
    let val = run_get("x = [3, nil, 1, nil, 2].compact()", "x");
    assert_int(&list_get(&val, 0), 3);
    assert_int(&list_get(&val, 1), 1);
    assert_int(&list_get(&val, 2), 2);
}

// ═══════════════════════════════════════════════════════
// interleave with empty list
// ═══════════════════════════════════════════════════════

#[test]
fn test_interleave_with_empty() {
    let val = run_get("x = [1, 2, 3].interleave([])", "x");
    assert_eq!(list_len(&val), 3);
}

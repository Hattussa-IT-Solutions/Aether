use aether::interpreter;
use aether::interpreter::environment::Environment;
use aether::interpreter::values::Value;
use aether::lexer::scanner::Scanner;
use aether::parser::parser::Parser;

// ═══════════════════════════════════════════════════════════════
// Helper functions
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

fn run_err(source: &str) -> String {
    let mut scanner = Scanner::new(source, "test.ae".to_string());
    let tokens = scanner.scan_tokens();
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("parse failed");
    let mut env = Environment::new();
    interpreter::register_builtins(&mut env);
    interpreter::interpret(&program, &mut env).unwrap_err()
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
            (f - expected).abs() < 1e-9,
            "expected Float({}), got Float({})",
            expected,
            f
        ),
        Value::Int(n) => assert!(
            (*n as f64 - expected).abs() < 1e-9,
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
    match val {
        Value::Nil => {}
        _ => panic!("expected Nil, got {:?}", val),
    }
}

// ═══════════════════════════════════════════════════════════════
// 1. Arithmetic with Int (12 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_int_add() {
    assert_int(&run_get("x = 3 + 7", "x"), 10);
}

#[test]
fn test_int_add_negative() {
    assert_int(&run_get("x = -5 + 3", "x"), -2);
}

#[test]
fn test_int_sub() {
    assert_int(&run_get("x = 20 - 8", "x"), 12);
}

#[test]
fn test_int_sub_negative_result() {
    assert_int(&run_get("x = 3 - 10", "x"), -7);
}

#[test]
fn test_int_mul() {
    assert_int(&run_get("x = 6 * 7", "x"), 42);
}

#[test]
fn test_int_mul_zero() {
    assert_int(&run_get("x = 999 * 0", "x"), 0);
}

#[test]
fn test_int_div() {
    assert_int(&run_get("x = 15 / 3", "x"), 5);
}

#[test]
fn test_int_div_truncates() {
    assert_int(&run_get("x = 10 / 3", "x"), 3);
}

#[test]
fn test_int_mod() {
    assert_int(&run_get("x = 17 % 5", "x"), 2);
}

#[test]
fn test_int_mod_even_check() {
    assert_int(&run_get("x = 10 % 2", "x"), 0);
}

#[test]
fn test_int_pow() {
    assert_int(&run_get("x = 2 ** 10", "x"), 1024);
}

#[test]
fn test_int_pow_zero() {
    assert_int(&run_get("x = 5 ** 0", "x"), 1);
}

// ═══════════════════════════════════════════════════════════════
// 2. Arithmetic with Float (12 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_float_add() {
    assert_float_approx(&run_get("x = 1.5 + 2.5", "x"), 4.0);
}

#[test]
fn test_float_add_precision() {
    assert_float_approx(&run_get("x = 0.1 + 0.2", "x"), 0.30000000000000004);
}

#[test]
fn test_float_sub() {
    assert_float_approx(&run_get("x = 10.5 - 3.2", "x"), 7.3);
}

#[test]
fn test_float_sub_negative() {
    assert_float_approx(&run_get("x = 1.0 - 5.5", "x"), -4.5);
}

#[test]
fn test_float_mul() {
    assert_float_approx(&run_get("x = 3.0 * 4.5", "x"), 13.5);
}

#[test]
fn test_float_mul_small() {
    assert_float_approx(&run_get("x = 0.5 * 0.5", "x"), 0.25);
}

#[test]
fn test_float_div() {
    assert_float_approx(&run_get("x = 10.0 / 4.0", "x"), 2.5);
}

#[test]
fn test_float_div_irrational() {
    assert_float_approx(&run_get("x = 1.0 / 3.0", "x"), 1.0 / 3.0);
}

#[test]
fn test_float_mod() {
    assert_float_approx(&run_get("x = 7.5 % 2.0", "x"), 1.5);
}

#[test]
fn test_float_mod_small() {
    assert_float_approx(&run_get("x = 5.5 % 1.5", "x"), 5.5_f64 % 1.5_f64);
}

#[test]
fn test_float_pow() {
    assert_float_approx(&run_get("x = 2.0 ** 3.0", "x"), 8.0);
}

#[test]
fn test_float_pow_fractional() {
    assert_float_approx(&run_get("x = 9.0 ** 0.5", "x"), 3.0);
}

// ═══════════════════════════════════════════════════════════════
// 3. Mixed Int/Float arithmetic (12 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_mixed_add_int_float() {
    assert_float_approx(&run_get("x = 1 + 2.5", "x"), 3.5);
}

#[test]
fn test_mixed_add_float_int() {
    assert_float_approx(&run_get("x = 3.14 + 2", "x"), 5.14);
}

#[test]
fn test_mixed_sub_int_float() {
    assert_float_approx(&run_get("x = 10 - 3.5", "x"), 6.5);
}

#[test]
fn test_mixed_sub_float_int() {
    assert_float_approx(&run_get("x = 5.5 - 3", "x"), 2.5);
}

#[test]
fn test_mixed_mul_int_float() {
    assert_float_approx(&run_get("x = 3 * 2.5", "x"), 7.5);
}

#[test]
fn test_mixed_mul_float_int() {
    assert_float_approx(&run_get("x = 2.5 * 4", "x"), 10.0);
}

#[test]
fn test_mixed_div_int_float() {
    assert_float_approx(&run_get("x = 7 / 2.0", "x"), 3.5);
}

#[test]
fn test_mixed_div_float_int() {
    assert_float_approx(&run_get("x = 7.0 / 2", "x"), 3.5);
}

#[test]
fn test_mixed_mod_int_float() {
    assert_float_approx(&run_get("x = 7 % 2.5", "x"), 7.0_f64 % 2.5_f64);
}

#[test]
fn test_mixed_mod_float_int() {
    assert_float_approx(&run_get("x = 7.5 % 2", "x"), 7.5_f64 % 2.0_f64);
}

#[test]
fn test_mixed_pow_int_float() {
    assert_float_approx(&run_get("x = 4 ** 0.5", "x"), 2.0);
}

#[test]
fn test_mixed_pow_float_int() {
    assert_float_approx(&run_get("x = 2.5 ** 2", "x"), 6.25);
}

// ═══════════════════════════════════════════════════════════════
// 4. String operations (10 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_str_concat() {
    assert_str(&run_get("x = \"hello\" + \" world\"", "x"), "hello world");
}

#[test]
fn test_str_concat_empty() {
    assert_str(&run_get("x = \"\" + \"abc\"", "x"), "abc");
}

#[test]
fn test_str_concat_both_empty() {
    assert_str(&run_get("x = \"\" + \"\"", "x"), "");
}

#[test]
fn test_str_concat_with_int() {
    // Str + non-Str auto-converts right operand
    assert_str(&run_get("x = \"value: \" + 42", "x"), "value: 42");
}

#[test]
fn test_str_concat_with_bool() {
    assert_str(&run_get("x = \"flag: \" + true", "x"), "flag: true");
}

#[test]
fn test_str_eq() {
    assert_bool(&run_get("x = \"abc\" == \"abc\"", "x"), true);
}

#[test]
fn test_str_neq() {
    assert_bool(&run_get("x = \"abc\" != \"xyz\"", "x"), true);
}

#[test]
fn test_str_lt() {
    assert_bool(&run_get("x = \"abc\" < \"abd\"", "x"), true);
}

#[test]
fn test_str_gt() {
    assert_bool(&run_get("x = \"b\" > \"a\"", "x"), true);
}

#[test]
fn test_str_lteq() {
    assert_bool(&run_get("x = \"abc\" <= \"abc\"", "x"), true);
}

// ═══════════════════════════════════════════════════════════════
// 5. Comparison operators (30 tests)
// ═══════════════════════════════════════════════════════════════

// -- Int comparisons --

#[test]
fn test_cmp_int_eq_true() {
    assert_bool(&run_get("x = 5 == 5", "x"), true);
}

#[test]
fn test_cmp_int_eq_false() {
    assert_bool(&run_get("x = 5 == 6", "x"), false);
}

#[test]
fn test_cmp_int_neq_true() {
    assert_bool(&run_get("x = 5 != 3", "x"), true);
}

#[test]
fn test_cmp_int_neq_false() {
    assert_bool(&run_get("x = 5 != 5", "x"), false);
}

#[test]
fn test_cmp_int_lt() {
    assert_bool(&run_get("x = 3 < 5", "x"), true);
}

#[test]
fn test_cmp_int_lt_false() {
    assert_bool(&run_get("x = 5 < 5", "x"), false);
}

#[test]
fn test_cmp_int_gt() {
    assert_bool(&run_get("x = 10 > 3", "x"), true);
}

#[test]
fn test_cmp_int_gt_false() {
    assert_bool(&run_get("x = 3 > 10", "x"), false);
}

#[test]
fn test_cmp_int_lteq_equal() {
    assert_bool(&run_get("x = 5 <= 5", "x"), true);
}

#[test]
fn test_cmp_int_lteq_less() {
    assert_bool(&run_get("x = 4 <= 5", "x"), true);
}

// -- Float comparisons --

#[test]
fn test_cmp_float_eq() {
    assert_bool(&run_get("x = 3.14 == 3.14", "x"), true);
}

#[test]
fn test_cmp_float_neq() {
    assert_bool(&run_get("x = 3.14 != 2.71", "x"), true);
}

#[test]
fn test_cmp_float_lt() {
    assert_bool(&run_get("x = 1.5 < 2.5", "x"), true);
}

#[test]
fn test_cmp_float_gt() {
    assert_bool(&run_get("x = 9.9 > 9.1", "x"), true);
}

#[test]
fn test_cmp_float_gteq() {
    assert_bool(&run_get("x = 5.0 >= 5.0", "x"), true);
}

// -- Mixed Int/Float comparisons --

#[test]
fn test_cmp_mixed_eq() {
    assert_bool(&run_get("x = 5 == 5.0", "x"), true);
}

#[test]
fn test_cmp_mixed_neq() {
    assert_bool(&run_get("x = 5 != 5.1", "x"), true);
}

#[test]
fn test_cmp_mixed_lt() {
    assert_bool(&run_get("x = 3 < 3.5", "x"), true);
}

#[test]
fn test_cmp_mixed_gt() {
    assert_bool(&run_get("x = 4.0 > 3", "x"), true);
}

#[test]
fn test_cmp_mixed_lteq() {
    assert_bool(&run_get("x = 5 <= 5.0", "x"), true);
}

// -- String comparisons --

#[test]
fn test_cmp_str_eq_true() {
    assert_bool(&run_get("x = \"hello\" == \"hello\"", "x"), true);
}

#[test]
fn test_cmp_str_eq_false() {
    assert_bool(&run_get("x = \"hello\" == \"world\"", "x"), false);
}

#[test]
fn test_cmp_str_neq_true() {
    assert_bool(&run_get("x = \"abc\" != \"def\"", "x"), true);
}

#[test]
fn test_cmp_str_lt_lex() {
    assert_bool(&run_get("x = \"apple\" < \"banana\"", "x"), true);
}

#[test]
fn test_cmp_str_gteq() {
    assert_bool(&run_get("x = \"z\" >= \"a\"", "x"), true);
}

// -- Bool comparisons --

#[test]
fn test_cmp_bool_eq_true() {
    assert_bool(&run_get("x = true == true", "x"), true);
}

#[test]
fn test_cmp_bool_eq_false() {
    assert_bool(&run_get("x = true == false", "x"), false);
}

#[test]
fn test_cmp_bool_neq() {
    assert_bool(&run_get("x = true != false", "x"), true);
}

// -- Nil comparisons --

#[test]
fn test_cmp_nil_eq_nil() {
    assert_bool(&run_get("x = nil == nil", "x"), true);
}

#[test]
fn test_cmp_nil_neq_int() {
    assert_bool(&run_get("x = nil != 0", "x"), true);
}

// ═══════════════════════════════════════════════════════════════
// 6. Logical operators (15 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_logical_and_true() {
    assert_bool(&run_get("x = true && true", "x"), true);
}

#[test]
fn test_logical_and_false() {
    assert_bool(&run_get("x = true && false", "x"), false);
}

#[test]
fn test_logical_and_both_false() {
    assert_bool(&run_get("x = false && false", "x"), false);
}

#[test]
fn test_logical_or_true() {
    assert_bool(&run_get("x = false || true", "x"), true);
}

#[test]
fn test_logical_or_both_true() {
    assert_bool(&run_get("x = true || true", "x"), true);
}

#[test]
fn test_logical_or_both_false() {
    assert_bool(&run_get("x = false || false", "x"), false);
}

#[test]
fn test_logical_not_true() {
    assert_bool(&run_get("x = !true", "x"), false);
}

#[test]
fn test_logical_not_false() {
    assert_bool(&run_get("x = !false", "x"), true);
}

#[test]
fn test_logical_and_keyword() {
    assert_bool(&run_get("x = true and false", "x"), false);
}

#[test]
fn test_logical_or_keyword() {
    assert_bool(&run_get("x = false or true", "x"), true);
}

#[test]
fn test_logical_not_keyword() {
    assert_bool(&run_get("x = not true", "x"), false);
}

#[test]
fn test_logical_short_circuit_and() {
    // false && ... should not evaluate the right side
    assert_bool(&run_get("x = false && (1 / 0 == 0)", "x"), false);
}

#[test]
fn test_logical_short_circuit_or() {
    // true || ... should not evaluate the right side
    assert_bool(&run_get("x = true || (1 / 0 == 0)", "x"), true);
}

#[test]
fn test_logical_truthy_int() {
    // Non-zero int is truthy
    assert_bool(&run_get("x = !0", "x"), true);
}

#[test]
fn test_logical_truthy_nonempty_string() {
    assert_bool(&run_get("x = !\"\"", "x"), true);
}

// ═══════════════════════════════════════════════════════════════
// 7. Division by zero (3 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_div_by_zero_int() {
    let err = run_err("x = 10 / 0");
    assert!(err.contains("division by zero"), "expected division by zero error, got: {}", err);
}

#[test]
fn test_div_by_zero_float() {
    let err = run_err("x = 10.0 / 0.0");
    assert!(err.contains("division by zero"), "expected division by zero error, got: {}", err);
}

#[test]
fn test_div_by_zero_mixed() {
    let err = run_err("x = 10 / 0.0");
    assert!(err.contains("division by zero"), "expected division by zero error, got: {}", err);
}

// ═══════════════════════════════════════════════════════════════
// 8. Nil operations (10 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_nil_eq_nil() {
    assert_bool(&run_get("x = nil == nil", "x"), true);
}

#[test]
fn test_nil_neq_int() {
    assert_bool(&run_get("x = nil != 42", "x"), true);
}

#[test]
fn test_nil_neq_str() {
    assert_bool(&run_get("x = nil != \"hello\"", "x"), true);
}

#[test]
fn test_nil_neq_bool() {
    assert_bool(&run_get("x = nil != false", "x"), true);
}

#[test]
fn test_nil_eq_int_false() {
    assert_bool(&run_get("x = nil == 0", "x"), false);
}

#[test]
fn test_nil_coalesce_nil() {
    assert_int(&run_get("x = nil ?? 42", "x"), 42);
}

#[test]
fn test_nil_coalesce_non_nil() {
    assert_int(&run_get("x = 10 ?? 42", "x"), 10);
}

#[test]
fn test_nil_coalesce_str() {
    assert_str(&run_get("x = nil ?? \"default\"", "x"), "default");
}

#[test]
fn test_nil_is_falsy() {
    assert_bool(&run_get("x = !nil", "x"), true);
}

#[test]
fn test_nil_coalesce_chain() {
    assert_int(&run_get("x = nil ?? nil ?? 99", "x"), 99);
}

// ═══════════════════════════════════════════════════════════════
// 9. List operations (8 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_list_eq_same() {
    assert_bool(&run_get("x = [1, 2, 3] == [1, 2, 3]", "x"), true);
}

#[test]
fn test_list_eq_different() {
    assert_bool(&run_get("x = [1, 2, 3] == [1, 2, 4]", "x"), false);
}

#[test]
fn test_list_neq() {
    assert_bool(&run_get("x = [1, 2] != [1, 2, 3]", "x"), true);
}

#[test]
fn test_list_eq_empty() {
    assert_bool(&run_get("x = [] == []", "x"), true);
}

#[test]
fn test_list_neq_empty_vs_nonempty() {
    assert_bool(&run_get("x = [] != [1]", "x"), true);
}

#[test]
fn test_list_eq_nested() {
    assert_bool(&run_get("x = [[1, 2], [3]] == [[1, 2], [3]]", "x"), true);
}

#[test]
fn test_list_neq_nested() {
    assert_bool(&run_get("x = [[1, 2], [3]] != [[1, 2], [4]]", "x"), true);
}

#[test]
fn test_list_eq_mixed_types() {
    assert_bool(&run_get("x = [1, \"two\", true] == [1, \"two\", true]", "x"), true);
}

// ═══════════════════════════════════════════════════════════════
// 10. Map operations (5 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_map_eq_same() {
    assert_bool(
        &run_get("a = {\"x\": 1, \"y\": 2}\nb = {\"x\": 1, \"y\": 2}\nx = a == b", "x"),
        true,
    );
}

#[test]
fn test_map_eq_different_values() {
    assert_bool(
        &run_get("a = {\"x\": 1, \"y\": 2}\nb = {\"x\": 1, \"y\": 3}\nx = a == b", "x"),
        false,
    );
}

#[test]
fn test_map_neq() {
    assert_bool(
        &run_get("a = {\"x\": 1}\nb = {\"x\": 2}\nx = a != b", "x"),
        true,
    );
}

#[test]
fn test_map_eq_empty() {
    assert_bool(&run_get("x = {} == {}", "x"), true);
}

#[test]
fn test_map_neq_different_keys() {
    assert_bool(
        &run_get("a = {\"x\": 1}\nb = {\"y\": 1}\nx = a != b", "x"),
        true,
    );
}

// ═══════════════════════════════════════════════════════════════
// 11. Type coercion: int(), float(), str() (15 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_int_from_float() {
    assert_int(&run_get("x = int(3.9)", "x"), 3);
}

#[test]
fn test_int_from_string() {
    assert_int(&run_get("x = int(\"42\")", "x"), 42);
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
fn test_int_from_int() {
    assert_int(&run_get("x = int(7)", "x"), 7);
}

#[test]
fn test_float_from_int() {
    assert_float_approx(&run_get("x = float(5)", "x"), 5.0);
}

#[test]
fn test_float_from_string() {
    assert_float_approx(&run_get("x = float(\"3.14\")", "x"), 3.14);
}

#[test]
fn test_float_from_float() {
    assert_float_approx(&run_get("x = float(2.718)", "x"), 2.718);
}

#[test]
fn test_str_from_int() {
    assert_str(&run_get("x = str(42)", "x"), "42");
}

#[test]
fn test_str_from_float() {
    assert_str(&run_get("x = str(3.14)", "x"), "3.14");
}

#[test]
fn test_str_from_bool_true() {
    assert_str(&run_get("x = str(true)", "x"), "true");
}

#[test]
fn test_str_from_bool_false() {
    assert_str(&run_get("x = str(false)", "x"), "false");
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
fn test_type_of_str() {
    assert_str(&run_get("x = type(\"hello\")", "x"), "Str");
}

// ═══════════════════════════════════════════════════════════════
// 12. Unary operators (8 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_unary_neg_int() {
    assert_int(&run_get("x = -42", "x"), -42);
}

#[test]
fn test_unary_neg_float() {
    assert_float_approx(&run_get("x = -3.14", "x"), -3.14);
}

#[test]
fn test_unary_neg_double() {
    // Double negation
    assert_int(&run_get("x = -(-5)", "x"), 5);
}

#[test]
fn test_unary_not_bool() {
    assert_bool(&run_get("x = !true", "x"), false);
}

#[test]
fn test_unary_not_nil() {
    assert_bool(&run_get("x = !nil", "x"), true);
}

#[test]
fn test_unary_not_zero() {
    assert_bool(&run_get("x = !0", "x"), true);
}

#[test]
fn test_unary_bitnot() {
    assert_int(&run_get("x = ~0", "x"), -1);
}

#[test]
fn test_unary_bitnot_value() {
    assert_int(&run_get("x = ~255", "x"), !255_i64);
}

// ═══════════════════════════════════════════════════════════════
// 13. Assignment operators (12 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_assign_add_int() {
    assert_int(&run_get("x = 10\nx += 5", "x"), 15);
}

#[test]
fn test_assign_sub_int() {
    assert_int(&run_get("x = 10\nx -= 3", "x"), 7);
}

#[test]
fn test_assign_mul_int() {
    assert_int(&run_get("x = 6\nx *= 7", "x"), 42);
}

#[test]
fn test_assign_div_int() {
    assert_int(&run_get("x = 20\nx /= 4", "x"), 5);
}

#[test]
fn test_assign_mod_int() {
    assert_int(&run_get("x = 17\nx %= 5", "x"), 2);
}

#[test]
fn test_assign_pow_int() {
    assert_int(&run_get("x = 2\nx **= 8", "x"), 256);
}

#[test]
fn test_assign_add_float() {
    assert_float_approx(&run_get("x = 1.5\nx += 2.5", "x"), 4.0);
}

#[test]
fn test_assign_sub_float() {
    assert_float_approx(&run_get("x = 10.0\nx -= 3.5", "x"), 6.5);
}

#[test]
fn test_assign_mul_float() {
    assert_float_approx(&run_get("x = 2.5\nx *= 4.0", "x"), 10.0);
}

#[test]
fn test_assign_add_str() {
    assert_str(&run_get("x = \"hello\"\nx += \" world\"", "x"), "hello world");
}

#[test]
fn test_assign_add_mixed() {
    assert_float_approx(&run_get("x = 5\nx += 2.5", "x"), 7.5);
}

#[test]
fn test_assign_sub_mixed() {
    assert_float_approx(&run_get("x = 10\nx -= 3.5", "x"), 6.5);
}

// ═══════════════════════════════════════════════════════════════
// 14. Bitwise operators (12 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_bitwise_and() {
    // 12 & 10 = 8  (1100 & 1010 = 1000)
    assert_int(&run_get("x = 12 & 10", "x"), 8);
}

#[test]
fn test_bitwise_and_mask() {
    assert_int(&run_get("x = 255 & 15", "x"), 15);
}

#[test]
fn test_bitwise_or() {
    // 12 | 10 = 14  (1100 | 1010 = 1110)
    assert_int(&run_get("x = 12 | 10", "x"), 14);
}

#[test]
fn test_bitwise_or_flags() {
    assert_int(&run_get("x = 1 | 2 | 4", "x"), 7);
}

#[test]
fn test_bitwise_xor() {
    // 12 ^ 10 = 6  (1100 ^ 1010 = 0110)
    assert_int(&run_get("x = 12 ^ 10", "x"), 6);
}

#[test]
fn test_bitwise_xor_self() {
    assert_int(&run_get("x = 42 ^ 42", "x"), 0);
}

#[test]
fn test_bitwise_shl() {
    assert_int(&run_get("x = 1 << 3", "x"), 8);
}

#[test]
fn test_bitwise_shl_mul() {
    assert_int(&run_get("x = 5 << 2", "x"), 20);
}

#[test]
fn test_bitwise_shr() {
    assert_int(&run_get("x = 16 >> 2", "x"), 4);
}

#[test]
fn test_bitwise_shr_large() {
    assert_int(&run_get("x = 255 >> 4", "x"), 15);
}

#[test]
fn test_bitwise_not() {
    assert_int(&run_get("x = ~0", "x"), -1);
}

#[test]
fn test_bitwise_not_invert() {
    // ~(~x) == x
    assert_int(&run_get("x = ~(~42)", "x"), 42);
}

// ═══════════════════════════════════════════════════════════════
// 15. Error cases (15 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_err_sub_strings() {
    let err = run_err("x = \"hello\" - \"world\"");
    assert!(!err.is_empty(), "expected an error for string subtraction");
}

#[test]
fn test_err_mul_strings() {
    let err = run_err("x = \"hello\" * \"world\"");
    assert!(!err.is_empty(), "expected an error for string multiplication");
}

#[test]
fn test_err_div_strings() {
    let err = run_err("x = \"hello\" / \"world\"");
    assert!(!err.is_empty(), "expected an error for string division");
}

#[test]
fn test_err_mod_strings() {
    let err = run_err("x = \"hello\" % \"world\"");
    assert!(!err.is_empty(), "expected an error for string modulus");
}

#[test]
fn test_err_add_bool_int() {
    let err = run_err("x = true + 1");
    assert!(!err.is_empty(), "expected an error for bool + int");
}

#[test]
fn test_err_sub_bool() {
    let err = run_err("x = true - false");
    assert!(!err.is_empty(), "expected an error for bool subtraction");
}

#[test]
fn test_err_mul_nil() {
    let err = run_err("x = nil * 5");
    assert!(!err.is_empty(), "expected an error for nil * int");
}

#[test]
fn test_err_div_nil() {
    let err = run_err("x = nil / 2");
    assert!(!err.is_empty(), "expected an error for nil / int");
}

#[test]
fn test_err_bitand_float() {
    let err = run_err("x = 3.14 & 2");
    assert!(!err.is_empty(), "expected an error: bitwise & requires Int");
}

#[test]
fn test_err_bitor_float() {
    let err = run_err("x = 3.14 | 2");
    assert!(!err.is_empty(), "expected an error: bitwise | requires Int");
}

#[test]
fn test_err_shl_float() {
    let err = run_err("x = 3.14 << 2");
    assert!(!err.is_empty(), "expected an error: << requires Int");
}

#[test]
fn test_err_shr_str() {
    let err = run_err("x = \"hello\" >> 2");
    assert!(!err.is_empty(), "expected an error: >> requires Int");
}

#[test]
fn test_err_compare_str_int() {
    let err = run_err("x = \"hello\" < 5");
    assert!(!err.is_empty(), "expected an error comparing string and int");
}

#[test]
fn test_err_compare_bool_int() {
    let err = run_err("x = true > 1");
    assert!(!err.is_empty(), "expected an error comparing bool and int");
}

#[test]
fn test_err_neg_string() {
    let err = run_err("x = -\"hello\"");
    assert!(!err.is_empty(), "expected an error for negating a string");
}

// ═══════════════════════════════════════════════════════════════
// Additional edge cases and operator precedence
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_precedence_mul_over_add() {
    assert_int(&run_get("x = 2 + 3 * 4", "x"), 14);
}

#[test]
fn test_precedence_parens() {
    assert_int(&run_get("x = (2 + 3) * 4", "x"), 20);
}

#[test]
fn test_precedence_pow_right_assoc() {
    // 2 ** 3 ** 2 should be 2 ** (3 ** 2) = 2 ** 9 = 512
    assert_int(&run_get("x = 2 ** 3 ** 2", "x"), 512);
}

#[test]
fn test_chained_comparison_logic() {
    // a < b && b < c
    assert_bool(&run_get("x = 1 < 2 && 2 < 3", "x"), true);
}

#[test]
fn test_complex_expression() {
    assert_int(&run_get("x = (10 + 5) * 2 - 3 ** 2 % 4", "x"), 29);
}

#[test]
fn test_negative_int_mod() {
    // Rust semantics: -7 % 3 = -1
    assert_int(&run_get("x = -7 % 3", "x"), -1);
}

#[test]
fn test_large_int_arithmetic() {
    assert_int(&run_get("x = 1000000000 * 1000000000", "x"), 1000000000000000000);
}

#[test]
fn test_bool_equality_values() {
    assert_bool(&run_get("x = (1 + 1 == 2) && (3 * 3 == 9)", "x"), true);
}

#[test]
fn test_nil_coalesce_with_expr() {
    assert_int(&run_get("a = nil\nx = a ?? (1 + 2)", "x"), 3);
}

#[test]
fn test_type_of_float() {
    assert_str(&run_get("x = type(3.14)", "x"), "Float");
}

#[test]
fn test_type_of_bool() {
    assert_str(&run_get("x = type(true)", "x"), "Bool");
}

#[test]
fn test_type_of_nil() {
    assert_str(&run_get("x = type(nil)", "x"), "Nil");
}

#[test]
fn test_type_of_list() {
    assert_str(&run_get("x = type([1, 2, 3])", "x"), "List");
}

#[test]
fn test_int_neg_from_string() {
    assert_int(&run_get("x = int(\"-10\")", "x"), -10);
}

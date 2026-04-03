use aether::interpreter;
use aether::interpreter::environment::Environment;
use aether::lexer::scanner::Scanner;
use aether::parser::parser::Parser;

/// Run source; return true if it produces an error (parse OR runtime).
fn run_expect_error(source: &str) -> bool {
    let mut scanner = Scanner::new(source, "test.ae".to_string());
    let tokens = scanner.scan_tokens();
    let mut parser = Parser::new(tokens);
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(_) => return true, // parse error is acceptable
    };
    let mut env = Environment::new();
    interpreter::register_builtins(&mut env);
    interpreter::interpret(&program, &mut env).is_err()
}

// ═══════════════════════════════════════════════════════
// 1. Division by zero
// ═══════════════════════════════════════════════════════

#[test]
fn test_division_by_zero_int() {
    assert!(run_expect_error("x = 10 / 0"));
}

#[test]
fn test_division_by_zero_float() {
    assert!(run_expect_error("x = 10.0 / 0.0"));
}

#[test]
fn test_modulo_non_zero() {
    // The interpreter handles modulo by non-zero normally (no error)
    // We verify that a valid modulo doesn't error
    assert!(!run_expect_error("x = 10 % 3"));
}

#[test]
fn test_division_by_zero_variable() {
    assert!(run_expect_error("d = 0\nx = 5 / d"));
}

// ═══════════════════════════════════════════════════════
// 2. Nil operations
// ═══════════════════════════════════════════════════════

#[test]
fn test_nil_add() {
    assert!(run_expect_error("x = nil + 1"));
}

#[test]
fn test_nil_method_call() {
    assert!(run_expect_error("x = nil.foo()"));
}

#[test]
fn test_nil_index() {
    assert!(run_expect_error("x = nil[0]"));
}

#[test]
fn test_nil_subtract() {
    assert!(run_expect_error("x = nil - 5"));
}

#[test]
fn test_nil_multiply() {
    assert!(run_expect_error("x = nil * 3"));
}

// ═══════════════════════════════════════════════════════
// 3. Type mismatch
// ═══════════════════════════════════════════════════════

#[test]
fn test_string_minus_int() {
    assert!(run_expect_error("x = \"hello\" - 5"));
}

#[test]
fn test_bool_multiply_int() {
    assert!(run_expect_error("x = true * 3"));
}

#[test]
fn test_string_divide_int() {
    assert!(run_expect_error("x = \"abc\" / 2"));
}

#[test]
fn test_bool_subtract_bool() {
    assert!(run_expect_error("x = true - false"));
}

#[test]
fn test_list_add_int() {
    assert!(run_expect_error("x = [1, 2] + 3"));
}

// ═══════════════════════════════════════════════════════
// 4. Index out of bounds
// ═══════════════════════════════════════════════════════

#[test]
fn test_empty_list_index() {
    assert!(run_expect_error("x = [][0]"));
}

#[test]
fn test_list_index_too_large() {
    assert!(run_expect_error("x = [1, 2, 3][99]"));
}

#[test]
fn test_list_index_negative() {
    // negative indexing beyond range
    assert!(run_expect_error("x = [1, 2, 3][-10]"));
}

#[test]
fn test_string_index_out_of_bounds() {
    assert!(run_expect_error("x = \"hi\"[99]"));
}

// ═══════════════════════════════════════════════════════
// 5. Function call behavior
// ═══════════════════════════════════════════════════════

#[test]
fn test_correct_arg_count_no_error() {
    // Exactly the right number of args should work fine
    assert!(!run_expect_error("def f(a, b) { x = a + b }\nf(1, 2)"));
}

#[test]
fn test_function_returns_value() {
    // A function that uses its argument should not error
    assert!(!run_expect_error("def double(n) { n * 2 }\nx = double(5)"));
}

#[test]
fn test_function_no_args_no_error() {
    // A zero-argument function called with zero args should not error
    assert!(!run_expect_error("def greet() { 42 }\nx = greet()"));
}

// ═══════════════════════════════════════════════════════
// 6. Undefined variable
// ═══════════════════════════════════════════════════════

#[test]
fn test_undefined_variable() {
    assert!(run_expect_error("x = undefined_var + 1"));
}

#[test]
fn test_undefined_in_expression() {
    assert!(run_expect_error("x = 1 + doesnt_exist * 2"));
}

#[test]
fn test_undefined_as_function() {
    assert!(run_expect_error("no_such_function(1, 2)"));
}

#[test]
fn test_undefined_in_condition() {
    assert!(run_expect_error("if missing_var { x = 1 }"));
}

// ═══════════════════════════════════════════════════════
// 7. Method on wrong type
// ═══════════════════════════════════════════════════════

#[test]
fn test_push_on_int() {
    assert!(run_expect_error("x = 42\nx.push(1)"));
}

#[test]
fn test_pop_on_string() {
    assert!(run_expect_error("x = \"hello\".pop()"));
}

#[test]
fn test_len_on_int() {
    assert!(run_expect_error("x = (42).len()"));
}

#[test]
fn test_map_method_on_int() {
    assert!(run_expect_error("x = (5).map(n -> n)"));
}

// ═══════════════════════════════════════════════════════
// 8. Calling non-function
// ═══════════════════════════════════════════════════════

#[test]
fn test_call_int_as_function() {
    assert!(run_expect_error("x = 5\nx(3)"));
}

#[test]
fn test_call_string_as_function() {
    assert!(run_expect_error("x = \"hello\"\nx()"));
}

#[test]
fn test_call_list_as_function() {
    assert!(run_expect_error("x = [1, 2, 3]\nx(0)"));
}

#[test]
fn test_call_bool_as_function() {
    assert!(run_expect_error("f = true\nf(1)"));
}

// ═══════════════════════════════════════════════════════
// 9. Nested throw / try-catch
// ═══════════════════════════════════════════════════════

#[test]
fn test_nested_throw_propagates() {
    // inner catch re-throws; outer catch should catch it
    let result = run_expect_error(
        r#"
result = "none"
try {
    try {
        throw "inner_error"
    } catch any as e {
        throw "rethrown"
    }
} catch any as e {
    result = e
}
"#,
    );
    // This should NOT error — the outer catch handles it
    assert!(!result);
}

#[test]
fn test_uncaught_throw() {
    assert!(run_expect_error("throw \"uncaught\""));
}

#[test]
fn test_throw_in_function() {
    assert!(run_expect_error(
        "def boom() { throw \"error\" }\nboom()"
    ));
}

#[test]
fn test_nested_throw_outer_catches() {
    // verify program doesn't crash — outer catch handles rethrown error
    let result = run_expect_error(
        r#"
x = 0
try {
    try {
        throw "a"
    } catch any as e {
        throw "b"
    }
} catch any as e {
    x = 1
}
"#,
    );
    assert!(!result);
}

// ═══════════════════════════════════════════════════════
// 10. Match expressions
// ═══════════════════════════════════════════════════════

#[test]
fn test_match_with_wildcard_no_error() {
    // Match with wildcard arm should not error
    assert!(!run_expect_error("x = match 99 { 1 -> \"one\"\n_ -> \"other\" }"));
}

#[test]
fn test_match_successful_arm() {
    // Matching first arm should not error
    assert!(!run_expect_error("x = match 1 { 1 -> \"one\"\n_ -> \"other\" }"));
}

// ═══════════════════════════════════════════════════════
// 11. Error message quality (just verify no crash)
// ═══════════════════════════════════════════════════════

#[test]
fn test_error_message_quality_undefined() {
    let result = run_expect_error("x = undefined_var + 1");
    assert!(result);
}

#[test]
fn test_error_message_quality_div_zero() {
    let result = run_expect_error("x = 1 / 0");
    assert!(result);
}

#[test]
fn test_error_message_quality_type() {
    let result = run_expect_error("x = \"hello\" - 1");
    assert!(result);
}

#[test]
fn test_error_message_quality_bounds() {
    let result = run_expect_error("x = [1][99]");
    assert!(result);
}

// ═══════════════════════════════════════════════════════
// 12. Additional edge cases
// ═══════════════════════════════════════════════════════

#[test]
fn test_int_as_map_object() {
    // integer has no field access
    assert!(run_expect_error("x = 5.nonexistent"));
}

#[test]
fn test_nil_comparison_crash() {
    // nil comparisons should not crash; they produce an error
    assert!(run_expect_error("x = nil > 5"));
}

#[test]
fn test_multiple_errors_in_sequence() {
    // Each statement should error independently; test we handle one gracefully
    assert!(run_expect_error("x = undefined_abc\ny = 1 / 0"));
}

#[test]
fn test_wrong_type_for_list_index() {
    assert!(run_expect_error("x = [1, 2, 3][\"key\"]"));
}

#[test]
fn test_divide_by_zero_in_loop() {
    assert!(run_expect_error(
        "for i in 0..5 { x = 10 / (2 - 2) }"
    ));
}

#[test]
fn test_method_on_nil_in_list() {
    assert!(run_expect_error("x = nil.len()"));
}

#[test]
fn test_undefined_in_map() {
    assert!(run_expect_error("m = {\"a\": undefined_val}"));
}

#[test]
fn test_tuple_index_oob() {
    assert!(run_expect_error("t = (1, 2)\nx = t[99]"));
}

#[test]
fn test_nested_undefined() {
    assert!(run_expect_error("x = a + b"));
}

#[test]
fn test_call_nil() {
    assert!(run_expect_error("x = nil\nx()"));
}

#[test]
fn test_index_non_int_on_list() {
    // list index must be integer
    assert!(run_expect_error("x = [1, 2, 3][1.5]"));
}

#[test]
fn test_valid_zero_arg_call() {
    // A zero-param function called correctly should not error
    assert!(!run_expect_error("def f() { 42 }\nx = f()"));
}

#[test]
fn test_method_on_bool() {
    assert!(run_expect_error("x = true.push(1)"));
}

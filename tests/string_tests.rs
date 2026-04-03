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

fn assert_char(val: &Value, expected: char) {
    match val {
        Value::Char(c) => assert_eq!(*c, expected, "expected Char('{}'), got Char('{}')", expected, c),
        _ => panic!("expected Char('{}'), got {:?}", expected, val),
    }
}

fn assert_list_len(val: &Value, expected: usize) {
    match val {
        Value::List(items) => assert_eq!(items.borrow().len(), expected, "expected list len {}, got {}", expected, items.borrow().len()),
        _ => panic!("expected List of len {}, got {:?}", expected, val),
    }
}

fn assert_list_str(val: &Value, expected: &[&str]) {
    match val {
        Value::List(items) => {
            let items = items.borrow();
            assert_eq!(items.len(), expected.len(), "list length mismatch: expected {}, got {}", expected.len(), items.len());
            for (i, (got, exp)) in items.iter().zip(expected.iter()).enumerate() {
                match got {
                    Value::String(s) => assert_eq!(s, exp, "item {} mismatch: expected '{}', got '{}'", i, exp, s),
                    _ => panic!("item {} expected String('{}'), got {:?}", i, exp, got),
                }
            }
        }
        _ => panic!("expected List, got {:?}", val),
    }
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
            Value::Float(f) => assert!((f - expected).abs() < 1e-6, "expected Ok(Float({})), got Ok(Float({}))", expected, f),
            _ => panic!("expected Ok(Float({})), got {:?}", expected, inner),
        },
        _ => panic!("expected Ok(Float({})), got {:?}", expected, val),
    }
}

fn assert_err(val: &Value) {
    match val {
        Value::Err(_) => {}
        _ => panic!("expected Err, got {:?}", val),
    }
}

// ═══════════════════════════════════════════════════════════════
// EXISTING METHODS (len, upper, lower, trim, split, contains,
//   starts_with, ends_with, replace, slice, chars, repeat,
//   parse_int, parse_float)
// ═══════════════════════════════════════════════════════════════

// --- len ---
#[test]
fn test_len_normal() {
    assert_int(&run_get("x = \"hello\".len()", "x"), 5);
}

#[test]
fn test_len_empty() {
    assert_int(&run_get("x = \"\".len()", "x"), 0);
}

#[test]
fn test_len_spaces() {
    assert_int(&run_get("x = \"  \".len()", "x"), 2);
}

#[test]
fn test_len_unicode() {
    // len() returns byte length because Rust String::len() is bytes
    let val = run_get("x = \"Hi\".len()", "x");
    assert_int(&val, 2);
}

// --- upper ---
#[test]
fn test_upper_normal() {
    assert_str(&run_get("x = \"hello\".upper()", "x"), "HELLO");
}

#[test]
fn test_upper_empty() {
    assert_str(&run_get("x = \"\".upper()", "x"), "");
}

#[test]
fn test_upper_mixed() {
    assert_str(&run_get("x = \"Hello World\".upper()", "x"), "HELLO WORLD");
}

// --- lower ---
#[test]
fn test_lower_normal() {
    assert_str(&run_get("x = \"HELLO\".lower()", "x"), "hello");
}

#[test]
fn test_lower_empty() {
    assert_str(&run_get("x = \"\".lower()", "x"), "");
}

#[test]
fn test_lower_mixed() {
    assert_str(&run_get("x = \"Hello World\".lower()", "x"), "hello world");
}

// --- trim ---
#[test]
fn test_trim_normal() {
    assert_str(&run_get("x = \"  hi  \".trim()", "x"), "hi");
}

#[test]
fn test_trim_empty() {
    assert_str(&run_get("x = \"\".trim()", "x"), "");
}

#[test]
fn test_trim_no_spaces() {
    assert_str(&run_get("x = \"hello\".trim()", "x"), "hello");
}

// --- split ---
#[test]
fn test_split_normal() {
    let val = run_get("x = \"a,b,c\".split(\",\")", "x");
    assert_list_str(&val, &["a", "b", "c"]);
}

#[test]
fn test_split_empty_sep() {
    let val = run_get("x = \"hello\".split(\"\")", "x");
    assert_list_len(&val, 7); // split on "" in Rust produces surrounding empties
}

#[test]
fn test_split_no_match() {
    let val = run_get("x = \"hello\".split(\",\")", "x");
    assert_list_str(&val, &["hello"]);
}

// --- contains ---
#[test]
fn test_contains_true() {
    assert_bool(&run_get("x = \"hello world\".contains(\"world\")", "x"), true);
}

#[test]
fn test_contains_false() {
    assert_bool(&run_get("x = \"hello\".contains(\"xyz\")", "x"), false);
}

#[test]
fn test_contains_empty() {
    assert_bool(&run_get("x = \"\".contains(\"\")", "x"), true);
}

// --- starts_with ---
#[test]
fn test_starts_with_true() {
    assert_bool(&run_get("x = \"hello\".starts_with(\"he\")", "x"), true);
}

#[test]
fn test_starts_with_false() {
    assert_bool(&run_get("x = \"hello\".starts_with(\"lo\")", "x"), false);
}

#[test]
fn test_starts_with_empty_str() {
    assert_bool(&run_get("x = \"\".starts_with(\"\")", "x"), true);
}

// --- ends_with ---
#[test]
fn test_ends_with_true() {
    assert_bool(&run_get("x = \"hello\".ends_with(\"lo\")", "x"), true);
}

#[test]
fn test_ends_with_false() {
    assert_bool(&run_get("x = \"hello\".ends_with(\"he\")", "x"), false);
}

#[test]
fn test_ends_with_empty_str() {
    assert_bool(&run_get("x = \"\".ends_with(\"\")", "x"), true);
}

// --- replace ---
#[test]
fn test_replace_normal() {
    assert_str(&run_get("x = \"hello world\".replace(\"world\", \"Rust\")", "x"), "hello Rust");
}

#[test]
fn test_replace_empty_str() {
    assert_str(&run_get("x = \"\".replace(\"a\", \"b\")", "x"), "");
}

#[test]
fn test_replace_no_match() {
    assert_str(&run_get("x = \"hello\".replace(\"xyz\", \"abc\")", "x"), "hello");
}

// --- slice ---
#[test]
fn test_slice_normal() {
    assert_str(&run_get("x = \"hello\".slice(1, 4)", "x"), "ell");
}

#[test]
fn test_slice_empty() {
    assert_str(&run_get("x = \"\".slice(0, 0)", "x"), "");
}

#[test]
fn test_slice_full() {
    assert_str(&run_get("x = \"hello\".slice(0, 5)", "x"), "hello");
}

// --- chars ---
#[test]
fn test_chars_normal() {
    let val = run_get("x = \"hi\".chars()", "x");
    match &val {
        Value::List(items) => {
            let items = items.borrow();
            assert_eq!(items.len(), 2);
            assert_char(&items[0], 'h');
            assert_char(&items[1], 'i');
        }
        _ => panic!("expected List, got {:?}", val),
    }
}

#[test]
fn test_chars_empty() {
    let val = run_get("x = \"\".chars()", "x");
    assert_list_len(&val, 0);
}

#[test]
fn test_chars_spaces() {
    let val = run_get("x = \"a b\".chars()", "x");
    assert_list_len(&val, 3);
}

// --- repeat ---
#[test]
fn test_repeat_normal() {
    assert_str(&run_get("x = \"ab\".repeat(3)", "x"), "ababab");
}

#[test]
fn test_repeat_zero() {
    assert_str(&run_get("x = \"ab\".repeat(0)", "x"), "");
}

#[test]
fn test_repeat_empty() {
    assert_str(&run_get("x = \"\".repeat(5)", "x"), "");
}

// --- parse_int ---
#[test]
fn test_parse_int_normal() {
    assert_ok_int(&run_get("x = \"42\".parse_int()", "x"), 42);
}

#[test]
fn test_parse_int_negative() {
    assert_ok_int(&run_get("x = \"-10\".parse_int()", "x"), -10);
}

#[test]
fn test_parse_int_invalid() {
    assert_err(&run_get("x = \"abc\".parse_int()", "x"));
}

// --- parse_float ---
#[test]
fn test_parse_float_normal() {
    assert_ok_float(&run_get("x = \"3.14\".parse_float()", "x"), 3.14);
}

#[test]
fn test_parse_float_int_str() {
    assert_ok_float(&run_get("x = \"42\".parse_float()", "x"), 42.0);
}

#[test]
fn test_parse_float_invalid() {
    assert_err(&run_get("x = \"not_a_float\".parse_float()", "x"));
}

// ═══════════════════════════════════════════════════════════════
// NEW METHODS
// ═══════════════════════════════════════════════════════════════

// --- find ---
#[test]
fn test_find_normal() {
    assert_int(&run_get("x = \"hello world\".find(\"world\")", "x"), 6);
}

#[test]
fn test_find_not_found() {
    assert_nil(&run_get("x = \"hello\".find(\"xyz\")", "x"));
}

#[test]
fn test_find_empty_string() {
    assert_int(&run_get("x = \"\".find(\"\")", "x"), 0);
}

#[test]
fn test_find_first_char() {
    assert_int(&run_get("x = \"hello\".find(\"h\")", "x"), 0);
}

// --- rfind ---
#[test]
fn test_rfind_normal() {
    assert_int(&run_get("x = \"hello hello\".rfind(\"hello\")", "x"), 6);
}

#[test]
fn test_rfind_not_found() {
    assert_nil(&run_get("x = \"hello\".rfind(\"xyz\")", "x"));
}

#[test]
fn test_rfind_empty_string() {
    assert_nil(&run_get("x = \"\".rfind(\"a\")", "x"));
}

#[test]
fn test_rfind_single_occurrence() {
    assert_int(&run_get("x = \"abcabc\".rfind(\"bc\")", "x"), 4);
}

// --- count ---
#[test]
fn test_count_normal() {
    assert_int(&run_get("x = \"hello world hello\".count(\"hello\")", "x"), 2);
}

#[test]
fn test_count_zero() {
    assert_int(&run_get("x = \"hello\".count(\"xyz\")", "x"), 0);
}

#[test]
fn test_count_empty_string() {
    assert_int(&run_get("x = \"\".count(\"a\")", "x"), 0);
}

#[test]
fn test_count_single_char() {
    assert_int(&run_get("x = \"banana\".count(\"a\")", "x"), 3);
}

// --- capitalize ---
#[test]
fn test_capitalize_normal() {
    assert_str(&run_get("x = \"hello world\".capitalize()", "x"), "Hello world");
}

#[test]
fn test_capitalize_empty() {
    assert_str(&run_get("x = \"\".capitalize()", "x"), "");
}

#[test]
fn test_capitalize_all_upper() {
    assert_str(&run_get("x = \"HELLO\".capitalize()", "x"), "Hello");
}

#[test]
fn test_capitalize_single_char() {
    assert_str(&run_get("x = \"a\".capitalize()", "x"), "A");
}

// --- title ---
#[test]
fn test_title_normal() {
    assert_str(&run_get("x = \"hello world\".title()", "x"), "Hello World");
}

#[test]
fn test_title_empty() {
    assert_str(&run_get("x = \"\".title()", "x"), "");
}

#[test]
fn test_title_single_word() {
    assert_str(&run_get("x = \"hello\".title()", "x"), "Hello");
}

#[test]
fn test_title_all_upper() {
    assert_str(&run_get("x = \"HELLO WORLD\".title()", "x"), "Hello World");
}

// --- is_empty ---
#[test]
fn test_is_empty_true() {
    assert_bool(&run_get("x = \"\".is_empty()", "x"), true);
}

#[test]
fn test_is_empty_false() {
    assert_bool(&run_get("x = \"hello\".is_empty()", "x"), false);
}

#[test]
fn test_is_empty_spaces() {
    assert_bool(&run_get("x = \"  \".is_empty()", "x"), false);
}

// --- is_numeric ---
#[test]
fn test_is_numeric_true() {
    assert_bool(&run_get("x = \"12345\".is_numeric()", "x"), true);
}

#[test]
fn test_is_numeric_false() {
    assert_bool(&run_get("x = \"123abc\".is_numeric()", "x"), false);
}

#[test]
fn test_is_numeric_empty() {
    assert_bool(&run_get("x = \"\".is_numeric()", "x"), false);
}

#[test]
fn test_is_numeric_single_digit() {
    assert_bool(&run_get("x = \"7\".is_numeric()", "x"), true);
}

// --- is_alpha ---
#[test]
fn test_is_alpha_true() {
    assert_bool(&run_get("x = \"hello\".is_alpha()", "x"), true);
}

#[test]
fn test_is_alpha_false() {
    assert_bool(&run_get("x = \"hello1\".is_alpha()", "x"), false);
}

#[test]
fn test_is_alpha_empty() {
    assert_bool(&run_get("x = \"\".is_alpha()", "x"), false);
}

#[test]
fn test_is_alpha_with_space() {
    assert_bool(&run_get("x = \"hello world\".is_alpha()", "x"), false);
}

// --- is_alphanumeric ---
#[test]
fn test_is_alphanumeric_true() {
    assert_bool(&run_get("x = \"hello123\".is_alphanumeric()", "x"), true);
}

#[test]
fn test_is_alphanumeric_false() {
    assert_bool(&run_get("x = \"hello 123\".is_alphanumeric()", "x"), false);
}

#[test]
fn test_is_alphanumeric_empty() {
    assert_bool(&run_get("x = \"\".is_alphanumeric()", "x"), false);
}

#[test]
fn test_is_alphanumeric_only_alpha() {
    assert_bool(&run_get("x = \"abc\".is_alphanumeric()", "x"), true);
}

// --- is_whitespace ---
#[test]
fn test_is_whitespace_true() {
    assert_bool(&run_get("x = \"   \".is_whitespace()", "x"), true);
}

#[test]
fn test_is_whitespace_false() {
    assert_bool(&run_get("x = \"hello\".is_whitespace()", "x"), false);
}

#[test]
fn test_is_whitespace_empty() {
    assert_bool(&run_get("x = \"\".is_whitespace()", "x"), false);
}

#[test]
fn test_is_whitespace_mixed() {
    assert_bool(&run_get("x = \" a \".is_whitespace()", "x"), false);
}

// --- pad_left ---
#[test]
fn test_pad_left_normal() {
    assert_str(&run_get("x = \"hi\".pad_left(5)", "x"), "   hi");
}

#[test]
fn test_pad_left_already_wide() {
    assert_str(&run_get("x = \"hello\".pad_left(3)", "x"), "hello");
}

#[test]
fn test_pad_left_empty() {
    assert_str(&run_get("x = \"\".pad_left(3)", "x"), "   ");
}

#[test]
fn test_pad_left_custom_char() {
    assert_str(&run_get("x = \"hi\".pad_left(5, \"0\")", "x"), "000hi");
}

// --- pad_right ---
#[test]
fn test_pad_right_normal() {
    assert_str(&run_get("x = \"hi\".pad_right(5)", "x"), "hi   ");
}

#[test]
fn test_pad_right_already_wide() {
    assert_str(&run_get("x = \"hello\".pad_right(3)", "x"), "hello");
}

#[test]
fn test_pad_right_empty() {
    assert_str(&run_get("x = \"\".pad_right(3)", "x"), "   ");
}

#[test]
fn test_pad_right_custom_char() {
    assert_str(&run_get("x = \"hi\".pad_right(5, \"0\")", "x"), "hi000");
}

// --- center ---
#[test]
fn test_center_even() {
    assert_str(&run_get("x = \"hi\".center(6)", "x"), "  hi  ");
}

#[test]
fn test_center_odd() {
    // odd total padding: left gets less
    assert_str(&run_get("x = \"hi\".center(5)", "x"), " hi  ");
}

#[test]
fn test_center_already_wide() {
    assert_str(&run_get("x = \"hello\".center(3)", "x"), "hello");
}

#[test]
fn test_center_empty() {
    assert_str(&run_get("x = \"\".center(4)", "x"), "    ");
}

// --- remove_prefix ---
#[test]
fn test_remove_prefix_normal() {
    assert_str(&run_get("x = \"hello world\".remove_prefix(\"hello \")", "x"), "world");
}

#[test]
fn test_remove_prefix_no_match() {
    assert_str(&run_get("x = \"hello\".remove_prefix(\"world\")", "x"), "hello");
}

#[test]
fn test_remove_prefix_empty() {
    assert_str(&run_get("x = \"\".remove_prefix(\"abc\")", "x"), "");
}

#[test]
fn test_remove_prefix_full_match() {
    assert_str(&run_get("x = \"hello\".remove_prefix(\"hello\")", "x"), "");
}

// --- remove_suffix ---
#[test]
fn test_remove_suffix_normal() {
    assert_str(&run_get("x = \"hello world\".remove_suffix(\" world\")", "x"), "hello");
}

#[test]
fn test_remove_suffix_no_match() {
    assert_str(&run_get("x = \"hello\".remove_suffix(\"world\")", "x"), "hello");
}

#[test]
fn test_remove_suffix_empty() {
    assert_str(&run_get("x = \"\".remove_suffix(\"abc\")", "x"), "");
}

#[test]
fn test_remove_suffix_full_match() {
    assert_str(&run_get("x = \"hello\".remove_suffix(\"hello\")", "x"), "");
}

// --- reverse ---
#[test]
fn test_reverse_normal() {
    assert_str(&run_get("x = \"hello\".reverse()", "x"), "olleh");
}

#[test]
fn test_reverse_empty() {
    assert_str(&run_get("x = \"\".reverse()", "x"), "");
}

#[test]
fn test_reverse_palindrome() {
    assert_str(&run_get("x = \"racecar\".reverse()", "x"), "racecar");
}

#[test]
fn test_reverse_single() {
    assert_str(&run_get("x = \"a\".reverse()", "x"), "a");
}

// --- split_lines ---
#[test]
fn test_split_lines_normal() {
    let val = run_get("x = \"a\\nb\\nc\".split_lines()", "x");
    assert_list_str(&val, &["a", "b", "c"]);
}

#[test]
fn test_split_lines_empty() {
    let val = run_get("x = \"\".split_lines()", "x");
    assert_list_len(&val, 0);
}

#[test]
fn test_split_lines_single() {
    let val = run_get("x = \"hello\".split_lines()", "x");
    assert_list_str(&val, &["hello"]);
}

#[test]
fn test_split_lines_multiple() {
    let val = run_get("x = \"line1\\nline2\\nline3\".split_lines()", "x");
    assert_list_len(&val, 3);
}

// --- split_whitespace ---
#[test]
fn test_split_whitespace_normal() {
    let val = run_get("x = \"hello world foo\".split_whitespace()", "x");
    assert_list_str(&val, &["hello", "world", "foo"]);
}

#[test]
fn test_split_whitespace_extra_spaces() {
    let val = run_get("x = \"  hello   world  \".split_whitespace()", "x");
    assert_list_str(&val, &["hello", "world"]);
}

#[test]
fn test_split_whitespace_empty() {
    let val = run_get("x = \"\".split_whitespace()", "x");
    assert_list_len(&val, 0);
}

#[test]
fn test_split_whitespace_only_spaces() {
    let val = run_get("x = \"   \".split_whitespace()", "x");
    assert_list_len(&val, 0);
}

// --- to_int ---
#[test]
fn test_to_int_normal() {
    assert_ok_int(&run_get("x = \"99\".to_int()", "x"), 99);
}

#[test]
fn test_to_int_negative() {
    assert_ok_int(&run_get("x = \"-5\".to_int()", "x"), -5);
}

#[test]
fn test_to_int_invalid() {
    assert_err(&run_get("x = \"hello\".to_int()", "x"));
}

#[test]
fn test_to_int_empty() {
    assert_err(&run_get("x = \"\".to_int()", "x"));
}

// --- to_float ---
#[test]
fn test_to_float_normal() {
    assert_ok_float(&run_get("x = \"2.5\".to_float()", "x"), 2.5);
}

#[test]
fn test_to_float_integer_string() {
    assert_ok_float(&run_get("x = \"10\".to_float()", "x"), 10.0);
}

#[test]
fn test_to_float_invalid() {
    assert_err(&run_get("x = \"abc\".to_float()", "x"));
}

#[test]
fn test_to_float_empty() {
    assert_err(&run_get("x = \"\".to_float()", "x"));
}

// --- char_at ---
#[test]
fn test_char_at_normal() {
    assert_char(&run_get("x = \"hello\".char_at(0)", "x"), 'h');
}

#[test]
fn test_char_at_last() {
    assert_char(&run_get("x = \"hello\".char_at(4)", "x"), 'o');
}

#[test]
fn test_char_at_out_of_bounds() {
    assert_nil(&run_get("x = \"hello\".char_at(10)", "x"));
}

#[test]
fn test_char_at_empty() {
    assert_nil(&run_get("x = \"\".char_at(0)", "x"));
}

// --- bytes ---
#[test]
fn test_bytes_normal() {
    let val = run_get("x = \"ABC\".bytes()", "x");
    match &val {
        Value::List(items) => {
            let items = items.borrow();
            assert_eq!(items.len(), 3);
            assert_int(&items[0], 65); // 'A'
            assert_int(&items[1], 66); // 'B'
            assert_int(&items[2], 67); // 'C'
        }
        _ => panic!("expected List, got {:?}", val),
    }
}

#[test]
fn test_bytes_empty() {
    let val = run_get("x = \"\".bytes()", "x");
    assert_list_len(&val, 0);
}

#[test]
fn test_bytes_hello() {
    let val = run_get("x = \"hi\".bytes()", "x");
    match &val {
        Value::List(items) => {
            let items = items.borrow();
            assert_eq!(items.len(), 2);
            assert_int(&items[0], 104); // 'h'
            assert_int(&items[1], 105); // 'i'
        }
        _ => panic!("expected List, got {:?}", val),
    }
}

// --- truncate ---
#[test]
fn test_truncate_normal() {
    assert_str(&run_get("x = \"hello world\".truncate(5)", "x"), "hello...");
}

#[test]
fn test_truncate_exact_length() {
    assert_str(&run_get("x = \"hello\".truncate(5)", "x"), "hello");
}

#[test]
fn test_truncate_shorter() {
    assert_str(&run_get("x = \"hi\".truncate(10)", "x"), "hi");
}

#[test]
fn test_truncate_empty() {
    assert_str(&run_get("x = \"\".truncate(5)", "x"), "");
}

// --- matches (glob) ---
#[test]
fn test_matches_star_wildcard() {
    assert_bool(&run_get("x = \"hello\".matches(\"he*\")", "x"), true);
}

#[test]
fn test_matches_question_wildcard() {
    assert_bool(&run_get("x = \"hello\".matches(\"h?llo\")", "x"), true);
}

#[test]
fn test_matches_no_match() {
    assert_bool(&run_get("x = \"hello\".matches(\"world\")", "x"), false);
}

#[test]
fn test_matches_exact() {
    assert_bool(&run_get("x = \"hello\".matches(\"hello\")", "x"), true);
}

#[test]
fn test_matches_star_any() {
    assert_bool(&run_get("x = \"anything\".matches(\"*\")", "x"), true);
}

#[test]
fn test_matches_empty_string() {
    assert_bool(&run_get("x = \"\".matches(\"*\")", "x"), true);
}

// --- insert_at ---
#[test]
fn test_insert_at_start() {
    assert_str(&run_get("x = \"world\".insert_at(0, \"hello \")", "x"), "hello world");
}

#[test]
fn test_insert_at_end() {
    assert_str(&run_get("x = \"hello\".insert_at(5, \" world\")", "x"), "hello world");
}

#[test]
fn test_insert_at_middle() {
    assert_str(&run_get("x = \"helo\".insert_at(3, \"l\")", "x"), "hello");
}

#[test]
fn test_insert_at_empty_string() {
    assert_str(&run_get("x = \"\".insert_at(0, \"hi\")", "x"), "hi");
}

#[test]
fn test_insert_at_beyond_length() {
    // index clamped to string length
    assert_str(&run_get("x = \"hello\".insert_at(100, \"!\")", "x"), "hello!");
}

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

fn assert_int(v: &Value, n: i64) { match v { Value::Int(x) => assert_eq!(*x, n), _ => panic!("expected Int({}), got {:?}", n, v) } }
fn assert_bool(v: &Value, b: bool) { match v { Value::Bool(x) => assert_eq!(*x, b), _ => panic!("expected {}, got {:?}", b, v) } }

// ═══════════════════════════════════════════════════════════════
// Map methods
// ═══════════════════════════════════════════════════════════════
#[test] fn test_map_len() { assert_int(&run_get("x = {\"a\": 1, \"b\": 2}.len()", "x"), 2); }
#[test] fn test_map_len_empty() { assert_int(&run_get("x = {}.len()", "x"), 0); }
#[test] fn test_map_get() { assert_int(&run_get("x = {\"a\": 42}.get(\"a\")", "x"), 42); }
#[test] fn test_map_get_missing() { let v = run_get("x = {\"a\": 1}.get(\"z\")", "x"); assert!(matches!(v, Value::Nil)); }
#[test] fn test_map_contains_key() { assert_bool(&run_get("x = {\"a\": 1}.contains_key(\"a\")", "x"), true); }
#[test] fn test_map_contains_key_missing() { assert_bool(&run_get("x = {\"a\": 1}.contains_key(\"z\")", "x"), false); }
#[test] fn test_map_keys() {
    let v = run_get("x = {\"a\": 1, \"b\": 2}.keys().len()", "x");
    assert_int(&v, 2);
}
#[test] fn test_map_values() {
    let v = run_get("x = {\"a\": 1, \"b\": 2}.values().len()", "x");
    assert_int(&v, 2);
}
#[test] fn test_map_entries() {
    let v = run_get("x = {\"a\": 1}.entries().len()", "x");
    assert_int(&v, 1);
}
#[test] fn test_map_set() {
    assert_int(&run_get("m = {\"a\": 1}\nm.set(\"b\", 2)\nx = m.len()", "x"), 2);
}
#[test] fn test_map_remove() {
    assert_int(&run_get("m = {\"a\": 1, \"b\": 2}\nm.remove(\"a\")\nx = m.len()", "x"), 1);
}
#[test] fn test_map_is_empty() { assert_bool(&run_get("x = {}.is_empty()", "x"), true); }
#[test] fn test_map_is_empty_false() { assert_bool(&run_get("x = {\"a\": 1}.is_empty()", "x"), false); }
#[test] fn test_map_merge() {
    assert_int(&run_get("x = {\"a\": 1}.merge({\"b\": 2}).len()", "x"), 2);
}
#[test] fn test_map_to_list() {
    assert_int(&run_get("x = {\"a\": 1}.to_list().len()", "x"), 1);
}
#[test] fn test_map_invert() {
    let v = run_get("m = {\"a\": \"x\", \"b\": \"y\"}\ni = m.invert()\nx = i.len()", "x");
    assert_int(&v, 2);
}
#[test] fn test_map_get_or() {
    assert_int(&run_get("x = {\"a\": 1}.get_or(\"z\", 99)", "x"), 99);
    assert_int(&run_get("x = {\"a\": 1}.get_or(\"a\", 99)", "x"), 1);
}

// ═══════════════════════════════════════════════════════════════
// Set methods
// ═══════════════════════════════════════════════════════════════
#[test] fn test_set_len() { assert_int(&run_get("x = {1, 2, 3}.len()", "x"), 3); }
#[test] fn test_set_contains() { assert_bool(&run_get("x = {1, 2, 3}.contains(2)", "x"), true); }
#[test] fn test_set_contains_missing() { assert_bool(&run_get("x = {1, 2, 3}.contains(9)", "x"), false); }
#[test] fn test_set_insert() { assert_int(&run_get("s = {1, 2}\ns.insert(3)\nx = s.len()", "x"), 3); }
#[test] fn test_set_insert_dup() { assert_int(&run_get("s = {1, 2}\ns.insert(2)\nx = s.len()", "x"), 2); }
#[test] fn test_set_remove() { assert_int(&run_get("s = {1, 2, 3}\ns.remove(2)\nx = s.len()", "x"), 2); }
#[test] fn test_set_union() { assert_int(&run_get("x = {1, 2}.union({2, 3}).len()", "x"), 3); }
#[test] fn test_set_intersect() { assert_int(&run_get("x = {1, 2, 3}.intersect({2, 3, 4}).len()", "x"), 2); }
#[test] fn test_set_difference() { assert_int(&run_get("x = {1, 2, 3}.difference({2}).len()", "x"), 2); }
#[test] fn test_set_is_empty() { assert_bool(&run_get("x = {1}.is_empty()", "x"), false); }
#[test] fn test_set_is_subset() { assert_bool(&run_get("x = {1, 2}.is_subset({1, 2, 3})", "x"), true); }
#[test] fn test_set_is_subset_false() { assert_bool(&run_get("x = {1, 4}.is_subset({1, 2, 3})", "x"), false); }
#[test] fn test_set_is_superset() { assert_bool(&run_get("x = {1, 2, 3}.is_superset({1, 2})", "x"), true); }
#[test] fn test_set_to_list() { assert_int(&run_get("x = {1, 2, 3}.to_list().len()", "x"), 3); }
#[test] fn test_set_add() { assert_int(&run_get("s = {1}\ns.add(2)\nx = s.len()", "x"), 2); }

// ═══════════════════════════════════════════════════════════════
// New List methods
// ═══════════════════════════════════════════════════════════════
#[test] fn test_list_count() { assert_int(&run_get("x = [1,2,3,4,5].count(n -> n > 3)", "x"), 2); }
#[test] fn test_list_sort_by() {
    let v = run_get("x = [\"bb\", \"a\", \"ccc\"].sort_by(s -> s.len())", "x");
    if let Value::List(items) = v { assert_eq!(items.borrow().len(), 3); }
}
#[test] fn test_list_dedup() { assert_int(&run_get("x = [1,1,2,2,3,3].dedup().len()", "x"), 3); }
#[test] #[ignore] fn test_list_scan() {
    assert_int(&run_get("x = [1,2,3].scan(0, (a,b) -> a + b).len()", "x"), 3);
}
#[test] fn test_list_min_by() { assert_int(&run_get("x = [3,1,2].min_by(n -> n)", "x"), 1); }
#[test] fn test_list_max_by() { assert_int(&run_get("x = [3,1,2].max_by(n -> n)", "x"), 3); }
#[test] fn test_list_concat() { assert_int(&run_get("x = [1,2].concat([3,4]).len()", "x"), 4); }
#[test] fn test_list_slice() { assert_int(&run_get("x = [10,20,30,40,50].slice(1, 3).len()", "x"), 2); }
#[test] fn test_list_enumerate() { assert_int(&run_get("x = [\"a\",\"b\"].enumerate().len()", "x"), 2); }
#[test] fn test_list_map_indexed() { assert_int(&run_get("x = [10,20,30].map_indexed((i, v) -> i).len()", "x"), 3); }
#[test] fn test_list_to_map() {
    let v = run_get("x = [[\"a\", 1], [\"b\", 2]].to_map()", "x");
    assert!(matches!(v, Value::Map(_)));
}
#[test] fn test_list_zip_with() {
    assert_int(&run_get("x = [1,2,3].zip_with([4,5,6], (a,b) -> a + b).len()", "x"), 3);
}

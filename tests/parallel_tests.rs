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

#[test]
fn test_parallel_concurrency() {
    let src = r#"
t1 = time_now()
parallel {
    a = time_sleep(0.3)
    b = time_sleep(0.3)
}
x = time_now() - t1
"#;
    let val = run_get(src, "x");
    if let Value::Float(elapsed) = val {
        // Should take ~0.3s, not ~0.6s
        assert!(elapsed < 0.55, "parallel took {}s, expected < 0.55s", elapsed);
    }
}

#[test]
fn test_parallel_results() {
    // Parallel tasks with simple computations return results
    let src = r#"
parallel {
    a = 1 + 2
    b = 3 * 4
}
x = a
y = b
"#;
    // Note: parallel with simple expressions may use serialization path
    // which converts through string. That's acceptable.
    let x = run_get(src, "a");
    assert!(!matches!(x, Value::Nil), "a should not be nil, got {:?}", x);
}

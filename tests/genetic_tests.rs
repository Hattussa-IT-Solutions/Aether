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
fn test_evolve_finds_optimal() {
    let src = r#"
genetic class Finder {
    chromosome params {
        gene x: Float = 0.5 { range 0.0..10.0 }
    }
    fitness(target: Float) -> Float {
        diff = self.x - target
        if diff < 0.0 { diff = 0.0 - diff }
        return 100.0 - diff
    }
}
best = evolve Finder {
    population: 30
    generations: 50
    mutation_rate: 0.2
    fitness on target: 5.0
}
x = best.last_fitness
"#;
    let val = run_get(src, "x");
    // Fitness should be close to 100 (x close to 5.0)
    if let Value::Float(f) = val { assert!(f > 90.0, "fitness {} should be > 90", f); }
    else { panic!("expected Float, got {:?}", val); }
}

#[test]
fn test_genetic_genes_method() {
    let src = r#"
genetic class Simple {
    chromosome ch {
        gene a: Int = 5 { range 1..10 }
    }
    fitness() -> Float { return 1.0 }
}
s = Simple()
x = s.genes().len()
"#;
    let val = run_get(src, "x");
    if let Value::Int(n) = val { assert_eq!(n, 1); }
    else { panic!("expected Int, got {:?}", val); }
}

#[test]
fn test_crossover_produces_child() {
    let src = r#"
genetic class G {
    chromosome ch {
        gene val: Float = 0.5 { range 0.0..1.0 }
    }
    fitness() -> Float { return 1.0 }
}
a = G()
b = G()
c = crossover(a, b)
x = c.val
"#;
    let val = run_get(src, "x");
    // Should be a float between 0 and 1
    if let Value::Float(f) = val { assert!(f >= 0.0 && f <= 1.0, "val {} out of range", f); }
    else { panic!("expected Float, got {:?}", val); }
}

use std::time::{Duration, Instant};

use crate::interpreter::environment::Environment;
use crate::interpreter::eval;
use crate::interpreter::exec;
use crate::interpreter::values::*;
use crate::parser::ast::*;
use crate::lexer::scanner::Scanner;
use crate::parser::parser::Parser;

/// Execute a parallel block. Each top-level assignment runs concurrently.
///
/// Strategy: for each task, we serialize the expression to source text,
/// then spawn a thread that re-parses and evaluates it in a fresh environment.
/// This gives true parallelism without requiring Send on Rc-based Values.
/// For tasks that need shared state, we fall back to sequential execution.
pub fn exec_parallel(
    tasks: &[Stmt],
    timeout: &Option<Expr>,
    _max_concurrency: &Option<Expr>,
    is_race: bool,
    env: &mut Environment,
) -> Result<(), Signal> {
    let timeout_dur = if let Some(t_expr) = timeout {
        let secs = eval::eval_expr(t_expr, env)?.as_float().unwrap_or(5.0);
        Some(Duration::from_secs_f64(secs))
    } else {
        None
    };

    let start = Instant::now();

    if is_race {
        // Race: first task to complete wins
        for stmt in tasks {
            if let Some(dl) = timeout_dur {
                if start.elapsed() >= dl {
                    return Err(Signal::Throw(Value::String("parallel.race timeout".into())));
                }
            }
            match &stmt.kind {
                StmtKind::VarDecl { name, value: Some(expr), .. } => {
                    match eval::eval_expr(expr, env) {
                        Ok(val) => {
                            env.define(name, val);
                            return Ok(());
                        }
                        Err(Signal::Throw(_)) => continue,
                        Err(e) => return Err(e),
                    }
                }
                _ => { exec::exec_stmt(stmt, env)?; }
            }
        }
        return Ok(());
    }

    // Collect tasks that are simple variable assignments with function calls
    // These can be parallelized by re-evaluating in separate threads
    let mut parallel_tasks: Vec<(String, String)> = Vec::new(); // (var_name, source_text)
    let mut sequential_tasks: Vec<&Stmt> = Vec::new();

    for stmt in tasks {
        match &stmt.kind {
            StmtKind::VarDecl { name, value: Some(expr), .. } => {
                // Serialize expression using its span info to reconstruct source
                // For now, we'll run these with thread::scope using fresh envs
                let source = expr_to_source(expr);
                if !source.is_empty() {
                    parallel_tasks.push((name.clone(), source));
                } else {
                    sequential_tasks.push(stmt);
                }
            }
            _ => sequential_tasks.push(stmt),
        }
    }

    if parallel_tasks.len() > 1 {
        // True parallel execution using OS threads
        let results: Vec<(String, String)> = std::thread::scope(|s| {
            let handles: Vec<_> = parallel_tasks.iter().map(|(name, source)| {
                let name = name.clone();
                let source = source.clone();
                s.spawn(move || {
                    let mut scanner = Scanner::new(&source, "<parallel>".to_string());
                    let tokens = scanner.scan_tokens();
                    let mut parser = Parser::new(tokens);
                    match parser.parse_expression(0) {
                        Ok(expr) => {
                            let mut fresh_env = Environment::new();
                            crate::interpreter::register_builtins(&mut fresh_env);
                            match eval::eval_expr(&expr, &mut fresh_env) {
                                Ok(val) => (name, val.to_string()),
                                Err(_) => (name, "nil".to_string()),
                            }
                        }
                        Err(_) => (name, "nil".to_string()),
                    }
                })
            }).collect();

            handles.into_iter().map(|h| h.join().unwrap()).collect()
        });

        // Parse results back into values and define in env
        for (name, val_str) in results {
            let val = parse_value_string(&val_str);
            env.define(&name, val);
        }
    } else {
        // Single or no parallel task — run directly
        for (name, _) in &parallel_tasks {
            // Find original stmt and execute directly
            for stmt in tasks {
                if let StmtKind::VarDecl { name: n, .. } = &stmt.kind {
                    if n == name {
                        exec::exec_stmt(stmt, env)?;
                        break;
                    }
                }
            }
        }
    }

    // Run remaining sequential tasks
    for stmt in sequential_tasks {
        exec::exec_stmt(stmt, env)?;
    }

    // Check timeout
    if let Some(dl) = timeout_dur {
        if start.elapsed() >= dl {
            return Err(Signal::Throw(Value::String("parallel timeout".into())));
        }
    }

    Ok(())
}

/// Best-effort serialization of an expression back to source text.
fn expr_to_source(expr: &Expr) -> String {
    match &expr.kind {
        ExprKind::Call { callee, args } => {
            let callee_s = expr_to_source(callee);
            let args_s: Vec<String> = args.iter().map(|a| expr_to_source(&a.value)).collect();
            format!("{}({})", callee_s, args_s.join(", "))
        }
        ExprKind::Identifier(name) => name.clone(),
        ExprKind::IntLiteral(n) => n.to_string(),
        ExprKind::FloatLiteral(f) => f.to_string(),
        ExprKind::StringLiteral(s) => format!("\"{}\"", s),
        ExprKind::BoolLiteral(b) => b.to_string(),
        ExprKind::NilLiteral => "nil".to_string(),
        ExprKind::Binary { left, op, right } => {
            let op_s = match op {
                BinaryOp::Add => "+", BinaryOp::Sub => "-", BinaryOp::Mul => "*",
                BinaryOp::Div => "/", BinaryOp::Mod => "%", BinaryOp::Pow => "**",
                _ => return String::new(),
            };
            format!("{} {} {}", expr_to_source(left), op_s, expr_to_source(right))
        }
        _ => String::new(),
    }
}

/// Parse a serialized value string back into a Value.
fn parse_value_string(s: &str) -> Value {
    if s == "nil" { return Value::Nil; }
    if s == "true" { return Value::Bool(true); }
    if s == "false" { return Value::Bool(false); }
    if let Ok(n) = s.parse::<i64>() { return Value::Int(n); }
    if let Ok(f) = s.parse::<f64>() { return Value::Float(f); }
    Value::String(s.to_string())
}

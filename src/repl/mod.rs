use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use crate::interpreter;
use crate::interpreter::environment::Environment;
use crate::interpreter::values::Value;
use crate::lexer::scanner::Scanner;
use crate::parser::parser::Parser;

/// Start the Aether REPL.
pub fn start_repl() {
    println!("Aether REPL v{}", env!("CARGO_PKG_VERSION"));
    println!("Type :help for help, :quit to exit\n");

    let mut rl = DefaultEditor::new().expect("Failed to create line editor");
    let mut env = Environment::new();
    interpreter::register_builtins(&mut env);

    loop {
        let prompt = "aether> ";
        match rl.readline(prompt) {
            Ok(line) => {
                let line = line.trim().to_string();
                if line.is_empty() { continue; }

                let _ = rl.add_history_entry(&line);

                // Commands
                match line.as_str() {
                    ":quit" | ":q" | ":exit" => {
                        println!("Goodbye!");
                        break;
                    }
                    ":help" | ":h" => {
                        println!("Commands:");
                        println!("  :help          Show this help");
                        println!("  :quit          Exit REPL");
                        println!("  :clear         Clear environment");
                        println!("  :load <file>   Load and execute a .ae file");
                        println!("  :time <expr>   Time an expression");
                        println!("  :type <expr>   Show type of expression result");
                        println!("  :tokens <code> Show token stream");
                        println!("  :ast <code>    Show AST");
                        println!("  :locals        Show all variables");
                        continue;
                    }
                    ":clear" => {
                        env = Environment::new();
                        interpreter::register_builtins(&mut env);
                        println!("Environment cleared.");
                        continue;
                    }
                    ":locals" => {
                        for (name, val) in env.all_named_values() {
                            // Skip builtins
                            if matches!(val, Value::NativeFunction(_)) { continue; }
                            println!("  {} = {}", name, val);
                        }
                        continue;
                    }
                    _ if line.starts_with(":load ") => {
                        let path = line[6..].trim();
                        match std::fs::read_to_string(path) {
                            Ok(source) => {
                                let mut s = Scanner::new(&source, path.to_string());
                                let tokens = s.scan_tokens();
                                let mut p = Parser::new(tokens);
                                match p.parse_program() {
                                    Ok(program) => {
                                        match interpreter::interpret(&program, &mut env) {
                                            Ok(()) => println!("Loaded {}", path),
                                            Err(e) => eprintln!("Error: {}", e),
                                        }
                                    }
                                    Err(errors) => {
                                        for e in &errors { eprintln!("{}", e); }
                                    }
                                }
                            }
                            Err(e) => eprintln!("Cannot read '{}': {}", path, e),
                        }
                        continue;
                    }
                    _ if line.starts_with(":time ") => {
                        let expr_str = &line[6..];
                        let start = std::time::Instant::now();
                        let mut s = Scanner::new(expr_str, "<repl>".to_string());
                        let tokens = s.scan_tokens();
                        let mut p = Parser::new(tokens);
                        match p.parse_program() {
                            Ok(program) => {
                                let _ = interpreter::interpret(&program, &mut env);
                            }
                            Err(_) => {}
                        }
                        let elapsed = start.elapsed();
                        println!("  Time: {:.3}ms", elapsed.as_secs_f64() * 1000.0);
                        continue;
                    }
                    _ if line.starts_with(":type ") => {
                        let expr_str = &line[6..];
                        let mut s = Scanner::new(expr_str, "<repl>".to_string());
                        let tokens = s.scan_tokens();
                        let mut p = Parser::new(tokens);
                        match p.parse_expression(0) {
                            Ok(expr) => {
                                match crate::interpreter::eval::eval_expr(&expr, &mut env) {
                                    Ok(val) => {
                                        let type_name = match &val {
                                            Value::Int(_) => "Int",
                                            Value::Float(_) => "Float",
                                            Value::String(_) => "Str",
                                            Value::Bool(_) => "Bool",
                                            Value::Nil => "Nil",
                                            Value::List(_) => "List",
                                            Value::Map(_) => "Map",
                                            Value::Set(_) => "Set",
                                            Value::Tuple(_) => "Tuple",
                                            Value::Function(_) => "Function",
                                            Value::NativeFunction(_) => "NativeFunction",
                                            Value::Class(c) => &c.name,
                                            Value::Instance(i) => &i.borrow().class_name,
                                            _ => "Unknown",
                                        };
                                        println!("  {}", type_name);
                                    }
                                    Err(_) => eprintln!("  Could not evaluate"),
                                }
                            }
                            Err(e) => eprintln!("  Parse error: {}", e),
                        }
                        continue;
                    }
                    _ if line.starts_with(":tokens ") => {
                        let code = &line[8..];
                        let mut s = Scanner::new(code, "<repl>".to_string());
                        let tokens = s.scan_tokens();
                        for t in &tokens {
                            if !matches!(t.kind, crate::lexer::tokens::TokenKind::Eof) {
                                println!("  {:?}", t.kind);
                            }
                        }
                        continue;
                    }
                    _ if line.starts_with(":ast ") => {
                        let code = &line[5..];
                        let mut s = Scanner::new(code, "<repl>".to_string());
                        let tokens = s.scan_tokens();
                        let mut p = Parser::new(tokens);
                        match p.parse_program() {
                            Ok(program) => {
                                for stmt in &program.statements {
                                    println!("  {:?}", stmt.kind);
                                }
                            }
                            Err(errors) => {
                                for e in &errors { eprintln!("  {}", e); }
                            }
                        }
                        continue;
                    }
                    _ => {}
                }

                // Collect multiline input if braces are unbalanced
                let mut input = line;
                while count_braces(&input) > 0 {
                    match rl.readline("  ... ") {
                        Ok(cont) => {
                            let _ = rl.add_history_entry(&cont);
                            input.push('\n');
                            input.push_str(&cont);
                        }
                        Err(_) => break,
                    }
                }

                // Parse and execute
                let mut scanner = Scanner::new(&input, "<repl>".to_string());
                let tokens = scanner.scan_tokens();
                let mut parser = Parser::new(tokens);

                match parser.parse_program() {
                    Ok(program) => {
                        match interpreter::interpret(&program, &mut env) {
                            Ok(()) => {
                                // If the input was a single expression, print its value
                                if let Some(val) = env.get("_") {
                                    if !matches!(val, Value::Nil) {
                                        println!("{}", val);
                                    }
                                }
                            }
                            Err(e) => eprintln!("Error: {}", e),
                        }
                    }
                    Err(errors) => {
                        for e in &errors {
                            eprintln!("Parse error: {}", e);
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("Goodbye!");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }
}

fn count_braces(s: &str) -> i32 {
    let mut count = 0i32;
    for ch in s.chars() {
        match ch {
            '{' | '(' | '[' => count += 1,
            '}' | ')' | ']' => count -= 1,
            _ => {}
        }
    }
    count
}

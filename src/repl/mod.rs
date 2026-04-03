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
                        println!("  :help    Show this help");
                        println!("  :quit    Exit REPL");
                        println!("  :clear   Clear environment");
                        continue;
                    }
                    ":clear" => {
                        env = Environment::new();
                        interpreter::register_builtins(&mut env);
                        println!("Environment cleared.");
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

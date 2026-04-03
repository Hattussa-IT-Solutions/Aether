//! CLI Debugger for the Aether programming language.
//!
//! Wraps the interpreter to add breakpoint and stepping support.
//! Does not modify the interpreter internals — operates as a wrapper
//! that iterates top-level statements and checks breakpoints before
//! each one.

use std::collections::HashMap;
use std::fs;
use std::process;

use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

use crate::interpreter::environment::Environment;
use crate::interpreter::exec;
use crate::interpreter::eval;
use crate::interpreter::values::{Signal, Value};
use crate::lexer::scanner::Scanner;
use crate::parser::parser::Parser;

// ─── Step mode ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum StepMode {
    /// Run freely until the next breakpoint.
    Run,
    /// Stop at the next top-level statement (step over).
    StepOver,
}

// ─── Debugger state ──────────────────────────────────────────────────────────

pub struct DebugState {
    /// line number -> enabled flag
    breakpoints: HashMap<usize, bool>,
    step_mode: StepMode,
    source_lines: Vec<String>,
    filename: String,
    current_line: usize,
}

impl DebugState {
    fn new(filename: &str, source: &str) -> Self {
        let source_lines = source.lines().map(|l| l.to_string()).collect();
        Self {
            breakpoints: HashMap::new(),
            step_mode: StepMode::StepOver, // pause before first statement
            source_lines,
            filename: filename.to_string(),
            current_line: 0,
        }
    }

    /// Returns true if execution should pause before the statement at `line`.
    fn should_pause(&self, line: usize) -> bool {
        if self.step_mode == StepMode::StepOver {
            return true;
        }
        // Check for an enabled breakpoint at this line.
        self.breakpoints.get(&line).copied().unwrap_or(false)
    }

    fn set_breakpoint(&mut self, line: usize) {
        self.breakpoints.insert(line, true);
        println!("Breakpoint set at line {}", line);
    }

    fn delete_breakpoint(&mut self, line: usize) {
        if self.breakpoints.remove(&line).is_some() {
            println!("Breakpoint at line {} removed", line);
        } else {
            println!("No breakpoint at line {}", line);
        }
    }

    fn list_breakpoints(&self) {
        if self.breakpoints.is_empty() {
            println!("No breakpoints set.");
            return;
        }
        let mut lines: Vec<usize> = self.breakpoints.keys().cloned().collect();
        lines.sort();
        println!("Breakpoints:");
        for l in lines {
            let enabled = if self.breakpoints[&l] { "enabled" } else { "disabled" };
            println!("  line {} ({})", l, enabled);
        }
    }

    /// Show source context around `line` (1-based).
    fn show_context(&self, line: usize, context: usize) {
        let total = self.source_lines.len();
        if total == 0 || line == 0 {
            return;
        }
        let start = line.saturating_sub(context).max(1);
        let end = (line + context).min(total);

        println!("  --> {}:{}", self.filename, line);
        for i in start..=end {
            let src = &self.source_lines[i - 1];
            let current_marker = if i == line { ">>" } else { "  " };
            let bp_marker = if self.breakpoints.get(&i).copied().unwrap_or(false) { "*" } else { " " };
            println!("{}{} {:4} | {}", current_marker, bp_marker, i, src);
        }
    }

    fn show_current(&self) {
        self.show_context(self.current_line, 3);
    }
}

// ─── Command loop ────────────────────────────────────────────────────────────

/// What the command loop tells the outer execution loop to do next.
enum DebugAction {
    /// Continue running (step mode = Run).
    Continue,
    /// Execute exactly one statement, then pause again.
    Step,
    /// Quit the debugger entirely.
    Quit,
}

/// Print the debugger help message.
fn print_help() {
    println!("Aether Debugger Commands:");
    println!("  run / r / continue / c  — run until next breakpoint");
    println!("  next / n                — execute one statement and pause");
    println!("  step / s                — same as next (step into not yet supported)");
    println!("  break N / b N           — set breakpoint at line N");
    println!("  delete N / d N          — remove breakpoint at line N");
    println!("  list / l                — list all breakpoints");
    println!("  print EXPR / p EXPR     — evaluate EXPR in current scope");
    println!("  locals                  — show all variables in scope");
    println!("  where / w               — show current position");
    println!("  stack / bt              — show call stack info");
    println!("  help / h                — show this help");
    println!("  quit / q                — exit debugger");
}

/// Evaluate an expression string in the given environment and print the result.
fn cmd_print(expr_str: &str, env: &mut Environment) {
    if expr_str.trim().is_empty() {
        println!("Usage: print <expression>");
        return;
    }
    let mut scanner = Scanner::new(expr_str, "<debug>".into());
    let tokens = scanner.scan_tokens();
    let mut parser = Parser::new(tokens);
    let expr = match parser.parse_expression(0) {
        Ok(e) => e,
        Err(e) => {
            println!("Parse error: {}", e);
            return;
        }
    };
    match eval::eval_expr(&expr, env) {
        Ok(val) => println!("{}", val),
        Err(Signal::Throw(val)) => println!("Error: {}", val),
        Err(Signal::Return(val)) => println!("{}", val),
        Err(_) => println!("(evaluation signal)"),
    }
}

/// Show all variables visible in the current environment.
fn cmd_locals(env: &Environment) {
    let pairs = env.all_named_values();
    if pairs.is_empty() {
        println!("(no variables in scope)");
        return;
    }
    println!("Local variables:");
    for (name, val) in &pairs {
        // Skip built-in functions for a cleaner display
        if matches!(val, Value::NativeFunction(_)) {
            continue;
        }
        println!("  {} = {}", name, val);
    }
}

/// Enter the interactive command loop when the debugger is paused.
/// Returns the `DebugAction` selected by the user.
fn command_loop(state: &mut DebugState, env: &mut Environment, rl: &mut DefaultEditor) -> DebugAction {
    state.show_current();

    loop {
        let prompt = format!("(adb) ");
        let readline = rl.readline(&prompt);
        match readline {
            Ok(line) => {
                let line = line.trim().to_string();
                if !line.is_empty() {
                    let _ = rl.add_history_entry(&line);
                }

                // Split into command + rest
                let mut parts = line.splitn(2, ' ');
                let cmd = parts.next().unwrap_or("").trim();
                let rest = parts.next().unwrap_or("").trim();

                match cmd {
                    // Continue running
                    "run" | "r" | "continue" | "c" => {
                        state.step_mode = StepMode::Run;
                        return DebugAction::Continue;
                    }
                    // Next / step
                    "next" | "n" | "step" | "s" => {
                        state.step_mode = StepMode::StepOver;
                        return DebugAction::Step;
                    }
                    // Break
                    "break" | "b" => {
                        match rest.parse::<usize>() {
                            Ok(line_no) => state.set_breakpoint(line_no),
                            Err(_) => println!("Usage: break <line>"),
                        }
                    }
                    // Delete breakpoint
                    "delete" | "d" => {
                        match rest.parse::<usize>() {
                            Ok(line_no) => state.delete_breakpoint(line_no),
                            Err(_) => println!("Usage: delete <line>"),
                        }
                    }
                    // List breakpoints
                    "list" | "l" => {
                        state.list_breakpoints();
                    }
                    // Print expression
                    "print" | "p" => {
                        cmd_print(rest, env);
                    }
                    // Locals
                    "locals" => {
                        cmd_locals(env);
                    }
                    // Where
                    "where" | "w" => {
                        state.show_context(state.current_line, 5);
                    }
                    // Stack / backtrace
                    "stack" | "bt" => {
                        println!("Stack depth: {} scope(s)", env.depth());
                        println!("  [0] {} line {}", state.filename, state.current_line);
                        println!("  (full call stack requires deeper integration)");
                    }
                    // Help
                    "help" | "h" | "?" => {
                        print_help();
                    }
                    // Quit
                    "quit" | "q" => {
                        return DebugAction::Quit;
                    }
                    // Empty line: repeat last action (treat as next)
                    "" => {
                        state.step_mode = StepMode::StepOver;
                        return DebugAction::Step;
                    }
                    _ => {
                        println!("Unknown command: '{}'. Type 'help' for commands.", cmd);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C (type 'quit' to exit)");
            }
            Err(ReadlineError::Eof) => {
                println!("^D");
                return DebugAction::Quit;
            }
            Err(e) => {
                eprintln!("readline error: {}", e);
                return DebugAction::Quit;
            }
        }
    }
}

// ─── Main entry point ────────────────────────────────────────────────────────

/// Run the Aether debugger for the given source file.
pub fn run_debugger(filename: &str) {
    // Read source
    let source = match fs::read_to_string(filename) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading '{}': {}", filename, e);
            process::exit(1);
        }
    };

    // Lex
    let mut scanner = Scanner::new(&source, filename.to_string());
    let tokens = scanner.scan_tokens();

    // Parse
    let mut parser_inst = Parser::new(tokens);
    let program = match parser_inst.parse_program() {
        Ok(p) => p,
        Err(errors) => {
            for e in &errors {
                eprintln!("{}", e);
            }
            process::exit(1);
        }
    };

    // Set up environment
    let mut env = Environment::new();
    crate::interpreter::register_builtins(&mut env);

    // Set up debug state
    let mut state = DebugState::new(filename, &source);

    // Set up rustyline editor
    let mut rl = match DefaultEditor::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to create line editor: {}", e);
            process::exit(1);
        }
    };

    println!("Aether Debugger — '{}' ({} statements)", filename, program.statements.len());
    println!("Type 'help' for commands. Press Enter to step, 'r' to run.");
    println!();

    // ── Main execution loop ───────────────────────────────────────────────────
    let stmts = &program.statements;
    let mut idx = 0;

    while idx < stmts.len() {
        let stmt = &stmts[idx];
        let line = stmt.span.line;
        state.current_line = line;

        // Check whether to pause at this statement
        if state.should_pause(line) {
            let action = command_loop(&mut state, &mut env, &mut rl);
            match action {
                DebugAction::Continue => {
                    // step_mode is now Run; re-check breakpoint for this stmt
                    // (the stmt hasn't executed yet — we just decided to continue)
                    // If there's a breakpoint here and we just typed 'c', skip the bp this once.
                    // Re-run the pause check: in Run mode, only stop at BPs.
                    // Since we're already at this statement, execute it now.
                }
                DebugAction::Step => {
                    // step_mode is StepOver — will pause before the NEXT statement.
                    // Execute the current statement now (fall through).
                }
                DebugAction::Quit => {
                    println!("Debugger exited.");
                    return;
                }
            }
        }

        // Execute the statement
        match exec::exec_stmt(stmt, &mut env) {
            Ok(()) => {}
            Err(Signal::Return(val)) => {
                if !matches!(val, Value::Nil) {
                    println!("{}", val);
                }
                // Top-level return ends execution
                break;
            }
            Err(Signal::Throw(val)) => {
                eprintln!("Unhandled error at line {}: {}", line, val);
                // Pause so the user can inspect
                state.current_line = line;
                let action = command_loop(&mut state, &mut env, &mut rl);
                if matches!(action, DebugAction::Quit) {
                    return;
                }
                // After inspection, stop execution
                break;
            }
            Err(Signal::Break(_)) => {
                eprintln!("'break' outside of loop at line {}", line);
                break;
            }
            Err(Signal::Next(_)) => {
                eprintln!("'next' outside of loop at line {}", line);
                break;
            }
        }

        idx += 1;
    }

    println!("\n[Program finished]");
}

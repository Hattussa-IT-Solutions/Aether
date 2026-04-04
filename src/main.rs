#![allow(dead_code, unused_imports, unused_variables, unused_mut)]

use std::env;
use std::fs;
use std::process;

mod lexer;
mod parser;
mod types;
mod interpreter;
mod compiler;
mod codegen;
mod stdlib;
mod bridge;
mod repl;
mod diagnostics;
mod forge;
mod lsp;
mod dap;
mod debugger;
mod watcher;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    match args[1].as_str() {
        "run" => {
            // Parse --max-instructions flag
            if let Some(pos) = args.iter().position(|a| a.starts_with("--max-instructions")) {
                let limit: u64 = if args[pos].contains('=') {
                    args[pos].split('=').nth(1).and_then(|v| v.parse().ok()).unwrap_or(interpreter::eval::DEFAULT_MAX_INSTRUCTIONS)
                } else if pos + 1 < args.len() {
                    args[pos + 1].parse().unwrap_or(interpreter::eval::DEFAULT_MAX_INSTRUCTIONS)
                } else {
                    interpreter::eval::DEFAULT_MAX_INSTRUCTIONS
                };
                interpreter::eval::set_max_instructions(limit);
            }

            if args.len() < 3 {
                // If aether.toml exists, run src/main.ae
                if std::path::Path::new("aether.toml").exists() && std::path::Path::new("src/main.ae").exists() {
                    run_file("src/main.ae");
                } else {
                    eprintln!("Usage: aether run <file.ae>");
                    process::exit(1);
                }
            } else {
                let use_vm = args.iter().any(|a| a == "--vm");
                let use_watch = args.iter().any(|a| a == "--watch");
                let use_profile = args.iter().any(|a| a == "--profile");
                // Find the .ae file argument (skip flags)
                let file = args.iter().skip(2)
                    .find(|a| !a.starts_with('-'))
                    .map(|s| s.as_str())
                    .unwrap_or(&args[2]);
                if use_watch {
                    watcher::run_with_watch(file);
                } else if use_vm {
                    run_file_vm(file);
                } else {
                    run_file_with_options(file, use_profile);
                }
            }
        }
        "repl" => {
            repl::start_repl();
        }
        "new" => {
            if args.len() < 3 {
                eprintln!("Usage: aether new <project-name>");
                process::exit(1);
            }
            if let Err(e) = forge::create_project(&args[2]) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        "init" => {
            if let Err(e) = forge::init_project() {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        "add" => {
            if args.len() < 3 {
                eprintln!("Usage: aether add <package>[@version]");
                process::exit(1);
            }
            let dev = args.iter().any(|a| a == "--dev");
            let pkg = match args.iter().skip(2).find(|a| !a.starts_with('-')) {
                Some(p) => p,
                None => {
                    eprintln!("Usage: aether add <package>[@version]");
                    process::exit(1);
                }
            };
            if let Err(e) = forge::add_dependency(pkg, dev) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        "remove" => {
            if args.len() < 3 {
                eprintln!("Usage: aether remove <package>");
                process::exit(1);
            }
            if let Err(e) = forge::remove_dependency(&args[2]) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        "install" => {
            if let Err(e) = forge::install_deps() {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        "list" => {
            if let Err(e) = forge::list_dependencies() {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        "build" => {
            if let Err(e) = forge::build_project() {
                eprintln!("{}", e);
                process::exit(1);
            }
        }
        "test" => {
            let dir = if args.len() >= 3 { &args[2] } else { "." };
            if let Err(e) = forge::run_tests(dir) {
                eprintln!("{}", e);
                process::exit(1);
            }
        }
        "check" => {
            if args.len() < 3 {
                eprintln!("Usage: aether check <file.ae>");
                process::exit(1);
            }
            check_file(&args[2]);
        }
        "fmt" => {
            if args.len() < 3 {
                eprintln!("Usage: aether fmt <file.ae>");
                process::exit(1);
            }
            if let Err(e) = forge::format_file(&args[2]) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        "jit" => {
            if args.len() < 3 {
                eprintln!("Usage: aether jit <file.ae>");
                process::exit(1);
            }
            jit_file(&args[2]);
        }
        "lsp" => {
            if let Err(e) = lsp::server::run_lsp() {
                eprintln!("LSP error: {}", e);
                process::exit(1);
            }
        }
        "dap" => {
            dap::server::run_dap();
        }
        "debug" => {
            if args.len() < 3 {
                eprintln!("Usage: aether debug <file.ae>");
                process::exit(1);
            }
            debugger::debugger::run_debugger(&args[2]);
        }
        "--version" | "-V" => {
            println!("aether {}", env!("CARGO_PKG_VERSION"));
        }
        "--help" | "-h" => {
            print_usage();
        }
        // If arg is a .ae file, run it directly
        path if path.ends_with(".ae") => {
            run_file(path);
        }
        cmd => {
            eprintln!("Unknown command: {}", cmd);
            print_usage();
            process::exit(1);
        }
    }
}

fn run_file(filename: &str) {
    run_file_with_options(filename, false);
}

fn run_file_with_options(filename: &str, profile: bool) {
    let source = match fs::read_to_string(filename) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading '{}': {}", filename, e);
            process::exit(1);
        }
    };

    // Lex
    let mut scanner = lexer::scanner::Scanner::new(&source, filename.to_string());
    let tokens = scanner.scan_tokens();

    // Parse
    let mut parser_inst = parser::parser::Parser::new(tokens);
    let program = match parser_inst.parse_program() {
        Ok(p) => p,
        Err(errors) => {
            for e in &errors {
                eprintln!("{}", e);
            }
            process::exit(1);
        }
    };

    // Interpret
    let mut env = interpreter::environment::Environment::new();
    interpreter::register_builtins(&mut env);
    env.profiling = profile;

    let start = std::time::Instant::now();
    let result = interpreter::interpret(&program, &mut env);
    let elapsed = start.elapsed();

    if let Err(e) = result {
        eprintln!("{}", e);
        process::exit(1);
    }

    if profile {
        eprintln!("\n=== Profile ===");
        eprintln!("Total time:          {:.1}ms", elapsed.as_secs_f64() * 1000.0);
        eprintln!("Instructions:        {}", env.instruction_count);
        eprintln!("Function calls:      {}", env.profile_function_calls);
        eprintln!("Variable lookups:    {}", env.profile_var_lookups);
        if env.instruction_count > 0 {
            let ns_per = (elapsed.as_nanos() as f64) / (env.instruction_count as f64);
            eprintln!("Time per instruction: {:.0}ns", ns_per);
        }
    }
}

fn check_file(filename: &str) {
    let source = match fs::read_to_string(filename) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading '{}': {}", filename, e);
            process::exit(1);
        }
    };

    let mut scanner = lexer::scanner::Scanner::new(&source, filename.to_string());
    let tokens = scanner.scan_tokens();
    let mut parser_inst = parser::parser::Parser::new(tokens);

    let program = match parser_inst.parse_program() {
        Ok(p) => p,
        Err(errors) => {
            for e in &errors { eprintln!("{}", e); }
            process::exit(1);
        }
    };

    let strict = program.directives.iter().any(|d| d.name == "strict");
    let mut checker = types::checker::TypeChecker::new(strict);
    let errors = checker.check_program(&program);

    if errors.is_empty() {
        println!("No type errors found.");
    } else {
        let output = diagnostics::format_errors(&errors, &source);
        eprint!("{}", output);
        process::exit(1);
    }
}

fn jit_file(filename: &str) {
    let source = match fs::read_to_string(filename) {
        Ok(s) => s,
        Err(e) => { eprintln!("Error: {}", e); process::exit(1); }
    };
    let mut scanner = lexer::scanner::Scanner::new(&source, filename.to_string());
    let tokens = scanner.scan_tokens();
    let mut parser_inst = parser::parser::Parser::new(tokens);
    let program = match parser_inst.parse_program() {
        Ok(p) => p,
        Err(errors) => { for e in &errors { eprintln!("{}", e); } process::exit(1); }
    };
    match codegen::cranelift::jit_compile_and_run(&program) {
        Ok(result) => println!("{}", result),
        Err(e) => { eprintln!("JIT error: {}", e); process::exit(1); }
    }
}

fn run_file_vm(filename: &str) {
    let source = match fs::read_to_string(filename) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading '{}': {}", filename, e);
            process::exit(1);
        }
    };

    let mut scanner = lexer::scanner::Scanner::new(&source, filename.to_string());
    let tokens = scanner.scan_tokens();
    let mut parser_inst = parser::parser::Parser::new(tokens);
    let program = match parser_inst.parse_program() {
        Ok(p) => p,
        Err(errors) => {
            for e in &errors { eprintln!("{}", e); }
            process::exit(1);
        }
    };

    let mut comp = compiler::compiler::Compiler::new();
    let chunks = comp.compile_program(&program);

    let mut vm = compiler::vm::VM::new();
    if let Err(e) = vm.execute_all(&chunks) {
        eprintln!("VM error: {}", e);
        process::exit(1);
    }
}

fn print_usage() {
    eprintln!("Aether Programming Language v{}", env!("CARGO_PKG_VERSION"));
    eprintln!();
    eprintln!("Usage: aether <command> [args]");
    eprintln!();
    eprintln!("Run & Execute:");
    eprintln!("  run [file.ae]      Run a source file (or src/main.ae if in project)");
    eprintln!("  run file.ae --vm   Run using bytecode VM");
    eprintln!("  jit <file.ae>      Compile and run via Cranelift JIT");
    eprintln!("  repl               Start interactive REPL");
    eprintln!();
    eprintln!("Project Management (Forge):");
    eprintln!("  new <name>         Create a new project");
    eprintln!("  init               Initialize aether.toml in current directory");
    eprintln!("  add <pkg>          Add a dependency (use @version for specific)");
    eprintln!("  add --dev <pkg>    Add a dev dependency");
    eprintln!("  remove <pkg>       Remove a dependency");
    eprintln!("  install            Install all dependencies");
    eprintln!("  list               List dependencies");
    eprintln!("  build              Parse and type-check project");
    eprintln!("  test [dir]         Run test files");
    eprintln!("  fmt <file.ae>      Format source code");
    eprintln!();
    eprintln!("Tools:");
    eprintln!("  check <file.ae>    Type-check a file");
    eprintln!("  debug <file.ae>    Start the CLI debugger");
    eprintln!("  dap                Start DAP debug server (for editors)");
    eprintln!("  lsp                Start Language Server (for editors)");
    eprintln!("  --version          Print version");
    eprintln!("  --help             Show this help");
}

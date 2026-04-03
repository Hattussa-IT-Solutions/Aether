use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;
use notify::{Watcher, RecursiveMode, Config, Event, EventKind};

/// Run a file with watch mode — re-run on every .ae file change.
pub fn run_with_watch(filename: &str) {
    let path = Path::new(filename).canonicalize().unwrap_or_else(|_| Path::new(filename).to_path_buf());
    let watch_dir = path.parent().unwrap_or_else(|| Path::new("."));

    println!("\x1b[34m[watching]\x1b[0m {} (Ctrl+C to stop)", filename);
    println!();

    // Initial run
    run_file(filename);

    // Set up file watcher
    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();

    let mut watcher = notify::recommended_watcher(tx).expect("Failed to create file watcher");
    watcher.watch(watch_dir, RecursiveMode::Recursive).expect("Failed to watch directory");

    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(Ok(event)) => {
                // Only react to .ae file modifications
                let is_ae_change = matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_))
                    && event.paths.iter().any(|p| {
                        p.extension().and_then(|e| e.to_str()) == Some("ae")
                    });

                if is_ae_change {
                    // Debounce: drain any pending events
                    while rx.recv_timeout(Duration::from_millis(200)).is_ok() {}

                    // Clear screen and re-run
                    print!("\x1b[2J\x1b[1;1H"); // ANSI clear screen + move cursor to top
                    println!("\x1b[33m[restarting]\x1b[0m {}", filename);
                    println!();
                    run_file(filename);
                    println!();
                    println!("\x1b[34m[watching]\x1b[0m Waiting for changes...");
                }
            }
            Ok(Err(e)) => {
                eprintln!("\x1b[31m[error]\x1b[0m Watch error: {}", e);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // Normal — no events, keep waiting
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                eprintln!("File watcher disconnected");
                break;
            }
        }
    }
}

fn run_file(filename: &str) {
    let source = match std::fs::read_to_string(filename) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("\x1b[31m[error]\x1b[0m Cannot read '{}': {}", filename, e);
            return;
        }
    };

    let mut scanner = crate::lexer::scanner::Scanner::new(&source, filename.to_string());
    let tokens = scanner.scan_tokens();
    let mut parser = crate::parser::parser::Parser::new(tokens);

    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(errors) => {
            eprintln!("\x1b[31m[error]\x1b[0m Parse errors:");
            for e in &errors { eprintln!("  {}", e); }
            return;
        }
    };

    let mut env = crate::interpreter::environment::Environment::new();
    crate::interpreter::register_builtins(&mut env);

    match crate::interpreter::interpret(&program, &mut env) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("\x1b[31m[error]\x1b[0m {}", e);
        }
    }
}

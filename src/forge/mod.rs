pub mod toml_parser;
pub mod resolver;

use std::fs;
use std::path::Path;

/// Create a new Aether project.
pub fn create_project(name: &str) -> Result<(), String> {
    let project_dir = Path::new(name);
    if project_dir.exists() {
        return Err(format!("Directory '{}' already exists", name));
    }

    fs::create_dir_all(project_dir.join("src")).map_err(|e| e.to_string())?;
    fs::create_dir_all(project_dir.join("tests")).map_err(|e| e.to_string())?;

    // aether.toml
    let toml_content = format!(
        r#"[project]
name = "{}"
version = "0.1.0"

[dependencies]
"#,
        name
    );
    fs::write(project_dir.join("aether.toml"), toml_content).map_err(|e| e.to_string())?;

    // src/main.ae
    fs::write(
        project_dir.join("src/main.ae"),
        "print(\"Hello from Aether!\")\n",
    ).map_err(|e| e.to_string())?;

    // tests/test_main.ae
    fs::write(
        project_dir.join("tests/test_main.ae"),
        "// Add tests here\n",
    ).map_err(|e| e.to_string())?;

    println!("Created new Aether project: {}", name);
    Ok(())
}

/// Run tests in the tests/ directory.
pub fn run_tests(dir: &str) -> Result<(), String> {
    let test_dir = Path::new(dir).join("tests");
    if !test_dir.exists() {
        return Err("No tests/ directory found".into());
    }

    let mut passed = 0;
    let mut failed = 0;

    for entry in fs::read_dir(&test_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("ae") {
            let source = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let filename = path.display().to_string();

            let mut scanner = crate::lexer::scanner::Scanner::new(&source, filename.clone());
            let tokens = scanner.scan_tokens();
            let mut parser = crate::parser::parser::Parser::new(tokens);

            match parser.parse_program() {
                Ok(program) => {
                    let mut env = crate::interpreter::environment::Environment::new();
                    crate::interpreter::register_builtins(&mut env);
                    match crate::interpreter::interpret(&program, &mut env) {
                        Ok(()) => {
                            println!("  PASS: {}", filename);
                            passed += 1;
                        }
                        Err(e) => {
                            println!("  FAIL: {} — {}", filename, e);
                            failed += 1;
                        }
                    }
                }
                Err(errors) => {
                    println!("  FAIL: {} — parse errors:", filename);
                    for e in &errors { println!("    {}", e); }
                    failed += 1;
                }
            }
        }
    }

    println!("\n{} passed, {} failed", passed, failed);
    if failed > 0 { Err(format!("{} test(s) failed", failed)) }
    else { Ok(()) }
}

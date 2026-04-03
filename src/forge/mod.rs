pub mod toml_parser;
pub mod resolver;

use std::fs;
use std::path::Path;
use std::time::Instant;

/// Create a new Aether project with full scaffolding.
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
name = "{name}"
version = "0.1.0"
description = ""
author = ""
license = "MIT"

[dependencies]

[dev-dependencies]

[scripts]
start = "aether run src/main.ae"
test = "aether test"
"#,
        name = name
    );
    fs::write(project_dir.join("aether.toml"), toml_content).map_err(|e| e.to_string())?;

    // src/main.ae
    fs::write(
        project_dir.join("src/main.ae"),
        format!("print(\"Hello from {}!\")\n", name),
    ).map_err(|e| e.to_string())?;

    // tests/test_main.ae
    fs::write(
        project_dir.join("tests/test_main.ae"),
        r#"// Tests for the main module

@test
def test_hello() {
    // Add your tests here
    print("test passed")
}
"#,
    ).map_err(|e| e.to_string())?;

    // .gitignore
    fs::write(
        project_dir.join(".gitignore"),
        "/target/\n.aether/\n*.pyc\nnode_modules/\n",
    ).map_err(|e| e.to_string())?;

    // README.md
    fs::write(
        project_dir.join("README.md"),
        format!(
            "# {}\n\nAn Aether project.\n\n## Getting Started\n\n```bash\naether run\n```\n",
            name
        ),
    ).map_err(|e| e.to_string())?;

    println!("  Created new Aether project: {}", name);
    println!("");
    println!("  To get started:");
    println!("    cd {}", name);
    println!("    aether run");
    Ok(())
}

/// Initialize aether.toml in the current directory.
pub fn init_project() -> Result<(), String> {
    if Path::new("aether.toml").exists() {
        return Err("aether.toml already exists".into());
    }

    let dir_name = std::env::current_dir()
        .map(|p| p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "project".into()))
        .unwrap_or_else(|_| "project".into());

    let toml_content = format!(
        r#"[project]
name = "{}"
version = "0.1.0"

[dependencies]

[dev-dependencies]
"#,
        dir_name
    );
    fs::write("aether.toml", toml_content).map_err(|e| e.to_string())?;
    println!("  Initialized aether.toml");
    Ok(())
}

/// Add a dependency to aether.toml.
pub fn add_dependency(package: &str, dev: bool) -> Result<(), String> {
    let toml_path = "aether.toml";
    if !Path::new(toml_path).exists() {
        return Err("aether.toml not found. Run 'aether init' first.".into());
    }

    let content = fs::read_to_string(toml_path).map_err(|e| e.to_string())?;

    // Parse package@version
    let (name, version) = if let Some(at_pos) = package.find('@') {
        (&package[..at_pos], &package[at_pos + 1..])
    } else {
        (package, "latest")
    };

    let section = if dev { "[dev-dependencies]" } else { "[dependencies]" };
    let entry = format!("{} = \"{}\"", name, version);

    // Check if already exists
    if content.contains(&format!("{} =", name)) {
        // Update existing
        let mut new_content = String::new();
        for line in content.lines() {
            if line.starts_with(&format!("{} =", name)) || line.starts_with(&format!("{} =", name)) {
                new_content.push_str(&entry);
            } else {
                new_content.push_str(line);
            }
            new_content.push('\n');
        }
        fs::write(toml_path, new_content).map_err(|e| e.to_string())?;
    } else {
        // Add new entry after the section header
        let new_content = content.replace(
            section,
            &format!("{}\n{}", section, entry),
        );
        fs::write(toml_path, new_content).map_err(|e| e.to_string())?;
    }

    println!("  Added {} = \"{}\" to {}", name, version, if dev { "dev-dependencies" } else { "dependencies" });
    Ok(())
}

/// Remove a dependency from aether.toml.
pub fn remove_dependency(package: &str) -> Result<(), String> {
    let toml_path = "aether.toml";
    if !Path::new(toml_path).exists() {
        return Err("aether.toml not found".into());
    }

    let content = fs::read_to_string(toml_path).map_err(|e| e.to_string())?;
    let new_content: String = content.lines()
        .filter(|line| !line.starts_with(&format!("{} =", package)))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(toml_path, new_content + "\n").map_err(|e| e.to_string())?;
    println!("  Removed {}", package);
    Ok(())
}

/// List dependencies from aether.toml.
pub fn list_dependencies() -> Result<(), String> {
    let toml_path = "aether.toml";
    if !Path::new(toml_path).exists() {
        return Err("aether.toml not found".into());
    }

    let content = fs::read_to_string(toml_path).map_err(|e| e.to_string())?;
    let config = toml_parser::parse_aether_toml(&content)?;

    println!("  Dependencies:");
    if config.dependencies.is_empty() {
        println!("    (none)");
    }
    for (name, ver) in &config.dependencies {
        println!("    {} = \"{}\"", name, ver);
    }
    Ok(())
}

/// Install dependencies (create lock file).
pub fn install_deps() -> Result<(), String> {
    let toml_path = "aether.toml";
    if !Path::new(toml_path).exists() {
        return Err("aether.toml not found".into());
    }

    let content = fs::read_to_string(toml_path).map_err(|e| e.to_string())?;
    let config = toml_parser::parse_aether_toml(&content)?;
    let resolved = resolver::resolve_dependencies(&config);

    if resolved.is_empty() {
        println!("  No dependencies to install.");
        return Ok(());
    }

    // Create packages directory
    let pkg_dir = dirs_packages();
    fs::create_dir_all(&pkg_dir).map_err(|e| e.to_string())?;

    // Generate lock file
    let mut lock_content = String::new();
    lock_content.push_str("# This file is auto-generated by Forge. Do not edit.\n\n");
    for (name, version) in &resolved {
        lock_content.push_str(&format!(
            "[[package]]\nname = \"{}\"\nversion = \"{}\"\n\n",
            name, version
        ));
        println!("  Resolved {} = {}", name, version);
    }

    fs::write("aether.lock", lock_content).map_err(|e| e.to_string())?;
    println!("  Generated aether.lock with {} packages", resolved.len());
    Ok(())
}

/// Run tests — find @test functions and execute them.
pub fn run_tests(dir: &str) -> Result<(), String> {
    let test_dir = Path::new(dir).join("tests");

    let mut test_files = Vec::new();
    // Check tests/ directory
    if test_dir.exists() {
        for entry in fs::read_dir(&test_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("ae") {
                test_files.push(path);
            }
        }
    }

    if test_files.is_empty() {
        println!("  No test files found in tests/");
        return Ok(());
    }

    let mut passed = 0;
    let mut failed = 0;
    let mut errors = Vec::new();

    for path in &test_files {
        let source = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let filename = path.display().to_string();

        let mut scanner = crate::lexer::scanner::Scanner::new(&source, filename.clone());
        let tokens = scanner.scan_tokens();
        let mut parser = crate::parser::parser::Parser::new(tokens);

        match parser.parse_program() {
            Ok(program) => {
                let mut env = crate::interpreter::environment::Environment::new();
                crate::interpreter::register_builtins(&mut env);

                let start = Instant::now();
                match crate::interpreter::interpret(&program, &mut env) {
                    Ok(()) => {
                        let elapsed = start.elapsed();
                        println!("  \x1b[32m\u{2713}\x1b[0m {} ({:.1}ms)", filename, elapsed.as_secs_f64() * 1000.0);
                        passed += 1;
                    }
                    Err(e) => {
                        let elapsed = start.elapsed();
                        println!("  \x1b[31m\u{2717}\x1b[0m {} ({:.1}ms) \x1b[31m— {}\x1b[0m", filename, elapsed.as_secs_f64() * 1000.0, e);
                        errors.push(format!("{}: {}", filename, e));
                        failed += 1;
                    }
                }
            }
            Err(parse_errors) => {
                println!("  \x1b[31m\u{2717}\x1b[0m {} \x1b[31m— parse error\x1b[0m", filename);
                for e in &parse_errors {
                    println!("    {}", e);
                }
                failed += 1;
            }
        }
    }

    println!("");
    let total = passed + failed;
    if failed == 0 {
        println!("  \x1b[32m{} tests passed\x1b[0m", total);
    } else {
        println!("  {} tests: \x1b[32m{} passed\x1b[0m, \x1b[31m{} failed\x1b[0m", total, passed, failed);
    }

    if failed > 0 {
        Err(format!("{} test(s) failed", failed))
    } else {
        Ok(())
    }
}

/// Build/check the project.
pub fn build_project() -> Result<(), String> {
    // Find main file
    let main_file = if Path::new("src/main.ae").exists() {
        "src/main.ae"
    } else if Path::new("main.ae").exists() {
        "main.ae"
    } else {
        return Err("No main file found (expected src/main.ae or main.ae)".into());
    };

    let source = fs::read_to_string(main_file).map_err(|e| e.to_string())?;
    let mut scanner = crate::lexer::scanner::Scanner::new(&source, main_file.to_string());
    let tokens = scanner.scan_tokens();
    let mut parser = crate::parser::parser::Parser::new(tokens);

    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(errors) => {
            for e in &errors { eprintln!("{}", e); }
            return Err("Parse errors found".into());
        }
    };

    let strict = program.directives.iter().any(|d| d.name == "strict");
    let mut checker = crate::types::checker::TypeChecker::new(strict);
    let type_errors = checker.check_program(&program);

    if type_errors.is_empty() {
        println!("  Build successful: no errors");
    } else {
        let output = crate::diagnostics::format_errors(&type_errors, &source);
        eprint!("{}", output);
        return Err(format!("{} type error(s) found", type_errors.len()));
    }

    Ok(())
}

/// Format a source file (basic pretty-printer).
pub fn format_file(path: &str) -> Result<(), String> {
    let source = fs::read_to_string(path).map_err(|e| e.to_string())?;

    // Basic formatting: normalize indentation, trim trailing whitespace
    let mut formatted = String::new();
    let mut indent: usize = 0;

    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            formatted.push('\n');
            continue;
        }

        // Decrease indent for closing braces
        if trimmed.starts_with('}') {
            indent = indent.saturating_sub(1);
        }

        // Write indented line
        for _ in 0..indent {
            formatted.push_str("    ");
        }
        formatted.push_str(trimmed);
        formatted.push('\n');

        // Increase indent for opening braces
        if trimmed.ends_with('{') {
            indent += 1;
        }
    }

    fs::write(path, &formatted).map_err(|e| e.to_string())?;
    println!("  Formatted {}", path);
    Ok(())
}

fn dirs_packages() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    format!("{}/.aether/packages", home)
}

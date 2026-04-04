/// Project-level Python-to-Aether converter.
///
/// Handles converting entire Python project directories to Aether projects,
/// including directory scanning, file conversion, dependency mapping,
/// and report generation.
use std::fs;
use std::path::{Path, PathBuf};

use super::python;

/// Skip these directories when scanning a Python project.
const SKIP_DIRS: &[&str] = &[
    "__pycache__",
    ".git",
    ".svn",
    ".hg",
    ".tox",
    ".mypy_cache",
    ".pytest_cache",
    ".eggs",
    "egg-info",
    "node_modules",
    "venv",
    ".venv",
    "env",
    ".env",
    "dist",
    "build",
    ".idea",
    ".vscode",
];

/// Skip these file patterns.
const SKIP_EXTENSIONS: &[&str] = &[".pyc", ".pyo", ".pyd", ".so", ".egg"];

/// Result of converting an entire project.
pub struct ProjectReport {
    /// Files that were converted from Python to Aether.
    pub files_converted: Vec<FileReport>,
    /// Files that were copied as-is (non-Python).
    pub files_copied: Vec<String>,
    /// Files that were skipped.
    pub files_skipped: Vec<String>,
    /// Dependencies found in requirements.txt.
    pub dependencies: Vec<Dependency>,
    /// All TODO items across all files.
    pub all_todos: Vec<String>,
    /// Input directory.
    pub input_dir: String,
    /// Output directory.
    pub output_dir: String,
}

/// Report for a single converted file.
pub struct FileReport {
    /// Original Python file path (relative to project root).
    pub source_path: String,
    /// Output Aether file path (relative to output root).
    pub output_path: String,
    /// Number of transformations applied.
    pub transformations: usize,
    /// Number of TODOs generated.
    pub todo_count: usize,
}

/// A Python dependency found in requirements.txt.
pub struct Dependency {
    /// Package name.
    pub name: String,
    /// Version constraint (if any).
    pub version: String,
    /// How it maps to Aether.
    pub mapping: String,
}

impl ProjectReport {
    /// Generate a human-readable summary of the conversion.
    pub fn summary(&self) -> String {
        let mut s = String::new();
        s.push_str("Aether Project Conversion Complete\n");
        s.push_str("===================================\n\n");
        s.push_str(&format!("Input:  {}\n", self.input_dir));
        s.push_str(&format!("Output: {}\n\n", self.output_dir));
        s.push_str(&format!("Files converted: {}\n", self.files_converted.len()));
        s.push_str(&format!("Files copied:    {}\n", self.files_copied.len()));
        s.push_str(&format!("Files skipped:   {}\n", self.files_skipped.len()));

        let total_transforms: usize = self.files_converted.iter().map(|f| f.transformations).sum();
        let total_todos: usize = self.files_converted.iter().map(|f| f.todo_count).sum();
        s.push_str(&format!("Transformations: {}\n", total_transforms));
        s.push_str(&format!("TODOs:           {}\n", total_todos));

        if !self.dependencies.is_empty() {
            s.push_str(&format!("\nDependencies: {}\n", self.dependencies.len()));
            for dep in &self.dependencies {
                s.push_str(&format!("  {} {} -> {}\n", dep.name, dep.version, dep.mapping));
            }
        }

        if !self.all_todos.is_empty() {
            s.push_str("\nTODO items requiring manual attention:\n");
            for (i, todo) in self.all_todos.iter().enumerate().take(20) {
                s.push_str(&format!("  {}. {}\n", i + 1, todo));
            }
            if self.all_todos.len() > 20 {
                s.push_str(&format!("  ... and {} more (see CONVERSION_REPORT.md)\n",
                    self.all_todos.len() - 20));
            }
        }

        s
    }

    /// Generate the full CONVERSION_REPORT.md content.
    fn generate_report_md(&self) -> String {
        let mut s = String::new();
        s.push_str("# Aether Conversion Report\n\n");
        s.push_str(&format!("**Source:** `{}`\n", self.input_dir));
        s.push_str(&format!("**Output:** `{}`\n\n", self.output_dir));

        // Summary
        s.push_str("## Summary\n\n");
        s.push_str("| Metric | Count |\n");
        s.push_str("|--------|-------|\n");
        s.push_str(&format!("| Files converted | {} |\n", self.files_converted.len()));
        s.push_str(&format!("| Files copied | {} |\n", self.files_copied.len()));
        s.push_str(&format!("| Files skipped | {} |\n", self.files_skipped.len()));
        let total_transforms: usize = self.files_converted.iter().map(|f| f.transformations).sum();
        let total_todos: usize = self.files_converted.iter().map(|f| f.todo_count).sum();
        s.push_str(&format!("| Total transformations | {} |\n", total_transforms));
        s.push_str(&format!("| Total TODOs | {} |\n\n", total_todos));

        // Per-file table
        s.push_str("## Converted Files\n\n");
        s.push_str("| Source | Output | Transforms | TODOs |\n");
        s.push_str("|--------|--------|-----------|-------|\n");
        for f in &self.files_converted {
            s.push_str(&format!("| `{}` | `{}` | {} | {} |\n",
                f.source_path, f.output_path, f.transformations, f.todo_count));
        }
        s.push('\n');

        // Dependencies
        if !self.dependencies.is_empty() {
            s.push_str("## Dependencies\n\n");
            s.push_str("| Python Package | Version | Aether Mapping |\n");
            s.push_str("|---------------|---------|----------------|\n");
            for dep in &self.dependencies {
                s.push_str(&format!("| {} | {} | {} |\n",
                    dep.name, dep.version, dep.mapping));
            }
            s.push('\n');
        }

        // TODOs
        if !self.all_todos.is_empty() {
            s.push_str("## TODO Items\n\n");
            s.push_str("These items require manual attention:\n\n");
            for todo in &self.all_todos {
                s.push_str(&format!("- {}\n", todo));
            }
            s.push('\n');
        }

        // Next steps
        s.push_str("## Next Steps\n\n");
        s.push_str("1. Review all `TODO(convert)` comments in the generated `.ae` files\n");
        s.push_str("2. Fix any `with`-statement conversions (resource management)\n");
        s.push_str("3. Convert generator functions to iterators or lists\n");
        s.push_str("4. Add type annotations where the converter could not infer types\n");
        s.push_str("5. Replace Python bridge imports with native Aether equivalents where possible\n");
        s.push_str("6. Run `aether check` on each file to find type errors\n");
        s.push_str("7. Run `aether test` to verify functionality\n");

        s
    }
}

/// Convert an entire Python project directory to an Aether project.
///
/// Scans the input directory for Python files, converts them, copies non-Python
/// files, parses requirements.txt, and generates a conversion report.
pub fn convert_project(input_dir: &str, output_dir: &str) -> Result<ProjectReport, String> {
    let input_path = Path::new(input_dir);
    if !input_path.exists() {
        return Err(format!("Input directory '{}' does not exist", input_dir));
    }
    if !input_path.is_dir() {
        return Err(format!("'{}' is not a directory", input_dir));
    }

    // Create output directory
    let output_path = Path::new(output_dir);
    fs::create_dir_all(output_path)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    let mut report = ProjectReport {
        files_converted: Vec::new(),
        files_copied: Vec::new(),
        files_skipped: Vec::new(),
        dependencies: Vec::new(),
        all_todos: Vec::new(),
        input_dir: input_dir.to_string(),
        output_dir: output_dir.to_string(),
    };

    // Scan and process files
    scan_directory(input_path, input_path, output_path, &mut report)?;

    // Parse requirements.txt if present
    let req_path = input_path.join("requirements.txt");
    if req_path.exists() {
        report.dependencies = parse_requirements(&req_path)?;
    }
    // Also check for setup.py / pyproject.toml
    let pyproject_path = input_path.join("pyproject.toml");
    if pyproject_path.exists() {
        report.all_todos.push("pyproject.toml found — review for additional dependencies".to_string());
    }
    let setup_path = input_path.join("setup.py");
    if setup_path.exists() {
        report.all_todos.push("setup.py found — review for additional dependencies".to_string());
    }

    // Generate aether.toml
    generate_aether_toml(output_path, &report)?;

    // Generate CONVERSION_REPORT.md
    let report_content = report.generate_report_md();
    let report_path = output_path.join("CONVERSION_REPORT.md");
    fs::write(&report_path, &report_content)
        .map_err(|e| format!("Failed to write conversion report: {}", e))?;

    Ok(report)
}

/// Recursively scan a directory and process files.
fn scan_directory(
    dir: &Path,
    root: &Path,
    output_root: &Path,
    report: &mut ProjectReport,
) -> Result<(), String> {
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory '{}': {}", dir.display(), e))?;

    let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();

        // Get relative path from root
        let rel_path = path.strip_prefix(root).unwrap_or(&path);

        if path.is_dir() {
            // Skip excluded directories
            if SKIP_DIRS.iter().any(|d| file_name == *d || file_name.ends_with(".egg-info")) {
                report.files_skipped.push(rel_path.display().to_string());
                continue;
            }

            // Create corresponding output directory
            let out_dir = output_root.join(rel_path);
            fs::create_dir_all(&out_dir)
                .map_err(|e| format!("Failed to create directory '{}': {}", out_dir.display(), e))?;

            scan_directory(&path, root, output_root, report)?;
        } else {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

            // Skip binary/compiled files
            if SKIP_EXTENSIONS.iter().any(|e| file_name.ends_with(e)) {
                report.files_skipped.push(rel_path.display().to_string());
                continue;
            }

            if ext == "py" {
                // Convert Python file
                convert_py_file(&path, rel_path, root, output_root, report)?;
            } else if file_name == "requirements.txt" {
                // Already handled separately
                report.files_skipped.push(rel_path.display().to_string());
            } else {
                // Copy non-Python file as-is
                let out_path = output_root.join(rel_path);
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent).ok();
                }
                fs::copy(&path, &out_path)
                    .map_err(|e| format!("Failed to copy '{}': {}", path.display(), e))?;
                report.files_copied.push(rel_path.display().to_string());
            }
        }
    }

    Ok(())
}

/// Convert a single Python file and write the output.
fn convert_py_file(
    path: &Path,
    rel_path: &Path,
    _root: &Path,
    output_root: &Path,
    report: &mut ProjectReport,
) -> Result<(), String> {
    let source = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read '{}': {}", path.display(), e))?;

    let filename = rel_path.display().to_string();

    // Pre-process: rewrite internal imports to use Aether module paths
    // "from src.models.user import User" → "from models.user import User"
    // "from .auth import login" → "from routes.auth import login" (relative to current module)
    let source = rewrite_internal_imports(&source, rel_path);

    // Convert
    let result = python::convert_python_to_aether(&source, &filename);

    // Determine output filename
    let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");
    let output_rel = if file_stem == "__init__" {
        // __init__.py -> mod.ae
        rel_path.with_file_name("mod.ae")
    } else {
        rel_path.with_extension("ae")
    };

    let out_path = output_root.join(&output_rel);
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).ok();
    }

    fs::write(&out_path, &result.source)
        .map_err(|e| format!("Failed to write '{}': {}", out_path.display(), e))?;

    let todo_count = result.todos.len();
    report.all_todos.extend(result.todos);

    report.files_converted.push(FileReport {
        source_path: rel_path.display().to_string(),
        output_path: output_rel.display().to_string(),
        transformations: result.transformations,
        todo_count,
    });

    Ok(())
}

/// Rewrite Python internal imports to Aether module paths.
/// - `from src.models.user import User` → `from models.user import User`
/// - `from .auth import login` → direct `import auth` style
fn rewrite_internal_imports(source: &str, current_file: &Path) -> String {
    let mut lines: Vec<String> = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("from ") && trimmed.contains(" import ") {
            // Handle relative imports: from .module or from ..module
            if let Some(rest) = trimmed.strip_prefix("from .") {
                if let Some(idx) = rest.find(" import ") {
                    let rel_module = &rest[..idx];
                    let names = &rest[idx + 8..];
                    // Resolve relative to current file's directory
                    if let Some(parent) = current_file.parent() {
                        let parent_str = parent.to_str().unwrap_or("");
                        if rel_module.starts_with('.') {
                            // from ..module — go up two levels
                            if let Some(grandparent) = parent.parent() {
                                let gp_str = grandparent.to_str().unwrap_or("");
                                let mod_part = rel_module.trim_start_matches('.');
                                let full = if gp_str.is_empty() || gp_str == "." {
                                    mod_part.to_string()
                                } else {
                                    format!("{}.{}", gp_str.replace(['/', '\\'], "."), mod_part)
                                };
                                lines.push(format!("from {} import {}", full, names));
                                continue;
                            }
                        } else {
                            // from .module — same package
                            let full = if parent_str.is_empty() || parent_str == "." {
                                rel_module.to_string()
                            } else {
                                format!("{}.{}", parent_str.replace(['/', '\\'], "."), rel_module)
                            };
                            lines.push(format!("from {} import {}", full, names));
                            continue;
                        }
                    }
                }
            }

            // Strip common project root prefixes (src., app., lib.)
            let rewritten = trimmed
                .replace("from src.", "from ")
                .replace("from app.", "from ")
                .replace("from lib.", "from ");
            lines.push(rewritten);
            continue;
        }

        // Also handle `import src.X` → `import X`
        if trimmed.starts_with("import src.") {
            lines.push(trimmed.replace("import src.", "import "));
            continue;
        }

        lines.push(line.to_string());
    }
    lines.join("\n")
}

/// Parse a requirements.txt file into a list of dependencies.
fn parse_requirements(path: &Path) -> Result<Vec<Dependency>, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read requirements.txt: {}", e))?;

    let mut deps = Vec::new();

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') || line.starts_with('-') {
            continue;
        }

        // Parse package==version, package>=version, package~=version, package
        let (name, version) = if let Some(idx) = line.find("==") {
            (line[..idx].trim().to_string(), line[idx..].trim().to_string())
        } else if let Some(idx) = line.find(">=") {
            (line[..idx].trim().to_string(), line[idx..].trim().to_string())
        } else if let Some(idx) = line.find("<=") {
            (line[..idx].trim().to_string(), line[idx..].trim().to_string())
        } else if let Some(idx) = line.find("~=") {
            (line[..idx].trim().to_string(), line[idx..].trim().to_string())
        } else if let Some(idx) = line.find("!=") {
            (line[..idx].trim().to_string(), line[idx..].trim().to_string())
        } else {
            (line.to_string(), String::new())
        };

        // Strip extras like [security] from package name
        let name = if let Some(bracket) = name.find('[') {
            name[..bracket].to_string()
        } else {
            name
        };

        let mapping = match python::python_to_aether_import(&name) {
            python::ImportMapping::Builtin(s) => format!("builtin: {}", s),
            python::ImportMapping::Bridge(s) => format!("bridge: {}", s),
            python::ImportMapping::Skip(reason) => format!("skip: {}", reason),
        };

        deps.push(Dependency {
            name,
            version,
            mapping,
        });
    }

    Ok(deps)
}

/// Generate an aether.toml for the converted project.
fn generate_aether_toml(output_path: &Path, report: &ProjectReport) -> Result<(), String> {
    let project_name = Path::new(&report.input_dir)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("converted-project");

    let mut toml = String::new();
    toml.push_str("[project]\n");
    toml.push_str(&format!("name = \"{}\"\n", project_name));
    toml.push_str("version = \"0.1.0\"\n");
    toml.push_str("description = \"Converted from Python\"\n\n");

    // Separate bridge dependencies from builtins
    let bridge_deps: Vec<&Dependency> = report.dependencies.iter()
        .filter(|d| d.mapping.starts_with("bridge:"))
        .collect();

    if !bridge_deps.is_empty() {
        toml.push_str("[python-bridge]\n");
        for dep in &bridge_deps {
            let version = if dep.version.is_empty() {
                "\"*\"".to_string()
            } else {
                format!("\"{}\"", dep.version.trim_start_matches(&['=', '>', '<', '~', '!'][..]))
            };
            toml.push_str(&format!("{} = {}\n", dep.name, version));
        }
        toml.push('\n');
    }

    // Notes section
    toml.push_str("[notes]\n");
    toml.push_str("# This project was auto-converted from Python.\n");
    toml.push_str("# Review CONVERSION_REPORT.md for details.\n");

    let builtin_deps: Vec<&Dependency> = report.dependencies.iter()
        .filter(|d| d.mapping.starts_with("builtin:"))
        .collect();
    if !builtin_deps.is_empty() {
        toml.push_str("# The following Python packages map to Aether builtins:\n");
        for dep in &builtin_deps {
            toml.push_str(&format!("#   {} -> {}\n", dep.name, dep.mapping));
        }
    }

    let skip_deps: Vec<&Dependency> = report.dependencies.iter()
        .filter(|d| d.mapping.starts_with("skip:"))
        .collect();
    if !skip_deps.is_empty() {
        toml.push_str("# The following Python packages are not needed in Aether:\n");
        for dep in &skip_deps {
            toml.push_str(&format!("#   {} -> {}\n", dep.name, dep.mapping));
        }
    }

    let toml_path = output_path.join("aether.toml");
    fs::write(&toml_path, &toml)
        .map_err(|e| format!("Failed to write aether.toml: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_test_project(name: &str) -> (PathBuf, PathBuf) {
        let base = std::env::temp_dir().join(format!("aether_test_{}", name));
        let input = base.join("input");
        let output = base.join("output");
        // Clean up any previous run
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&input).unwrap();
        (input, output)
    }

    #[test]
    fn test_convert_simple_project() {
        let (input, output) = setup_test_project("simple");

        // Create a simple Python file
        fs::write(input.join("main.py"), "def hello():\n    print(\"hello\")\n\nhello()\n").unwrap();

        let report = convert_project(
            input.to_str().unwrap(),
            output.to_str().unwrap(),
        ).unwrap();

        assert_eq!(report.files_converted.len(), 1);
        assert!(output.join("main.ae").exists());
        assert!(output.join("aether.toml").exists());
        assert!(output.join("CONVERSION_REPORT.md").exists());

        let content = fs::read_to_string(output.join("main.ae")).unwrap();
        assert!(content.contains("def hello() {"));

        // Cleanup
        let _ = fs::remove_dir_all(input.parent().unwrap());
    }

    #[test]
    fn test_init_to_mod() {
        let (input, output) = setup_test_project("initmod");

        let pkg_dir = input.join("mypackage");
        fs::create_dir_all(&pkg_dir).unwrap();
        fs::write(pkg_dir.join("__init__.py"), "# package init\n").unwrap();
        fs::write(pkg_dir.join("utils.py"), "def helper():\n    pass\n").unwrap();

        let report = convert_project(
            input.to_str().unwrap(),
            output.to_str().unwrap(),
        ).unwrap();

        assert!(output.join("mypackage").join("mod.ae").exists());
        assert!(output.join("mypackage").join("utils.ae").exists());

        let _ = fs::remove_dir_all(input.parent().unwrap());
    }

    #[test]
    fn test_skip_pycache() {
        let (input, output) = setup_test_project("pycache");

        fs::write(input.join("main.py"), "x = 1\n").unwrap();
        let cache_dir = input.join("__pycache__");
        fs::create_dir_all(&cache_dir).unwrap();
        fs::write(cache_dir.join("main.cpython-310.pyc"), "binary").unwrap();

        let report = convert_project(
            input.to_str().unwrap(),
            output.to_str().unwrap(),
        ).unwrap();

        assert!(!output.join("__pycache__").exists());
        assert!(report.files_skipped.len() >= 1);

        let _ = fs::remove_dir_all(input.parent().unwrap());
    }

    #[test]
    fn test_requirements_parsing() {
        let (input, output) = setup_test_project("reqs");

        fs::write(input.join("main.py"), "import json\n").unwrap();
        fs::write(input.join("requirements.txt"),
            "requests==2.28.0\nnumpy>=1.21\nflask\n# comment\npytest\n").unwrap();

        let report = convert_project(
            input.to_str().unwrap(),
            output.to_str().unwrap(),
        ).unwrap();

        assert_eq!(report.dependencies.len(), 4);
        assert_eq!(report.dependencies[0].name, "requests");
        assert!(report.dependencies[0].mapping.contains("builtin"));

        // Check aether.toml was generated
        let toml_content = fs::read_to_string(output.join("aether.toml")).unwrap();
        assert!(toml_content.contains("[project]"));

        let _ = fs::remove_dir_all(input.parent().unwrap());
    }

    #[test]
    fn test_copy_non_python_files() {
        let (input, output) = setup_test_project("nonpy");

        fs::write(input.join("main.py"), "x = 1\n").unwrap();
        fs::write(input.join("config.yaml"), "key: value\n").unwrap();
        fs::write(input.join("README.md"), "# Project\n").unwrap();

        let report = convert_project(
            input.to_str().unwrap(),
            output.to_str().unwrap(),
        ).unwrap();

        assert!(output.join("config.yaml").exists());
        assert!(output.join("README.md").exists());
        assert_eq!(report.files_copied.len(), 2);

        let _ = fs::remove_dir_all(input.parent().unwrap());
    }

    #[test]
    fn test_project_report_summary() {
        let report = ProjectReport {
            files_converted: vec![FileReport {
                source_path: "main.py".to_string(),
                output_path: "main.ae".to_string(),
                transformations: 10,
                todo_count: 2,
            }],
            files_copied: vec!["config.yaml".to_string()],
            files_skipped: vec!["__pycache__".to_string()],
            dependencies: vec![],
            all_todos: vec!["fix generator".to_string()],
            input_dir: "/tmp/input".to_string(),
            output_dir: "/tmp/output".to_string(),
        };

        let summary = report.summary();
        assert!(summary.contains("Files converted: 1"));
        assert!(summary.contains("Files copied:    1"));
        assert!(summary.contains("Transformations: 10"));
    }
}

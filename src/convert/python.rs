/// Core Python-to-Aether single-file converter.
///
/// Uses pattern-based line-by-line transformations with indentation tracking
/// to convert Python source code into Aether syntax. Not a full Python parser;
/// handles ~80% of common patterns and leaves TODO comments for the rest.
use std::fs;

/// Result of converting a Python source file to Aether.
pub struct ConvertResult {
    /// The converted Aether source code.
    pub source: String,
    /// Number of transformations applied.
    pub transformations: usize,
    /// TODO items requiring manual attention.
    pub todos: Vec<String>,
    /// Warnings generated during conversion.
    pub warnings: Vec<String>,
}

/// Describes how a Python import maps to Aether.
pub enum ImportMapping {
    /// Aether has a built-in equivalent module.
    Builtin(String),
    /// Use the Python bridge to access this module.
    Bridge(String),
    /// Skip this import (functionality handled differently in Aether).
    Skip(String),
}

/// Map a Python module name to its Aether equivalent.
pub fn python_to_aether_import(python_module: &str) -> ImportMapping {
    match python_module {
        "json" => ImportMapping::Builtin("use json".to_string()),
        "os" => ImportMapping::Builtin("use fs".to_string()),
        "os.path" => ImportMapping::Builtin("use fs".to_string()),
        "sys" => ImportMapping::Builtin("use sys".to_string()),
        "math" => ImportMapping::Builtin("use math".to_string()),
        "random" => ImportMapping::Builtin("use math.random".to_string()),
        "time" => ImportMapping::Builtin("use time".to_string()),
        "datetime" => ImportMapping::Builtin("use time".to_string()),
        "re" => ImportMapping::Builtin("use regex".to_string()),
        "collections" => ImportMapping::Builtin("use collections".to_string()),
        "itertools" => ImportMapping::Builtin("use iter".to_string()),
        "functools" => ImportMapping::Builtin("use func".to_string()),
        "io" => ImportMapping::Builtin("use io".to_string()),
        "pathlib" => ImportMapping::Builtin("use fs".to_string()),
        "typing" => ImportMapping::Skip("typing handled natively in Aether".to_string()),
        "abc" => ImportMapping::Skip("interfaces handled natively in Aether".to_string()),
        "dataclasses" => ImportMapping::Skip("classes/structs handled natively in Aether".to_string()),
        "unittest" => ImportMapping::Skip("use @test decorator instead".to_string()),
        "pytest" => ImportMapping::Skip("use @test decorator instead".to_string()),
        "requests" => ImportMapping::Builtin("use net.http".to_string()),
        "http" => ImportMapping::Builtin("use net.http".to_string()),
        "urllib" => ImportMapping::Builtin("use net.http".to_string()),
        "asyncio" => ImportMapping::Skip("async/await is native in Aether".to_string()),
        "concurrent" => ImportMapping::Skip("parallel blocks are native in Aether".to_string()),
        "threading" => ImportMapping::Skip("parallel blocks are native in Aether".to_string()),
        "multiprocessing" => ImportMapping::Skip("parallel blocks are native in Aether".to_string()),
        "subprocess" => ImportMapping::Builtin("use process".to_string()),
        "csv" => ImportMapping::Builtin("use csv".to_string()),
        "sqlite3" => ImportMapping::Bridge("use python.sqlite3".to_string()),
        "numpy" | "np" => ImportMapping::Bridge("use python.numpy as np".to_string()),
        "pandas" | "pd" => ImportMapping::Bridge("use python.pandas as pd".to_string()),
        "torch" => ImportMapping::Bridge("use python.torch".to_string()),
        "tensorflow" | "tf" => ImportMapping::Bridge("use python.tensorflow as tf".to_string()),
        "sklearn" | "scikit-learn" => ImportMapping::Bridge("use python.sklearn".to_string()),
        "matplotlib" => ImportMapping::Bridge("use python.matplotlib".to_string()),
        "scipy" => ImportMapping::Bridge("use python.scipy".to_string()),
        "flask" => ImportMapping::Bridge("use python.flask".to_string()),
        "django" => ImportMapping::Bridge("use python.django".to_string()),
        "fastapi" => ImportMapping::Bridge("use python.fastapi".to_string()),
        "PIL" | "Pillow" => ImportMapping::Bridge("use python.PIL".to_string()),
        other => ImportMapping::Bridge(format!("use python.{}", other)),
    }
}

/// Read a Python file from disk and convert it to Aether source.
pub fn convert_file(path: &str) -> Result<String, String> {
    let source = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read '{}': {}", path, e))?;
    let filename = std::path::Path::new(path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string());
    let result = convert_python_to_aether(&source, &filename);
    Ok(result.source)
}

/// Convert Python source code to Aether source code.
///
/// Works line-by-line with state tracking for indentation-to-brace conversion.
/// Handles common Python patterns and leaves TODO comments for unsupported constructs.
pub fn convert_python_to_aether(source: &str, filename: &str) -> ConvertResult {
    let mut output_lines: Vec<String> = Vec::new();
    let mut transformations: usize = 0;
    let mut todos: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // Indentation tracking: stack of indent levels (in spaces).
    // Each entry represents a block opening.
    let mut indent_stack: Vec<usize> = vec![0];

    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    // Track if we are inside a multiline string (triple-quote)
    let mut in_triple_quote = false;
    let mut triple_quote_char = "\"\"\"";

    while i < lines.len() {
        let raw_line = lines[i];

        // Handle triple-quoted strings passthrough
        if in_triple_quote {
            if raw_line.contains(triple_quote_char) {
                in_triple_quote = false;
            }
            output_lines.push(raw_line.to_string());
            i += 1;
            continue;
        }

        // Check for triple-quote start
        let trimmed = raw_line.trim();
        if (trimmed.contains("\"\"\"") || trimmed.contains("'''")) && !trimmed.starts_with('#') {
            let dq_count = trimmed.matches("\"\"\"").count();
            let sq_count = trimmed.matches("'''").count();
            if dq_count == 1 {
                in_triple_quote = true;
                triple_quote_char = "\"\"\"";
                output_lines.push(raw_line.to_string());
                i += 1;
                continue;
            }
            if sq_count == 1 {
                in_triple_quote = true;
                triple_quote_char = "'''";
                output_lines.push(raw_line.to_string());
                i += 1;
                continue;
            }
            // If count is 2, it opens and closes on same line — passthrough
        }

        // Skip empty lines
        if trimmed.is_empty() {
            output_lines.push(String::new());
            i += 1;
            continue;
        }

        // Calculate current indentation (number of leading spaces)
        let current_indent = count_indent(raw_line);

        // Close blocks: if current indent is less than top of stack, pop and emit closing braces
        while indent_stack.len() > 1 && current_indent < *indent_stack.last().unwrap() {
            indent_stack.pop();
            let brace_indent = *indent_stack.last().unwrap();
            output_lines.push(format!("{}}}", " ".repeat(brace_indent)));
            transformations += 1;
        }

        // Now transform the line
        let (converted, did_transform, line_todos, line_warnings, opens_block) =
            transform_line(trimmed, current_indent, i + 1, filename);

        if did_transform {
            transformations += 1;
        }
        todos.extend(line_todos);
        warnings.extend(line_warnings);

        output_lines.push(format!("{}{}", " ".repeat(current_indent), converted));

        // If this line opens a block, push the expected indent level
        if opens_block {
            // Peek at next non-empty line to determine the indent level of the block body
            let next_indent = find_next_indent(&lines, i + 1);
            if next_indent > current_indent {
                indent_stack.push(next_indent);
            } else {
                // Empty block — close it immediately
                output_lines.push(format!("{}}}", " ".repeat(current_indent)));
            }
        }

        i += 1;
    }

    // Close any remaining open blocks
    while indent_stack.len() > 1 {
        indent_stack.pop();
        let brace_indent = *indent_stack.last().unwrap();
        output_lines.push(format!("{}}}", " ".repeat(brace_indent)));
        transformations += 1;
    }

    ConvertResult {
        source: output_lines.join("\n") + "\n",
        transformations,
        todos,
        warnings,
    }
}

/// Count leading spaces in a line.
fn count_indent(line: &str) -> usize {
    let mut count = 0;
    for ch in line.chars() {
        match ch {
            ' ' => count += 1,
            '\t' => count += 4,
            _ => break,
        }
    }
    count
}

/// Find the indent level of the next non-empty line.
fn find_next_indent(lines: &[&str], start: usize) -> usize {
    for line in lines.iter().skip(start) {
        let t = line.trim();
        if !t.is_empty() {
            return count_indent(line);
        }
    }
    0
}

/// Transform a single trimmed line of Python to Aether.
/// Returns: (converted_line, did_transform, todos, warnings, opens_block)
fn transform_line(
    trimmed: &str,
    _indent: usize,
    line_num: usize,
    filename: &str,
) -> (String, bool, Vec<String>, Vec<String>, bool) {
    let mut todos = Vec::new();
    let mut warnings = Vec::new();
    let mut transformed = false;
    let mut opens_block = false;

    // --- Comment lines ---
    if let Some(comment_text) = trimmed.strip_prefix('#') {
        // Special: shebang
        if trimmed.starts_with("#!") {
            return (format!("// {}", trimmed), true, todos, warnings, false);
        }
        return (format!("//{}", comment_text), true, todos, warnings, false);
    }

    // --- pass ---
    if trimmed == "pass" {
        return ("// pass".to_string(), true, todos, warnings, false);
    }

    // --- yield ---
    if trimmed.starts_with("yield") {
        todos.push(format!("{}:{}: generators not supported in Aether", filename, line_num));
        return (
            format!("// TODO(convert): generators not supported — {}", trimmed),
            true, todos, warnings, false,
        );
    }

    // --- with statement ---
    if trimmed.starts_with("with ") && trimmed.ends_with(':') {
        todos.push(format!("{}:{}: with-statement needs manual conversion", filename, line_num));
        return (
            format!("// TODO(convert): with-statement needs manual conversion — {}", trimmed),
            true, todos, warnings, true,
        );
    }

    // --- if __name__ == "__main__": ---
    if trimmed.contains("__name__") && trimmed.contains("__main__") {
        return ("// Main entry point".to_string(), true, todos, warnings, true);
    }

    // --- Decorators (pass through) ---
    if trimmed.starts_with('@') {
        return (trimmed.to_string(), false, todos, warnings, false);
    }

    // --- import / from...import ---
    if let Some(rest) = trimmed.strip_prefix("import ") {
        let result = convert_import(rest);
        return (result, true, todos, warnings, false);
    }
    if trimmed.starts_with("from ") {
        let result = convert_from_import(trimmed);
        return (result, true, todos, warnings, false);
    }

    // --- class definition ---
    if trimmed.starts_with("class ") && trimmed.ends_with(':') {
        let inner = &trimmed[6..trimmed.len()-1].trim();
        let result = convert_class_def(inner);
        return (result, true, todos, warnings, true);
    }

    // --- def / async def ---
    if (trimmed.starts_with("def ") || trimmed.starts_with("async def ")) && trimmed.ends_with(':') {
        let result = convert_function_def(trimmed);
        return (result, true, todos, warnings, true);
    }

    // --- if / elif / else ---
    if trimmed.starts_with("if ") && trimmed.ends_with(':') {
        let cond = &trimmed[3..trimmed.len()-1];
        let cond = convert_expression(cond);
        return (format!("if {} {{", cond), true, todos, warnings, true);
    }
    if trimmed.starts_with("elif ") && trimmed.ends_with(':') {
        let cond = &trimmed[5..trimmed.len()-1];
        let cond = convert_expression(cond);
        return (format!("}} else if {} {{", cond), true, todos, warnings, true);
    }
    if trimmed == "else:" {
        return ("} else {".to_string(), true, todos, warnings, true);
    }

    // --- for loop ---
    if trimmed.starts_with("for ") && trimmed.ends_with(':') {
        let body = &trimmed[4..trimmed.len()-1];
        let result = convert_for_loop(body);
        return (result, true, todos, warnings, true);
    }

    // --- while loop ---
    if trimmed.starts_with("while ") && trimmed.ends_with(':') {
        let cond = &trimmed[6..trimmed.len()-1];
        let cond = convert_expression(cond);
        return (format!("loop while {} {{", cond), true, todos, warnings, true);
    }

    // --- try / except / finally ---
    if trimmed == "try:" {
        return ("try {".to_string(), true, todos, warnings, true);
    }
    if trimmed.starts_with("except") && trimmed.ends_with(':') {
        let result = convert_except(trimmed);
        return (result, true, todos, warnings, true);
    }
    if trimmed == "finally:" {
        return ("} finally {".to_string(), true, todos, warnings, true);
    }

    // --- raise ---
    if let Some(rest) = trimmed.strip_prefix("raise ") {
        // raise Exception("msg") -> throw "msg"
        if let Some(msg) = extract_exception_message(rest) {
            return (format!("throw {}", msg), true, todos, warnings, false);
        }
        return (format!("throw {}", convert_expression(rest)), true, todos, warnings, false);
    }

    // --- assert ---
    if let Some(rest) = trimmed.strip_prefix("assert ") {
        let result = convert_assert(rest);
        return (result, true, todos, warnings, false);
    }

    // --- return ---
    if let Some(rest) = trimmed.strip_prefix("return ") {
        return (format!("return {}", convert_expression(rest)), true, todos, warnings, false);
    }
    if trimmed == "return" {
        return ("return".to_string(), false, todos, warnings, false);
    }

    // --- General expression/statement line ---
    let converted = convert_expression(trimmed);
    if converted != trimmed {
        transformed = true;
    }
    (converted, transformed, todos, warnings, opens_block)
}

/// Convert a Python `import module` or `import module as alias` statement.
fn convert_import(rest: &str) -> String {
    // import module as alias
    if let Some(idx) = rest.find(" as ") {
        let module = rest[..idx].trim();
        let alias = rest[idx+4..].trim();
        match python_to_aether_import(module) {
            ImportMapping::Builtin(s) => format!("{} as {}", s, alias),
            ImportMapping::Bridge(s) => {
                // Already has module name, add alias
                if s.contains(" as ") {
                    format!("use python.{} as {}", module, alias)
                } else {
                    format!("{} as {}", s, alias)
                }
            }
            ImportMapping::Skip(reason) => format!("// Skipped import {}: {}", module, reason),
        }
    } else {
        let module = rest.trim();
        match python_to_aether_import(module) {
            ImportMapping::Builtin(s) => s,
            ImportMapping::Bridge(s) => s,
            ImportMapping::Skip(reason) => format!("// Skipped import {}: {}", module, reason),
        }
    }
}

/// Convert a Python `from module import name` statement.
fn convert_from_import(line: &str) -> String {
    // from module import name1, name2
    // from module import name as alias
    let without_from = &line[5..];
    if let Some(idx) = without_from.find(" import ") {
        let module = without_from[..idx].trim();
        let names = without_from[idx+8..].trim();

        // Check for relative imports
        if module.starts_with('.') {
            let clean = module.trim_start_matches('.');
            return format!("use {}  // imports {}", clean, names);
        }

        // Check if this looks like a project-internal import (lowercase, has dots, not a known package)
        let first_part = module.split('.').next().unwrap_or(module);
        let is_likely_internal = first_part.chars().next().is_some_and(|c| c.is_lowercase())
            && module.contains('.')
            && !matches!(first_part, "os" | "sys" | "io" | "re" | "csv" | "http" | "json"
                | "math" | "time" | "datetime" | "pathlib" | "collections" | "itertools"
                | "functools" | "typing" | "abc" | "unittest" | "pytest" | "requests"
                | "asyncio" | "concurrent" | "threading" | "subprocess" | "numpy" | "np"
                | "pandas" | "pd" | "torch" | "tensorflow" | "sklearn" | "flask"
                | "django" | "fastapi" | "matplotlib" | "scipy" | "PIL" | "sqlite3");

        if is_likely_internal {
            return format!("use {}  // imports {}", module, names);
        }

        match python_to_aether_import(module) {
            ImportMapping::Builtin(s) => format!("{}  // imports {}", s, names),
            ImportMapping::Bridge(s) => format!("{}  // imports {}", s, names),
            ImportMapping::Skip(reason) => format!("// Skipped: {} — {}", line, reason),
        }
    } else {
        format!("// TODO(convert): unusual import — {}", line)
    }
}

/// Convert a Python class definition.
fn convert_class_def(inner: &str) -> String {
    // inner is the part between "class " and ":"
    if let Some(paren_start) = inner.find('(') {
        let name = &inner[..paren_start];
        let parents = &inner[paren_start+1..inner.len()-1]; // strip parens
        let parents = parents.trim();

        // Filter out metaclass=..., ABC, object
        let parent_list: Vec<&str> = parents.split(',')
            .map(|p| p.trim())
            .filter(|p| {
                !p.is_empty()
                    && !p.starts_with("metaclass")
                    && *p != "object"
                    && *p != "ABC"
            })
            .collect();

        if parent_list.is_empty() {
            format!("class {} {{", name)
        } else {
            format!("class {} : {} {{", name, parent_list.join(", "))
        }
    } else {
        format!("class {} {{", inner)
    }
}

/// Convert a Python function/method definition.
fn convert_function_def(trimmed: &str) -> String {
    let is_async = trimmed.starts_with("async ");
    let rest = if is_async {
        &trimmed[10..trimmed.len()-1] // skip "async def " ... ":"
    } else {
        &trimmed[4..trimmed.len()-1] // skip "def " ... ":"
    };

    // Split into name(params) -> optional return annotation
    let paren_start = match rest.find('(') {
        Some(i) => i,
        None => {
            let prefix = if is_async { "async " } else { "" };
            return format!("{}def {} {{", prefix, rest);
        }
    };
    let name = &rest[..paren_start];

    // Find matching close paren
    let after_name = &rest[paren_start..];
    let paren_end = find_matching_paren(after_name).unwrap_or(after_name.len() - 1);
    let params_str = &after_name[1..paren_end]; // between parens
    let after_params = &after_name[paren_end+1..].trim();

    // Handle return type annotation: -> Type
    let return_type = after_params.strip_prefix("->").map(|rt| convert_type_hint(rt.trim()));

    // Convert params: remove 'self', convert type hints
    let converted_params = convert_params(params_str);

    // Special method conversions
    let prefix = if is_async { "async " } else { "" };

    if name == "__init__" {
        // init constructor
        return format!("{}init({}) {{", prefix, converted_params);
    }

    if name == "__str__" || name == "__repr__" {
        return format!("{}def to_string({}) -> Str {{", prefix, converted_params);
    }

    if name == "__eq__" {
        return format!("{}operator ==({}) -> Bool {{", prefix, converted_params);
    }

    if name == "__lt__" {
        return format!("{}operator <({}) -> Bool {{", prefix, converted_params);
    }

    if name == "__add__" {
        return format!("{}operator +({}) -> Self {{", prefix, converted_params);
    }

    if name == "__len__" {
        return format!("{}def len({}) -> Int {{", prefix, converted_params);
    }

    if name == "__del__" {
        return format!("{}deinit {{", prefix);
    }

    match return_type {
        Some(rt) => format!("{}def {}({}) -> {} {{", prefix, name, converted_params, rt),
        None => format!("{}def {}({}) {{", prefix, name, converted_params),
    }
}

/// Convert Python parameter list, removing 'self' and converting type hints.
fn convert_params(params_str: &str) -> String {
    if params_str.trim().is_empty() {
        return String::new();
    }

    let params: Vec<&str> = split_params(params_str);
    let mut result = Vec::new();

    for param in params {
        let p = param.trim();
        if p == "self" || p == "cls" {
            continue;
        }
        // Remove 'self, ' prefix case
        if p.starts_with("self,") || p.starts_with("cls,") {
            continue;
        }

        // Handle default values: name: Type = default
        if let Some(eq_idx) = p.find('=') {
            let before_eq = p[..eq_idx].trim();
            let default_val = p[eq_idx+1..].trim();
            let default_val = convert_expression(default_val);

            if let Some(colon_idx) = before_eq.find(':') {
                let name = &before_eq[..colon_idx].trim();
                let type_hint = &before_eq[colon_idx+1..].trim();
                let aether_type = convert_type_hint(type_hint);
                result.push(format!("{}: {} = {}", name, aether_type, default_val));
            } else {
                result.push(format!("{} = {}", before_eq, default_val));
            }
        } else if let Some(colon_idx) = p.find(':') {
            let name = &p[..colon_idx].trim();
            let type_hint = &p[colon_idx+1..].trim();
            let aether_type = convert_type_hint(type_hint);
            result.push(format!("{}: {}", name, aether_type));
        } else {
            result.push(p.to_string());
        }
    }

    result.join(", ")
}

/// Split parameters by comma, respecting nested brackets/parens.
fn split_params(s: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut depth = 0;
    let mut start = 0;

    for (i, ch) in s.char_indices() {
        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                result.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < s.len() {
        result.push(&s[start..]);
    }
    result
}

/// Convert a Python type hint to Aether type.
fn convert_type_hint(hint: &str) -> String {
    let h = hint.trim();
    match h {
        "int" => "Int".to_string(),
        "float" => "Float".to_string(),
        "str" => "Str".to_string(),
        "bool" => "Bool".to_string(),
        "None" | "NoneType" => "nil".to_string(),
        "bytes" => "Byte[]".to_string(),
        "any" | "Any" => "any".to_string(),
        _ => {
            // Optional[X] -> X?
            if h.starts_with("Optional[") && h.ends_with(']') {
                let inner = &h[9..h.len()-1];
                return format!("{}?", convert_type_hint(inner));
            }
            // List[X] -> X[]
            if h.starts_with("List[") && h.ends_with(']') {
                let inner = &h[5..h.len()-1];
                return format!("{}[]", convert_type_hint(inner));
            }
            // list[X] -> X[]
            if h.starts_with("list[") && h.ends_with(']') {
                let inner = &h[5..h.len()-1];
                return format!("{}[]", convert_type_hint(inner));
            }
            // Dict[K, V] -> {K: V}
            if h.starts_with("Dict[") && h.ends_with(']') {
                let inner = &h[5..h.len()-1];
                let parts: Vec<&str> = split_params(inner);
                if parts.len() == 2 {
                    return format!("{{{}: {}}}", convert_type_hint(parts[0]), convert_type_hint(parts[1]));
                }
            }
            // dict[K, V] -> {K: V}
            if h.starts_with("dict[") && h.ends_with(']') {
                let inner = &h[5..h.len()-1];
                let parts: Vec<&str> = split_params(inner);
                if parts.len() == 2 {
                    return format!("{{{}: {}}}", convert_type_hint(parts[0]), convert_type_hint(parts[1]));
                }
            }
            // Tuple[X, Y] -> (X, Y)
            if h.starts_with("Tuple[") && h.ends_with(']') {
                let inner = &h[6..h.len()-1];
                let parts: Vec<&str> = split_params(inner);
                let converted: Vec<String> = parts.iter().map(|p| convert_type_hint(p)).collect();
                return format!("({})", converted.join(", "));
            }
            // Set[X] -> Set<X>
            if h.starts_with("Set[") && h.ends_with(']') {
                let inner = &h[4..h.len()-1];
                return format!("Set<{}>", convert_type_hint(inner));
            }
            // Union[X, Y] -> leave as-is with comment
            if h.starts_with("Union[") {
                return format!("any /* {} */", h);
            }
            // Callable -> leave as-is
            if h.starts_with("Callable") {
                return format!("/* {} */", h);
            }
            // Otherwise keep as-is (likely a custom class name)
            h.to_string()
        }
    }
}

/// Convert a Python for-loop body (between "for " and ":").
fn convert_for_loop(body: &str) -> String {
    // for x in range(n): -> for x in 0..n {
    // for x in range(a, b): -> for x in a..b {
    // for i, x in enumerate(lst): -> for i, x in lst {
    // for x in iterable: -> for x in iterable {

    if let Some(in_idx) = body.find(" in ") {
        let var_part = &body[..in_idx];
        let iter_part = body[in_idx+4..].trim();

        // range(n) or range(a, b) or range(a, b, step)
        if iter_part.starts_with("range(") && iter_part.ends_with(')') {
            let args = &iter_part[6..iter_part.len()-1];
            let parts: Vec<&str> = split_params(args);
            match parts.len() {
                1 => return format!("for {} in 0..{} {{", var_part, parts[0].trim()),
                2 => return format!("for {} in {}..{} {{", var_part, parts[0].trim(), parts[1].trim()),
                3 => return format!("for {} in {}..{} step {} {{", var_part, parts[0].trim(), parts[1].trim(), parts[2].trim()),
                _ => {}
            }
        }

        // enumerate(lst)
        if iter_part.starts_with("enumerate(") && iter_part.ends_with(')') {
            let inner = &iter_part[10..iter_part.len()-1];
            return format!("for {} in {} {{", var_part, convert_expression(inner));
        }

        // reversed(lst)
        if iter_part.starts_with("reversed(") && iter_part.ends_with(')') {
            let inner = &iter_part[9..iter_part.len()-1];
            return format!("for {} in {}.reversed() {{", var_part, convert_expression(inner));
        }

        // sorted(lst)
        if iter_part.starts_with("sorted(") && iter_part.ends_with(')') {
            let inner = &iter_part[7..iter_part.len()-1];
            return format!("for {} in {}.sorted() {{", var_part, convert_expression(inner));
        }

        // zip(a, b)
        if iter_part.starts_with("zip(") && iter_part.ends_with(')') {
            let inner = &iter_part[4..iter_part.len()-1];
            return format!("for {} in zip({}) {{", var_part, convert_expression(inner));
        }

        return format!("for {} in {} {{", var_part, convert_expression(iter_part));
    }

    format!("for {} {{", convert_expression(body))
}

/// Convert a Python except clause.
fn convert_except(trimmed: &str) -> String {
    // except: -> } catch any {
    if trimmed == "except:" {
        return "} catch any {".to_string();
    }
    // except Exception as e: -> } catch any as e {
    // except TypeError as e: -> } catch TypeError as e {
    let rest = &trimmed[6..]; // skip "except"
    let rest = rest.trim();
    let rest = rest.trim_end_matches(':');

    if let Some(as_idx) = rest.find(" as ") {
        let exc_type = rest[..as_idx].trim();
        let var_name = rest[as_idx+4..].trim();
        let exc_type = if exc_type == "Exception" || exc_type == "BaseException" {
            "any"
        } else {
            exc_type
        };
        format!("}} catch {} as {} {{", exc_type, var_name)
    } else {
        let exc_type = rest.trim();
        if exc_type.is_empty() || exc_type == "Exception" || exc_type == "BaseException" {
            "} catch any {".to_string()
        } else {
            format!("}} catch {} {{", exc_type)
        }
    }
}

/// Convert a Python assert statement.
fn convert_assert(rest: &str) -> String {
    // assert x == y -> assert_eq(x, y)
    // assert x != y -> assert(x != y)
    // assert x -> assert(x)
    let rest = rest.trim();

    // Check for assert with message: assert cond, "msg"
    // Split on top-level comma
    let parts: Vec<&str> = split_params(rest);
    let cond = parts[0].trim();

    if cond.contains(" == ") {
        if let Some(eq_idx) = cond.find(" == ") {
            let left = convert_expression(&cond[..eq_idx]);
            let right = convert_expression(&cond[eq_idx+4..]);
            return format!("assert_eq({}, {})", left, right);
        }
    }

    format!("assert({})", convert_expression(cond))
}

/// Convert a general Python expression to Aether.
fn convert_expression(expr: &str) -> String {
    let mut s = expr.to_string();

    // True/False/None
    s = replace_word(&s, "True", "true");
    s = replace_word(&s, "False", "false");
    s = replace_word(&s, "None", "nil");

    // is None -> == nil (already replaced None above)
    s = s.replace(" is nil", " == nil");
    s = s.replace(" is not nil", " != nil");

    // not x -> !x (only standalone 'not ', not 'not_' or 'cannot')
    s = replace_not(&s);

    // f-strings: f"..." -> "..."
    s = convert_fstrings(&s);

    // lambda x: expr -> x -> expr
    s = convert_lambdas(&s);

    // .append( -> .push(
    s = s.replace(".append(", ".push(");

    // len(x) -> x.len() — simple cases only
    s = convert_len_calls(&s);

    // print() stays the same

    // range(n) -> 0..n in expression context
    s = convert_range_expr(&s);

    // isinstance(x, Type) -> x is Type
    s = convert_isinstance(&s);

    // ** (power) -> ** (same in Aether)
    // // (integer division) -> / (with comment)
    // Note: Python // is floor division; leave as-is and add warning

    s
}

/// Replace a whole word only (not substrings).
fn replace_word(s: &str, word: &str, replacement: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let word_chars: Vec<char> = word.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if i + word_chars.len() <= chars.len()
            && &chars[i..i+word_chars.len()] == word_chars.as_slice()
        {
            // Check boundaries
            let before_ok = i == 0 || !chars[i-1].is_alphanumeric() && chars[i-1] != '_';
            let after_ok = i + word_chars.len() >= chars.len()
                || !chars[i+word_chars.len()].is_alphanumeric() && chars[i+word_chars.len()] != '_';

            if before_ok && after_ok {
                result.push_str(replacement);
                i += word_chars.len();
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

/// Convert Python `not x` to `!x`, being careful about word boundaries.
fn replace_not(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    let s_bytes = s.as_bytes();
    let mut i = 0;

    while i < s.len() {
        if i + 4 <= s.len() && &s[i..i+4] == "not " {
            // Check it's a word boundary before
            let before_ok = i == 0 || (!s_bytes[i-1].is_ascii_alphanumeric() && s_bytes[i-1] != b'_');
            if before_ok {
                result.push('!');
                i += 4; // skip "not "
                continue;
            }
        }
        result.push(s.as_bytes()[i] as char);
        i += 1;
    }
    result
}

/// Convert f-strings: f"text {expr}" -> "text {expr}"
fn convert_fstrings(s: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == 'f' && i + 1 < chars.len() && chars[i+1] == '"' {
            // Check word boundary before 'f'
            let before_ok = i == 0 || !chars[i-1].is_alphanumeric() && chars[i-1] != '_';
            if before_ok {
                // Skip the 'f', keep the rest
                i += 1;
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

/// Convert lambda expressions: lambda x: expr -> x -> expr
fn convert_lambdas(s: &str) -> String {
    let mut result = s.to_string();

    while let Some(idx) = result.find("lambda ") {
        // Check word boundary
        if idx > 0 {
            let prev = result.as_bytes()[idx - 1];
            if prev.is_ascii_alphanumeric() || prev == b'_' {
                break;
            }
        }

        let after = &result[idx + 7..]; // skip "lambda "
        // Find the colon that separates params from body
        // Need to handle nested structures
        if let Some(colon_idx) = find_lambda_colon(after) {
            let params = after[..colon_idx].trim();
            let body = after[colon_idx+1..].trim();

            let aether_lambda = if params.contains(',') {
                format!("({}) -> {}", params, body)
            } else {
                format!("{} -> {}", params, body)
            };

            result = format!("{}{}", &result[..idx], aether_lambda);
        } else {
            break;
        }
    }
    result
}

/// Find the colon in a lambda expression (not nested in brackets).
fn find_lambda_colon(s: &str) -> Option<usize> {
    let mut depth = 0;
    for (i, ch) in s.char_indices() {
        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => {
                if depth > 0 { depth -= 1; }
            }
            ':' if depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

/// Convert len(x) to x.len() for simple cases.
fn convert_len_calls(s: &str) -> String {
    let mut result = s.to_string();

    // Simple pattern: len(identifier)
    while let Some(idx) = result.find("len(") {
        // Check word boundary
        if idx > 0 {
            let prev = result.as_bytes()[idx - 1];
            if prev.is_ascii_alphanumeric() || prev == b'_' || prev == b'.' {
                // Part of another identifier like "strlen(" — skip
                break;
            }
        }
        let after = &result[idx + 4..];
        if let Some(close) = find_matching_paren_in(&format!("({}", after)) {
            let inner = &after[..close - 1];
            // Only do simple transforms (no nested function calls to avoid breaking things)
            if !inner.contains('(') || inner.contains('.') {
                let converted = format!("{}.len()", inner);
                result = format!("{}{}{}", &result[..idx], converted, &result[idx + 4 + close..]);
            } else {
                break;
            }
        } else {
            break;
        }
    }
    result
}

/// Convert range(n) expressions in non-for contexts.
fn convert_range_expr(s: &str) -> String {
    let mut result = s.to_string();

    while let Some(idx) = result.find("range(") {
        if idx > 0 {
            let prev = result.as_bytes()[idx - 1];
            if prev.is_ascii_alphanumeric() || prev == b'_' || prev == b'.' {
                break;
            }
        }
        let after = &result[idx + 6..];
        if let Some(close_idx) = find_matching_paren_in(&format!("({}", after)) {
            let args = &after[..close_idx - 1];
            let parts: Vec<&str> = split_params(args);
            let replacement = match parts.len() {
                1 => format!("0..{}", parts[0].trim()),
                2 => format!("{}..{}", parts[0].trim(), parts[1].trim()),
                3 => format!("{}..{} step {}", parts[0].trim(), parts[1].trim(), parts[2].trim()),
                _ => break,
            };
            result = format!("{}{}{}", &result[..idx], replacement, &result[idx + 6 + close_idx..]);
        } else {
            break;
        }
    }
    result
}

/// Convert isinstance(x, Type) to x is Type.
fn convert_isinstance(s: &str) -> String {
    let mut result = s.to_string();

    while let Some(idx) = result.find("isinstance(") {
        if idx > 0 {
            let prev = result.as_bytes()[idx - 1];
            if prev.is_ascii_alphanumeric() || prev == b'_' {
                break;
            }
        }
        let after = &result[idx + 11..];
        if let Some(close) = find_matching_paren_in(&format!("({}", after)) {
            let inner = &after[..close - 1];
            let parts: Vec<&str> = split_params(inner);
            if parts.len() == 2 {
                let obj = parts[0].trim();
                let typ = parts[1].trim();
                let replacement = format!("{} is {}", obj, typ);
                result = format!("{}{}{}", &result[..idx], replacement, &result[idx + 11 + close..]);
            } else {
                break;
            }
        } else {
            break;
        }
    }
    result
}

/// Find matching close paren in a string starting with '('.
fn find_matching_paren(s: &str) -> Option<usize> {
    let mut depth = 0;
    for (i, ch) in s.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Find matching close paren, returning position relative to after the opening paren.
fn find_matching_paren_in(s: &str) -> Option<usize> {
    find_matching_paren(s)
}

/// Extract the message from a raise Exception("msg") pattern.
fn extract_exception_message(s: &str) -> Option<String> {
    let s = s.trim();
    // Pattern: ExceptionType("message")
    if let Some(paren_start) = s.find('(') {
        if s.ends_with(')') {
            let msg = &s[paren_start+1..s.len()-1];
            return Some(msg.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_function() {
        let py = "def hello():\n    print(\"hello\")\n";
        let result = convert_python_to_aether(py, "test.py");
        assert!(result.source.contains("def hello() {"));
        assert!(result.source.contains("print(\"hello\")"));
        assert!(result.source.contains("}"));
    }

    #[test]
    fn test_class_conversion() {
        let py = "class Dog:\n    def __init__(self, name):\n        self.name = name\n    def bark(self):\n        print(self.name)\n";
        let result = convert_python_to_aether(py, "test.py");
        assert!(result.source.contains("class Dog {"));
        assert!(result.source.contains("init(name) {"));
        assert!(result.source.contains("def bark() {"));
    }

    #[test]
    fn test_class_inheritance() {
        let py = "class Admin(User):\n    pass\n";
        let result = convert_python_to_aether(py, "test.py");
        assert!(result.source.contains("class Admin : User {"));
    }

    #[test]
    fn test_if_elif_else() {
        let py = "if x > 0:\n    print(\"pos\")\nelif x < 0:\n    print(\"neg\")\nelse:\n    print(\"zero\")\n";
        let result = convert_python_to_aether(py, "test.py");
        assert!(result.source.contains("if x > 0 {"));
        assert!(result.source.contains("} else if x < 0 {"));
        assert!(result.source.contains("} else {"));
    }

    #[test]
    fn test_for_range() {
        let py = "for i in range(10):\n    print(i)\n";
        let result = convert_python_to_aether(py, "test.py");
        assert!(result.source.contains("for i in 0..10 {"));
    }

    #[test]
    fn test_while_loop() {
        let py = "while x > 0:\n    x = x - 1\n";
        let result = convert_python_to_aether(py, "test.py");
        assert!(result.source.contains("loop while x > 0 {"));
    }

    #[test]
    fn test_true_false_none() {
        let py = "x = True\ny = False\nz = None\n";
        let result = convert_python_to_aether(py, "test.py");
        assert!(result.source.contains("x = true"));
        assert!(result.source.contains("y = false"));
        assert!(result.source.contains("z = nil"));
    }

    #[test]
    fn test_fstring() {
        let py = "print(f\"hello {name}\")\n";
        let result = convert_python_to_aether(py, "test.py");
        assert!(result.source.contains("print(\"hello {name}\")"));
    }

    #[test]
    fn test_import() {
        let py = "import json\nimport numpy as np\n";
        let result = convert_python_to_aether(py, "test.py");
        assert!(result.source.contains("use json"));
        assert!(result.source.contains("use python.numpy as np"));
    }

    #[test]
    fn test_lambda() {
        let py = "f = lambda x: x * 2\n";
        let result = convert_python_to_aether(py, "test.py");
        assert!(result.source.contains("x -> x * 2"));
    }

    #[test]
    fn test_try_except() {
        let py = "try:\n    risky()\nexcept Exception as e:\n    handle(e)\nfinally:\n    cleanup()\n";
        let result = convert_python_to_aether(py, "test.py");
        assert!(result.source.contains("try {"));
        assert!(result.source.contains("} catch any as e {"));
        assert!(result.source.contains("} finally {"));
    }

    #[test]
    fn test_raise() {
        let py = "raise ValueError(\"bad value\")\n";
        let result = convert_python_to_aether(py, "test.py");
        assert!(result.source.contains("throw \"bad value\""));
    }

    #[test]
    fn test_comment() {
        let py = "# This is a comment\n";
        let result = convert_python_to_aether(py, "test.py");
        assert!(result.source.contains("// This is a comment"));
    }

    #[test]
    fn test_pass() {
        let py = "pass\n";
        let result = convert_python_to_aether(py, "test.py");
        assert!(result.source.contains("// pass"));
    }

    #[test]
    fn test_assert_eq() {
        let py = "assert x == 5\n";
        let result = convert_python_to_aether(py, "test.py");
        assert!(result.source.contains("assert_eq(x, 5)"));
    }

    #[test]
    fn test_append_to_push() {
        let py = "items.append(42)\n";
        let result = convert_python_to_aether(py, "test.py");
        assert!(result.source.contains("items.push(42)"));
    }

    #[test]
    fn test_is_none() {
        let py = "if x is None:\n    pass\n";
        let result = convert_python_to_aether(py, "test.py");
        assert!(result.source.contains("if x == nil {"));
    }

    #[test]
    fn test_type_hints() {
        assert_eq!(convert_type_hint("int"), "Int");
        assert_eq!(convert_type_hint("str"), "Str");
        assert_eq!(convert_type_hint("float"), "Float");
        assert_eq!(convert_type_hint("bool"), "Bool");
        assert_eq!(convert_type_hint("List[int]"), "Int[]");
        assert_eq!(convert_type_hint("Optional[str]"), "Str?");
        assert_eq!(convert_type_hint("Dict[str, int]"), "{Str: Int}");
    }

    #[test]
    fn test_async_function() {
        let py = "async def fetch(url):\n    data = await get(url)\n    return data\n";
        let result = convert_python_to_aether(py, "test.py");
        assert!(result.source.contains("async def fetch(url) {"));
        assert!(result.source.contains("await"));
    }

    #[test]
    fn test_yield_todo() {
        let py = "yield x\n";
        let result = convert_python_to_aether(py, "test.py");
        assert!(result.source.contains("TODO(convert): generators not supported"));
        assert!(!result.todos.is_empty());
    }

    #[test]
    fn test_with_statement_todo() {
        let py = "with open('file.txt') as f:\n    data = f.read()\n";
        let result = convert_python_to_aether(py, "test.py");
        assert!(result.source.contains("TODO(convert): with-statement needs manual conversion"));
    }

    #[test]
    fn test_dunder_name_main() {
        let py = "if __name__ == \"__main__\":\n    main()\n";
        let result = convert_python_to_aether(py, "test.py");
        assert!(result.source.contains("// Main entry point"));
        assert!(result.source.contains("main()"));
    }

    #[test]
    fn test_import_mapping() {
        match python_to_aether_import("json") {
            ImportMapping::Builtin(s) => assert_eq!(s, "use json"),
            _ => panic!("json should be builtin"),
        }
        match python_to_aether_import("numpy") {
            ImportMapping::Bridge(s) => assert!(s.contains("python.numpy")),
            _ => panic!("numpy should be bridge"),
        }
        match python_to_aether_import("typing") {
            ImportMapping::Skip(_) => {},
            _ => panic!("typing should be skip"),
        }
    }

    #[test]
    fn test_for_enumerate() {
        let py = "for i, item in enumerate(items):\n    print(i)\n";
        let result = convert_python_to_aether(py, "test.py");
        assert!(result.source.contains("for i, item in items {"));
    }

    #[test]
    fn test_method_self_removal() {
        let py = "def greet(self, name):\n    print(name)\n";
        let result = convert_python_to_aether(py, "test.py");
        assert!(result.source.contains("def greet(name) {"));
    }

    #[test]
    fn test_isinstance_conversion() {
        let result = convert_isinstance("isinstance(x, int)");
        assert_eq!(result, "x is int");
    }

    #[test]
    fn test_not_conversion() {
        let result = replace_not("not x");
        assert_eq!(result, "!x");
    }

    #[test]
    fn test_typed_params() {
        let py = "def add(a: int, b: int) -> int:\n    return a + b\n";
        let result = convert_python_to_aether(py, "test.py");
        assert!(result.source.contains("def add(a: Int, b: Int) -> Int {"));
    }
}

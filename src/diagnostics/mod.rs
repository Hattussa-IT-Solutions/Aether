use crate::types::checker::TypeError;

/// Format a type error with source context for display.
pub fn format_error(error: &TypeError, source: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let mut output = String::new();

    // Header
    output.push_str(&format!(
        "\x1b[1;31merror\x1b[0m: {}\n", error.message
    ));

    // Location
    output.push_str(&format!(
        "  \x1b[1;34m-->\x1b[0m {}:{}:{}\n", error.file, error.line, error.column
    ));

    // Source line
    if error.line > 0 && error.line <= lines.len() {
        let line = lines[error.line - 1];
        let line_num = format!("{}", error.line);
        let padding = " ".repeat(line_num.len());

        output.push_str(&format!("  {} \x1b[1;34m|\x1b[0m\n", padding));
        output.push_str(&format!("  \x1b[1;34m{} |\x1b[0m {}\n", line_num, line));

        // Underline
        let col = error.column.saturating_sub(1);
        let underline = " ".repeat(col) + "^^^";
        output.push_str(&format!("  {} \x1b[1;34m|\x1b[0m \x1b[1;31m{}\x1b[0m\n", padding, underline));
    }

    output
}

/// Format multiple errors.
pub fn format_errors(errors: &[TypeError], source: &str) -> String {
    errors.iter().map(|e| format_error(e, source)).collect::<Vec<_>>().join("\n")
}

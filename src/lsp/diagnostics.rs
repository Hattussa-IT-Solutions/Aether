use lsp_types::*;

/// Run the Aether parser and type checker on source code, return LSP diagnostics.
pub fn get_diagnostics(uri: &Url, source: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let filename = uri.path().to_string();

    // Parse
    let mut scanner = crate::lexer::scanner::Scanner::new(source, filename.clone());
    let tokens = scanner.scan_tokens();
    let mut parser = crate::parser::parser::Parser::new(tokens);

    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(errors) => {
            for e in errors {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position { line: e.span.line.saturating_sub(1) as u32, character: e.span.column.saturating_sub(1) as u32 },
                        end: Position { line: e.span.line.saturating_sub(1) as u32, character: (e.span.column + 10) as u32 },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    source: Some("aether".to_string()),
                    message: e.message,
                    ..Default::default()
                });
            }
            return diagnostics;
        }
    };

    // Type check
    let strict = program.directives.iter().any(|d| d.name == "strict");
    let mut checker = crate::types::checker::TypeChecker::new(strict);
    let type_errors = checker.check_program(&program);

    for e in type_errors {
        diagnostics.push(Diagnostic {
            range: Range {
                start: Position { line: e.line.saturating_sub(1) as u32, character: e.column.saturating_sub(1) as u32 },
                end: Position { line: e.line.saturating_sub(1) as u32, character: (e.column + 10) as u32 },
            },
            severity: Some(if strict { DiagnosticSeverity::ERROR } else { DiagnosticSeverity::WARNING }),
            source: Some("aether".to_string()),
            message: e.message,
            ..Default::default()
        });
    }

    diagnostics
}

/// Extract symbol information from parsed source for completions/hover.
pub fn extract_symbols(source: &str) -> Vec<(String, String, String)> {
    let mut symbols = Vec::new();
    let filename = "analysis".to_string();

    let mut scanner = crate::lexer::scanner::Scanner::new(source, filename);
    let tokens = scanner.scan_tokens();
    let mut parser = crate::parser::parser::Parser::new(tokens);

    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(_) => return symbols,
    };

    for stmt in &program.statements {
        extract_stmt_symbols(stmt, &mut symbols);
    }

    symbols
}

fn extract_stmt_symbols(stmt: &crate::parser::ast::Stmt, symbols: &mut Vec<(String, String, String)>) {
    use crate::parser::ast::StmtKind;

    match &stmt.kind {
        StmtKind::VarDecl { name, type_ann, .. } => {
            let type_str = type_ann.as_ref().map(|t| format!("{:?}", t)).unwrap_or_else(|| "auto".to_string());
            symbols.push((name.clone(), "variable".to_string(), type_str));
        }
        StmtKind::FuncDef(fd) => {
            let params: Vec<String> = fd.params.iter().map(|p| {
                if let Some(t) = &p.type_ann {
                    format!("{}: {:?}", p.name, t)
                } else {
                    p.name.clone()
                }
            }).collect();
            let ret = fd.return_type.as_ref().map(|t| format!(" -> {:?}", t)).unwrap_or_default();
            symbols.push((fd.name.clone(), "function".to_string(), format!("def {}({}){}", fd.name, params.join(", "), ret)));

            // Add parameters as local variables
            for p in &fd.params {
                symbols.push((p.name.clone(), "variable".to_string(), "parameter".to_string()));
            }
        }
        StmtKind::ClassDef(cd) => {
            symbols.push((cd.name.clone(), "class".to_string(), format!("class {}", cd.name)));
            for field in &cd.fields {
                symbols.push((field.name.clone(), "field".to_string(), format!("{}.{}", cd.name, field.name)));
            }
            for method in &cd.methods {
                symbols.push((method.name.clone(), "function".to_string(), format!("{}.{}()", cd.name, method.name)));
            }
        }
        StmtKind::StructDef(sd) => {
            symbols.push((sd.name.clone(), "class".to_string(), format!("struct {}", sd.name)));
        }
        StmtKind::EnumDef(ed) => {
            symbols.push((ed.name.clone(), "class".to_string(), format!("enum {}", ed.name)));
        }
        _ => {}
    }
}

use lsp_types::*;

/// Find the definition location of the symbol at the cursor.
pub fn get_definition(
    source: &str,
    uri: &Url,
    line: u32,
    character: u32,
) -> Option<GotoDefinitionResponse> {
    let lines: Vec<&str> = source.lines().collect();
    let current_line = lines.get(line as usize)?;

    // Extract word at cursor
    let word = extract_word(current_line, character as usize)?;

    // Search source for definition
    for (i, src_line) in lines.iter().enumerate() {
        let trimmed = src_line.trim();

        // Function definition: def name(
        if trimmed.starts_with("def ") {
            let after_def = &trimmed[4..];
            if let Some(paren) = after_def.find('(') {
                let fname = after_def[..paren].trim();
                if fname == word {
                    return Some(GotoDefinitionResponse::Scalar(Location {
                        uri: uri.clone(),
                        range: Range {
                            start: Position { line: i as u32, character: 4 },
                            end: Position { line: i as u32, character: (4 + fname.len()) as u32 },
                        },
                    }));
                }
            }
        }

        // Class definition: class Name
        if trimmed.starts_with("class ") {
            let after_class = &trimmed[6..];
            let cname = after_class.split_whitespace().next().unwrap_or("");
            if cname == word {
                return Some(GotoDefinitionResponse::Scalar(Location {
                    uri: uri.clone(),
                    range: Range {
                        start: Position { line: i as u32, character: 6 },
                        end: Position { line: i as u32, character: (6 + cname.len()) as u32 },
                    },
                }));
            }
        }

        // Struct definition
        if trimmed.starts_with("struct ") {
            let name = trimmed[7..].split_whitespace().next().unwrap_or("");
            if name == word {
                return Some(GotoDefinitionResponse::Scalar(Location {
                    uri: uri.clone(),
                    range: Range {
                        start: Position { line: i as u32, character: 7 },
                        end: Position { line: i as u32, character: (7 + name.len()) as u32 },
                    },
                }));
            }
        }

        // Enum definition
        if trimmed.starts_with("enum ") {
            let name = trimmed[5..].split_whitespace().next().unwrap_or("");
            if name == word {
                return Some(GotoDefinitionResponse::Scalar(Location {
                    uri: uri.clone(),
                    range: Range {
                        start: Position { line: i as u32, character: 5 },
                        end: Position { line: i as u32, character: (5 + name.len()) as u32 },
                    },
                }));
            }
        }

        // Variable assignment: name = ...
        if !trimmed.starts_with("//") {
            if let Some(eq_pos) = trimmed.find(" = ") {
                let vname = trimmed[..eq_pos].trim();
                // Skip if it has dots (field access) or other operators
                if vname == word && !vname.contains('.') && !vname.contains('(') {
                    let col = src_line.find(vname).unwrap_or(0);
                    return Some(GotoDefinitionResponse::Scalar(Location {
                        uri: uri.clone(),
                        range: Range {
                            start: Position { line: i as u32, character: col as u32 },
                            end: Position { line: i as u32, character: (col + vname.len()) as u32 },
                        },
                    }));
                }
            }
        }
    }

    None
}

fn extract_word(line: &str, col: usize) -> Option<String> {
    if col > line.len() { return None; }
    let bytes = line.as_bytes();
    let mut start = col;
    let mut end = col;
    while start > 0 && (bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_') {
        start -= 1;
    }
    while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
        end += 1;
    }
    if start == end { return None; }
    Some(line[start..end].to_string())
}

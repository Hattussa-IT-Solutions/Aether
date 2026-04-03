use lsp_types::*;

/// Generate hover information for the word at the given position.
pub fn get_hover(
    source: &str,
    line: u32,
    character: u32,
    symbols: &[(String, String, String)], // (name, kind, detail)
) -> Option<Hover> {
    let lines: Vec<&str> = source.lines().collect();
    let current_line = lines.get(line as usize)?;

    // Extract the word at the cursor position
    let word = extract_word_at(current_line, character as usize)?;

    // Check built-in types
    let type_info = match word {
        "Int" => Some("**Int** — 64-bit signed integer\n\nLiteral: `42`, `1_000`, `0xFF`, `0b1010`"),
        "Float" => Some("**Float** — 64-bit IEEE 754 floating point\n\nLiteral: `3.14`, `1.0e10`"),
        "Str" => Some("**Str** — UTF-8 immutable string\n\nLiteral: `\"hello\"`, `\"Hello {name}\"`\n\nMethods: `len()`, `upper()`, `lower()`, `trim()`, `split()`, `contains()`, `replace()`"),
        "Bool" => Some("**Bool** — Boolean value\n\nLiterals: `true`, `false`"),
        "Nil" | "nil" => Some("**nil** — Absence of value"),
        "List" => Some("**List** — Dynamic array\n\nLiteral: `[1, 2, 3]`\n\nMethods: `push()`, `pop()`, `map()`, `filter()`, `reduce()`, `sort()`, `sum()`"),
        "Map" => Some("**Map** — Key-value dictionary\n\nLiteral: `{\"key\": value}`\n\nMethods: `get()`, `set()`, `keys()`, `values()`, `entries()`"),
        "Set" => Some("**Set** — Unique value collection\n\nLiteral: `{1, 2, 3}`\n\nMethods: `insert()`, `remove()`, `contains()`, `union()`, `intersect()`"),
        "Result" => Some("**Result<T, E>** — Success or error value\n\nConstructors: `Ok(value)`, `Err(message)`\n\nUse `?` to propagate errors"),
        _ => None,
    };

    if let Some(info) = type_info {
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: info.to_string(),
            }),
            range: None,
        });
    }

    // Check keywords
    let keyword_info = match word {
        "def" => Some("**def** — Define a function\n\n```aether\ndef name(params) -> ReturnType {\n    body\n}\n```"),
        "let" => Some("**let** — Declare an immutable variable\n\n```aether\nlet x = 42  // Cannot be reassigned\n```"),
        "const" => Some("**const** — Compile-time constant\n\n```aether\nconst MAX = 100\n```"),
        "class" => Some("**class** — Define a reference-type class\n\n```aether\nclass Name {\n    field: Type\n    init(params) { }\n    def method() { }\n}\n```"),
        "struct" => Some("**struct** — Define a value-type struct (copy on assign)\n\n```aether\nstruct Point {\n    x: Float\n    y: Float\n}\n```"),
        "enum" => Some("**enum** — Algebraic data type with variants\n\n```aether\nenum Shape {\n    Circle(radius: Float)\n    Rect(width: Float, height: Float)\n}\n```"),
        "match" => Some("**match** — Pattern matching expression\n\n```aether\nmatch value {\n    pattern -> result\n    _ -> default\n}\n```"),
        "parallel" => Some("**parallel** — Execute tasks concurrently\n\n```aether\nparallel {\n    a = task1()\n    b = task2()\n}\n```"),
        "genetic" => Some("**genetic class** — Class with evolvable genes\n\n```aether\ngenetic class Strategy {\n    chromosome params {\n        gene x: Float = 0.5 { range 0.0..1.0 }\n    }\n    fitness(data) -> Float { ... }\n}\n```"),
        "evolve" => Some("**evolve** — Run genetic evolution\n\n```aether\nbest = evolve Strategy {\n    population: 50\n    generations: 100\n    fitness on data: my_data\n}\n```"),
        "reactive" => Some("**reactive** — Property that auto-recomputes when dependencies change\n\n```aether\nreactive total: Float = items.sum(p -> p.price)\n```"),
        "temporal" => Some("**temporal** — Property with automatic history tracking\n\n```aether\ntemporal(keep: 100) temperature: Float = 20.0\n```"),
        _ => None,
    };

    if let Some(info) = keyword_info {
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: info.to_string(),
            }),
            range: None,
        });
    }

    // Check user-defined symbols
    for (name, kind, detail) in symbols {
        if name == word {
            let md = format!("**{}** — {}\n\n{}", name, kind, detail);
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: md,
                }),
                range: None,
            });
        }
    }

    // Check built-in functions
    let builtin_info = match word {
        "print" => Some("**print(args...)** — Print values to stdout, separated by spaces"),
        "len" => Some("**len(val)** — Get the length of a string, list, map, set, or tuple"),
        "sqrt" => Some("**sqrt(x: Float) -> Float** — Square root"),
        "abs" => Some("**abs(x: Num) -> Num** — Absolute value"),
        _ => None,
    };

    if let Some(info) = builtin_info {
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: info.to_string(),
            }),
            range: None,
        });
    }

    None
}

fn extract_word_at(line: &str, col: usize) -> Option<&str> {
    if col > line.len() { return None; }

    let bytes = line.as_bytes();
    let mut start = col;
    let mut end = col;

    // Walk backwards to find word start
    while start > 0 && (bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_') {
        start -= 1;
    }
    // Walk forwards to find word end
    while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
        end += 1;
    }

    if start == end { return None; }
    Some(&line[start..end])
}

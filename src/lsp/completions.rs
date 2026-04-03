use lsp_types::*;

/// All Aether keywords for completion.
const KEYWORDS: &[&str] = &[
    "def", "let", "const", "if", "else", "match", "guard", "for", "in",
    "loop", "times", "while", "until", "step", "break", "next", "return",
    "class", "struct", "enum", "interface", "impl", "self", "super",
    "init", "deinit", "pub", "priv", "prot", "readonly", "static", "lazy",
    "async", "await", "try", "catch", "finally", "throw", "use", "mod",
    "as", "type", "parallel", "after", "device", "reactive", "temporal",
    "mutation", "evolving", "genetic", "gene", "chromosome", "fitness",
    "crossover", "breed", "evolve", "weave", "bond", "face", "extend",
    "delegate", "select", "exclude", "morph", "true", "false", "nil",
    "and", "or", "not", "then", "operator", "override", "where", "with",
];

/// Built-in type names.
const TYPES: &[&str] = &[
    "Int", "Float", "Num", "Bool", "Str", "Char", "Byte", "Nil",
    "Int8", "Int16", "Int32", "Int64", "UInt8", "UInt16", "UInt32", "UInt64",
    "Float32", "Float64", "List", "Map", "Set", "Result", "Tensor", "Self",
];

/// Built-in functions.
const BUILTINS: &[(&str, &str)] = &[
    ("print", "print(args...) — Print values to stdout"),
    ("input", "input(prompt) — Read line from stdin"),
    ("len", "len(val) — Get length of collection"),
    ("type", "type(val) — Get type name as string"),
    ("str", "str(val) — Convert to string"),
    ("int", "int(val) — Convert to integer"),
    ("float", "float(val) — Convert to float"),
    ("sqrt", "sqrt(x) — Square root"),
    ("abs", "abs(x) — Absolute value"),
    ("sin", "sin(x) — Sine"),
    ("cos", "cos(x) — Cosine"),
    ("tan", "tan(x) — Tangent"),
    ("log", "log(x) — Natural logarithm"),
    ("pow", "pow(base, exp) — Power"),
    ("min", "min(a, b) — Minimum"),
    ("max", "max(a, b) — Maximum"),
    ("round", "round(x) — Round to nearest integer"),
    ("floor", "floor(x) — Round down"),
    ("ceil", "ceil(x) — Round up"),
    ("random", "random() — Random float 0..1"),
    ("range", "range(start, end) — Create range"),
    ("fs_read", "fs_read(path) — Read file to string"),
    ("fs_write", "fs_write(path, content) — Write string to file"),
    ("fs_exists", "fs_exists(path) — Check if file exists"),
    ("time_now", "time_now() — Current time as float seconds"),
    ("time_sleep", "time_sleep(seconds) — Sleep"),
    ("json_encode", "json_encode(val) — Encode value to JSON string"),
    ("json_decode", "json_decode(str) — Decode JSON string to value"),
    ("crossover", "crossover(a, b) — Genetic crossover"),
    ("breed", "breed(a, b, rate) — Crossover with mutation"),
    ("Tensor_zeros", "Tensor_zeros(shape) — Create zero tensor"),
    ("Tensor_ones", "Tensor_ones(shape) — Create ones tensor"),
    ("Tensor_random", "Tensor_random(shape) — Create random tensor"),
    ("tensor_matmul", "tensor_matmul(a, b) — Matrix multiply"),
];

/// String methods for autocomplete after `.`
const STR_METHODS: &[(&str, &str)] = &[
    ("len()", "Length of string"),
    ("upper()", "Convert to uppercase"),
    ("lower()", "Convert to lowercase"),
    ("trim()", "Remove leading/trailing whitespace"),
    ("split(sep)", "Split by separator"),
    ("contains(sub)", "Check if contains substring"),
    ("starts_with(s)", "Check prefix"),
    ("ends_with(s)", "Check suffix"),
    ("replace(old, new)", "Replace occurrences"),
    ("slice(start, end)", "Substring"),
    ("chars()", "List of characters"),
    ("repeat(n)", "Repeat n times"),
    ("parse_int()", "Parse to integer"),
    ("parse_float()", "Parse to float"),
];

/// List methods for autocomplete after `.`
const LIST_METHODS: &[(&str, &str)] = &[
    ("len()", "Number of elements"),
    ("push(item)", "Add item to end"),
    ("pop()", "Remove and return last item"),
    ("first()", "First element"),
    ("last()", "Last element"),
    ("map(fn)", "Transform each element"),
    ("filter(fn)", "Keep elements matching predicate"),
    ("reduce(init, fn)", "Fold elements"),
    ("sort()", "Sort elements"),
    ("reverse()", "Reverse order"),
    ("contains(item)", "Check if item exists"),
    ("index_of(item)", "Find index of item"),
    ("sum()", "Sum all elements"),
    ("min()", "Minimum element"),
    ("max()", "Maximum element"),
    ("any(fn)", "True if any match"),
    ("all(fn)", "True if all match"),
    ("unique()", "Remove duplicates"),
    ("flat_map(fn)", "Map and flatten"),
    ("zip(other)", "Pair with another list"),
    ("chunks(n)", "Split into chunks of size n"),
    ("join(sep)", "Join elements with separator"),
];

/// Generate completions for a given trigger context.
pub fn get_completions(
    source: &str,
    line: u32,
    character: u32,
    symbols: &[(String, String, String)], // (name, kind, detail)
) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    // Get the current line text up to cursor
    let lines: Vec<&str> = source.lines().collect();
    let current_line = lines.get(line as usize).unwrap_or(&"");
    let prefix = if (character as usize) <= current_line.len() {
        &current_line[..character as usize]
    } else {
        current_line
    };

    // Check if we're after a `.` (method completion)
    if prefix.ends_with('.') || prefix.contains(".") {
        let before_dot = prefix.rsplit('.').nth(1).unwrap_or("");
        let var_name = before_dot.split_whitespace().last().unwrap_or("");

        // Determine type from symbols or heuristics
        let methods = guess_type_methods(var_name, symbols);
        for (name, detail) in methods {
            items.push(CompletionItem {
                label: name.to_string(),
                kind: Some(CompletionItemKind::METHOD),
                detail: Some(detail.to_string()),
                insert_text: Some(name.trim_end_matches("()").to_string()),
                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                ..Default::default()
            });
        }
        return items;
    }

    // Get the word being typed
    let word = prefix.split_whitespace().last().unwrap_or("");

    // Keywords
    for kw in KEYWORDS {
        if kw.starts_with(word) || word.is_empty() {
            items.push(CompletionItem {
                label: kw.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("keyword".to_string()),
                ..Default::default()
            });
        }
    }

    // Types
    for ty in TYPES {
        if ty.starts_with(word) || word.is_empty() {
            items.push(CompletionItem {
                label: ty.to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some("type".to_string()),
                ..Default::default()
            });
        }
    }

    // Built-in functions
    for (name, doc) in BUILTINS {
        if name.starts_with(word) || word.is_empty() {
            items.push(CompletionItem {
                label: name.to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some(doc.to_string()),
                ..Default::default()
            });
        }
    }

    // User-defined symbols from the file
    for (name, kind, detail) in symbols {
        if name.starts_with(word) || word.is_empty() {
            let kind = match kind.as_str() {
                "function" => CompletionItemKind::FUNCTION,
                "class" => CompletionItemKind::CLASS,
                "variable" => CompletionItemKind::VARIABLE,
                "field" => CompletionItemKind::FIELD,
                _ => CompletionItemKind::TEXT,
            };
            items.push(CompletionItem {
                label: name.to_string(),
                kind: Some(kind),
                detail: Some(detail.to_string()),
                ..Default::default()
            });
        }
    }

    // Snippet completions
    add_snippets(&mut items, word);

    items
}

fn guess_type_methods(var_name: &str, symbols: &[(String, String, String)]) -> Vec<(&'static str, &'static str)> {
    // Check symbols for type hints
    for (name, _kind, detail) in symbols {
        if name == var_name {
            if detail.contains("Str") || detail.contains("String") {
                return STR_METHODS.to_vec();
            }
            if detail.contains("List") || detail.contains("[]") {
                return LIST_METHODS.to_vec();
            }
        }
    }
    // Default: show all common methods
    let mut all = Vec::new();
    all.extend_from_slice(STR_METHODS);
    all.extend_from_slice(LIST_METHODS);
    all
}

fn add_snippets(items: &mut Vec<CompletionItem>, word: &str) {
    let snippets = vec![
        ("def", "def ${1:name}(${2:params}) {\n\t$0\n}", "Define a function"),
        ("class", "class ${1:Name} {\n\t${2:field}: ${3:Type}\n\n\tinit(${4:params}) {\n\t\t$0\n\t}\n}", "Define a class"),
        ("for", "for ${1:item} in ${2:collection} {\n\t$0\n}", "For loop"),
        ("match", "match ${1:value} {\n\t${2:pattern} -> ${3:expr}\n\t_ -> ${4:default}\n}", "Match expression"),
        ("if", "if ${1:condition} {\n\t$0\n}", "If statement"),
        ("parallel", "parallel {\n\t${1:task} = ${2:expr}\n\t$0\n}", "Parallel block"),
        ("try", "try {\n\t$0\n} catch ${1:any} as ${2:e} {\n\t${3:handle(e)}\n}", "Try/catch block"),
        ("loop", "loop ${1:5} times {\n\t$0\n}", "Loop N times"),
        ("struct", "struct ${1:Name} {\n\t${2:field}: ${3:Type}\n}", "Define a struct"),
        ("enum", "enum ${1:Name} {\n\t${2:Variant}\n}", "Define an enum"),
        ("genetic", "genetic class ${1:Name} {\n\tchromosome ${2:params} {\n\t\tgene ${3:name}: ${4:Float} = ${5:0.5} { range ${6:0.0}..${7:1.0} }\n\t}\n\tfitness(${8:data}: ${9:Float}) -> Float {\n\t\t$0\n\t}\n}", "Genetic class"),
    ];

    for (trigger, body, desc) in snippets {
        if trigger.starts_with(word) || word.is_empty() {
            items.push(CompletionItem {
                label: format!("{} (snippet)", trigger),
                kind: Some(CompletionItemKind::SNIPPET),
                detail: Some(desc.to_string()),
                insert_text: Some(body.to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            });
        }
    }
}

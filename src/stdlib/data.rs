use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::interpreter::values::*;

// ─── helpers ──────────────────────────────────────────────────────────────────

/// Extract rows from a DataFrame (List<Map>). Returns an error string on bad input.
fn get_rows(val: &Value) -> Result<Vec<Value>, String> {
    match val {
        Value::List(lst) => Ok(lst.borrow().clone()),
        _ => Err("data: expected a List<Map> DataFrame".into()),
    }
}

/// Make a Value::Map row from a HashMap.
fn make_row(map: HashMap<String, Value>) -> Value {
    Value::Map(Rc::new(RefCell::new(map)))
}

/// Make a Value::List from a Vec<Value>.
fn make_list(v: Vec<Value>) -> Value {
    Value::List(Rc::new(RefCell::new(v)))
}

/// Try to extract a float from any numeric Value.
fn to_float(v: &Value) -> Option<f64> {
    v.as_float()
}

/// Get an ordered list of column names from the first row.
fn columns_of(rows: &[Value]) -> Vec<String> {
    match rows.first() {
        Some(Value::Map(m)) => {
            let mut keys: Vec<String> = m.borrow().keys().cloned().collect();
            keys.sort();
            keys
        }
        _ => vec![],
    }
}

/// Coerce a value from CSV (all-string) into Int/Float if possible.
fn coerce(s: &str) -> Value {
    if let Ok(i) = s.parse::<i64>() {
        return Value::Int(i);
    }
    if let Ok(f) = s.parse::<f64>() {
        return Value::Float(f);
    }
    Value::String(s.to_string())
}

/// Parse a single CSV row handling quoted fields.
fn parse_csv_row(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                if in_quotes {
                    if chars.peek() == Some(&'"') {
                        chars.next();
                        current.push('"');
                    } else {
                        in_quotes = false;
                    }
                } else {
                    in_quotes = true;
                }
            }
            ',' if !in_quotes => {
                fields.push(current.trim().to_string());
                current = String::new();
            }
            _ => current.push(c),
        }
    }
    fields.push(current.trim().to_string());
    fields
}

/// Escape a single CSV field.
fn csv_escape(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

/// Compare two Values for ordering. Returns None if incomparable.
fn compare_values(a: &Value, b: &Value) -> std::cmp::Ordering {
    match (a.as_float(), b.as_float()) {
        (Some(fa), Some(fb)) => fa.partial_cmp(&fb).unwrap_or(std::cmp::Ordering::Equal),
        _ => {
            let sa = a.to_string();
            let sb = b.to_string();
            sa.cmp(&sb)
        }
    }
}

// ─── PRNG state for data_sample (simple xorshift) ────────────────────────────

static SAMPLE_SEED: std::sync::Mutex<u64> = std::sync::Mutex::new(0);

fn sample_rand() -> u64 {
    let mut g = SAMPLE_SEED.lock().unwrap();
    if *g == 0 {
        let ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos() as u64;
        let s = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        *g = (s ^ (ns << 32)).wrapping_add(0x9e3779b97f4a7c15);
        if *g == 0 { *g = 0xcafe_babe_dead_beef; }
    }
    let mut x = *g;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *g = x;
    x
}

// ─── public register function ─────────────────────────────────────────────────

pub fn register(env: &mut crate::interpreter::environment::Environment) {

    // ── 1. data_from_csv(text) -> List<Map> ──────────────────────────────────
    env.define("data_from_csv", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "data_from_csv".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let text = match args.first() {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("data_from_csv: argument must be a string".into()),
            };
            let mut lines = text.lines();
            let header_line = match lines.next() {
                Some(l) => l,
                None => return Ok(make_list(vec![])),
            };
            let headers: Vec<String> = parse_csv_row(header_line);
            let mut rows = Vec::new();
            for line in lines {
                let line = line.trim();
                if line.is_empty() { continue; }
                let fields = parse_csv_row(line);
                let mut map = HashMap::new();
                for (i, h) in headers.iter().enumerate() {
                    let raw = fields.get(i).map(|s| s.as_str()).unwrap_or("");
                    map.insert(h.clone(), coerce(raw));
                }
                rows.push(make_row(map));
            }
            Ok(make_list(rows))
        }),
    })));

    // ── 2. data_from_list(list) -> List<Map> ─────────────────────────────────
    env.define("data_from_list", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "data_from_list".into(),
        arity: Some(1),
        func: Box::new(|args| {
            match args.into_iter().next() {
                Some(v @ Value::List(_)) => Ok(v),
                _ => Err("data_from_list: argument must be a list of maps".into()),
            }
        }),
    })));

    // ── 3. data_columns(df) -> List<Str> ─────────────────────────────────────
    env.define("data_columns", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "data_columns".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let rows = get_rows(args.first().ok_or("data_columns: missing argument")?)?;
            let cols = columns_of(&rows);
            Ok(make_list(cols.into_iter().map(Value::String).collect()))
        }),
    })));

    // ── 4. data_rows(df) -> Int ───────────────────────────────────────────────
    env.define("data_rows", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "data_rows".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let rows = get_rows(args.first().ok_or("data_rows: missing argument")?)?;
            Ok(Value::Int(rows.len() as i64))
        }),
    })));

    // ── 5. data_select(df, columns) -> List<Map> ─────────────────────────────
    env.define("data_select", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "data_select".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let rows = get_rows(args.first().ok_or("data_select: missing df")?)?;
            let cols: Vec<String> = match args.get(1) {
                Some(Value::List(lst)) => lst.borrow().iter().map(|v| v.to_string()).collect(),
                _ => return Err("data_select: second argument must be a list of column names".into()),
            };
            let result: Vec<Value> = rows.iter().map(|row| {
                if let Value::Map(m) = row {
                    let src = m.borrow();
                    let mut new_map = HashMap::new();
                    for col in &cols {
                        if let Some(v) = src.get(col) {
                            new_map.insert(col.clone(), v.clone());
                        }
                    }
                    make_row(new_map)
                } else {
                    row.clone()
                }
            }).collect();
            Ok(make_list(result))
        }),
    })));

    // ── 6. data_where(df, column, op, value) -> List<Map> ────────────────────
    env.define("data_where", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "data_where".into(),
        arity: Some(4),
        func: Box::new(|args| {
            let rows = get_rows(args.first().ok_or("data_where: missing df")?)?;
            let col = match args.get(1) {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("data_where: second arg must be column name string".into()),
            };
            let op = match args.get(2) {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("data_where: third arg must be operator string".into()),
            };
            let filter_val = args.get(3).ok_or("data_where: missing filter value")?.clone();

            let result: Vec<Value> = rows.into_iter().filter(|row| {
                if let Value::Map(m) = row {
                    let map = m.borrow();
                    if let Some(cell) = map.get(&col) {
                        return apply_op(cell, &op, &filter_val);
                    }
                }
                false
            }).collect();
            Ok(make_list(result))
        }),
    })));

    // ── 7. data_sort_by(df, column, desc) -> List<Map> ───────────────────────
    env.define("data_sort_by", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "data_sort_by".into(),
        arity: Some(3),
        func: Box::new(|args| {
            let rows = get_rows(args.first().ok_or("data_sort_by: missing df")?)?;
            let col = match args.get(1) {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("data_sort_by: second arg must be column name string".into()),
            };
            let desc = match args.get(2) {
                Some(Value::Bool(b)) => *b,
                _ => false,
            };
            let mut result = rows;
            result.sort_by(|a, b| {
                let va = cell_val(a, &col);
                let vb = cell_val(b, &col);
                let ord = compare_values(&va, &vb);
                if desc { ord.reverse() } else { ord }
            });
            Ok(make_list(result))
        }),
    })));

    // ── 8. data_top(df, n) -> List<Map> ──────────────────────────────────────
    env.define("data_top", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "data_top".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let rows = get_rows(args.first().ok_or("data_top: missing df")?)?;
            let n = match args.get(1) {
                Some(v) => v.as_int().ok_or_else(|| "data_top: second arg must be int".to_string())? as usize,
                None => return Err("data_top: missing n".into()),
            };
            Ok(make_list(rows.into_iter().take(n).collect()))
        }),
    })));

    // ── 9. data_add_column(df, name, value) -> List<Map> ─────────────────────
    env.define("data_add_column", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "data_add_column".into(),
        arity: Some(3),
        func: Box::new(|args| {
            let rows = get_rows(args.first().ok_or("data_add_column: missing df")?)?;
            let col_name = match args.get(1) {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("data_add_column: second arg must be column name string".into()),
            };
            let col_val = args.get(2).ok_or("data_add_column: missing value")?.clone();
            let result: Vec<Value> = rows.into_iter().map(|row| {
                if let Value::Map(m) = &row {
                    let mut new_map = m.borrow().clone();
                    new_map.insert(col_name.clone(), col_val.clone());
                    make_row(new_map)
                } else {
                    row
                }
            }).collect();
            Ok(make_list(result))
        }),
    })));

    // ── 10. data_column(df, name) -> List ────────────────────────────────────
    env.define("data_column", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "data_column".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let rows = get_rows(args.first().ok_or("data_column: missing df")?)?;
            let col = match args.get(1) {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("data_column: second arg must be column name string".into()),
            };
            let vals: Vec<Value> = rows.iter().map(|row| cell_val(row, &col)).collect();
            Ok(make_list(vals))
        }),
    })));

    // ── 11. data_group_count(df, column) -> List<Map> ────────────────────────
    env.define("data_group_count", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "data_group_count".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let rows = get_rows(args.first().ok_or("data_group_count: missing df")?)?;
            let col = match args.get(1) {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("data_group_count: second arg must be column name string".into()),
            };
            let mut counts: Vec<(String, i64)> = Vec::new();
            for row in &rows {
                let key = cell_val(row, &col).to_string();
                if let Some(entry) = counts.iter_mut().find(|(k, _)| k == &key) {
                    entry.1 += 1;
                } else {
                    counts.push((key, 1));
                }
            }
            let result: Vec<Value> = counts.into_iter().map(|(k, c)| {
                let mut m = HashMap::new();
                m.insert(col.clone(), Value::String(k));
                m.insert("count".into(), Value::Int(c));
                make_row(m)
            }).collect();
            Ok(make_list(result))
        }),
    })));

    // ── 12. data_describe(df) -> Nil (prints stats) ───────────────────────────
    env.define("data_describe", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "data_describe".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let rows = get_rows(args.first().ok_or("data_describe: missing df")?)?;
            if rows.is_empty() {
                println!("(empty DataFrame)");
                return Ok(Value::Nil);
            }
            let cols = columns_of(&rows);
            // Find numeric columns
            let numeric_cols: Vec<String> = cols.iter().filter(|col| {
                rows.iter().any(|row| to_float(&cell_val(row, col)).is_some())
            }).cloned().collect();

            if numeric_cols.is_empty() {
                println!("(no numeric columns to describe)");
                return Ok(Value::Nil);
            }

            // Header
            let stat_labels = ["column", "count", "mean", "std", "min", "max"];
            let col_w = 12usize;
            let stat_w = 10usize;

            // Top border
            print!("┌{}", "─".repeat(col_w + 2));
            for _ in &stat_labels[1..] {
                print!("┬{}", "─".repeat(stat_w + 2));
            }
            println!("┐");

            // Header row
            print!("│ {:<col_w$} ", "column");
            for label in &stat_labels[1..] {
                print!("│ {:>stat_w$} ", label);
            }
            println!("│");

            // Separator
            print!("├{}", "─".repeat(col_w + 2));
            for _ in &stat_labels[1..] {
                print!("┼{}", "─".repeat(stat_w + 2));
            }
            println!("┤");

            // Data rows
            for col in &numeric_cols {
                let vals: Vec<f64> = rows.iter()
                    .filter_map(|row| to_float(&cell_val(row, col)))
                    .collect();
                let n = vals.len();
                let mean = vals.iter().sum::<f64>() / n as f64;
                let variance = vals.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n as f64;
                let std_dev = variance.sqrt();
                let min = vals.iter().cloned().fold(f64::INFINITY, f64::min);
                let max = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

                let col_display = if col.len() > col_w { &col[..col_w] } else { col.as_str() };
                print!("│ {:<col_w$} ", col_display);
                print!("│ {:>stat_w$} ", n);
                print!("│ {:>stat_w$.4} ", mean);
                print!("│ {:>stat_w$.4} ", std_dev);
                print!("│ {:>stat_w$.4} ", min);
                print!("│ {:>stat_w$.4} ", max);
                println!("│");
            }

            // Bottom border
            print!("└{}", "─".repeat(col_w + 2));
            for _ in &stat_labels[1..] {
                print!("┴{}", "─".repeat(stat_w + 2));
            }
            println!("┘");

            Ok(Value::Nil)
        }),
    })));

    // ── 13. data_print(df) -> Nil (prints table) ─────────────────────────────
    env.define("data_print", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "data_print".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let rows = get_rows(args.first().ok_or("data_print: missing df")?)?;
            if rows.is_empty() {
                println!("(empty DataFrame)");
                return Ok(Value::Nil);
            }

            // Collect columns from the first row (preserve insertion order if possible)
            let cols: Vec<String> = match rows.first() {
                Some(Value::Map(m)) => {
                    let mut keys: Vec<String> = m.borrow().keys().cloned().collect();
                    keys.sort();
                    keys
                }
                _ => vec![],
            };
            if cols.is_empty() {
                println!("(no columns)");
                return Ok(Value::Nil);
            }

            const MAX_COL: usize = 16;
            const MIN_COL: usize = 4;

            // Compute column widths: max of header len and cell display len (capped).
            let widths: Vec<usize> = cols.iter().map(|col| {
                let header_w = col.len().min(MAX_COL).max(MIN_COL);
                let data_w = rows.iter().map(|row| {
                    cell_display(&cell_val(row, col)).len().min(MAX_COL)
                }).max().unwrap_or(MIN_COL);
                header_w.max(data_w).max(MIN_COL)
            }).collect();

            // Top border  ┌──┬──┐
            print!("┌");
            for (i, w) in widths.iter().enumerate() {
                if i > 0 { print!("┬"); }
                print!("{}", "─".repeat(w + 2));
            }
            println!("┐");

            // Header row  │ Col │
            print!("│");
            for (i, col) in cols.iter().enumerate() {
                let w = widths[i];
                let header = truncate(col, w);
                print!(" {:<w$} │", header);
            }
            println!();

            // Header separator  ├──┼──┤
            print!("├");
            for (i, w) in widths.iter().enumerate() {
                if i > 0 { print!("┼"); }
                print!("{}", "─".repeat(w + 2));
            }
            println!("┤");

            // Data rows
            for row in &rows {
                print!("│");
                for (i, col) in cols.iter().enumerate() {
                    let w = widths[i];
                    let raw = cell_val(row, col);
                    let display = cell_display(&raw);
                    let cell = truncate(&display, w);
                    // Right-align numbers, left-align strings
                    if raw.as_float().is_some() {
                        print!(" {:>w$} │", cell);
                    } else {
                        print!(" {:<w$} │", cell);
                    }
                }
                println!();
            }

            // Bottom border  └──┴──┘
            print!("└");
            for (i, w) in widths.iter().enumerate() {
                if i > 0 { print!("┴"); }
                print!("{}", "─".repeat(w + 2));
            }
            println!("┘");

            Ok(Value::Nil)
        }),
    })));

    // ── 14. data_to_csv(df) -> Str ────────────────────────────────────────────
    env.define("data_to_csv", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "data_to_csv".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let rows = get_rows(args.first().ok_or("data_to_csv: missing df")?)?;
            if rows.is_empty() {
                return Ok(Value::String(String::new()));
            }
            let cols = columns_of(&rows);
            let mut lines: Vec<String> = Vec::new();
            // Header
            lines.push(cols.iter().map(|c| csv_escape(c)).collect::<Vec<_>>().join(","));
            // Rows
            for row in &rows {
                if let Value::Map(m) = row {
                    let map = m.borrow();
                    let fields: Vec<String> = cols.iter().map(|c| {
                        let s = match map.get(c) {
                            Some(Value::String(s)) => s.clone(),
                            Some(v) => v.to_string(),
                            None => String::new(),
                        };
                        csv_escape(&s)
                    }).collect();
                    lines.push(fields.join(","));
                }
            }
            Ok(Value::String(lines.join("\n")))
        }),
    })));

    // ── 15. data_join(df1, df2, on) -> List<Map> ─────────────────────────────
    env.define("data_join", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "data_join".into(),
        arity: Some(3),
        func: Box::new(|args| {
            let left = get_rows(args.first().ok_or("data_join: missing df1")?)?;
            let right = get_rows(args.get(1).ok_or("data_join: missing df2")?)?;
            let on = match args.get(2) {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("data_join: third arg must be join-key column name string".into()),
            };
            let mut result = Vec::new();
            for lrow in &left {
                let lkey = cell_val(lrow, &on).to_string();
                for rrow in &right {
                    let rkey = cell_val(rrow, &on).to_string();
                    if lkey == rkey {
                        // Merge maps; left takes precedence on duplicate keys
                        let mut merged = HashMap::new();
                        if let Value::Map(rm) = rrow {
                            for (k, v) in rm.borrow().iter() {
                                merged.insert(k.clone(), v.clone());
                            }
                        }
                        if let Value::Map(lm) = lrow {
                            for (k, v) in lm.borrow().iter() {
                                merged.insert(k.clone(), v.clone());
                            }
                        }
                        result.push(make_row(merged));
                    }
                }
            }
            Ok(make_list(result))
        }),
    })));

    // ── 16. data_value_counts(df, column) -> List<Map> ───────────────────────
    env.define("data_value_counts", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "data_value_counts".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let rows = get_rows(args.first().ok_or("data_value_counts: missing df")?)?;
            let col = match args.get(1) {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("data_value_counts: second arg must be column name string".into()),
            };
            let total = rows.len() as f64;
            let mut counts: Vec<(String, i64)> = Vec::new();
            for row in &rows {
                let key = cell_val(row, &col).to_string();
                if let Some(entry) = counts.iter_mut().find(|(k, _)| k == &key) {
                    entry.1 += 1;
                } else {
                    counts.push((key, 1));
                }
            }
            // Sort descending by count
            counts.sort_by(|a, b| b.1.cmp(&a.1));
            let result: Vec<Value> = counts.into_iter().map(|(k, c)| {
                let percent = if total > 0.0 { (c as f64 / total) * 100.0 } else { 0.0 };
                let mut m = HashMap::new();
                // Round percent to 2 decimal places for readable display
                let percent_rounded = (percent * 100.0).round() / 100.0;
                m.insert("value".into(), Value::String(k));
                m.insert("count".into(), Value::Int(c));
                m.insert("percent".into(), Value::Float(percent_rounded));
                make_row(m)
            }).collect();
            Ok(make_list(result))
        }),
    })));

    // ── 17. data_unique(df, column) -> List<Map> ─────────────────────────────
    env.define("data_unique", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "data_unique".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let rows = get_rows(args.first().ok_or("data_unique: missing df")?)?;
            let col = match args.get(1) {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("data_unique: second arg must be column name string".into()),
            };
            let mut seen: Vec<String> = Vec::new();
            let result: Vec<Value> = rows.into_iter().filter(|row| {
                let key = cell_val(row, &col).to_string();
                if seen.contains(&key) {
                    false
                } else {
                    seen.push(key);
                    true
                }
            }).collect();
            Ok(make_list(result))
        }),
    })));

    // ── 18. data_rename(df, old, new) -> List<Map> ───────────────────────────
    env.define("data_rename", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "data_rename".into(),
        arity: Some(3),
        func: Box::new(|args| {
            let rows = get_rows(args.first().ok_or("data_rename: missing df")?)?;
            let old_name = match args.get(1) {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("data_rename: second arg must be old column name string".into()),
            };
            let new_name = match args.get(2) {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("data_rename: third arg must be new column name string".into()),
            };
            let result: Vec<Value> = rows.into_iter().map(|row| {
                if let Value::Map(m) = &row {
                    let mut new_map = m.borrow().clone();
                    if let Some(val) = new_map.remove(&old_name) {
                        new_map.insert(new_name.clone(), val);
                    }
                    make_row(new_map)
                } else {
                    row
                }
            }).collect();
            Ok(make_list(result))
        }),
    })));

    // ── 19. data_fill_nulls(df, column, default) -> List<Map> ────────────────
    env.define("data_fill_nulls", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "data_fill_nulls".into(),
        arity: Some(3),
        func: Box::new(|args| {
            let rows = get_rows(args.first().ok_or("data_fill_nulls: missing df")?)?;
            let col = match args.get(1) {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("data_fill_nulls: second arg must be column name string".into()),
            };
            let default = args.get(2).ok_or("data_fill_nulls: missing default value")?.clone();
            let result: Vec<Value> = rows.into_iter().map(|row| {
                if let Value::Map(m) = &row {
                    let mut new_map = m.borrow().clone();
                    let is_null = match new_map.get(&col) {
                        None | Some(Value::Nil) => true,
                        _ => false,
                    };
                    if is_null {
                        new_map.insert(col.clone(), default.clone());
                    }
                    make_row(new_map)
                } else {
                    row
                }
            }).collect();
            Ok(make_list(result))
        }),
    })));

    // ── 20. data_sample(df, n) -> List<Map> ──────────────────────────────────
    env.define("data_sample", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "data_sample".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let rows = get_rows(args.first().ok_or("data_sample: missing df")?)?;
            let n = match args.get(1) {
                Some(v) => v.as_int().ok_or_else(|| "data_sample: second arg must be int".to_string())? as usize,
                None => return Err("data_sample: missing n".into()),
            };
            let n = n.min(rows.len());
            let mut indices: Vec<usize> = (0..rows.len()).collect();
            // Partial Fisher-Yates
            for i in 0..n {
                let j = i + (sample_rand() as usize) % (rows.len() - i);
                indices.swap(i, j);
            }
            let sample: Vec<Value> = indices[..n].iter().map(|&i| rows[i].clone()).collect();
            Ok(make_list(sample))
        }),
    })));
}

// ─── private helpers ──────────────────────────────────────────────────────────

/// Get a cell value from a row map (or Nil if missing).
fn cell_val(row: &Value, col: &str) -> Value {
    match row {
        Value::Map(m) => m.borrow().get(col).cloned().unwrap_or(Value::Nil),
        _ => Value::Nil,
    }
}

/// Display a Value as a plain string (no quotes around strings).
fn cell_display(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Nil => "nil".into(),
        other => other.to_string(),
    }
}

/// Truncate a string to at most `max` chars, appending "…" if needed.
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else if max <= 1 {
        s.chars().take(max).collect()
    } else {
        let mut t: String = s.chars().take(max - 1).collect();
        t.push('…');
        t
    }
}

/// Apply a comparison operator between a cell value and a filter value.
fn apply_op(cell: &Value, op: &str, filter: &Value) -> bool {
    if op == "contains" {
        let cs = match cell { Value::String(s) => s.clone(), v => v.to_string() };
        let fs = match filter { Value::String(s) => s.clone(), v => v.to_string() };
        return cs.contains(&fs);
    }
    // Numeric comparison path
    if let (Some(cv), Some(fv)) = (cell.as_float(), filter.as_float()) {
        return match op {
            "==" => (cv - fv).abs() < f64::EPSILON,
            "!=" => (cv - fv).abs() >= f64::EPSILON,
            ">"  => cv > fv,
            "<"  => cv < fv,
            ">=" => cv >= fv,
            "<=" => cv <= fv,
            _ => false,
        };
    }
    // String comparison path
    let cs = match cell { Value::String(s) => s.clone(), v => v.to_string() };
    let fs = match filter { Value::String(s) => s.clone(), v => v.to_string() };
    match op {
        "==" => cs == fs,
        "!=" => cs != fs,
        ">"  => cs > fs,
        "<"  => cs < fs,
        ">=" => cs >= fs,
        "<=" => cs <= fs,
        _ => false,
    }
}

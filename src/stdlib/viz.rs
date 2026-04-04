use std::rc::Rc;
use std::cell::RefCell;
use crate::interpreter::values::*;

// ═══════════════════════════════════════════════════════════════
// ANSI color helpers
// ═══════════════════════════════════════════════════════════════

const RESET: &str = "\x1b[0m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const CYAN: &str = "\x1b[36m";
const BOLD: &str = "\x1b[1m";

// Heatmap ANSI background colors (256-color mode)
fn heatmap_color(level: usize, total_levels: usize) -> String {
    // Map level 0..total_levels to blue->cyan->green->yellow->red
    let ratio = if total_levels <= 1 { 0.5 } else { level as f64 / (total_levels - 1) as f64 };
    if ratio < 0.25 {
        "\x1b[34m".to_string() // blue
    } else if ratio < 0.5 {
        "\x1b[36m".to_string() // cyan
    } else if ratio < 0.75 {
        "\x1b[32m".to_string() // green
    } else if ratio < 0.9 {
        "\x1b[33m".to_string() // yellow
    } else {
        "\x1b[31m".to_string() // red
    }
}

// ═══════════════════════════════════════════════════════════════
// Helper: call a Value::Function or NativeFunction with args
// ═══════════════════════════════════════════════════════════════

fn call_fn(func: &Value, args: Vec<Value>) -> Result<Value, String> {
    match func {
        Value::NativeFunction(nf) => (nf.func)(args),
        Value::Function(_) => {
            crate::interpreter::eval::call_function(
                func,
                args,
                &[],
                &mut crate::interpreter::environment::Environment::new(),
            )
            .map_err(|e| match e {
                Signal::Throw(v) => v.to_string(),
                Signal::Return(v) => v.to_string(),
                _ => "function error".to_string(),
            })
        }
        _ => Err("expected a callable function".into()),
    }
}

// ═══════════════════════════════════════════════════════════════
// viz_bar — horizontal bar chart
// ═══════════════════════════════════════════════════════════════

fn viz_bar_impl(args: Vec<Value>) -> Result<Value, String> {
    let labels: Vec<String> = match args.first() {
        Some(Value::List(l)) => l.borrow().iter().map(|v| v.to_string()).collect(),
        _ => return Err("viz_bar: first arg must be a list of labels".into()),
    };
    let values: Vec<f64> = match args.get(1) {
        Some(Value::List(l)) => {
            let borrow = l.borrow();
            borrow.iter().map(|v| v.as_float().unwrap_or(0.0)).collect()
        }
        _ => return Err("viz_bar: second arg must be a list of numbers".into()),
    };
    let title = match args.get(2) {
        Some(Value::String(s)) => Some(s.clone()),
        Some(v) if !matches!(v, Value::Nil) => Some(v.to_string()),
        _ => None,
    };

    if labels.len() != values.len() {
        return Err("viz_bar: labels and values must have the same length".into());
    }

    let max_val = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    if max_val <= 0.0 {
        println!("(no data)");
        return Ok(Value::Nil);
    }

    // Calculate label padding
    let max_label_len = labels.iter().map(|l| l.len()).max().unwrap_or(0);
    // Bar width available (60 chars total, minus label, value display, spacing)
    let value_width = values.iter().map(|v| format!("{:.0}", v).len()).max().unwrap_or(1);
    let bar_max = 60usize.saturating_sub(max_label_len + value_width + 4);

    if let Some(t) = &title {
        println!("  {}{}{}", BOLD, t, RESET);
    }

    let colors = [GREEN, CYAN, YELLOW, BLUE, RED];
    for (i, (label, &val)) in labels.iter().zip(values.iter()).enumerate() {
        let bar_len = if max_val > 0.0 {
            ((val / max_val) * bar_max as f64).round() as usize
        } else {
            0
        };
        let bar: String = "█".repeat(bar_len);
        let color = colors[i % colors.len()];
        println!(
            "  {:>width$}  {}{}{}  {:.0}",
            label,
            color,
            bar,
            RESET,
            val,
            width = max_label_len
        );
    }

    Ok(Value::Nil)
}

// ═══════════════════════════════════════════════════════════════
// viz_line — ASCII line chart
// ═══════════════════════════════════════════════════════════════

fn viz_line_impl(args: Vec<Value>) -> Result<Value, String> {
    let x_vals: Vec<f64> = match args.first() {
        Some(Value::List(l)) => {
            let b = l.borrow();
            b.iter().map(|v| v.as_float().unwrap_or(0.0)).collect()
        }
        _ => return Err("viz_line: first arg must be a list of x values".into()),
    };
    let y_vals: Vec<f64> = match args.get(1) {
        Some(Value::List(l)) => {
            let b = l.borrow();
            b.iter().map(|v| v.as_float().unwrap_or(0.0)).collect()
        }
        _ => return Err("viz_line: second arg must be a list of y values".into()),
    };
    let title = match args.get(2) {
        Some(Value::String(s)) => Some(s.clone()),
        Some(v) if !matches!(v, Value::Nil) => Some(v.to_string()),
        _ => None,
    };

    render_line_chart(&x_vals, &y_vals, title.as_deref())
}

fn render_line_chart(x_vals: &[f64], y_vals: &[f64], title: Option<&str>) -> Result<Value, String> {
    if x_vals.is_empty() || y_vals.is_empty() {
        println!("(no data)");
        return Ok(Value::Nil);
    }

    let width: usize = 40;
    let height: usize = 12;

    let x_min = x_vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let x_max = x_vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let y_min = y_vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let y_max = y_vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let x_range = if (x_max - x_min).abs() < 1e-12 { 1.0 } else { x_max - x_min };
    let y_range = if (y_max - y_min).abs() < 1e-12 { 1.0 } else { y_max - y_min };

    // 2D grid of chars
    let mut grid: Vec<Vec<char>> = vec![vec![' '; width]; height];

    // Plot each point
    for (&x, &y) in x_vals.iter().zip(y_vals.iter()) {
        let col = ((x - x_min) / x_range * (width - 1) as f64).round() as usize;
        let row_f = (y - y_min) / y_range * (height - 1) as f64;
        let row = height - 1 - row_f.round() as usize;
        let col = col.min(width - 1);
        let row = row.min(height - 1);
        grid[row][col] = '*';
    }

    // Y-axis label width
    let y_label_w = format!("{:.0}", y_max).len().max(format!("{:.0}", y_min).len()) + 1;

    if let Some(t) = title {
        println!("  {}{}{}", BOLD, t, RESET);
    }

    for (r, row) in grid.iter().enumerate() {
        let frac = if height <= 1 { 1.0 } else { 1.0 - r as f64 / (height - 1) as f64 };
        let y_label_val = y_min + frac * y_range;
        let y_label = format!("{:.0}", y_label_val);
        let row_str: String = row.iter().collect();
        println!(
            "  {:>width$} │{}{}{}",
            y_label,
            CYAN,
            row_str,
            RESET,
            width = y_label_w
        );
    }

    // X axis
    let axis_line = "─".repeat(width);
    println!("  {:>width$}─┼{}", "", axis_line, width = y_label_w);

    // X-axis labels
    let x_label_left = format!("{:.0}", x_min);
    let x_label_right = format!("{:.0}", x_max);
    let x_label_mid_val = x_min + x_range / 2.0;
    let x_label_mid = format!("{:.0}", x_label_mid_val);
    let pad = y_label_w + 2;
    let available = width;
    let left_space = 0usize;
    let right_space = available.saturating_sub(x_label_left.len() + x_label_right.len() + x_label_mid.len() + 2);
    let mid_space = right_space / 2;
    let empty = String::new();
    println!(
        "  {:pad$}{}{}{}{:>mid$}{}",
        empty,
        x_label_left,
        " ".repeat(mid_space.saturating_sub(x_label_mid.len() / 2)),
        x_label_mid,
        x_label_right,
        empty,
        pad = pad,
        mid = mid_space + x_label_mid.len(),
    );

    Ok(Value::Nil)
}

// ═══════════════════════════════════════════════════════════════
// viz_hist — histogram
// ═══════════════════════════════════════════════════════════════

fn viz_hist_impl(args: Vec<Value>) -> Result<Value, String> {
    let data: Vec<f64> = match args.first() {
        Some(Value::List(l)) => {
            let b = l.borrow();
            b.iter().map(|v| v.as_float().unwrap_or(0.0)).collect()
        }
        _ => return Err("viz_hist: first arg must be a list of numbers".into()),
    };
    let bins = match args.get(1) {
        Some(Value::Int(n)) => *n as usize,
        Some(Value::Float(f)) => *f as usize,
        Some(Value::Nil) | None => 10,
        _ => 10,
    };
    let bins = bins.max(1);
    let title = match args.get(2) {
        Some(Value::String(s)) => Some(s.clone()),
        Some(v) if !matches!(v, Value::Nil) => Some(v.to_string()),
        _ => None,
    };

    if data.is_empty() {
        println!("(no data)");
        return Ok(Value::Nil);
    }

    let min_val = data.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_val = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = max_val - min_val;

    // Build bin edges and counts
    let mut counts: Vec<usize> = vec![0; bins];
    for &v in &data {
        let bin = if range == 0.0 {
            0
        } else {
            let b = ((v - min_val) / range * bins as f64).floor() as usize;
            b.min(bins - 1)
        };
        counts[bin] += 1;
    }

    // Build labels
    let labels: Vec<String> = (0..bins).map(|i| {
        let lo = min_val + (i as f64 / bins as f64) * range;
        let hi = min_val + ((i + 1) as f64 / bins as f64) * range;
        format!("{:.0}-{:.0}", lo, hi)
    }).collect();

    let count_values: Vec<f64> = counts.iter().map(|&c| c as f64).collect();

    // Reuse bar chart logic
    let labels_val = Value::List(Rc::new(RefCell::new(
        labels.into_iter().map(Value::String).collect()
    )));
    let values_val = Value::List(Rc::new(RefCell::new(
        count_values.into_iter().map(Value::Float).collect()
    )));

    let mut bar_args = vec![labels_val, values_val];
    if let Some(t) = title {
        bar_args.push(Value::String(t));
    }

    viz_bar_impl(bar_args)
}

// ═══════════════════════════════════════════════════════════════
// viz_scatter — scatter plot
// ═══════════════════════════════════════════════════════════════

fn viz_scatter_impl(args: Vec<Value>) -> Result<Value, String> {
    let x_vals: Vec<f64> = match args.first() {
        Some(Value::List(l)) => {
            let b = l.borrow();
            b.iter().map(|v| v.as_float().unwrap_or(0.0)).collect()
        }
        _ => return Err("viz_scatter: first arg must be a list of x values".into()),
    };
    let y_vals: Vec<f64> = match args.get(1) {
        Some(Value::List(l)) => {
            let b = l.borrow();
            b.iter().map(|v| v.as_float().unwrap_or(0.0)).collect()
        }
        _ => return Err("viz_scatter: second arg must be a list of y values".into()),
    };
    let title = match args.get(2) {
        Some(Value::String(s)) => Some(s.clone()),
        Some(v) if !matches!(v, Value::Nil) => Some(v.to_string()),
        _ => None,
    };

    if x_vals.is_empty() || y_vals.is_empty() {
        println!("(no data)");
        return Ok(Value::Nil);
    }

    let width: usize = 40;
    let height: usize = 12;

    let x_min = x_vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let x_max = x_vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let y_min = y_vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let y_max = y_vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let x_range = if (x_max - x_min).abs() < 1e-12 { 1.0 } else { x_max - x_min };
    let y_range = if (y_max - y_min).abs() < 1e-12 { 1.0 } else { y_max - y_min };

    let mut grid: Vec<Vec<char>> = vec![vec![' '; width]; height];

    for (&x, &y) in x_vals.iter().zip(y_vals.iter()) {
        let col = ((x - x_min) / x_range * (width - 1) as f64).round() as usize;
        let row_f = (y - y_min) / y_range * (height - 1) as f64;
        let row = height - 1 - (row_f.round() as usize).min(height - 1);
        let col = col.min(width - 1);
        grid[row][col] = '*';
    }

    let y_label_w = format!("{:.0}", y_max).len().max(format!("{:.0}", y_min).len()) + 1;

    if let Some(t) = &title {
        println!("  {}{}{}", BOLD, t, RESET);
    }

    for (r, row) in grid.iter().enumerate() {
        let frac = if height <= 1 { 1.0 } else { 1.0 - r as f64 / (height - 1) as f64 };
        let y_label_val = y_min + frac * y_range;
        let y_label = format!("{:.0}", y_label_val);
        let row_str: String = row.iter().collect();
        println!(
            "  {:>width$} │{}{}{}",
            y_label,
            YELLOW,
            row_str,
            RESET,
            width = y_label_w
        );
    }

    let axis_line = "─".repeat(width);
    println!("  {:>width$}─┼{}", "", axis_line, width = y_label_w);

    let x_label_left = format!("{:.0}", x_min);
    let x_label_right = format!("{:.0}", x_max);
    let pad = y_label_w + 2;
    let gap = width.saturating_sub(x_label_left.len() + x_label_right.len());
    println!("  {:pad$}{}{:>gap$}", "", x_label_left, x_label_right, pad = pad, gap = gap);

    Ok(Value::Nil)
}

// ═══════════════════════════════════════════════════════════════
// viz_table — formatted table
// ═══════════════════════════════════════════════════════════════

fn viz_table_impl(args: Vec<Value>) -> Result<Value, String> {
    // Two signatures:
    //   viz_table(headers: List<Str>, rows: List<List>)
    //   viz_table(data: List<Map>)
    match (args.first(), args.get(1)) {
        // List<Map> signature: single list of maps
        (Some(Value::List(lst)), None) | (Some(Value::List(lst)), Some(Value::Nil)) => {
            let items = lst.borrow();
            if items.is_empty() {
                println!("(empty table)");
                return Ok(Value::Nil);
            }
            // Check if it's a list of maps
            if let Value::Map(_) = &items[0] {
                let headers: Vec<String> = if let Value::Map(m) = &items[0] {
                    let mut keys: Vec<String> = m.borrow().keys().cloned().collect();
                    keys.sort();
                    keys
                } else {
                    vec![]
                };

                let rows: Vec<Vec<String>> = items.iter().map(|item| {
                    if let Value::Map(m) = item {
                        let m = m.borrow();
                        headers.iter().map(|h| m.get(h).map(|v| v.to_string()).unwrap_or_default()).collect()
                    } else {
                        headers.iter().map(|_| String::new()).collect()
                    }
                }).collect();

                return render_table(&headers, &rows);
            }
            // Fallback: list of lists with no headers
            let rows: Vec<Vec<String>> = items.iter().map(|item| {
                if let Value::List(row) = item {
                    row.borrow().iter().map(|v| v.to_string()).collect()
                } else {
                    vec![item.to_string()]
                }
            }).collect();
            let max_cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
            let headers: Vec<String> = (0..max_cols).map(|i| format!("Col{}", i + 1)).collect();
            render_table(&headers, &rows)
        }
        // (headers, rows) signature
        (Some(Value::List(headers_v)), Some(Value::List(rows_v))) => {
            let headers: Vec<String> = headers_v.borrow().iter().map(|v| v.to_string()).collect();
            let rows: Vec<Vec<String>> = rows_v.borrow().iter().map(|row| {
                if let Value::List(row_list) = row {
                    row_list.borrow().iter().map(|v| v.to_string()).collect()
                } else {
                    vec![row.to_string()]
                }
            }).collect();
            render_table(&headers, &rows)
        }
        _ => Err("viz_table: expected (headers, rows) or (list of maps)".into()),
    }
}

fn render_table(headers: &[String], rows: &[Vec<String>]) -> Result<Value, String> {
    let ncols = headers.len();
    if ncols == 0 {
        println!("(empty table)");
        return Ok(Value::Nil);
    }

    // Calculate column widths
    let mut col_widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        for (j, cell) in row.iter().enumerate() {
            if j < ncols {
                col_widths[j] = col_widths[j].max(cell.len());
            }
        }
    }

    let make_horiz = |left: &str, mid: &str, right: &str, fill: &str| -> String {
        let mut s = left.to_string();
        for (i, &w) in col_widths.iter().enumerate() {
            s.push_str(&fill.repeat(w + 2));
            if i < ncols - 1 { s.push_str(mid); } else { s.push_str(right); }
        }
        s
    };

    let top = make_horiz("┌", "┬", "┐", "─");
    let mid = make_horiz("├", "┼", "┤", "─");
    let bot = make_horiz("└", "┴", "┘", "─");

    println!("{}", top);

    // Header row
    let mut header_row = String::from("│");
    for (i, h) in headers.iter().enumerate() {
        header_row.push_str(&format!(" {}{}{:<width$} ", BOLD, h, RESET, width = col_widths[i]));
        header_row.push('│');
    }
    println!("{}", header_row);
    println!("{}", mid);

    // Data rows
    for row in rows {
        let mut row_str = String::from("│");
        #[allow(clippy::needless_range_loop)]
        for j in 0..ncols {
            let cell = row.get(j).map(|s| s.as_str()).unwrap_or("");
            // right-align if looks numeric
            let aligned = if cell.parse::<f64>().is_ok() {
                format!(" {:>width$} ", cell, width = col_widths[j])
            } else {
                format!(" {:<width$} ", cell, width = col_widths[j])
            };
            row_str.push_str(&aligned);
            row_str.push('│');
        }
        println!("{}", row_str);
    }

    println!("{}", bot);
    Ok(Value::Nil)
}

// ═══════════════════════════════════════════════════════════════
// viz_heatmap — heatmap using block characters
// ═══════════════════════════════════════════════════════════════

fn viz_heatmap_impl(args: Vec<Value>) -> Result<Value, String> {
    let matrix: Vec<Vec<f64>> = match args.first() {
        Some(Value::List(outer)) => {
            outer.borrow().iter().map(|row| {
                if let Value::List(inner) = row {
                    inner.borrow().iter().map(|v| v.as_float().unwrap_or(0.0)).collect()
                } else {
                    vec![row.as_float().unwrap_or(0.0)]
                }
            }).collect()
        }
        _ => return Err("viz_heatmap: first arg must be a 2D list of numbers".into()),
    };
    let title = match args.get(1) {
        Some(Value::String(s)) => Some(s.clone()),
        Some(v) if !matches!(v, Value::Nil) => Some(v.to_string()),
        _ => None,
    };

    if matrix.is_empty() {
        println!("(no data)");
        return Ok(Value::Nil);
    }

    // Find global min/max
    let mut global_min = f64::INFINITY;
    let mut global_max = f64::NEG_INFINITY;
    for row in &matrix {
        for &v in row {
            if v < global_min { global_min = v; }
            if v > global_max { global_max = v; }
        }
    }
    let range = if (global_max - global_min).abs() < 1e-12 { 1.0 } else { global_max - global_min };

    let blocks = ['░', '▒', '▓', '█'];
    let n_levels = blocks.len();

    if let Some(t) = &title {
        println!("  {}{}{}", BOLD, t, RESET);
    }

    for row in &matrix {
        let mut line = String::from("  ");
        for &v in row {
            let norm = (v - global_min) / range;
            let level = ((norm * (n_levels - 1) as f64).round() as usize).min(n_levels - 1);
            let color = heatmap_color(level, n_levels);
            line.push_str(&format!("{}{}{}", color, blocks[level], RESET));
        }
        println!("{}", line);
    }

    // Legend
    println!();
    let mut legend = String::from("  Scale: ");
    for (i, block) in blocks.iter().enumerate().take(n_levels) {
        let color = heatmap_color(i, n_levels);
        legend.push_str(&format!("{}{}{}", color, block, RESET));
    }
    legend.push_str(&format!("  {:.2} … {:.2}", global_min, global_max));
    println!("{}", legend);

    Ok(Value::Nil)
}

// ═══════════════════════════════════════════════════════════════
// viz_progress — progress bar
// ═══════════════════════════════════════════════════════════════

fn viz_progress_impl(args: Vec<Value>) -> Result<Value, String> {
    let current = match args.first() {
        Some(v) => v.as_int().ok_or_else(|| "viz_progress: first arg must be a number".to_string())?,
        None => return Err("viz_progress requires at least 2 args".into()),
    };
    let total = match args.get(1) {
        Some(v) => v.as_int().ok_or_else(|| "viz_progress: second arg must be a number".to_string())?,
        None => return Err("viz_progress requires at least 2 args".into()),
    };
    let label = match args.get(2) {
        Some(Value::String(s)) => Some(s.clone()),
        Some(v) if !matches!(v, Value::Nil) => Some(v.to_string()),
        _ => None,
    };

    if total <= 0 {
        return Err("viz_progress: total must be > 0".into());
    }

    let current = current.max(0).min(total);
    let pct = (current as f64 / total as f64 * 100.0).round() as usize;
    let bar_width = 30usize;
    let filled = (current as f64 / total as f64 * bar_width as f64).round() as usize;
    let empty = bar_width.saturating_sub(filled);

    let bar = format!(
        "{}{}{}{}{}",
        GREEN,
        "█".repeat(filled),
        RESET,
        "░".repeat(empty),
        RESET
    );

    let label_str = label.map(|l| format!(" {} ({}/{})", l, current, total)).unwrap_or_default();
    println!("  [{}] {}{}%{}{}", bar, BOLD, pct, RESET, label_str);

    Ok(Value::Nil)
}

// ═══════════════════════════════════════════════════════════════
// viz_plot_fn — plot a math function
// ═══════════════════════════════════════════════════════════════

fn viz_plot_fn_impl(args: Vec<Value>) -> Result<Value, String> {
    let func = match args.first() {
        Some(v @ Value::Function(_)) | Some(v @ Value::NativeFunction(_)) => v.clone(),
        _ => return Err("viz_plot_fn: first arg must be a function".into()),
    };
    let x_min = match args.get(1) {
        Some(v) => v.as_float().ok_or_else(|| "viz_plot_fn: x_min must be a number".to_string())?,
        None => return Err("viz_plot_fn requires at least 3 args".into()),
    };
    let x_max = match args.get(2) {
        Some(v) => v.as_float().ok_or_else(|| "viz_plot_fn: x_max must be a number".to_string())?,
        None => return Err("viz_plot_fn requires at least 3 args".into()),
    };
    let title = match args.get(3) {
        Some(Value::String(s)) => Some(s.clone()),
        Some(v) if !matches!(v, Value::Nil) => Some(v.to_string()),
        _ => None,
    };

    let n_points = 40usize;
    let mut x_vals = Vec::with_capacity(n_points);
    let mut y_vals = Vec::with_capacity(n_points);

    for i in 0..n_points {
        let x = x_min + (i as f64 / (n_points - 1) as f64) * (x_max - x_min);
        let y = call_fn(&func, vec![Value::Float(x)])?;
        let y_f = y.as_float().unwrap_or(0.0);
        x_vals.push(x);
        y_vals.push(y_f);
    }

    render_line_chart(&x_vals, &y_vals, title.as_deref())
}

// ═══════════════════════════════════════════════════════════════
// viz_sparkline — inline sparkline
// ═══════════════════════════════════════════════════════════════

fn viz_sparkline_impl(args: Vec<Value>) -> Result<Value, String> {
    let data: Vec<f64> = match args.first() {
        Some(Value::List(l)) => {
            let b = l.borrow();
            b.iter().map(|v| v.as_float().unwrap_or(0.0)).collect()
        }
        _ => return Err("viz_sparkline: first arg must be a list of numbers".into()),
    };

    if data.is_empty() {
        println!("  (empty)");
        return Ok(Value::Nil);
    }

    let blocks = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let n = blocks.len();
    let min_val = data.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_val = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = if (max_val - min_val).abs() < 1e-12 { 1.0 } else { max_val - min_val };

    let line: String = data.iter().map(|&v| {
        let idx = ((v - min_val) / range * (n - 1) as f64).round() as usize;
        blocks[idx.min(n - 1)]
    }).collect();

    println!("  {}{}{}", CYAN, line, RESET);
    Ok(Value::Nil)
}

// ═══════════════════════════════════════════════════════════════
// register — expose all viz functions
// ═══════════════════════════════════════════════════════════════

pub fn register(env: &mut crate::interpreter::environment::Environment) {
    env.define("viz_bar", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "viz_bar".into(),
        arity: None,
        func: Box::new(viz_bar_impl),
    })));

    env.define("viz_line", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "viz_line".into(),
        arity: None,
        func: Box::new(viz_line_impl),
    })));

    env.define("viz_hist", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "viz_hist".into(),
        arity: None,
        func: Box::new(viz_hist_impl),
    })));

    env.define("viz_scatter", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "viz_scatter".into(),
        arity: None,
        func: Box::new(viz_scatter_impl),
    })));

    env.define("viz_table", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "viz_table".into(),
        arity: None,
        func: Box::new(viz_table_impl),
    })));

    env.define("viz_heatmap", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "viz_heatmap".into(),
        arity: None,
        func: Box::new(viz_heatmap_impl),
    })));

    env.define("viz_progress", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "viz_progress".into(),
        arity: None,
        func: Box::new(viz_progress_impl),
    })));

    env.define("viz_plot_fn", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "viz_plot_fn".into(),
        arity: None,
        func: Box::new(viz_plot_fn_impl),
    })));

    env.define("viz_sparkline", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "viz_sparkline".into(),
        arity: None,
        func: Box::new(viz_sparkline_impl),
    })));
}

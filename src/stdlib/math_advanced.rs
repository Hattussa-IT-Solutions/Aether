use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use crate::interpreter::values::*;

pub fn register(env: &mut crate::interpreter::environment::Environment) {
    // ── Statistics on lists ──────────────────────────────
    env.define("stats_mean", native("stats_mean", 1, |a| {
        let vals = extract_floats(&a[0])?;
        if vals.is_empty() { return Ok(Value::Float(0.0)); }
        Ok(Value::Float(vals.iter().sum::<f64>() / vals.len() as f64))
    }));

    env.define("stats_median", native("stats_median", 1, |a| {
        let mut vals = extract_floats(&a[0])?;
        if vals.is_empty() { return Ok(Value::Float(0.0)); }
        vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mid = vals.len() / 2;
        if vals.len() % 2 == 0 { Ok(Value::Float((vals[mid - 1] + vals[mid]) / 2.0)) }
        else { Ok(Value::Float(vals[mid])) }
    }));

    env.define("stats_mode", native("stats_mode", 1, |a| {
        let vals = extract_floats(&a[0])?;
        let mut counts: HashMap<i64, usize> = HashMap::new();
        for v in &vals { *counts.entry((*v * 1000.0) as i64).or_insert(0) += 1; }
        let max_count = counts.values().max().copied().unwrap_or(0);
        let mode_key = counts.iter().find(|(_, &c)| c == max_count).map(|(k, _)| *k).unwrap_or(0);
        Ok(Value::Float(mode_key as f64 / 1000.0))
    }));

    env.define("stats_std", native("stats_std", 1, |a| {
        let vals = extract_floats(&a[0])?;
        Ok(Value::Float(std_dev(&vals)))
    }));

    env.define("stats_variance", native("stats_variance", 1, |a| {
        let vals = extract_floats(&a[0])?;
        let s = std_dev(&vals);
        Ok(Value::Float(s * s))
    }));

    env.define("stats_percentile", native_var("stats_percentile", |a| {
        let vals = extract_floats(&a[0])?;
        let p = a.get(1).and_then(|v| v.as_float()).unwrap_or(50.0);
        Ok(Value::Float(percentile(&vals, p)))
    }));

    env.define("stats_quartiles", native("stats_quartiles", 1, |a| {
        let vals = extract_floats(&a[0])?;
        Ok(Value::Tuple(vec![
            Value::Float(percentile(&vals, 25.0)),
            Value::Float(percentile(&vals, 50.0)),
            Value::Float(percentile(&vals, 75.0)),
        ]))
    }));

    env.define("stats_iqr", native("stats_iqr", 1, |a| {
        let vals = extract_floats(&a[0])?;
        Ok(Value::Float(percentile(&vals, 75.0) - percentile(&vals, 25.0)))
    }));

    env.define("stats_outliers", native("stats_outliers", 1, |a| {
        let vals = extract_floats(&a[0])?;
        let q1 = percentile(&vals, 25.0);
        let q3 = percentile(&vals, 75.0);
        let iqr = q3 - q1;
        let lo = q1 - 1.5 * iqr;
        let hi = q3 + 1.5 * iqr;
        let outliers: Vec<Value> = vals.iter().filter(|&&v| v < lo || v > hi).map(|v| Value::Float(*v)).collect();
        Ok(Value::List(Rc::new(RefCell::new(outliers))))
    }));

    env.define("stats_z_scores", native("stats_z_scores", 1, |a| {
        let vals = extract_floats(&a[0])?;
        let m = mean(&vals);
        let s = std_dev(&vals);
        let zs: Vec<Value> = vals.iter().map(|v| Value::Float(if s > 0.0 { (v - m) / s } else { 0.0 })).collect();
        Ok(Value::List(Rc::new(RefCell::new(zs))))
    }));

    env.define("stats_normalize", native("stats_normalize", 1, |a| {
        let vals = extract_floats(&a[0])?;
        let min = vals.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = max - min;
        let normed: Vec<Value> = vals.iter().map(|v| Value::Float(if range > 0.0 { (v - min) / range } else { 0.0 })).collect();
        Ok(Value::List(Rc::new(RefCell::new(normed))))
    }));

    env.define("stats_cumsum", native("stats_cumsum", 1, |a| {
        let vals = extract_floats(&a[0])?;
        let mut sum = 0.0;
        let cs: Vec<Value> = vals.iter().map(|v| { sum += v; Value::Float(sum) }).collect();
        Ok(Value::List(Rc::new(RefCell::new(cs))))
    }));

    env.define("stats_diff", native("stats_diff", 1, |a| {
        let vals = extract_floats(&a[0])?;
        let d: Vec<Value> = vals.windows(2).map(|w| Value::Float(w[1] - w[0])).collect();
        Ok(Value::List(Rc::new(RefCell::new(d))))
    }));

    env.define("stats_moving_average", native_var("stats_moving_average", |a| {
        let vals = extract_floats(&a[0])?;
        let window = a.get(1).and_then(|v| v.as_int()).unwrap_or(3) as usize;
        let ma: Vec<Value> = vals.windows(window).map(|w| Value::Float(w.iter().sum::<f64>() / w.len() as f64)).collect();
        Ok(Value::List(Rc::new(RefCell::new(ma))))
    }));

    env.define("stats_correlation", native_var("stats_correlation", |a| {
        let x = extract_floats(&a[0])?;
        let y = extract_floats(&a[1])?;
        Ok(Value::Float(correlation(&x, &y)))
    }));

    // ── Linear Regression ────────────────────────────────
    env.define("math_linear_regression", native_var("math_linear_regression", |a| {
        let x = extract_floats(&a[0])?;
        let y = extract_floats(&a[1])?;
        let n = x.len().min(y.len()) as f64;
        if n < 2.0 { return Err("need at least 2 data points".into()); }
        let mx = mean(&x);
        let my = mean(&y);
        let mut num = 0.0;
        let mut den = 0.0;
        for i in 0..x.len().min(y.len()) {
            num += (x[i] - mx) * (y[i] - my);
            den += (x[i] - mx) * (x[i] - mx);
        }
        let slope = if den != 0.0 { num / den } else { 0.0 };
        let intercept = my - slope * mx;
        // R-squared
        let ss_res: f64 = x.iter().zip(y.iter()).map(|(xi, yi)| { let pred = slope * xi + intercept; (yi - pred).powi(2) }).sum();
        let ss_tot: f64 = y.iter().map(|yi| (yi - my).powi(2)).sum();
        let r_squared = if ss_tot > 0.0 { 1.0 - ss_res / ss_tot } else { 0.0 };
        let mut result = HashMap::new();
        result.insert("slope".into(), Value::Float(slope));
        result.insert("intercept".into(), Value::Float(intercept));
        result.insert("r_squared".into(), Value::Float(r_squared));
        Ok(Value::Map(Rc::new(RefCell::new(result))))
    }));

    // ── Calculus (numerical) ─────────────────────────────
    env.define("math_derivative", native_var("math_derivative", |a| {
        let f = &a[0];
        let x = a[1].as_float().unwrap_or(0.0);
        let h = 1e-8;
        let f_plus = call_fn_float(f, x + h)?;
        let f_minus = call_fn_float(f, x - h)?;
        Ok(Value::Float((f_plus - f_minus) / (2.0 * h)))
    }));

    env.define("math_integrate", native_var("math_integrate", |a| {
        let f = &a[0];
        let lo = a[1].as_float().unwrap_or(0.0);
        let hi = a[2].as_float().unwrap_or(1.0);
        let n = 1000;
        let h = (hi - lo) / n as f64;
        // Simpson's rule
        let mut sum = call_fn_float(f, lo)? + call_fn_float(f, hi)?;
        for i in 1..n {
            let x = lo + i as f64 * h;
            let coeff = if i % 2 == 0 { 2.0 } else { 4.0 };
            sum += coeff * call_fn_float(f, x)?;
        }
        Ok(Value::Float(sum * h / 3.0))
    }));

    env.define("math_find_root", native_var("math_find_root", |a| {
        let f = &a[0];
        let mut lo = a.get(1).and_then(|v| v.as_float()).unwrap_or(-10.0);
        let mut hi = a.get(2).and_then(|v| v.as_float()).unwrap_or(10.0);
        // Bisection method
        for _ in 0..100 {
            let mid = (lo + hi) / 2.0;
            let f_mid = call_fn_float(f, mid)?;
            if f_mid.abs() < 1e-10 { return Ok(Value::Float(mid)); }
            let f_lo = call_fn_float(f, lo)?;
            if f_lo * f_mid < 0.0 { hi = mid; } else { lo = mid; }
        }
        Ok(Value::Float((lo + hi) / 2.0))
    }));

    env.define("math_find_min", native_var("math_find_min", |a| {
        let f = &a[0];
        let mut lo = a.get(1).and_then(|v| v.as_float()).unwrap_or(-10.0);
        let mut hi = a.get(2).and_then(|v| v.as_float()).unwrap_or(10.0);
        let phi = (5.0_f64.sqrt() - 1.0) / 2.0;
        for _ in 0..100 {
            let x1 = hi - phi * (hi - lo);
            let x2 = lo + phi * (hi - lo);
            if call_fn_float(f, x1)? < call_fn_float(f, x2)? { hi = x2; } else { lo = x1; }
        }
        let x = (lo + hi) / 2.0;
        let y = call_fn_float(f, x)?;
        let mut result = HashMap::new();
        result.insert("x".into(), Value::Float(x));
        result.insert("y".into(), Value::Float(y));
        Ok(Value::Map(Rc::new(RefCell::new(result))))
    }));

    // ── Distance / Similarity ────────────────────────────
    env.define("math_distance", native_var("math_distance", |a| {
        let x = extract_floats(&a[0])?;
        let y = extract_floats(&a[1])?;
        let d: f64 = x.iter().zip(y.iter()).map(|(a, b)| (a - b).powi(2)).sum();
        Ok(Value::Float(d.sqrt()))
    }));

    env.define("math_cosine_similarity", native_var("math_cosine_similarity", |a| {
        let x = extract_floats(&a[0])?;
        let y = extract_floats(&a[1])?;
        let dot: f64 = x.iter().zip(y.iter()).map(|(a, b)| a * b).sum();
        let nx: f64 = x.iter().map(|a| a * a).sum::<f64>().sqrt();
        let ny: f64 = y.iter().map(|a| a * a).sum::<f64>().sqrt();
        Ok(Value::Float(if nx * ny > 0.0 { dot / (nx * ny) } else { 0.0 }))
    }));

    // ── Linear Algebra ───────────────────────────────────
    env.define("math_solve", native_var("math_solve", |a| {
        // Solve Ax = b using Gaussian elimination
        let a_mat = extract_matrix(&a[0])?;
        let b_vec = extract_floats(&a[1])?;
        let n = b_vec.len();
        let mut aug: Vec<Vec<f64>> = Vec::new();
        for i in 0..n {
            let mut row = a_mat[i].clone();
            row.push(b_vec[i]);
            aug.push(row);
        }
        // Forward elimination
        for i in 0..n {
            let mut max_row = i;
            for k in (i + 1)..n { if aug[k][i].abs() > aug[max_row][i].abs() { max_row = k; } }
            aug.swap(i, max_row);
            if aug[i][i].abs() < 1e-12 { return Err("singular matrix".into()); }
            for k in (i + 1)..n {
                let factor = aug[k][i] / aug[i][i];
                #[allow(clippy::needless_range_loop)]
                for j in i..=n { aug[k][j] -= factor * aug[i][j]; }
            }
        }
        // Back substitution
        let mut x = vec![0.0; n];
        for i in (0..n).rev() {
            x[i] = aug[i][n];
            for j in (i + 1)..n { x[i] -= aug[i][j] * x[j]; }
            x[i] /= aug[i][i];
        }
        let result: Vec<Value> = x.iter().map(|v| Value::Float(*v)).collect();
        Ok(Value::List(Rc::new(RefCell::new(result))))
    }));

    env.define("math_det", native("math_det", 1, |a| {
        let mat = extract_matrix(&a[0])?;
        Ok(Value::Float(determinant(&mat)))
    }));

    // ── Clustering ───────────────────────────────────────
    env.define("math_kmeans", native_var("math_kmeans", |a| {
        let data = extract_matrix(&a[0])?;
        let k = a.get(1).and_then(|v| v.as_int()).unwrap_or(3) as usize;
        let n = data.len();
        if n == 0 || k == 0 { return Err("empty data or k=0".into()); }
        let dims = data[0].len();
        // Initialize centroids: first k points
        let mut centroids: Vec<Vec<f64>> = data[..k.min(n)].to_vec();
        let mut labels = vec![0usize; n];
        for _ in 0..50 {
            // Assign
            for i in 0..n {
                let mut best_d = f64::INFINITY;
                for (j, centroid) in centroids.iter().enumerate().take(k) {
                    let d: f64 = (0..dims).map(|d| (data[i][d] - centroid[d]).powi(2)).sum();
                    if d < best_d { best_d = d; labels[i] = j; }
                }
            }
            // Update centroids
            let mut new_centroids = vec![vec![0.0; dims]; k];
            let mut counts = vec![0usize; k];
            for i in 0..n {
                let l = labels[i];
                counts[l] += 1;
                for d in 0..dims { new_centroids[l][d] += data[i][d]; }
            }
            for j in 0..k {
                if counts[j] > 0 {
                    for item in new_centroids[j].iter_mut().take(dims) { *item /= counts[j] as f64; }
                }
            }
            centroids = new_centroids;
        }
        let label_vals: Vec<Value> = labels.iter().map(|l| Value::Int(*l as i64)).collect();
        let mut result = HashMap::new();
        result.insert("labels".into(), Value::List(Rc::new(RefCell::new(label_vals))));
        Ok(Value::Map(Rc::new(RefCell::new(result))))
    }));

    // ── Number theory extras ─────────────────────────────
    env.define("math_primes_up_to", native("math_primes_up_to", 1, |a| {
        let n = a[0].as_int().unwrap_or(0) as usize;
        let mut sieve = vec![true; n + 1];
        sieve[0] = false; if n > 0 { sieve[1] = false; }
        for i in 2..=((n as f64).sqrt() as usize) {
            if sieve[i] { for j in (i*i..=n).step_by(i) { sieve[j] = false; } }
        }
        let primes: Vec<Value> = (2..=n).filter(|&i| sieve[i]).map(|i| Value::Int(i as i64)).collect();
        Ok(Value::List(Rc::new(RefCell::new(primes))))
    }));

    env.define("math_prime_factors", native("math_prime_factors", 1, |a| {
        let mut n = a[0].as_int().unwrap_or(0);
        let mut factors = Vec::new();
        let mut d = 2;
        while d * d <= n { while n % d == 0 { factors.push(Value::Int(d)); n /= d; } d += 1; }
        if n > 1 { factors.push(Value::Int(n)); }
        Ok(Value::List(Rc::new(RefCell::new(factors))))
    }));

    env.define("math_binomial", native_var("math_binomial", |a| {
        let n = a[0].as_int().unwrap_or(0);
        let k = a[1].as_int().unwrap_or(0);
        if k > n || k < 0 { return Ok(Value::Int(0)); }
        let k = k.min(n - k);
        let mut result: i64 = 1;
        for i in 0..k { result = result * (n - i) / (i + 1); }
        Ok(Value::Int(result))
    }));

    // ── Constants ────────────────────────────────────────
    env.define("PHI", Value::Float(1.618033988749895));
    env.define("SQRT2", Value::Float(std::f64::consts::SQRT_2));
    env.define("LN2", Value::Float(std::f64::consts::LN_2));
    env.define("LN10", Value::Float(std::f64::consts::LN_10));
    env.define("NAN", Value::Float(f64::NAN));
    env.define("NEG_INF", Value::Float(f64::NEG_INFINITY));
}

// ── Helpers ──────────────────────────────────────────────

fn native(name: &str, arity: usize, f: impl Fn(Vec<Value>) -> Result<Value, String> + 'static) -> Value {
    Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: name.to_string(), arity: Some(arity), func: Box::new(f),
    }))
}

fn native_var(name: &str, f: impl Fn(Vec<Value>) -> Result<Value, String> + 'static) -> Value {
    Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: name.to_string(), arity: None, func: Box::new(f),
    }))
}

fn extract_floats(val: &Value) -> Result<Vec<f64>, String> {
    match val {
        Value::List(items) => Ok(items.borrow().iter().map(|v| v.as_float().unwrap_or(0.0)).collect()),
        Value::Tuple(items) => Ok(items.iter().map(|v| v.as_float().unwrap_or(0.0)).collect()),
        _ => Err("expected a list of numbers".into()),
    }
}

fn extract_matrix(val: &Value) -> Result<Vec<Vec<f64>>, String> {
    match val {
        Value::List(rows) => {
            let rows = rows.borrow();
            let mut mat = Vec::new();
            for row in rows.iter() {
                match row {
                    Value::List(cols) => {
                        mat.push(cols.borrow().iter().map(|v| v.as_float().unwrap_or(0.0)).collect());
                    }
                    Value::Tuple(cols) => {
                        mat.push(cols.iter().map(|v| v.as_float().unwrap_or(0.0)).collect());
                    }
                    _ => return Err("matrix rows must be lists".into()),
                }
            }
            Ok(mat)
        }
        _ => Err("expected a matrix (list of lists)".into()),
    }
}

fn mean(vals: &[f64]) -> f64 {
    if vals.is_empty() { 0.0 } else { vals.iter().sum::<f64>() / vals.len() as f64 }
}

fn std_dev(vals: &[f64]) -> f64 {
    let m = mean(vals);
    let var = vals.iter().map(|x| (x - m).powi(2)).sum::<f64>() / vals.len().max(1) as f64;
    var.sqrt()
}

fn percentile(vals: &[f64], p: f64) -> f64 {
    if vals.is_empty() { return 0.0; }
    let mut sorted = vals.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = (p / 100.0 * (sorted.len() - 1) as f64).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn correlation(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len().min(y.len()) as f64;
    if n < 2.0 { return 0.0; }
    let mx = mean(x);
    let my = mean(y);
    let mut cov = 0.0;
    let mut sx = 0.0;
    let mut sy = 0.0;
    for i in 0..x.len().min(y.len()) {
        cov += (x[i] - mx) * (y[i] - my);
        sx += (x[i] - mx).powi(2);
        sy += (y[i] - my).powi(2);
    }
    if sx * sy > 0.0 { cov / (sx * sy).sqrt() } else { 0.0 }
}

fn determinant(mat: &[Vec<f64>]) -> f64 {
    let n = mat.len();
    if n == 1 { return mat[0][0]; }
    if n == 2 { return mat[0][0] * mat[1][1] - mat[0][1] * mat[1][0]; }
    let mut det = 0.0;
    for j in 0..n {
        let minor: Vec<Vec<f64>> = (1..n).map(|i| {
            (0..n).filter(|&k| k != j).map(|k| mat[i][k]).collect()
        }).collect();
        let sign = if j % 2 == 0 { 1.0 } else { -1.0 };
        det += sign * mat[0][j] * determinant(&minor);
    }
    det
}

fn call_fn_float(f: &Value, x: f64) -> Result<f64, String> {
    let result = crate::interpreter::eval::call_function(
        f, vec![Value::Float(x)], &[], &mut crate::interpreter::environment::Environment::new()
    ).map_err(|_| "function call failed".to_string())?;
    result.as_float().ok_or_else(|| "expected numeric result".into())
}

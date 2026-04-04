use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::interpreter::values::*;

/// A dense tensor stored as a flat Vec<f64> with a shape.
#[derive(Debug, Clone)]
pub struct TensorData {
    pub data: Vec<f64>,
    pub shape: Vec<usize>,
}

impl TensorData {
    pub fn zeros(shape: Vec<usize>) -> Self {
        let size: usize = shape.iter().product();
        Self { data: vec![0.0; size], shape }
    }

    pub fn ones(shape: Vec<usize>) -> Self {
        let size: usize = shape.iter().product();
        Self { data: vec![1.0; size], shape }
    }

    pub fn from_flat(data: Vec<f64>, shape: Vec<usize>) -> Self {
        Self { data, shape }
    }

    pub fn random(shape: Vec<usize>) -> Self {
        use std::time::SystemTime;
        let mut seed = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos() as u64;
        let size: usize = shape.iter().product();
        let mut data = Vec::with_capacity(size);
        for _ in 0..size {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            data.push((seed as f64 / u64::MAX as f64).abs());
        }
        Self { data, shape }
    }

    pub fn len(&self) -> usize { self.data.len() }
    pub fn is_empty(&self) -> bool { self.data.is_empty() }
    pub fn ndim(&self) -> usize { self.shape.len() }

    pub fn sum(&self) -> f64 { self.data.iter().sum() }
    pub fn mean(&self) -> f64 {
        if self.data.is_empty() { 0.0 } else { self.sum() / self.data.len() as f64 }
    }
    pub fn min(&self) -> f64 {
        self.data.iter().cloned().fold(f64::INFINITY, f64::min)
    }
    pub fn max(&self) -> f64 {
        self.data.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
    }
    pub fn std_dev(&self) -> f64 {
        let m = self.mean();
        let variance = self.data.iter().map(|x| (x - m).powi(2)).sum::<f64>() / self.data.len() as f64;
        variance.sqrt()
    }

    /// Element-wise binary operation.
    pub fn elementwise(&self, other: &TensorData, op: impl Fn(f64, f64) -> f64) -> Result<TensorData, String> {
        if self.shape != other.shape {
            // Try broadcasting: scalar op tensor
            if other.data.len() == 1 {
                let s = other.data[0];
                let data: Vec<f64> = self.data.iter().map(|x| op(*x, s)).collect();
                return Ok(TensorData { data, shape: self.shape.clone() });
            }
            if self.data.len() == 1 {
                let s = self.data[0];
                let data: Vec<f64> = other.data.iter().map(|x| op(s, *x)).collect();
                return Ok(TensorData { data, shape: other.shape.clone() });
            }
            return Err(format!("shape mismatch: {:?} vs {:?}", self.shape, other.shape));
        }
        let data: Vec<f64> = self.data.iter().zip(other.data.iter()).map(|(a, b)| op(*a, *b)).collect();
        Ok(TensorData { data, shape: self.shape.clone() })
    }

    /// Scalar operation.
    pub fn scalar_op(&self, scalar: f64, op: impl Fn(f64, f64) -> f64) -> TensorData {
        let data: Vec<f64> = self.data.iter().map(|x| op(*x, scalar)).collect();
        TensorData { data, shape: self.shape.clone() }
    }

    /// Matrix multiply (2D only).
    pub fn matmul(&self, other: &TensorData) -> Result<TensorData, String> {
        if self.shape.len() != 2 || other.shape.len() != 2 {
            return Err("matmul requires 2D tensors".into());
        }
        let (m, k1) = (self.shape[0], self.shape[1]);
        let (k2, n) = (other.shape[0], other.shape[1]);
        if k1 != k2 {
            return Err(format!("matmul dimension mismatch: {}x{} @ {}x{}", m, k1, k2, n));
        }
        let mut result = vec![0.0; m * n];
        for i in 0..m {
            for j in 0..n {
                let mut sum = 0.0;
                for k in 0..k1 {
                    sum += self.data[i * k1 + k] * other.data[k * n + j];
                }
                result[i * n + j] = sum;
            }
        }
        Ok(TensorData { data: result, shape: vec![m, n] })
    }

    /// Transpose (2D).
    pub fn transpose(&self) -> Result<TensorData, String> {
        if self.shape.len() != 2 {
            return Err("transpose requires 2D tensor".into());
        }
        let (rows, cols) = (self.shape[0], self.shape[1]);
        let mut result = vec![0.0; rows * cols];
        for i in 0..rows {
            for j in 0..cols {
                result[j * rows + i] = self.data[i * cols + j];
            }
        }
        Ok(TensorData { data: result, shape: vec![cols, rows] })
    }

    /// Reshape.
    pub fn reshape(&self, new_shape: Vec<usize>) -> Result<TensorData, String> {
        let new_size: usize = new_shape.iter().product();
        if new_size != self.data.len() {
            return Err(format!("cannot reshape {} elements to {:?}", self.data.len(), new_shape));
        }
        Ok(TensorData { data: self.data.clone(), shape: new_shape })
    }

    /// Negation.
    pub fn neg(&self) -> TensorData {
        let data: Vec<f64> = self.data.iter().map(|x| -x).collect();
        TensorData { data, shape: self.shape.clone() }
    }
}

impl fmt::Display for TensorData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.shape.len() == 1 {
            write!(f, "Tensor([")?;
            for (i, v) in self.data.iter().enumerate() {
                if i > 0 { write!(f, ", ")?; }
                if i > 8 { write!(f, "...")?; break; }
                write!(f, "{:.4}", v)?;
            }
            write!(f, "], shape={:?})", self.shape)
        } else if self.shape.len() == 2 {
            writeln!(f, "Tensor([")?;
            let cols = self.shape[1];
            for r in 0..self.shape[0].min(6) {
                write!(f, "  [")?;
                for c in 0..cols.min(6) {
                    if c > 0 { write!(f, ", ")?; }
                    write!(f, "{:.4}", self.data[r * cols + c])?;
                }
                if cols > 6 { write!(f, ", ...")?; }
                write!(f, "]")?;
                if r < self.shape[0] - 1 { write!(f, ",")?; }
                writeln!(f)?;
            }
            if self.shape[0] > 6 { writeln!(f, "  ...")?; }
            write!(f, "], shape={:?})", self.shape)
        } else {
            write!(f, "Tensor(shape={:?}, len={})", self.shape, self.data.len())
        }
    }
}

/// Register tensor functions as builtins.
pub fn register_tensor(env: &mut crate::interpreter::environment::Environment) {
    env.define("Tensor_zeros", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "Tensor_zeros".into(), arity: Some(1),
        func: Box::new(|args| {
            let shape = extract_shape(&args[0])?;
            let t = TensorData::zeros(shape);
            Ok(tensor_to_value(t))
        }),
    })));

    env.define("Tensor_ones", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "Tensor_ones".into(), arity: Some(1),
        func: Box::new(|args| {
            let shape = extract_shape(&args[0])?;
            let t = TensorData::ones(shape);
            Ok(tensor_to_value(t))
        }),
    })));

    env.define("Tensor_random", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "Tensor_random".into(), arity: Some(1),
        func: Box::new(|args| {
            let shape = extract_shape(&args[0])?;
            let t = TensorData::random(shape);
            Ok(tensor_to_value(t))
        }),
    })));

    env.define("Tensor_from_list", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "Tensor_from_list".into(), arity: Some(1),
        func: Box::new(|args| {
            match &args[0] {
                Value::List(items) => {
                    let items = items.borrow();
                    // Check if 2D (list of lists)
                    if let Some(Value::List(_)) = items.first() {
                        let mut data = Vec::new();
                        let rows = items.len();
                        let mut cols = 0;
                        for row in items.iter() {
                            if let Value::List(row_items) = row {
                                let row_items = row_items.borrow();
                                if cols == 0 { cols = row_items.len(); }
                                for v in row_items.iter() {
                                    data.push(v.as_float().unwrap_or(0.0));
                                }
                            }
                        }
                        Ok(tensor_to_value(TensorData::from_flat(data, vec![rows, cols])))
                    } else {
                        let data: Vec<f64> = items.iter().map(|v| v.as_float().unwrap_or(0.0)).collect();
                        let len = data.len();
                        Ok(tensor_to_value(TensorData::from_flat(data, vec![len])))
                    }
                }
                _ => Err("Tensor_from_list requires a list".into()),
            }
        }),
    })));

    // Tensor operations as functions
    env.define("tensor_add", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "tensor_add".into(), arity: Some(2),
        func: Box::new(|args| {
            let a = extract_tensor(&args[0])?;
            let b = extract_tensor(&args[1])?;
            let result = a.elementwise(&b, |x, y| x + y)?;
            Ok(tensor_to_value(result))
        }),
    })));

    env.define("tensor_mul", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "tensor_mul".into(), arity: Some(2),
        func: Box::new(|args| {
            let a = extract_tensor(&args[0])?;
            let b = extract_tensor(&args[1])?;
            let result = a.elementwise(&b, |x, y| x * y)?;
            Ok(tensor_to_value(result))
        }),
    })));

    env.define("tensor_matmul", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "tensor_matmul".into(), arity: Some(2),
        func: Box::new(|args| {
            let a = extract_tensor(&args[0])?;
            let b = extract_tensor(&args[1])?;
            let result = a.matmul(&b)?;
            Ok(tensor_to_value(result))
        }),
    })));

    env.define("tensor_transpose", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "tensor_transpose".into(), arity: Some(1),
        func: Box::new(|args| {
            let t = extract_tensor(&args[0])?;
            let result = t.transpose()?;
            Ok(tensor_to_value(result))
        }),
    })));

    env.define("tensor_reshape", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "tensor_reshape".into(), arity: Some(2),
        func: Box::new(|args| {
            let t = extract_tensor(&args[0])?;
            let shape = extract_shape(&args[1])?;
            let result = t.reshape(shape)?;
            Ok(tensor_to_value(result))
        }),
    })));

    env.define("tensor_sum", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "tensor_sum".into(), arity: Some(1),
        func: Box::new(|args| {
            let t = extract_tensor(&args[0])?;
            Ok(Value::Float(t.sum()))
        }),
    })));

    env.define("tensor_mean", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "tensor_mean".into(), arity: Some(1),
        func: Box::new(|args| {
            let t = extract_tensor(&args[0])?;
            Ok(Value::Float(t.mean()))
        }),
    })));

    env.define("tensor_shape", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "tensor_shape".into(), arity: Some(1),
        func: Box::new(|args| {
            let t = extract_tensor(&args[0])?;
            let shape: Vec<Value> = t.shape.iter().map(|s| Value::Int(*s as i64)).collect();
            Ok(Value::List(Rc::new(RefCell::new(shape))))
        }),
    })));
}

/// Convert a TensorData to a Value (stored as a Map with __tensor_data, shape, etc.)
fn tensor_to_value(t: TensorData) -> Value {
    let mut map = HashMap::new();
    map.insert("__type".to_string(), Value::String("Tensor".to_string()));
    let data_list: Vec<Value> = t.data.iter().map(|v| Value::Float(*v)).collect();
    map.insert("data".to_string(), Value::List(Rc::new(RefCell::new(data_list))));
    let shape_list: Vec<Value> = t.shape.iter().map(|s| Value::Int(*s as i64)).collect();
    map.insert("shape".to_string(), Value::List(Rc::new(RefCell::new(shape_list))));
    map.insert("ndim".to_string(), Value::Int(t.ndim() as i64));
    map.insert("len".to_string(), Value::Int(t.len() as i64));
    map.insert("__display".to_string(), Value::String(t.to_string()));
    Value::Map(Rc::new(RefCell::new(map)))
}

/// Extract TensorData from a Value (Map representation).
fn extract_tensor(val: &Value) -> Result<TensorData, String> {
    match val {
        Value::Map(map) => {
            let map = map.borrow();
            let data = match map.get("data") {
                Some(Value::List(items)) => {
                    items.borrow().iter().map(|v| v.as_float().unwrap_or(0.0)).collect()
                }
                _ => return Err("not a tensor".into()),
            };
            let shape = match map.get("shape") {
                Some(Value::List(items)) => {
                    items.borrow().iter().map(|v| v.as_int().unwrap_or(0) as usize).collect()
                }
                _ => return Err("not a tensor".into()),
            };
            Ok(TensorData { data, shape })
        }
        _ => Err("not a tensor".into()),
    }
}

/// Extract a shape from a Value (list of ints).
fn extract_shape(val: &Value) -> Result<Vec<usize>, String> {
    match val {
        Value::List(items) => {
            Ok(items.borrow().iter().map(|v| v.as_int().unwrap_or(0) as usize).collect())
        }
        Value::Tuple(items) => {
            Ok(items.iter().map(|v| v.as_int().unwrap_or(0) as usize).collect())
        }
        _ => Err("shape must be a list of integers".into()),
    }
}

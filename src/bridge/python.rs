use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use pyo3::prelude::*;
use pyo3::types::*;

use crate::interpreter::values::Value;

/// Import a Python module by name. Returns a wrapped PyObject as a Value.
pub fn python_import(module_name: &str) -> Result<Value, String> {
    Python::with_gil(|py| {
        let module = py.import_bound(module_name)
            .map_err(|e| format!("Python import error: {}", e))?;
        let obj: PyObject = module.into_py(py);
        Ok(Value::PythonObject(PythonObjectWrapper::new(obj)))
    })
}

/// Call a method/function on a Python object.
pub fn python_call(
    obj: &PythonObjectWrapper,
    method: &str,
    args: Vec<Value>,
) -> Result<Value, String> {
    Python::with_gil(|py| {
        let py_obj = obj.inner.bind(py);

        // Get the attribute
        let attr = py_obj.getattr(method)
            .map_err(|e| format!("Python attribute error: {}.{} — {}", obj.name, method, e))?;

        // Convert Aether args to Python args
        let py_args: Vec<PyObject> = args.iter()
            .map(|v| aether_to_python(py, v))
            .collect();

        let py_tuple = PyTuple::new_bound(py, &py_args);

        // Call it
        let result = attr.call1(py_tuple)
            .map_err(|e| format!("Python call error: {}.{}() — {}", obj.name, method, e))?;

        Ok(python_to_aether(py, &result))
    })
}

/// Access an attribute/field on a Python object.
pub fn python_getattr(
    obj: &PythonObjectWrapper,
    attr: &str,
) -> Result<Value, String> {
    Python::with_gil(|py| {
        let py_obj = obj.inner.bind(py);

        let val = py_obj.getattr(attr)
            .map_err(|e| format!("Python getattr error: {}.{} — {}", obj.name, attr, e))?;

        // Check if it's callable (a function/method) — if so, wrap as PythonObject
        if val.is_callable() {
            let wrapped: PyObject = val.into_py(py);
            Ok(Value::PythonObject(PythonObjectWrapper {
                inner: wrapped,
                name: format!("{}.{}", obj.name, attr),
            }))
        } else {
            Ok(python_to_aether(py, &val))
        }
    })
}

/// Call a Python callable object directly (for when we have a function reference).
pub fn python_call_direct(
    obj: &PythonObjectWrapper,
    args: Vec<Value>,
) -> Result<Value, String> {
    Python::with_gil(|py| {
        let py_obj = obj.inner.bind(py);

        let py_args: Vec<PyObject> = args.iter()
            .map(|v| aether_to_python(py, v))
            .collect();

        let py_tuple = PyTuple::new_bound(py, &py_args);

        let result = py_obj.call1(py_tuple)
            .map_err(|e| format!("Python call error: {}() — {}", obj.name, e))?;

        Ok(python_to_aether(py, &result))
    })
}

// ═══════════════════════════════════════════════════════════════
// Value conversion: Aether -> Python
// ═══════════════════════════════════════════════════════════════

fn aether_to_python(py: Python<'_>, value: &Value) -> PyObject {
    match value {
        Value::Int(n) => n.into_py(py),
        Value::Float(f) => f.into_py(py),
        Value::Bool(b) => b.into_py(py),
        Value::String(s) => s.into_py(py),
        Value::Char(c) => c.to_string().into_py(py),
        Value::Nil => py.None(),
        Value::List(items) => {
            let items = items.borrow();
            let py_list = PyList::new_bound(py, items.iter().map(|v| aether_to_python(py, v)));
            py_list.into_py(py)
        }
        Value::Map(map) => {
            let map = map.borrow();
            let py_dict = PyDict::new_bound(py);
            for (k, v) in map.iter() {
                py_dict.set_item(k, aether_to_python(py, v)).ok();
            }
            py_dict.into_py(py)
        }
        Value::Tuple(items) => {
            let py_items: Vec<PyObject> = items.iter().map(|v| aether_to_python(py, v)).collect();
            PyTuple::new_bound(py, &py_items).into_py(py)
        }
        Value::PythonObject(wrapper) => wrapper.inner.clone_ref(py),
        _ => {
            // For other types, convert to string
            value.to_string().into_py(py)
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Value conversion: Python -> Aether
// ═══════════════════════════════════════════════════════════════

fn python_to_aether(py: Python<'_>, obj: &Bound<'_, PyAny>) -> Value {
    // Check None first
    if obj.is_none() {
        return Value::Nil;
    }

    // Bool (must check before int since Python bool is a subclass of int)
    if let Ok(b) = obj.extract::<bool>() {
        return Value::Bool(b);
    }

    // Int
    if let Ok(n) = obj.extract::<i64>() {
        return Value::Int(n);
    }

    // Float
    if let Ok(f) = obj.extract::<f64>() {
        return Value::Float(f);
    }

    // String
    if let Ok(s) = obj.extract::<String>() {
        return Value::String(s);
    }

    // List
    if let Ok(list) = obj.downcast::<PyList>() {
        let items: Vec<Value> = list.iter()
            .map(|item| python_to_aether(py, &item))
            .collect();
        return Value::List(Rc::new(RefCell::new(items)));
    }

    // Tuple
    if let Ok(tuple) = obj.downcast::<PyTuple>() {
        let items: Vec<Value> = tuple.iter()
            .map(|item| python_to_aether(py, &item))
            .collect();
        return Value::Tuple(items);
    }

    // Dict
    if let Ok(dict) = obj.downcast::<PyDict>() {
        let mut map = HashMap::new();
        for (k, v) in dict.iter() {
            let key = k.extract::<String>().unwrap_or_else(|_| k.str().map(|s| s.to_string()).unwrap_or_default());
            map.insert(key, python_to_aether(py, &v));
        }
        return Value::Map(Rc::new(RefCell::new(map)));
    }

    // For anything else (numpy arrays, custom objects, etc.) — wrap as PythonObject
    let repr = obj.repr().map(|r| r.to_string()).unwrap_or_else(|_| "<python object>".to_string());
    let wrapped: PyObject = obj.clone().into_py(py);
    Value::PythonObject(PythonObjectWrapper {
        inner: wrapped,
        name: repr,
    })
}

// ═══════════════════════════════════════════════════════════════
// PythonObjectWrapper — wraps a PyObject for use in Aether
// ═══════════════════════════════════════════════════════════════

/// Wrapper around a Python object that can be stored in Aether's Value enum.
/// Uses pyo3's PyObject which is Send-safe (unlike Rc).
pub struct PythonObjectWrapper {
    pub inner: PyObject,
    pub name: String,
}

impl Clone for PythonObjectWrapper {
    fn clone(&self) -> Self {
        Python::with_gil(|py| {
            Self {
                inner: self.inner.clone_ref(py),
                name: self.name.clone(),
            }
        })
    }
}

impl PythonObjectWrapper {
    pub fn new(obj: PyObject) -> Self {
        let name = Python::with_gil(|py| {
            let bound = obj.bind(py);
            bound.getattr("__name__")
                .and_then(|n| n.extract::<String>())
                .unwrap_or_else(|_| {
                    bound.repr().map(|r| r.to_string()).unwrap_or_else(|_| "<python>".to_string())
                })
        });
        Self { inner: obj, name }
    }
}

impl std::fmt::Debug for PythonObjectWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<python: {}>", self.name)
    }
}

impl std::fmt::Display for PythonObjectWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Python::with_gil(|py| {
            let bound = self.inner.bind(py);
            let repr = bound.str()
                .map(|s| s.to_string())
                .unwrap_or_else(|_| format!("<python: {}>", self.name));
            write!(f, "{}", repr)
        })
    }
}

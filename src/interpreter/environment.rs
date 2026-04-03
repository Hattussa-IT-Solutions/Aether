use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::interpreter::values::Value;

/// High-performance variable scope using a flat Vec with scope boundary markers.
/// Globals use HashMap; locals use a flat Vec scanned backwards.
/// String allocations are minimized by reusing existing names on update.
#[derive(Debug, Clone)]
pub struct Environment {
    globals: HashMap<String, Value>,
    locals: Vec<(String, Value)>,
    scope_marks: Vec<usize>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            globals: HashMap::with_capacity(128),
            locals: Vec::with_capacity(64),
            scope_marks: Vec::with_capacity(16),
        }
    }

    pub fn new_shared() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self::new()))
    }

    #[inline(always)]
    pub fn push_scope(&mut self) {
        self.scope_marks.push(self.locals.len());
    }

    #[inline(always)]
    pub fn pop_scope(&mut self) {
        if let Some(mark) = self.scope_marks.pop() {
            self.locals.truncate(mark);
        }
    }

    /// Define a variable in the current scope. Avoids String allocation
    /// if the variable already exists in the current scope (update in place).
    #[inline(always)]
    pub fn define(&mut self, name: &str, value: Value) {
        if self.scope_marks.is_empty() {
            self.globals.insert(name.to_string(), value);
        } else {
            let scope_start = unsafe { *self.scope_marks.last().unwrap_unchecked() };
            // Scan current scope only — find existing to update
            let len = self.locals.len();
            let mut i = len;
            while i > scope_start {
                i -= 1;
                if self.locals[i].0.as_str() == name {
                    self.locals[i].1 = value;
                    return;
                }
            }
            self.locals.push((name.to_string(), value));
        }
    }

    /// Fast define — caller guarantees this is a NEW variable (not update).
    /// Avoids the backwards scan entirely.
    #[inline(always)]
    pub fn define_new(&mut self, name: &str, value: Value) {
        if self.scope_marks.is_empty() {
            self.globals.insert(name.to_string(), value);
        } else {
            self.locals.push((name.to_string(), value));
        }
    }

    /// Get a variable. Returns clone of the value.
    #[inline(always)]
    pub fn get(&self, name: &str) -> Option<Value> {
        let len = self.locals.len();
        let mut i = len;
        while i > 0 {
            i -= 1;
            if self.locals[i].0.as_str() == name {
                return Some(self.locals[i].1.clone());
            }
        }
        self.globals.get(name).cloned()
    }

    /// Get a variable that's expected to be in the top scope (common case for
    /// function parameters and loop variables). Only scans the top scope.
    #[inline(always)]
    pub fn get_local(&self, name: &str) -> Option<Value> {
        let scope_start = self.scope_marks.last().copied().unwrap_or(0);
        let len = self.locals.len();
        let mut i = len;
        while i > scope_start {
            i -= 1;
            if self.locals[i].0.as_str() == name {
                return Some(self.locals[i].1.clone());
            }
        }
        None
    }

    /// Set the LAST occurrence of a variable (update in place).
    #[inline(always)]
    pub fn set(&mut self, name: &str, value: Value) -> bool {
        let len = self.locals.len();
        let mut i = len;
        while i > 0 {
            i -= 1;
            if self.locals[i].0.as_str() == name {
                self.locals[i].1 = value;
                return true;
            }
        }
        if let Some(v) = self.globals.get_mut(name) {
            *v = value;
            return true;
        }
        false
    }

    #[inline(always)]
    pub fn define_or_set(&mut self, name: &str, value: Value) {
        // Try to update existing local
        let len = self.locals.len();
        let mut i = len;
        while i > 0 {
            i -= 1;
            if self.locals[i].0.as_str() == name {
                self.locals[i].1 = value;
                return;
            }
        }
        if let Some(v) = self.globals.get_mut(name) {
            *v = value;
            return;
        }
        self.define(name, value);
    }

    pub fn snapshot(&self) -> Environment { self.clone() }

    pub fn global_only(&self) -> Environment {
        Environment {
            globals: self.globals.clone(),
            locals: Vec::new(),
            scope_marks: Vec::new(),
        }
    }

    pub fn depth(&self) -> usize { self.scope_marks.len() + 1 }

    pub fn all_named_values(&self) -> Vec<(String, Value)> {
        let mut seen = std::collections::HashSet::new();
        let mut result = Vec::new();
        for i in (0..self.locals.len()).rev() {
            if seen.insert(self.locals[i].0.clone()) {
                result.push((self.locals[i].0.clone(), self.locals[i].1.clone()));
            }
        }
        for (name, val) in &self.globals {
            if seen.insert(name.clone()) {
                result.push((name.clone(), val.clone()));
            }
        }
        result.sort_by(|a, b| a.0.cmp(&b.0));
        result
    }

    pub fn all_values(&self) -> Vec<Value> {
        let mut seen = std::collections::HashSet::new();
        let mut result = Vec::new();
        for i in (0..self.locals.len()).rev() {
            if seen.insert(self.locals[i].0.clone()) {
                result.push(self.locals[i].1.clone());
            }
        }
        for (name, val) in &self.globals {
            if seen.insert(name.clone()) {
                result.push(val.clone());
            }
        }
        result
    }

    pub fn all_instances(&self) -> Vec<Value> {
        self.all_values().into_iter().filter(|v| matches!(v, Value::Instance(_))).collect()
    }
}

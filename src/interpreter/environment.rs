use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::interpreter::values::Value;

/// High-performance variable scope using a flat Vec with scope boundary markers.
///
/// Instead of Vec<HashMap>, we use a single Vec<(String, Value)> with scope
/// boundaries tracked separately. This avoids HashMap allocation/hashing on
/// every push_scope/pop_scope and makes variable lookup a simple linear scan
/// (which is faster than HashMap for scopes with < ~20 variables).
///
/// For the global scope (which may be large with builtins), we keep a HashMap.
#[derive(Debug, Clone)]
pub struct Environment {
    /// Global scope — HashMap for O(1) lookup of builtins.
    globals: HashMap<String, Value>,
    /// Local variable stack — flat array of (name, value) pairs.
    locals: Vec<(String, Value)>,
    /// Scope boundaries — each entry is the locals.len() at the time of push_scope.
    scope_marks: Vec<usize>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
            locals: Vec::new(),
            scope_marks: Vec::new(),
        }
    }

    pub fn new_shared() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self::new()))
    }

    /// Push a new scope — just record the current stack position.
    #[inline]
    pub fn push_scope(&mut self) {
        self.scope_marks.push(self.locals.len());
    }

    /// Pop the top scope — truncate locals back to the mark.
    #[inline]
    pub fn pop_scope(&mut self) {
        if let Some(mark) = self.scope_marks.pop() {
            self.locals.truncate(mark);
        }
    }

    /// Define a variable in the current (innermost) scope.
    #[inline]
    pub fn define(&mut self, name: &str, value: Value) {
        if self.scope_marks.is_empty() {
            // No local scopes — define in globals
            self.globals.insert(name.to_string(), value);
        } else {
            // Check if variable already exists in current scope (update it)
            let scope_start = *self.scope_marks.last().unwrap();
            for i in (scope_start..self.locals.len()).rev() {
                if self.locals[i].0 == name {
                    self.locals[i].1 = value;
                    return;
                }
            }
            // New variable in current scope
            self.locals.push((name.to_string(), value));
        }
    }

    /// Get a variable — search locals (innermost first), then globals.
    #[inline]
    pub fn get(&self, name: &str) -> Option<Value> {
        // Search locals from top of stack backwards (innermost scope first)
        for i in (0..self.locals.len()).rev() {
            if self.locals[i].0 == name {
                return Some(self.locals[i].1.clone());
            }
        }
        // Then check globals
        self.globals.get(name).cloned()
    }

    /// Set an existing variable (finds it in the nearest scope that has it).
    pub fn set(&mut self, name: &str, value: Value) -> bool {
        // Search locals first
        for i in (0..self.locals.len()).rev() {
            if self.locals[i].0 == name {
                self.locals[i].1 = value;
                return true;
            }
        }
        // Then globals
        if self.globals.contains_key(name) {
            self.globals.insert(name.to_string(), value);
            return true;
        }
        false
    }

    /// Define or set — if variable exists anywhere, update it; otherwise define in current scope.
    #[inline]
    pub fn define_or_set(&mut self, name: &str, value: Value) {
        // Try to set in existing scope first
        for i in (0..self.locals.len()).rev() {
            if self.locals[i].0 == name {
                self.locals[i].1 = value;
                return;
            }
        }
        if self.globals.contains_key(name) {
            self.globals.insert(name.to_string(), value);
            return;
        }
        // Not found — define in current scope
        self.define(name, value);
    }

    /// Get a snapshot of the current environment for closure capture.
    pub fn snapshot(&self) -> Environment {
        self.clone()
    }

    /// Return a new environment containing only the global scope.
    pub fn global_only(&self) -> Environment {
        Environment {
            globals: self.globals.clone(),
            locals: Vec::new(),
            scope_marks: Vec::new(),
        }
    }

    /// Get the number of scopes (for debugging).
    pub fn depth(&self) -> usize {
        self.scope_marks.len() + 1 // +1 for global
    }

    /// Collect all (name, value) pairs visible in the current environment.
    pub fn all_named_values(&self) -> Vec<(String, Value)> {
        let mut seen = std::collections::HashSet::new();
        let mut result = Vec::new();
        // Locals first (innermost)
        for i in (0..self.locals.len()).rev() {
            if seen.insert(self.locals[i].0.clone()) {
                result.push((self.locals[i].0.clone(), self.locals[i].1.clone()));
            }
        }
        // Then globals
        for (name, val) in &self.globals {
            if seen.insert(name.clone()) {
                result.push((name.clone(), val.clone()));
            }
        }
        result.sort_by(|a, b| a.0.cmp(&b.0));
        result
    }

    /// Collect all values (for atomic snapshots).
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

    /// Collect all Instance values (for mutation.atomic).
    pub fn all_instances(&self) -> Vec<Value> {
        self.all_values().into_iter().filter(|v| matches!(v, Value::Instance(_))).collect()
    }
}

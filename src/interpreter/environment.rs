use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::interpreter::values::Value;

/// Variable scope — a stack of hash maps.
/// Each scope level maps variable names to values.
#[derive(Debug, Clone)]
pub struct Environment {
    scopes: Vec<HashMap<String, Value>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()], // global scope
        }
    }

    /// Create an environment wrapped in Rc<RefCell> for sharing.
    pub fn new_shared() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self::new()))
    }

    /// Push a new scope onto the stack.
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Pop the top scope.
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Define a new variable in the current (top) scope.
    pub fn define(&mut self, name: &str, value: Value) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), value);
        }
    }

    /// Get a variable, searching from innermost scope outward.
    pub fn get(&self, name: &str) -> Option<Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Some(val.clone());
            }
        }
        None
    }

    /// Set an existing variable (finds it in the nearest scope that has it).
    /// Returns true if the variable was found and set.
    pub fn set(&mut self, name: &str, value: Value) -> bool {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return true;
            }
        }
        false
    }

    /// Define or set — if variable exists, update it; otherwise define in current scope.
    pub fn define_or_set(&mut self, name: &str, value: Value) {
        if !self.set(name, value.clone()) {
            self.define(name, value);
        }
    }

    /// Get a snapshot of the current environment for closure capture.
    pub fn snapshot(&self) -> Environment {
        self.clone()
    }

    /// Return a new environment containing only the outermost (global) scope.
    /// Used when we want a cheap environment for evaluating fitness functions
    /// that only reference `self` and literals, not the full call stack.
    pub fn global_only(&self) -> Environment {
        let global = self.scopes.first().cloned().unwrap_or_default();
        Environment {
            scopes: vec![global],
        }
    }

    /// Get the number of scopes (for debugging).
    pub fn depth(&self) -> usize {
        self.scopes.len()
    }

    /// Collect all values in this environment (for atomic snapshots).
    pub fn all_values(&self) -> Vec<Value> {
        let mut seen_names = std::collections::HashSet::new();
        let mut result = Vec::new();
        for scope in self.scopes.iter().rev() {
            for (name, val) in scope {
                if seen_names.insert(name.clone()) {
                    result.push(val.clone());
                }
            }
        }
        result
    }

    /// Collect all Instance values currently visible in this environment.
    /// Used by mutation.atomic to snapshot instances for rollback.
    pub fn all_instances(&self) -> Vec<Value> {
        let mut seen_names = std::collections::HashSet::new();
        let mut result = Vec::new();
        for scope in self.scopes.iter().rev() {
            for (name, val) in scope {
                if seen_names.insert(name.clone()) {
                    if matches!(val, Value::Instance(_)) {
                        result.push(val.clone());
                    }
                }
            }
        }
        result
    }
}

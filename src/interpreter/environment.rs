use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::interpreter::values::Value;

/// High-performance variable scope with slot-indexed locals.
///
/// Two modes:
/// 1. SLOT MODE (fast path): local_slots is a pre-sized Vec<Value> indexed by integer.
///    Used by the function call fast path when slot count is known.
///    get_slot(i) / set_slot(i, v) are O(1) — no strings involved.
///
/// 2. NAME MODE (fallback): locals is a Vec<(String, Value)> scanned backwards.
///    Used for closures, REPL, and any code not yet slot-resolved.
///
/// Globals always use HashMap (accessed infrequently after startup).
#[derive(Debug, Clone)]
pub struct Environment {
    pub globals: HashMap<String, Value>,
    locals: Vec<(String, Value)>,
    scope_marks: Vec<usize>,
    /// Slot-indexed frame for the current function call (fast path).
    pub local_slots: Vec<Value>,
    /// Name-to-slot-index map for the current function.
    slot_names: Option<HashMap<String, usize>>,
    /// Saved slot frames from outer function calls.
    slot_stack: Vec<(Vec<Value>, Option<HashMap<String, usize>>)>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            globals: HashMap::with_capacity(128),
            locals: Vec::with_capacity(64),
            scope_marks: Vec::with_capacity(16),
            local_slots: Vec::new(),
            slot_names: None,
            slot_stack: Vec::with_capacity(16),
        }
    }

    pub fn new_shared() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self::new()))
    }

    // ── Slot-indexed operations (fast path) ──────────────

    /// Push a new slot frame with name map (for function call).
    #[inline(always)]
    pub fn push_slot_frame_with_names(&mut self, slot_count: usize, names: Option<HashMap<String, usize>>) {
        let old_slots = std::mem::replace(&mut self.local_slots, vec![Value::Nil; slot_count]);
        let old_names = std::mem::replace(&mut self.slot_names, names);
        self.slot_stack.push((old_slots, old_names));
    }

    /// Push a new slot frame with the given capacity.
    #[inline(always)]
    pub fn push_slot_frame(&mut self, slot_count: usize) {
        self.push_slot_frame_with_names(slot_count, None);
    }

    /// Pop the current slot frame, restoring the caller's frame.
    #[inline(always)]
    pub fn pop_slot_frame(&mut self) {
        if let Some((old_slots, old_names)) = self.slot_stack.pop() {
            self.local_slots = old_slots;
            self.slot_names = old_names;
        } else {
            self.local_slots.clear();
            self.slot_names = None;
        }
    }

    /// Find slot index by name. O(n) but n is typically 1-5.
    #[inline(always)]
    pub fn find_slot(&self, name: &str) -> Option<usize> {
        self.slot_names.as_ref().and_then(|m| m.get(name).copied())
    }

    /// Get a value by slot index. O(1).
    #[inline(always)]
    pub fn get_slot(&self, index: usize) -> &Value {
        &self.local_slots[index]
    }

    /// Set a value by slot index. O(1).
    #[inline(always)]
    pub fn set_slot(&mut self, index: usize, value: Value) {
        self.local_slots[index] = value;
    }

    /// Check if we're in slot mode (have an active slot frame).
    #[inline(always)]
    pub fn has_slots(&self) -> bool {
        !self.local_slots.is_empty()
    }

    // ── Name-based operations (fallback) ─────────────────

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

    #[inline(always)]
    pub fn define(&mut self, name: &str, value: Value) {
        if self.scope_marks.is_empty() {
            self.globals.insert(name.to_string(), value);
        } else {
            let scope_start = unsafe { *self.scope_marks.last().unwrap_unchecked() };
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

    #[inline(always)]
    pub fn define_new(&mut self, name: &str, value: Value) {
        if self.scope_marks.is_empty() {
            self.globals.insert(name.to_string(), value);
        } else {
            self.locals.push((name.to_string(), value));
        }
    }

    #[inline(always)]
    pub fn get(&self, name: &str) -> Option<Value> {
        // Check slot frame first (slot names stored by convention)
        // Then check named locals
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
            local_slots: Vec::new(),
            slot_names: None,
            slot_stack: Vec::new(),
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

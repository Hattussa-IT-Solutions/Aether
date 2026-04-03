use std::collections::HashMap;
use crate::parser::ast::*;

/// Slot map: for each function (by name), maps variable names to slot indices.
/// Also stores the total slot count per function.
#[derive(Debug, Clone, Default)]
pub struct SlotMap {
    /// function_name -> { variable_name -> slot_index }
    pub slots: HashMap<String, HashMap<String, usize>>,
    /// function_name -> total slot count
    pub counts: HashMap<String, usize>,
    /// Top-level (global) slot map for the main script body
    pub global_slots: HashMap<String, usize>,
    pub global_count: usize,
}

impl SlotMap {
    /// Get the slot index for a variable in a function.
    #[inline(always)]
    pub fn get_slot(&self, func_name: &str, var_name: &str) -> Option<usize> {
        self.slots.get(func_name).and_then(|m| m.get(var_name).copied())
    }

    /// Get total slot count for a function.
    #[inline(always)]
    pub fn get_count(&self, func_name: &str) -> usize {
        self.counts.get(func_name).copied().unwrap_or(0)
    }
}

/// Resolve slot indices for all functions in a program.
pub fn resolve(program: &Program) -> SlotMap {
    let mut map = SlotMap::default();

    // Resolve top-level variables
    let mut global_idx = 0;
    for stmt in &program.statements {
        collect_vars_stmt(stmt, &mut map.global_slots, &mut global_idx);
    }
    map.global_count = global_idx;

    // Resolve each function
    for stmt in &program.statements {
        resolve_stmt(stmt, &mut map);
    }

    map
}

fn resolve_stmt(stmt: &Stmt, map: &mut SlotMap) {
    match &stmt.kind {
        StmtKind::FuncDef(fd) => {
            let mut var_map = HashMap::new();
            let mut idx = 0;

            // Parameters get the first slots
            for param in &fd.params {
                var_map.insert(param.name.clone(), idx);
                idx += 1;
            }

            // Walk the body to find all local variable assignments
            match &fd.body {
                FuncBody::Block(stmts) => {
                    for s in stmts {
                        collect_vars_stmt(s, &mut var_map, &mut idx);
                    }
                }
                FuncBody::Expression(_) => {}
            }

            map.counts.insert(fd.name.clone(), idx);
            map.slots.insert(fd.name.clone(), var_map);

            // Resolve nested functions
            match &fd.body {
                FuncBody::Block(stmts) => {
                    for s in stmts {
                        resolve_stmt(s, map);
                    }
                }
                _ => {}
            }
        }
        StmtKind::ClassDef(cd) => {
            // Resolve methods
            for method in &cd.methods {
                let mut var_map = HashMap::new();
                let mut idx = 0;

                // "self" is always slot 0 for methods
                var_map.insert("self".to_string(), idx);
                idx += 1;

                for param in &method.params {
                    var_map.insert(param.name.clone(), idx);
                    idx += 1;
                }

                match &method.body {
                    FuncBody::Block(stmts) => {
                        for s in stmts {
                            collect_vars_stmt(s, &mut var_map, &mut idx);
                        }
                    }
                    FuncBody::Expression(_) => {}
                }

                let method_key = format!("{}.{}", cd.name, method.name);
                map.counts.insert(method_key.clone(), idx);
                map.slots.insert(method_key, var_map);
            }

            // Resolve init
            if let Some(init_fn) = &cd.init {
                let mut var_map = HashMap::new();
                let mut idx = 0;
                var_map.insert("self".to_string(), idx);
                idx += 1;
                for param in &init_fn.params {
                    var_map.insert(param.name.clone(), idx);
                    idx += 1;
                }
                match &init_fn.body {
                    FuncBody::Block(stmts) => {
                        for s in stmts {
                            collect_vars_stmt(s, &mut var_map, &mut idx);
                        }
                    }
                    _ => {}
                }
                let init_key = format!("{}.init", cd.name);
                map.counts.insert(init_key.clone(), idx);
                map.slots.insert(init_key, var_map);
            }
        }
        StmtKind::If { then_block, else_if_blocks, else_block, .. } => {
            for s in then_block { resolve_stmt(s, map); }
            for (_, block) in else_if_blocks { for s in block { resolve_stmt(s, map); } }
            if let Some(eb) = else_block { for s in eb { resolve_stmt(s, map); } }
        }
        StmtKind::ForLoop { body, .. } => {
            for s in body { resolve_stmt(s, map); }
        }
        StmtKind::Loop { body, .. } => {
            for s in body { resolve_stmt(s, map); }
        }
        StmtKind::Block(stmts) => {
            for s in stmts { resolve_stmt(s, map); }
        }
        _ => {}
    }
}

/// Collect all variable names assigned in a statement, adding them to the slot map.
fn collect_vars_stmt(stmt: &Stmt, var_map: &mut HashMap<String, usize>, idx: &mut usize) {
    match &stmt.kind {
        StmtKind::VarDecl { name, .. } => {
            if !var_map.contains_key(name) {
                var_map.insert(name.clone(), *idx);
                *idx += 1;
            }
        }
        StmtKind::Assignment { target, .. } => {
            if let ExprKind::Identifier(name) = &target.kind {
                if !var_map.contains_key(name) {
                    var_map.insert(name.clone(), *idx);
                    *idx += 1;
                }
            }
        }
        StmtKind::ForLoop { pattern, body, .. } => {
            match pattern {
                ForPattern::Single(name) => {
                    if !var_map.contains_key(name) {
                        var_map.insert(name.clone(), *idx);
                        *idx += 1;
                    }
                }
                ForPattern::Enumerate(a, b) => {
                    if !var_map.contains_key(a) { var_map.insert(a.clone(), *idx); *idx += 1; }
                    if !var_map.contains_key(b) { var_map.insert(b.clone(), *idx); *idx += 1; }
                }
                ForPattern::Destructure(names) => {
                    for name in names {
                        if !var_map.contains_key(name) { var_map.insert(name.clone(), *idx); *idx += 1; }
                    }
                }
            }
            for s in body { collect_vars_stmt(s, var_map, idx); }
        }
        StmtKind::If { then_block, else_if_blocks, else_block, .. } => {
            for s in then_block { collect_vars_stmt(s, var_map, idx); }
            for (_, block) in else_if_blocks { for s in block { collect_vars_stmt(s, var_map, idx); } }
            if let Some(eb) = else_block { for s in eb { collect_vars_stmt(s, var_map, idx); } }
        }
        StmtKind::Loop { body, .. } => {
            for s in body { collect_vars_stmt(s, var_map, idx); }
        }
        StmtKind::Block(stmts) => {
            for s in stmts { collect_vars_stmt(s, var_map, idx); }
        }
        StmtKind::TryCatch { try_block, catches, finally_block, .. } => {
            for s in try_block { collect_vars_stmt(s, var_map, idx); }
            for c in catches {
                if let Some(binding) = &c.binding {
                    if !var_map.contains_key(binding) { var_map.insert(binding.clone(), *idx); *idx += 1; }
                }
                for s in &c.body { collect_vars_stmt(s, var_map, idx); }
            }
            if let Some(fb) = finally_block { for s in fb { collect_vars_stmt(s, var_map, idx); } }
        }
        _ => {}
    }
}

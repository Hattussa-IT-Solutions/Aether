use std::collections::HashMap;
use std::rc::Rc;

use crate::interpreter::environment::Environment;
use crate::interpreter::eval::*;
use crate::interpreter::values::*;
use crate::parser::ast::*;

/// Execute a block of statements.
pub fn exec_block(stmts: &[Stmt], env: &mut Environment) -> Result<(), Signal> {
    for stmt in stmts {
        exec_stmt(stmt, env)?;
    }
    Ok(())
}

/// Execute a block and return the value of the last expression statement (implicit return).
pub fn exec_block_with_value(stmts: &[Stmt], env: &mut Environment) -> Result<Value, Signal> {
    let mut last_val = Value::Nil;
    for stmt in stmts {
        match &stmt.kind {
            StmtKind::Expression(expr) => {
                last_val = eval_expr(expr, env)?;
            }
            StmtKind::Match { value, arms } => {
                let val = eval_expr(value, env)?;
                last_val = eval_match(&val, arms, env)?;
            }
            _ => {
                exec_stmt(stmt, env)?;
                last_val = Value::Nil;
            }
        }
    }
    Ok(last_val)
}

/// Execute a single statement.
pub fn exec_stmt(stmt: &Stmt, env: &mut Environment) -> Result<(), Signal> {
    match &stmt.kind {
        StmtKind::Expression(expr) => {
            eval_expr(expr, env)?;
            Ok(())
        }

        StmtKind::VarDecl { name, value, mutable: _, is_const: _, .. } => {
            let val = if let Some(expr) = value {
                let v = eval_expr(expr, env)?;
                // Struct copy-on-assign: deep clone struct instances
                if let Value::StructInstance(inst) = &v {
                    Value::StructInstance(std::rc::Rc::new(std::cell::RefCell::new(inst.borrow().clone())))
                } else {
                    v
                }
            } else {
                Value::Nil
            };
            env.define_or_set(name, val);
            Ok(())
        }

        StmtKind::Assignment { target, op, value } => {
            let new_val = eval_expr(value, env)?;
            exec_assignment(target, op, new_val, env)
        }

        StmtKind::FuncDef(fd) => {
            let func = Value::Function(Rc::new(FunctionValue {
                name: fd.name.clone(),
                params: fd.params.clone(),
                body: fd.body.clone(),
                closure_env: None,
                is_method: false,
            }));
            env.define(&fd.name, func);
            Ok(())
        }

        StmtKind::If { condition, then_block, else_if_blocks, else_block } => {
            let cond = eval_expr(condition, env)?;
            if cond.is_truthy() {
                env.push_scope();
                let result = exec_block(then_block, env);
                env.pop_scope();
                return result;
            }
            for (cond_expr, block) in else_if_blocks {
                let c = eval_expr(cond_expr, env)?;
                if c.is_truthy() {
                    env.push_scope();
                    let result = exec_block(block, env);
                    env.pop_scope();
                    return result;
                }
            }
            if let Some(else_blk) = else_block {
                env.push_scope();
                let result = exec_block(else_blk, env);
                env.pop_scope();
                return result;
            }
            Ok(())
        }

        StmtKind::Guard { pattern, value, else_block } => {
            let val = eval_expr(value, env)?;
            // guard let fails if value is nil or pattern doesn't match
            let should_fail = matches!(val, Value::Nil) || !match_pattern(&val, pattern, env);
            if should_fail {
                env.push_scope();
                let result = exec_block(else_block, env);
                env.pop_scope();
                return result;
            }
            // Pattern matched and value is not nil — bindings are in env
            Ok(())
        }

        StmtKind::IfLet { pattern, value, then_block, else_block } => {
            let val = eval_expr(value, env)?;
            if match_pattern(&val, pattern, env) {
                env.push_scope();
                let result = exec_block(then_block, env);
                env.pop_scope();
                result
            } else if let Some(else_blk) = else_block {
                env.push_scope();
                let result = exec_block(else_blk, env);
                env.pop_scope();
                result
            } else {
                Ok(())
            }
        }

        StmtKind::Match { value, arms } => {
            let val = eval_expr(value, env)?;
            eval_match(&val, arms, env)?;
            Ok(())
        }

        StmtKind::ForLoop { label, pattern, iterable, step, body, .. } => {
            let iter_val = eval_expr(iterable, env)?;
            let items = value_to_iter(&iter_val)?;

            let step_size = if let Some(step_expr) = step {
                eval_expr(step_expr, env)?.as_int().unwrap_or(1) as usize
            } else { 1 };

            env.push_scope();
            let mut idx = 0;
            for item in items.iter().step_by(step_size) {
                match pattern {
                    ForPattern::Single(name) => {
                        env.define(name, item.clone());
                    }
                    ForPattern::Enumerate(i_name, v_name) => {
                        env.define(i_name, Value::Int(idx as i64));
                        env.define(v_name, item.clone());
                    }
                    ForPattern::Destructure(names) => {
                        if let Value::Tuple(vals) = item {
                            for (i, name) in names.iter().enumerate() {
                                env.define(name, vals.get(i).cloned().unwrap_or(Value::Nil));
                            }
                        }
                    }
                }

                match exec_block(body, env) {
                    Ok(()) => {}
                    Err(Signal::Break(ref l)) if l == label || l.is_none() => break,
                    Err(Signal::Next(ref l)) if l == label || l.is_none() => { idx += 1; continue; }
                    Err(e) => { env.pop_scope(); return Err(e); }
                }
                idx += 1;
            }
            env.pop_scope();
            Ok(())
        }

        StmtKind::Loop { label, kind, body, until_condition } => {
            env.push_scope();
            match kind {
                LoopKind::Times(count_expr) => {
                    let count = eval_expr(count_expr, env)?.as_int().unwrap_or(0);
                    for _ in 0..count {
                        match exec_block(body, env) {
                            Ok(()) => {}
                            Err(Signal::Break(ref l)) if l == label || l.is_none() => break,
                            Err(Signal::Next(ref l)) if l == label || l.is_none() => continue,
                            Err(e) => { env.pop_scope(); return Err(e); }
                        }
                    }
                }
                LoopKind::While(cond_expr) => {
                    loop {
                        let cond = eval_expr(cond_expr, env)?;
                        if !cond.is_truthy() { break; }
                        match exec_block(body, env) {
                            Ok(()) => {}
                            Err(Signal::Break(ref l)) if l == label || l.is_none() => break,
                            Err(Signal::Next(ref l)) if l == label || l.is_none() => continue,
                            Err(e) => { env.pop_scope(); return Err(e); }
                        }
                    }
                }
                LoopKind::Infinite => {
                    loop {
                        match exec_block(body, env) {
                            Ok(()) => {}
                            Err(Signal::Break(ref l)) if l == label || l.is_none() => break,
                            Err(Signal::Next(ref l)) if l == label || l.is_none() => continue,
                            Err(e) => { env.pop_scope(); return Err(e); }
                        }
                        if let Some(until_cond) = until_condition {
                            let c = eval_expr(until_cond, env)?;
                            if c.is_truthy() { break; }
                        }
                    }
                }
            }
            env.pop_scope();
            Ok(())
        }

        StmtKind::Break { label } => Err(Signal::Break(label.clone())),
        StmtKind::Next { label, condition } => {
            if let Some(cond) = condition {
                let val = eval_expr(cond, env)?;
                if val.is_truthy() {
                    Err(Signal::Next(label.clone()))
                } else {
                    Ok(())
                }
            } else {
                Err(Signal::Next(label.clone()))
            }
        }
        StmtKind::Return(val_expr) => {
            let val = if let Some(expr) = val_expr {
                eval_expr(expr, env)?
            } else {
                Value::Nil
            };
            Err(Signal::Return(val))
        }
        StmtKind::Throw(expr) => {
            let val = eval_expr(expr, env)?;
            Err(Signal::Throw(val))
        }

        StmtKind::TryCatch { try_block, catches, finally_block } => {
            env.push_scope();
            let result = exec_block(try_block, env);
            env.pop_scope();

            match result {
                Ok(()) => {}
                Err(Signal::Throw(err_val)) => {
                    let mut caught = false;
                    for catch in catches {
                        // Simple type matching — match "any" or by error string type
                        let type_matches = match &catch.error_type {
                            None => true,
                            Some(t) if t == "any" => true,
                            Some(t) => {
                                // Match if the error value's "type" matches
                                match &err_val {
                                    Value::Instance(inst) => inst.borrow().class_name == *t,
                                    Value::String(_) => t == "Str" || t == "Error",
                                    _ => true,
                                }
                            }
                        };
                        if type_matches {
                            env.push_scope();
                            if let Some(binding) = &catch.binding {
                                env.define(binding, err_val.clone());
                            }
                            let catch_result = exec_block(&catch.body, env);
                            env.pop_scope();
                            caught = true;
                            if let Err(e) = catch_result {
                                if let Some(finally) = finally_block {
                                    env.push_scope();
                                    let _ = exec_block(finally, env);
                                    env.pop_scope();
                                }
                                return Err(e);
                            }
                            break;
                        }
                    }
                    if !caught {
                        if let Some(finally) = finally_block {
                            env.push_scope();
                            let _ = exec_block(finally, env);
                            env.pop_scope();
                        }
                        return Err(Signal::Throw(err_val));
                    }
                }
                Err(e) => {
                    if let Some(finally) = finally_block {
                        env.push_scope();
                        let _ = exec_block(finally, env);
                        env.pop_scope();
                    }
                    return Err(e);
                }
            }

            if let Some(finally) = finally_block {
                env.push_scope();
                exec_block(finally, env)?;
                env.pop_scope();
            }
            Ok(())
        }

        StmtKind::ClassDef(cd) => {
            exec_class_def(cd, env)
        }

        StmtKind::StructDef(sd) => {
            let mut methods = HashMap::new();
            for method in &sd.methods {
                methods.insert(method.name.clone(), Rc::new(FunctionValue {
                    name: method.name.clone(),
                    params: method.params.clone(),
                    body: method.body.clone(),
                    closure_env: None,
                    is_method: true,
                }));
            }
            let struct_val = Value::StructDef(Rc::new(StructDefValue {
                name: sd.name.clone(),
                fields: sd.fields.clone(),
                methods,
            }));
            env.define(&sd.name, struct_val);
            Ok(())
        }

        StmtKind::EnumDef(ed) => {
            let mut methods = HashMap::new();
            for method in &ed.methods {
                methods.insert(method.name.clone(), Rc::new(FunctionValue {
                    name: method.name.clone(),
                    params: method.params.clone(),
                    body: method.body.clone(),
                    closure_env: None,
                    is_method: true,
                }));
            }
            let enum_val = Value::EnumDef(Rc::new(EnumDefValue {
                name: ed.name.clone(),
                variants: ed.variants.clone(),
                methods,
            }));
            env.define(&ed.name, enum_val);
            Ok(())
        }

        StmtKind::InterfaceDef(idef) => {
            // Store interface as a list of required method names
            let mut required_methods: Vec<Value> = Vec::new();
            for m in &idef.methods {
                if m.default_body.is_none() {
                    required_methods.push(Value::String(m.name.clone()));
                }
            }
            env.define(&idef.name, Value::List(
                std::rc::Rc::new(std::cell::RefCell::new(required_methods))
            ));
            Ok(())
        }

        StmtKind::Use { path, alias } => {
            // Basic module loading — for now just register the path as a namespace
            let name = alias.as_ref().unwrap_or_else(|| path.last().unwrap());
            env.define(name, Value::String(format!("module:{}", path.join("."))));
            Ok(())
        }

        StmtKind::ModBlock { name, body } => {
            env.push_scope();
            exec_block(body, env)?;
            env.pop_scope();
            // Module exports not implemented yet
            env.define(name, Value::String(format!("module:{}", name)));
            Ok(())
        }

        StmtKind::TypeAlias { .. } => Ok(()),

        StmtKind::Parallel { tasks, timeout, max_concurrency, is_race } => {
            crate::interpreter::parallel::exec_parallel(tasks, timeout, max_concurrency, *is_race, env)
        }

        StmtKind::After { body, .. } => {
            env.push_scope();
            let result = exec_block(body, env);
            env.pop_scope();
            result
        }

        StmtKind::MutationAtomic { body } => {
            // Snapshot all instance fields in all reachable variables for rollback
            let snapshots: Vec<(std::rc::Rc<std::cell::RefCell<InstanceValue>>, HashMap<String, Value>)> =
                env.all_values().into_iter().filter_map(|v| {
                    if let Value::Instance(inst_rc) = v {
                        let field_snap: HashMap<String, Value> = inst_rc.borrow().fields.clone();
                        Some((inst_rc, field_snap))
                    } else {
                        None
                    }
                }).collect();

            env.push_scope();
            let result = exec_block(body, env);
            env.pop_scope();

            if result.is_err() {
                // Restore all snapshots on failure
                for (inst_rc, snap) in &snapshots {
                    inst_rc.borrow_mut().fields = snap.clone();
                }
            }

            result
        }

        StmtKind::Device { body, .. } => {
            env.push_scope();
            let result = exec_block(body, env);
            env.pop_scope();
            result
        }

        StmtKind::WeaveDef(wd) => {
            crate::interpreter::values::register_weave(&wd.name, wd.clone());
            Ok(())
        }

        StmtKind::ExtendBlock(eb) => {
            // Determine target type name from the TypeAnnotation
            let type_name = match &eb.target {
                TypeAnnotation::Simple(name) => name.clone(),
                TypeAnnotation::Generic(name, _) => name.clone(),
                _ => return Ok(()),
            };
            crate::interpreter::values::register_extension(&type_name, eb.methods.clone());
            Ok(())
        }

        StmtKind::Block(stmts) => {
            env.push_scope();
            let result = exec_block(stmts, env);
            env.pop_scope();
            result
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Assignment
// ═══════════════════════════════════════════════════════════════

fn exec_assignment(target: &Expr, op: &AssignOp, new_val: Value, env: &mut Environment) -> Result<(), Signal> {
    match &target.kind {
        ExprKind::Identifier(name) => {
            let final_val = if *op == AssignOp::Assign {
                new_val
            } else {
                let old = env.get(name).unwrap_or(Value::Nil);
                apply_compound_op(&old, op, &new_val)?
            };
            env.define_or_set(name, final_val);
            Ok(())
        }
        ExprKind::FieldAccess { object, field } => {
            let obj = eval_expr(object, env)?;
            match obj {
                Value::Instance(inst) => {
                    // Freeze check: reject assignment if instance is frozen
                    if inst.borrow().fields.get("__frozen").map(|v| matches!(v, Value::Bool(true))).unwrap_or(false) {
                        return Err(Signal::Throw(Value::String(format!(
                            "cannot assign field '{}' on a frozen instance", field
                        ))));
                    }

                    let old = inst.borrow().fields.get(field).cloned().unwrap_or(Value::Nil);
                    let final_val = if *op == AssignOp::Assign {
                        new_val
                    } else {
                        apply_compound_op(&old, op, &new_val)?
                    };

                    // Temporal property: push old value to history ring buffer
                    {
                        let cls_opt = inst.borrow().class.clone();
                        if let Some(cls) = cls_opt {
                            // Temporal tracking
                            if let Some(&(keep, _)) = cls.temporal_props.get(field.as_str()) {
                                let hist_key = format!("__temporal_hist_{}", field);
                                let prev_key = format!("__temporal_prev_{}", field);
                                // Push old value to history
                                let hist = inst.borrow().fields.get(&hist_key).cloned()
                                    .unwrap_or_else(|| Value::List(std::rc::Rc::new(std::cell::RefCell::new(Vec::new()))));
                                if let Value::List(ref list) = hist {
                                    let mut v = list.borrow_mut();
                                    v.push(old.clone());
                                    if v.len() > keep {
                                        v.remove(0);
                                    }
                                }
                                inst.borrow_mut().fields.insert(prev_key, old.clone());
                                inst.borrow_mut().fields.insert(hist_key, hist);
                            }
                            // Mutation tracking
                            if cls.mutation_tracked.contains(field) {
                                let mut_key = "__mutations".to_string();
                                let mutations = inst.borrow().fields.get(&mut_key).cloned()
                                    .unwrap_or_else(|| Value::List(std::rc::Rc::new(std::cell::RefCell::new(Vec::new()))));
                                if let Value::List(ref list) = mutations {
                                    use std::time::{SystemTime, UNIX_EPOCH};
                                    let ts = SystemTime::now()
                                        .duration_since(UNIX_EPOCH)
                                        .map(|d| d.as_secs() as i64)
                                        .unwrap_or(0);
                                    let mut entry = std::collections::HashMap::new();
                                    entry.insert("old".to_string(), old.clone());
                                    entry.insert("new".to_string(), final_val.clone());
                                    entry.insert("field".to_string(), Value::String(field.clone()));
                                    entry.insert("timestamp".to_string(), Value::Int(ts));
                                    list.borrow_mut().push(Value::Map(std::rc::Rc::new(std::cell::RefCell::new(entry))));
                                }
                                inst.borrow_mut().fields.insert(mut_key, mutations);
                            }
                            // Mutation undoable: maintain undo history
                            if let Some(&(depth, _)) = cls.mutation_undoable.get(field.as_str()) {
                                let undo_key = format!("__undo_{}", field);
                                let undo_hist = inst.borrow().fields.get(&undo_key).cloned()
                                    .unwrap_or_else(|| Value::List(std::rc::Rc::new(std::cell::RefCell::new(Vec::new()))));
                                if let Value::List(ref list) = undo_hist {
                                    let mut v = list.borrow_mut();
                                    v.push(old.clone());
                                    if v.len() > depth {
                                        v.remove(0);
                                    }
                                }
                                inst.borrow_mut().fields.insert(undo_key, undo_hist);
                                // Clear redo buffer on new assignment
                                let redo_key = format!("__redo_{}", field);
                                inst.borrow_mut().fields.insert(redo_key,
                                    Value::List(std::rc::Rc::new(std::cell::RefCell::new(Vec::new()))));
                            }
                        }
                    }

                    let final_val_clone = final_val.clone();
                    inst.borrow_mut().fields.insert(field.clone(), final_val.clone());

                    // Bond bidirectional sync: when a bond field is assigned, also update the
                    // reciprocal field on the target instance.
                    {
                        let cls_opt = inst.borrow().class.clone();
                        if let Some(cls) = cls_opt {
                            if let Some((_, via_field)) = cls.bonds.get(field.as_str()) {
                                let via_field = via_field.clone();
                                // final_val should be an Instance; set via_field on it to point back
                                if let Value::Instance(target_inst) = &final_val {
                                    let self_val = Value::Instance(inst.clone());
                                    // Check if target is not frozen before writing reciprocal
                                    let target_frozen = target_inst.borrow().fields
                                        .get("__frozen")
                                        .map(|v| matches!(v, Value::Bool(true)))
                                        .unwrap_or(false);
                                    if !target_frozen {
                                        target_inst.borrow_mut().fields.insert(via_field.clone(), self_val);
                                    }
                                }
                            }
                        }
                    }

                    // Observed property: fire did_change callback
                    {
                        let cls_opt = inst.borrow().class.clone();
                        if let Some(cls) = cls_opt {
                            if let Some(did_change_stmts) = cls.observed_props.get(field.as_str()) {
                                if !did_change_stmts.is_empty() {
                                    let stmts = did_change_stmts.clone();
                                    let mut cb_env = env.snapshot();
                                    cb_env.push_scope();
                                    cb_env.define("old", old.clone());
                                    cb_env.define("new", final_val_clone);
                                    cb_env.define("self", Value::Instance(inst.clone()));
                                    let _ = exec_block(&stmts, &mut cb_env);
                                }
                            }
                        }
                    }

                    Ok(())
                }
                Value::StructInstance(inst) => {
                    let final_val = if *op == AssignOp::Assign {
                        new_val
                    } else {
                        let old = inst.borrow().fields.get(field).cloned().unwrap_or(Value::Nil);
                        apply_compound_op(&old, op, &new_val)?
                    };
                    inst.borrow_mut().fields.insert(field.clone(), final_val);
                    Ok(())
                }
                _ => Err(Signal::Throw(Value::String(format!("cannot assign field on {}", obj)))),
            }
        }
        ExprKind::Index { object, index } => {
            let obj = eval_expr(object, env)?;
            let idx = eval_expr(index, env)?;
            match obj {
                Value::List(items) => {
                    let i = idx.as_int().unwrap_or(0) as usize;
                    let mut items = items.borrow_mut();
                    if i < items.len() {
                        let final_val = if *op == AssignOp::Assign {
                            new_val
                        } else {
                            apply_compound_op(&items[i], op, &new_val)?
                        };
                        items[i] = final_val;
                    }
                    Ok(())
                }
                Value::Map(map) => {
                    let key = idx.as_map_key().unwrap_or_default();
                    let final_val = if *op == AssignOp::Assign {
                        new_val
                    } else {
                        let old = map.borrow().get(&key).cloned().unwrap_or(Value::Nil);
                        apply_compound_op(&old, op, &new_val)?
                    };
                    map.borrow_mut().insert(key, final_val);
                    Ok(())
                }
                _ => Err(Signal::Throw(Value::String(format!("cannot index-assign on {}", obj)))),
            }
        }
        _ => Err(Signal::Throw(Value::String("invalid assignment target".into()))),
    }
}

fn apply_compound_op(old: &Value, op: &AssignOp, new_val: &Value) -> Result<Value, Signal> {
    match op {
        AssignOp::Assign => Ok(new_val.clone()),
        AssignOp::AddAssign => {
            match (old, new_val) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + *b as f64)),
                (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                _ => Err(Signal::Throw(Value::String(format!("cannot += {} and {}", old, new_val)))),
            }
        }
        AssignOp::SubAssign => {
            match (old, new_val) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - *b as f64)),
                _ => Err(Signal::Throw(Value::String(format!("cannot -= {} and {}", old, new_val)))),
            }
        }
        AssignOp::MulAssign => {
            match (old, new_val) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a * *b as f64)),
                _ => Err(Signal::Throw(Value::String(format!("cannot *= {} and {}", old, new_val)))),
            }
        }
        AssignOp::DivAssign => {
            match (old, new_val) {
                (Value::Int(a), Value::Int(b)) if *b != 0 => Ok(Value::Int(a / b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
                _ => Err(Signal::Throw(Value::String(format!("cannot /= {} and {}", old, new_val)))),
            }
        }
        AssignOp::ModAssign => {
            match (old, new_val) {
                (Value::Int(a), Value::Int(b)) if *b != 0 => Ok(Value::Int(a % b)),
                _ => Err(Signal::Throw(Value::String(format!("cannot %= {} and {}", old, new_val)))),
            }
        }
        AssignOp::PowAssign => {
            match (old, new_val) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.pow(*b as u32))),
                _ => Err(Signal::Throw(Value::String(format!("cannot **= {} and {}", old, new_val)))),
            }
        }
        _ => Ok(new_val.clone()),
    }
}

// ═══════════════════════════════════════════════════════════════
// Class definition
// ═══════════════════════════════════════════════════════════════

fn exec_class_def(cd: &ClassDef, env: &mut Environment) -> Result<(), Signal> {
    let parent = if let Some(parent_name) = &cd.parent {
        if let Some(Value::Class(parent_cls)) = env.get(parent_name) {
            Some(parent_cls)
        } else {
            None
        }
    } else {
        None
    };

    let mut methods = HashMap::new();
    // Inherit parent methods
    if let Some(ref p) = parent {
        for (name, method) in &p.methods {
            methods.insert(name.clone(), method.clone());
        }
    }
    // Own methods (override parent)
    for method in &cd.methods {
        methods.insert(method.name.clone(), Rc::new(FunctionValue {
            name: method.name.clone(),
            params: method.params.clone(),
            body: method.body.clone(),
            closure_env: None,
            is_method: true,
        }));
    }

    // Apply select/exclude inheritance filters
    if let Some(selected) = &cd.select {
        let selected_set: std::collections::HashSet<&String> = selected.iter().collect();
        methods.retain(|name, _| {
            // Keep own methods always; only filter inherited ones
            cd.methods.iter().any(|m| &m.name == name) || selected_set.contains(name)
        });
    }
    if let Some(excluded) = &cd.exclude {
        let excluded_set: std::collections::HashSet<&String> = excluded.iter().collect();
        methods.retain(|name, _| {
            // Never remove own methods, only filter inherited
            cd.methods.iter().any(|m| &m.name == name) || !excluded_set.contains(name)
        });
    }

    // Inject weave before/after wrappers around each method
    let weave_names = cd.weaves.clone();
    if !weave_names.is_empty() {
        let mut weaved_methods = HashMap::new();
        for (method_name, func) in &methods {
            let mut before_stmts: Vec<crate::parser::ast::Stmt> = Vec::new();
            let mut after_stmts: Vec<crate::parser::ast::Stmt> = Vec::new();
            for wname in &weave_names {
                if let Some(weave) = crate::interpreter::values::get_weave(wname) {
                    if let Some(before) = weave.before {
                        before_stmts.extend(before);
                    }
                    if let Some(after) = weave.after {
                        after_stmts.extend(after);
                    }
                }
            }
            if before_stmts.is_empty() && after_stmts.is_empty() {
                weaved_methods.insert(method_name.clone(), func.clone());
                continue;
            }
            // Build a new function body that: runs before stmts, delegates to original, runs after stmts
            let original_body = match &func.body {
                FuncBody::Block(stmts) => stmts.clone(),
                FuncBody::Expression(expr) => vec![crate::parser::ast::Stmt {
                    kind: crate::parser::ast::StmtKind::Return(Some(expr.clone())),
                    span: expr.span.clone(),
                }],
            };
            let mut wrapped_body = before_stmts;
            wrapped_body.extend(original_body);
            wrapped_body.extend(after_stmts);
            weaved_methods.insert(method_name.clone(), Rc::new(FunctionValue {
                name: func.name.clone(),
                params: func.params.clone(),
                body: FuncBody::Block(wrapped_body),
                closure_env: func.closure_env.clone(),
                is_method: true,
            }));
        }
        methods = weaved_methods;
    }

    let mut static_methods = HashMap::new();
    for method in &cd.static_methods {
        static_methods.insert(method.name.clone(), Rc::new(FunctionValue {
            name: method.name.clone(),
            params: method.params.clone(),
            body: method.body.clone(),
            closure_env: None,
            is_method: false,
        }));
    }

    let init = cd.init.as_ref().map(|init_fn| Rc::new(FunctionValue {
        name: "init".to_string(),
        params: init_fn.params.clone(),
        body: init_fn.body.clone(),
        closure_env: None,
        is_method: true,
    }));

    // Merge parent fields with own fields
    let mut all_fields = Vec::new();
    if let Some(ref p) = parent {
        all_fields.extend(p.fields.clone());
    }
    all_fields.extend(cd.fields.clone());

    // Collect operator overloads
    let mut operators = HashMap::new();
    for op in &cd.operators {
        operators.insert(op.op.clone(), op.clone());
    }

    // Collect computed properties
    let mut computed_props = HashMap::new();
    for cp in &cd.computed_props {
        computed_props.insert(cp.name.clone(), cp.body.clone());
    }

    // Collect reactive properties (stored same as computed — recomputed on access)
    let mut reactive_props = HashMap::new();
    for rp in &cd.reactive_props {
        reactive_props.insert(rp.name.clone(), rp.compute_expr.clone());
    }

    // Collect temporal property info
    let mut temporal_props = HashMap::new();
    for tp in &cd.temporal_props {
        temporal_props.insert(tp.name.clone(), (tp.keep, tp.default.clone()));
    }

    // Collect mutation property info
    let mut mutation_tracked = Vec::new();
    let mut mutation_undoable = HashMap::new();
    for mp in &cd.mutation_props {
        if mp.tracked {
            mutation_tracked.push(mp.name.clone());
        }
        if let Some(depth) = mp.undoable {
            mutation_undoable.insert(mp.name.clone(), (depth, mp.default.clone()));
        }
    }

    // Collect face definitions
    let mut faces = HashMap::new();
    for face in &cd.faces {
        faces.insert(face.name.clone(), face.visible_fields.clone());
    }

    // Collect delegate field names
    let mut delegates = Vec::new();
    for d in &cd.delegates {
        delegates.push(d.field.clone());
    }

    let fitness_fn = cd.fitness_fn.as_ref().map(|ff| Rc::new(FunctionValue {
        name: "fitness".to_string(),
        params: ff.params.clone(),
        body: ff.body.clone(),
        closure_env: None,
        is_method: true,
    }));

    // Collect lazy properties
    let mut lazy_props = HashMap::new();
    for lp in &cd.lazy_props {
        lazy_props.insert(lp.name.clone(), lp.initializer.clone());
    }

    // Collect observed properties
    let mut observed_props = HashMap::new();
    for op in &cd.observed_props {
        observed_props.insert(op.name.clone(), op.did_change.clone());
    }

    // Collect morph methods
    let mut morph_methods = HashMap::new();
    for md in &cd.morph_methods {
        morph_methods.insert(md.name.clone(), md.clone());
    }

    // Collect bond definitions: field_name -> (target_type, via_field)
    let mut bonds = HashMap::new();
    for bd in &cd.bonds {
        let target_type_name = match &bd.target_type {
            TypeAnnotation::Simple(n) => n.clone(),
            TypeAnnotation::Generic(n, _) => n.clone(),
            _ => continue,
        };
        bonds.insert(bd.name.clone(), (target_type_name, bd.via.clone()));
    }

    // Capabilities declared on this class
    let capabilities = cd.capabilities.clone().unwrap_or_default();

    // Interface conformance check
    for iface_name in &cd.interfaces {
        if let Some(Value::List(required)) = env.get(iface_name) {
            let required = required.borrow();
            for req in required.iter() {
                if let Value::String(method_name) = req {
                    if !methods.contains_key(method_name) {
                        return Err(Signal::Throw(Value::String(format!(
                            "class '{}' implements '{}' but missing required method '{}'",
                            cd.name, iface_name, method_name
                        ))));
                    }
                }
            }
        }
    }

    let class_val = Value::Class(Rc::new(ClassValue {
        name: cd.name.clone(),
        parent,
        fields: all_fields,
        methods,
        static_methods,
        operators,
        computed_props,
        init,
        is_genetic: cd.is_genetic,
        chromosomes: cd.chromosomes.clone(),
        fitness_fn,
        reactive_props,
        temporal_props,
        mutation_tracked,
        mutation_undoable,
        faces,
        delegates,
        weaves: weave_names,
        lazy_props,
        observed_props,
        interfaces: cd.interfaces.clone(),
        morph_methods,
        bonds,
        capabilities,
    }));

    env.define(&cd.name, class_val);
    Ok(())
}

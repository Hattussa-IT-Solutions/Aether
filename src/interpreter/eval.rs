use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::interpreter::environment::Environment;
use crate::interpreter::values::*;
use crate::parser::ast::*;

/// Thread-local slot map — set once before execution, used by all function calls.
thread_local! {
    pub static CURRENT_SLOT_MAP: RefCell<Option<crate::interpreter::resolver::SlotMap>> = RefCell::new(None);
    /// Current function's slot map (variable name -> index), updated on each call.
    pub static CURRENT_FUNC_SLOTS: RefCell<Option<HashMap<String, usize>>> = RefCell::new(None);
}

/// Install a slot map for the current execution.
pub fn set_slot_map(map: crate::interpreter::resolver::SlotMap) {
    CURRENT_SLOT_MAP.with(|sm| *sm.borrow_mut() = Some(map));
}

/// Evaluate an expression in the given environment.
pub fn eval_expr(expr: &Expr, env: &mut Environment) -> Result<Value, Signal> {
    match &expr.kind {
        // ── Literals ─────────────────────────────────────
        ExprKind::IntLiteral(n) => Ok(Value::Int(*n)),
        ExprKind::FloatLiteral(n) => Ok(Value::Float(*n)),
        ExprKind::StringLiteral(s) => Ok(Value::String(s.clone())),
        ExprKind::BoolLiteral(b) => Ok(Value::Bool(*b)),
        ExprKind::CharLiteral(c) => Ok(Value::Char(*c)),
        ExprKind::NilLiteral => Ok(Value::Nil),

        // ── String interpolation ─────────────────────────
        ExprKind::InterpolatedString(parts) => {
            let mut result = String::new();
            for part in parts {
                match part {
                    StringInterp::Literal(s) => result.push_str(s),
                    StringInterp::Expr(e) => {
                        let val = eval_expr(e, env)?;
                        result.push_str(&val.to_string());
                    }
                }
            }
            Ok(Value::String(result))
        }

        // ── Identifiers ──────────────────────────────────
        ExprKind::Identifier(name) => {
            // FAST PATH: check slot frame first (direct array index)
            if env.has_slots() {
                if let Some(idx) = env.find_slot(name) {
                    return Ok(env.get_slot(idx).clone());
                }
            }
            env.get(name).ok_or_else(|| {
                Signal::Throw(Value::String(format!("undefined variable: {}", name)))
            })
        }
        ExprKind::SelfExpr => {
            if env.has_slots() {
                if let Some(idx) = env.find_slot("self") {
                    return Ok(env.get_slot(idx).clone());
                }
            }
            env.get("self").ok_or_else(|| {
                Signal::Throw(Value::String("'self' used outside of class method".into()))
            })
        }
        ExprKind::SuperExpr => {
            env.get("super").ok_or_else(|| {
                Signal::Throw(Value::String("'super' used outside of class method".into()))
            })
        }

        // ── Unary operations ─────────────────────────────
        ExprKind::Unary { op, operand } => {
            let val = eval_expr(operand, env)?;
            match op {
                UnaryOp::Neg => match val {
                    Value::Int(n) => Ok(Value::Int(-n)),
                    Value::Float(n) => Ok(Value::Float(-n)),
                    _ => Err(Signal::Throw(Value::String(format!("cannot negate {}", val)))),
                },
                UnaryOp::Not => Ok(Value::Bool(!val.is_truthy())),
                UnaryOp::BitNot => match val {
                    Value::Int(n) => Ok(Value::Int(!n)),
                    _ => Err(Signal::Throw(Value::String("bitwise NOT requires Int".into()))),
                },
            }
        }

        // ── Binary operations (with inline fast paths for Int arithmetic) ──
        ExprKind::Binary { left, op, right } => {
            // FAST PATH: Int op Int — avoids full eval_binary dispatch
            match op {
                BinaryOp::Add => {
                    let l = eval_expr(left, env)?;
                    let r = eval_expr(right, env)?;
                    if let (Value::Int(a), Value::Int(b)) = (&l, &r) {
                        return Ok(Value::Int(a + b));
                    }
                    if let (Value::Float(a), Value::Float(b)) = (&l, &r) {
                        return Ok(Value::Float(a + b));
                    }
                    if let (Value::Int(a), Value::Float(b)) = (&l, &r) {
                        return Ok(Value::Float(*a as f64 + b));
                    }
                    if let (Value::Float(a), Value::Int(b)) = (&l, &r) {
                        return Ok(Value::Float(a + *b as f64));
                    }
                    if let (Value::String(a), Value::String(b)) = (&l, &r) {
                        return Ok(Value::String(format!("{}{}", a, b)));
                    }
                    if let Value::String(a) = &l {
                        return Ok(Value::String(format!("{}{}", a, r)));
                    }
                    eval_binary(op, left, right, env)
                }
                BinaryOp::Sub => {
                    let l = eval_expr(left, env)?;
                    let r = eval_expr(right, env)?;
                    match (&l, &r) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
                        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                        (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
                        (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - *b as f64)),
                        _ => eval_binary(op, left, right, env),
                    }
                }
                BinaryOp::Mul => {
                    let l = eval_expr(left, env)?;
                    let r = eval_expr(right, env)?;
                    match (&l, &r) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
                        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                        (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
                        (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a * *b as f64)),
                        _ => eval_binary(op, left, right, env),
                    }
                }
                BinaryOp::Lt | BinaryOp::LtEq | BinaryOp::Gt | BinaryOp::GtEq => {
                    let l = eval_expr(left, env)?;
                    let r = eval_expr(right, env)?;
                    if let (Value::Int(a), Value::Int(b)) = (&l, &r) {
                        return Ok(Value::Bool(match op {
                            BinaryOp::Lt => a < b,
                            BinaryOp::LtEq => a <= b,
                            BinaryOp::Gt => a > b,
                            BinaryOp::GtEq => a >= b,
                            _ => unreachable!(),
                        }));
                    }
                    eval_binary(op, left, right, env)
                }
                _ => eval_binary(op, left, right, env),
            }
        }

        // ── Function call ────────────────────────────────
        ExprKind::Call { callee, args } => {
            let func_val = eval_expr(callee, env)?;
            let mut arg_vals = Vec::new();
            for arg in args {
                arg_vals.push(eval_expr(&arg.value, env)?);
            }
            call_function(&func_val, arg_vals, args, env)
        }

        // ── Method call ──────────────────────────────────
        ExprKind::MethodCall { object, method, args } => {
            let mut arg_vals = Vec::new();
            for arg in args {
                arg_vals.push(eval_expr(&arg.value, env)?);
            }
            // Handle super.method() — dispatch to parent class
            if matches!(object.kind, ExprKind::SuperExpr) {
                let self_val = env.get("self").ok_or_else(|| {
                    Signal::Throw(Value::String("'super' used outside of class method".into()))
                })?;
                return call_super_method(&self_val, method, arg_vals, env);
            }
            let obj_val = eval_expr(object, env)?;
            call_method(&obj_val, method, arg_vals, env)
        }

        // ── Field access ─────────────────────────────────
        ExprKind::FieldAccess { object, field } => {
            let obj = eval_expr(object, env)?;
            access_field(&obj, field)
        }

        // ── Index access ─────────────────────────────────
        ExprKind::Index { object, index } => {
            let obj = eval_expr(object, env)?;
            let idx = eval_expr(index, env)?;
            match (&obj, &idx) {
                (Value::List(items), Value::Int(i)) => {
                    let items = items.borrow();
                    let i = if *i < 0 { items.len() as i64 + i } else { *i } as usize;
                    items.get(i).cloned().ok_or_else(|| {
                        Signal::Throw(Value::String(format!("index {} out of bounds", i)))
                    })
                }
                (Value::Map(map), _) => {
                    let key = idx.as_map_key().ok_or_else(|| {
                        Signal::Throw(Value::String("invalid map key type".into()))
                    })?;
                    let map = map.borrow();
                    Ok(map.get(&key).cloned().unwrap_or(Value::Nil))
                }
                (Value::Tuple(items), Value::Int(i)) => {
                    let i = *i as usize;
                    items.get(i).cloned().ok_or_else(|| {
                        Signal::Throw(Value::String(format!("tuple index {} out of bounds", i)))
                    })
                }
                (Value::String(s), Value::Int(i)) => {
                    let i = if *i < 0 { s.len() as i64 + i } else { *i } as usize;
                    s.chars().nth(i).map(|c| Value::Char(c)).ok_or_else(|| {
                        Signal::Throw(Value::String(format!("string index {} out of bounds", i)))
                    })
                }
                _ => Err(Signal::Throw(Value::String(format!("cannot index {} with {}", obj, idx)))),
            }
        }

        // ── Optional chaining ────────────────────────────
        ExprKind::OptionalChain { object, field } => {
            let obj = eval_expr(object, env)?;
            if matches!(obj, Value::Nil) {
                Ok(Value::Nil)
            } else {
                access_field(&obj, field)
            }
        }

        // ── Nil coalescing ───────────────────────────────
        ExprKind::NilCoalesce { value, default } => {
            let val = eval_expr(value, env)?;
            if matches!(val, Value::Nil) {
                eval_expr(default, env)
            } else {
                Ok(val)
            }
        }

        // ── Error propagation ────────────────────────────
        ExprKind::ErrorPropagate(inner) => {
            let val = eval_expr(inner, env)?;
            match val {
                Value::Ok(v) => Ok(*v),
                Value::Err(e) => Err(Signal::Return(Value::Err(e))),
                other => Ok(other),
            }
        }

        // ── Pipeline ─────────────────────────────────────
        ExprKind::Pipeline { left, right } => {
            let arg = eval_expr(left, env)?;
            let func = eval_expr(right, env)?;
            call_function(&func, vec![arg], &[], env)
        }

        // ── Lambda ───────────────────────────────────────
        ExprKind::Lambda { params, body } => {
            Ok(Value::Function(Rc::new(FunctionValue {
                name: "<lambda>".to_string(),
                params: params.clone(),
                body: FuncBody::Expression((**body).clone()),
                closure_env: Some(Rc::new(RefCell::new(env.snapshot()))),
                is_method: false,
            })))
        }

        // ── If expression ────────────────────────────────
        ExprKind::IfExpr { condition, then_expr, else_expr } => {
            let cond = eval_expr(condition, env)?;
            if cond.is_truthy() {
                eval_expr(then_expr, env)
            } else {
                eval_expr(else_expr, env)
            }
        }

        // ── Match expression ─────────────────────────────
        ExprKind::MatchExpr { value, arms } => {
            let val = eval_expr(value, env)?;
            eval_match(&val, arms, env)
        }

        // ── List literal ─────────────────────────────────
        ExprKind::ListLiteral(items) => {
            let mut vals = Vec::new();
            for item in items {
                vals.push(eval_expr(item, env)?);
            }
            Ok(Value::List(Rc::new(RefCell::new(vals))))
        }

        // ── Map literal ──────────────────────────────────
        ExprKind::MapLiteral(pairs) => {
            let mut map = HashMap::new();
            for (k, v) in pairs {
                let key_val = eval_expr(k, env)?;
                let key = key_val.as_map_key().ok_or_else(|| {
                    Signal::Throw(Value::String("invalid map key".into()))
                })?;
                let val = eval_expr(v, env)?;
                map.insert(key, val);
            }
            Ok(Value::Map(Rc::new(RefCell::new(map))))
        }

        // ── Set literal ──────────────────────────────────
        ExprKind::SetLiteral(items) => {
            let mut vals = Vec::new();
            for item in items {
                let val = eval_expr(item, env)?;
                // Simple dedup
                if !vals.iter().any(|v: &Value| v.equals(&val)) {
                    vals.push(val);
                }
            }
            Ok(Value::Set(Rc::new(RefCell::new(vals))))
        }

        // ── Tuple literal ────────────────────────────────
        ExprKind::TupleLiteral(items) => {
            let mut vals = Vec::new();
            for item in items {
                vals.push(eval_expr(item, env)?);
            }
            Ok(Value::Tuple(vals))
        }

        // ── Range ────────────────────────────────────────
        ExprKind::Range { start, end, inclusive, step } => {
            let s = eval_expr(start, env)?.as_int().unwrap_or(0);
            let e = eval_expr(end, env)?.as_int().unwrap_or(0);
            let st = if let Some(step_expr) = step {
                eval_expr(step_expr, env)?.as_int().unwrap_or(1)
            } else { 1 };
            Ok(Value::Range { start: s, end: e, inclusive: *inclusive, step: st })
        }

        // ── Comprehension ────────────────────────────────
        ExprKind::Comprehension { expr: body_expr, var, iterable, condition, kind: _ } => {
            let iter_val = eval_expr(iterable, env)?;
            let items = value_to_iter(&iter_val)?;
            let mut result = Vec::new();
            env.push_scope();
            for item in items {
                env.define(var, item);
                if let Some(cond) = condition {
                    let c = eval_expr(cond, env)?;
                    if !c.is_truthy() { continue; }
                }
                result.push(eval_expr(body_expr, env)?);
            }
            env.pop_scope();
            Ok(Value::List(Rc::new(RefCell::new(result))))
        }

        // ── Ok/Err constructors ──────────────────────────
        ExprKind::ResultOk(inner) => {
            let val = eval_expr(inner, env)?;
            Ok(Value::Ok(Box::new(val)))
        }
        ExprKind::ResultErr(inner) => {
            let val = eval_expr(inner, env)?;
            Ok(Value::Err(Box::new(val)))
        }

        // ── Enum variant ─────────────────────────────────
        ExprKind::EnumVariant { name, args } => {
            let mut field_vals = Vec::new();
            for arg in args {
                field_vals.push(eval_expr(arg, env)?);
            }
            Ok(Value::EnumVariant {
                enum_name: String::new(), // filled in by context
                variant: name.clone(),
                fields: field_vals,
            })
        }

        // ── Await (placeholder — runs synchronously) ─────
        ExprKind::Await(inner) => eval_expr(inner, env),

        // ── As cast ──────────────────────────────────────
        ExprKind::AsCast { value, target_type } => {
            let val = eval_expr(value, env)?;
            eval_cast(val, target_type)
        }

        // ── EvolveBlock ───────────────────────────────────
        ExprKind::EvolveBlock { target, config } => {
            eval_evolve_block(target, config, env)
        }

        // ── Crossover ─────────────────────────────────────
        ExprKind::Crossover { parent_a, parent_b } => {
            let a = eval_expr(parent_a, env)?;
            let b = eval_expr(parent_b, env)?;
            eval_crossover(&a, &b, 0.0, env)
        }

        // ── Breed ─────────────────────────────────────────
        ExprKind::Breed { parent_a, parent_b, mutation_rate } => {
            let a = eval_expr(parent_a, env)?;
            let b = eval_expr(parent_b, env)?;
            let rate = if let Some(mr) = mutation_rate {
                eval_expr(mr, env)?.as_float().unwrap_or(0.05)
            } else {
                0.05
            };
            eval_crossover(&a, &b, rate, env)
        }

        _ => Err(Signal::Throw(Value::String(format!(
            "unimplemented expression: {:?}", expr.kind
        )))),
    }
}

// ═══════════════════════════════════════════════════════════════
// Binary operations
// ═══════════════════════════════════════════════════════════════

fn eval_binary(op: &BinaryOp, left: &Expr, right: &Expr, env: &mut Environment) -> Result<Value, Signal> {
    // Short-circuit for logical operators
    if *op == BinaryOp::And {
        let l = eval_expr(left, env)?;
        if !l.is_truthy() { return Ok(Value::Bool(false)); }
        let r = eval_expr(right, env)?;
        return Ok(Value::Bool(r.is_truthy()));
    }
    if *op == BinaryOp::Or {
        let l = eval_expr(left, env)?;
        if l.is_truthy() { return Ok(Value::Bool(true)); }
        let r = eval_expr(right, env)?;
        return Ok(Value::Bool(r.is_truthy()));
    }

    let lval = eval_expr(left, env)?;
    let rval = eval_expr(right, env)?;

    // Check for operator overloading on class instances
    if let Value::Instance(ref inst) = lval {
        let op_name = match op {
            BinaryOp::Add => "+", BinaryOp::Sub => "-", BinaryOp::Mul => "*",
            BinaryOp::Div => "/", BinaryOp::Mod => "%", BinaryOp::Eq => "==",
            BinaryOp::NotEq => "!=", BinaryOp::Lt => "<", BinaryOp::Gt => ">",
            _ => "",
        };
        if !op_name.is_empty() {
            let inst_borrow = inst.borrow();
            if let Some(cls) = &inst_borrow.class {
                if let Some(op_def) = cls.operators.get(op_name) {
                    let op_def = op_def.clone();
                    drop(inst_borrow);
                    return call_operator_method(&lval, &op_def, vec![rval], env);
                }
            }
        }
    }

    match op {
        // Arithmetic
        BinaryOp::Add => numeric_op(&lval, &rval, |a, b| a + b, |a, b| a + b, "+"),
        BinaryOp::Sub => numeric_op(&lval, &rval, |a, b| a - b, |a, b| a - b, "-"),
        BinaryOp::Mul => numeric_op(&lval, &rval, |a, b| a * b, |a, b| a * b, "*"),
        BinaryOp::Div => {
            // Check for division by zero
            match (&lval, &rval) {
                (_, Value::Int(0)) => Err(Signal::Throw(Value::String("division by zero".into()))),
                (_, Value::Float(f)) if *f == 0.0 => Err(Signal::Throw(Value::String("division by zero".into()))),
                _ => numeric_op(&lval, &rval, |a, b| a / b, |a, b| a / b, "/"),
            }
        }
        BinaryOp::Mod => numeric_op(&lval, &rval, |a, b| a % b, |a, b| a % b, "%"),
        BinaryOp::Pow => {
            match (&lval, &rval) {
                (Value::Int(a), Value::Int(b)) => {
                    if *b >= 0 { Ok(Value::Int(a.pow(*b as u32))) }
                    else { Ok(Value::Float((*a as f64).powf(*b as f64))) }
                }
                _ => {
                    let a = lval.as_float().unwrap_or(0.0);
                    let b = rval.as_float().unwrap_or(0.0);
                    Ok(Value::Float(a.powf(b)))
                }
            }
        }

        // Comparison
        BinaryOp::Eq => Ok(Value::Bool(lval.equals(&rval))),
        BinaryOp::NotEq => Ok(Value::Bool(!lval.equals(&rval))),
        BinaryOp::Lt => compare_op(&lval, &rval, |o| o == std::cmp::Ordering::Less),
        BinaryOp::Gt => compare_op(&lval, &rval, |o| o == std::cmp::Ordering::Greater),
        BinaryOp::LtEq => compare_op(&lval, &rval, |o| o != std::cmp::Ordering::Greater),
        BinaryOp::GtEq => compare_op(&lval, &rval, |o| o != std::cmp::Ordering::Less),

        // Bitwise
        BinaryOp::BitAnd => int_op(&lval, &rval, |a, b| a & b, "&"),
        BinaryOp::BitOr => int_op(&lval, &rval, |a, b| a | b, "|"),
        BinaryOp::BitXor => int_op(&lval, &rval, |a, b| a ^ b, "^"),
        BinaryOp::Shl => int_op(&lval, &rval, |a, b| a << b, "<<"),
        BinaryOp::Shr => int_op(&lval, &rval, |a, b| a >> b, ">>"),

        // Range (handled in ExprKind::Range)
        BinaryOp::Range | BinaryOp::RangeInclusive => {
            let s = lval.as_int().unwrap_or(0);
            let e = rval.as_int().unwrap_or(0);
            Ok(Value::Range {
                start: s, end: e,
                inclusive: *op == BinaryOp::RangeInclusive,
                step: 1,
            })
        }

        BinaryOp::And | BinaryOp::Or => unreachable!(), // handled above
        BinaryOp::Pipeline => unreachable!(), // handled in ExprKind::Pipeline
    }
}

fn numeric_op(
    left: &Value, right: &Value,
    int_fn: impl Fn(i64, i64) -> i64,
    float_fn: impl Fn(f64, f64) -> f64,
    op_name: &str,
) -> Result<Value, Signal> {
    match (left, right) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(int_fn(*a, *b))),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(float_fn(*a, *b))),
        (Value::Int(a), Value::Float(b)) => Ok(Value::Float(float_fn(*a as f64, *b))),
        (Value::Float(a), Value::Int(b)) => Ok(Value::Float(float_fn(*a, *b as f64))),
        // String concatenation with +
        (Value::String(a), Value::String(b)) if op_name == "+" => {
            Ok(Value::String(format!("{}{}", a, b)))
        }
        (Value::String(a), b) if op_name == "+" => {
            Ok(Value::String(format!("{}{}", a, b)))
        }
        _ => Err(Signal::Throw(Value::String(format!(
            "cannot apply '{}' to {} and {}", op_name, left, right
        )))),
    }
}

fn int_op(
    left: &Value, right: &Value,
    op_fn: impl Fn(i64, i64) -> i64,
    op_name: &str,
) -> Result<Value, Signal> {
    match (left, right) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(op_fn(*a, *b))),
        _ => Err(Signal::Throw(Value::String(format!(
            "'{}' requires Int operands", op_name
        )))),
    }
}

fn compare_op(
    left: &Value, right: &Value,
    pred: impl Fn(std::cmp::Ordering) -> bool,
) -> Result<Value, Signal> {
    let ord = match (left, right) {
        (Value::Int(a), Value::Int(b)) => a.cmp(b),
        (Value::Float(a), Value::Float(b)) => a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
        (Value::Int(a), Value::Float(b)) => (*a as f64).partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
        (Value::Float(a), Value::Int(b)) => a.partial_cmp(&(*b as f64)).unwrap_or(std::cmp::Ordering::Equal),
        (Value::String(a), Value::String(b)) => a.cmp(b),
        _ => return Err(Signal::Throw(Value::String(format!("cannot compare {} and {}", left, right)))),
    };
    Ok(Value::Bool(pred(ord)))
}

// ═══════════════════════════════════════════════════════════════
// Function calling
// ═══════════════════════════════════════════════════════════════

pub fn call_function(
    func_val: &Value,
    mut arg_vals: Vec<Value>,
    named_args: &[Argument],
    env: &mut Environment,
) -> Result<Value, Signal> {
    match func_val {
        Value::Function(func) => {
            let has_closure = func.closure_env.is_some();
            let has_named = named_args.iter().any(|a| a.name.is_some());

            // Check if we have a slot map for this function
            let slot_count = CURRENT_SLOT_MAP.with(|sm| {
                sm.borrow().as_ref().and_then(|m| m.counts.get(&func.name).copied())
            });

            // Slot-indexed path disabled — Vec-scan is faster for small scopes (1-5 vars).
            // The slot overhead (HashMap clone per call) exceeds the string-scan cost.
            // To beat Python's 150ms on fib(30), we'd need compile-time slot assignment
            // that eliminates the HashMap entirely (integer-only frames).
            if false && !has_closure && !has_named && slot_count.is_some() {
                // FASTEST PATH: slot-indexed frame — no string operations at all
                let sc = slot_count.unwrap();
                let slot_map = CURRENT_SLOT_MAP.with(|sm| {
                    sm.borrow().as_ref().and_then(|m| m.slots.get(&func.name).cloned())
                });

                env.push_slot_frame_with_names(sc, slot_map.clone());

                // Bind params by slot index
                if let Some(ref sm) = slot_map {
                    for (i, param) in func.params.iter().enumerate() {
                        let val = if i < arg_vals.len() { arg_vals[i].clone() } else { Value::Nil };
                        if let Some(&slot) = sm.get(&param.name) {
                            env.set_slot(slot, val);
                        }
                    }
                }

                let result = match &func.body {
                    FuncBody::Expression(expr) => {
                        match eval_expr(expr, env) {
                            Ok(val) => Ok(val),
                            Err(Signal::Return(val)) => Ok(val),
                            Err(e) => Err(e),
                        }
                    }
                    FuncBody::Block(stmts) => {
                        match crate::interpreter::exec::exec_block_with_value(stmts, env) {
                            Ok(val) => Ok(val),
                            Err(Signal::Return(val)) => Ok(val),
                            Err(e) => Err(e),
                        }
                    }
                };

                env.pop_slot_frame();
                return result;
            } else if !has_closure && !has_named {
                // FAST PATH: push scope with string names (no slot map available)
                env.push_scope();
                for (i, param) in func.params.iter().enumerate() {
                    let val = if i < arg_vals.len() {
                        arg_vals[i].clone()
                    } else if let Some(default) = &param.default {
                        eval_expr(default, env)?
                    } else {
                        Value::Nil
                    };
                    env.define_new(&param.name, val);
                }

                let result = match &func.body {
                    FuncBody::Expression(expr) => {
                        match eval_expr(expr, env) {
                            Ok(val) => Ok(val),
                            Err(Signal::Return(val)) => Ok(val),
                            Err(e) => Err(e),
                        }
                    }
                    FuncBody::Block(stmts) => {
                        match crate::interpreter::exec::exec_block_with_value(stmts, env) {
                            Ok(val) => Ok(val),
                            Err(Signal::Return(val)) => Ok(val),
                            Err(e) => Err(e),
                        }
                    }
                };
                env.pop_scope();
                return result;
            }

            // SLOW PATH: closure or named args — need env clone
            let mut call_env = if let Some(closure) = &func.closure_env {
                closure.borrow().clone()
            } else {
                env.snapshot()
            };
            call_env.push_scope();

            // Bind parameters
            for (i, param) in func.params.iter().enumerate() {
                let val = if let Some(named) = named_args.iter().find(|a| a.name.as_deref() == Some(&param.name)) {
                    eval_expr(&named.value, env)?
                } else if i < arg_vals.len() {
                    arg_vals[i].clone()
                } else if let Some(default) = &param.default {
                    eval_expr(default, &mut call_env)?
                } else {
                    Value::Nil
                };
                call_env.define(&param.name, val);
            }

            match &func.body {
                FuncBody::Expression(expr) => {
                    let result = eval_expr(expr, &mut call_env);
                    match result {
                        Ok(val) => Ok(val),
                        Err(Signal::Return(val)) => Ok(val),
                        Err(e) => Err(e),
                    }
                }
                FuncBody::Block(stmts) => {
                    let result = crate::interpreter::exec::exec_block_with_value(stmts, &mut call_env);
                    match result {
                        Ok(val) => Ok(val),
                        Err(Signal::Return(val)) => Ok(val),
                        Err(e) => Err(e),
                    }
                }
            }
        }
        Value::NativeFunction(nf) => {
            (nf.func)(arg_vals).map_err(|e| Signal::Throw(Value::String(e)))
        }
        Value::Class(cls) => {
            // Constructor call: ClassName(args)
            let mut fields = HashMap::new();
            // Set default values
            for field in &cls.fields {
                if let Some(default) = &field.default {
                    fields.insert(field.name.clone(), eval_expr(default, env)?);
                }
            }

            // Temporal properties: initialize default values and history buffers
            for (tp_name, (_keep, default_expr)) in &cls.temporal_props {
                if !fields.contains_key(tp_name.as_str()) {
                    let initial = if let Some(def_expr) = default_expr {
                        eval_expr(def_expr, env)?
                    } else {
                        Value::Nil
                    };
                    fields.insert(tp_name.clone(), initial);
                }
                let hist_key = format!("__temporal_hist_{}", tp_name);
                fields.insert(hist_key, Value::List(Rc::new(RefCell::new(Vec::new()))));
                let prev_key = format!("__temporal_prev_{}", tp_name);
                fields.insert(prev_key, Value::Nil);
            }

            // Mutation undoable: initialize undo/redo stacks and default values
            for (mp_name, (_depth, default_expr)) in &cls.mutation_undoable {
                // Set default value if field not already set
                if !fields.contains_key(mp_name.as_str()) {
                    let initial = if let Some(def_expr) = default_expr {
                        eval_expr(def_expr, env)?
                    } else {
                        Value::Nil
                    };
                    fields.insert(mp_name.clone(), initial);
                }
                let undo_key = format!("__undo_{}", mp_name);
                fields.insert(undo_key, Value::List(Rc::new(RefCell::new(Vec::new()))));
                let redo_key = format!("__redo_{}", mp_name);
                fields.insert(redo_key, Value::List(Rc::new(RefCell::new(Vec::new()))));
            }
            // Also set mutation_tracked defaults
            for mp_name in &cls.mutation_tracked {
                if !fields.contains_key(mp_name.as_str()) {
                    fields.insert(mp_name.clone(), Value::Nil);
                }
            }

            // Mutation tracked: initialize mutations log
            if !cls.mutation_tracked.is_empty() {
                fields.insert("__mutations".to_string(), Value::List(Rc::new(RefCell::new(Vec::new()))));
            }

            // Genetic class: populate gene fields with randomised initial values
            if cls.is_genetic {
                let mut seed = rng_seed_from_env();
                for chromosome in &cls.chromosomes {
                    for gene in &chromosome.genes {
                        let val = random_gene_value(gene, &mut seed, env)?;
                        fields.insert(gene.name.clone(), val);
                    }
                }
                // Mark this instance as genetic by storing __is_genetic flag
                fields.insert("__is_genetic".to_string(), Value::Bool(true));
            }

            let instance = Rc::new(RefCell::new(InstanceValue {
                class_name: cls.name.clone(),
                fields,
                class: Some(cls.clone()),
            }));

            // Call init if available — use push/pop scope (fast path)
            if let Some(init_fn) = &cls.init {
                env.push_scope();
                env.define_new("self", Value::Instance(instance.clone()));

                for (i, param) in init_fn.params.iter().enumerate() {
                    let val = if i < arg_vals.len() { arg_vals[i].clone() } else { Value::Nil };
                    env.define_new(&param.name, val);
                }

                if let FuncBody::Block(stmts) = &init_fn.body {
                    match crate::interpreter::exec::exec_block(stmts, env) {
                        Ok(()) | Err(Signal::Return(_)) => {}
                        Err(e) => { env.pop_scope(); return Err(e); }
                    }
                }
                env.pop_scope();
                // No need to copy back — self shares the same Rc<RefCell>, mutations are direct
            } else {
                // No init — assign positional args to fields (skip for genetic classes)
                if !cls.is_genetic {
                    for (i, field) in cls.fields.iter().enumerate() {
                        if i < arg_vals.len() {
                            instance.borrow_mut().fields.insert(field.name.clone(), arg_vals[i].clone());
                        }
                    }
                }
            }

            Ok(Value::Instance(instance))
        }
        Value::StructDef(sd) => {
            let mut fields = HashMap::new();
            for (i, field) in sd.fields.iter().enumerate() {
                if i < arg_vals.len() {
                    fields.insert(field.name.clone(), arg_vals[i].clone());
                } else if let Some(default) = &field.default {
                    fields.insert(field.name.clone(), eval_expr(default, env)?);
                }
            }
            Ok(Value::StructInstance(Rc::new(RefCell::new(InstanceValue {
                class_name: sd.name.clone(),
                fields,
                class: None,
            }))))
        }
        // Call a Python callable directly
        Value::PythonObject(wrapper) => {
            crate::bridge::python::python_call_direct(wrapper, arg_vals)
                .map_err(|e| Signal::Throw(Value::String(e)))
        }
        _ => Err(Signal::Throw(Value::String(format!("{} is not callable", func_val)))),
    }
}

// ═══════════════════════════════════════════════════════════════
// Method calls
// ═══════════════════════════════════════════════════════════════

pub fn call_method(
    obj: &Value, method: &str, args: Vec<Value>, env: &mut Environment,
) -> Result<Value, Signal> {
    // Check for built-in methods on types first
    if let Some(result) = builtin_method(obj, method, &args)? {
        return Ok(result);
    }

    // Class/struct instance methods
    match obj {
        Value::Instance(inst) => {
            // ── Genetic built-in methods ──────────────────────────────────
            {
                let inst_borrow = inst.borrow();
                let is_genetic = inst_borrow.fields
                    .get("__is_genetic")
                    .map(|v| matches!(v, Value::Bool(true)))
                    .unwrap_or(false);
                if is_genetic {
                    match method {
                        "mutate" => {
                            // Randomise one or all genes within their ranges.
                            // mutate() with no args randomises all genes.
                            // mutate("gene_name") randomises a specific gene.
                            let cls_opt = inst_borrow.class.clone();
                            drop(inst_borrow);
                            if let Some(cls) = cls_opt {
                                let mut seed = rng_seed_from_env();
                                let target_gene = args.first().and_then(|a| {
                                    if let Value::String(s) = a { Some(s.clone()) } else { None }
                                });
                                let new_instance = inst.borrow().clone();
                                let new_inst_rc = Rc::new(RefCell::new(new_instance));
                                for chromosome in &cls.chromosomes {
                                    for gene in &chromosome.genes {
                                        if target_gene.is_none() || target_gene.as_deref() == Some(&gene.name) {
                                            let val = random_gene_value(gene, &mut seed, env)?;
                                            new_inst_rc.borrow_mut().fields.insert(gene.name.clone(), val);
                                        }
                                    }
                                }
                                return Ok(Value::Instance(new_inst_rc));
                            }
                            return Ok(obj.clone());
                        }
                        "fitness" => {
                            // Call the class fitness function with self.
                            let cls_opt = inst_borrow.class.clone();
                            drop(inst_borrow);
                            if let Some(cls) = cls_opt {
                                if let Some(fitness_fn) = &cls.fitness_fn {
                                    let fitness_fn = fitness_fn.clone();
                                    let mut method_env = env.snapshot();
                                    method_env.push_scope();
                                    method_env.define("self", obj.clone());
                                    // Pass optional data argument
                                    for (i, param) in fitness_fn.params.iter().enumerate() {
                                        let val = if i < args.len() { args[i].clone() } else { Value::Nil };
                                        method_env.define(&param.name, val);
                                    }
                                    return match &fitness_fn.body {
                                        FuncBody::Expression(expr) => eval_expr(expr, &mut method_env),
                                        FuncBody::Block(stmts) => {
                                            match crate::interpreter::exec::exec_block_with_value(stmts, &mut method_env) {
                                                Ok(val) => Ok(val),
                                                Err(Signal::Return(val)) => Ok(val),
                                                Err(e) => Err(e),
                                            }
                                        }
                                    };
                                }
                            }
                            // No fitness function defined — return 0
                            return Ok(Value::Float(0.0));
                        }
                        "genes" => {
                            // Return a map of all gene name -> value pairs.
                            let cls_opt = inst_borrow.class.clone();
                            let fields_snap = inst_borrow.fields.clone();
                            drop(inst_borrow);
                            if let Some(cls) = cls_opt {
                                let mut gene_map = std::collections::HashMap::new();
                                for chromosome in &cls.chromosomes {
                                    for gene in &chromosome.genes {
                                        let val = fields_snap.get(&gene.name).cloned().unwrap_or(Value::Nil);
                                        gene_map.insert(gene.name.clone(), val);
                                    }
                                }
                                return Ok(Value::Map(Rc::new(RefCell::new(gene_map))));
                            }
                            return Ok(Value::Map(Rc::new(RefCell::new(std::collections::HashMap::new()))));
                        }
                        _ => { drop(inst_borrow); }
                    }
                } else {
                    drop(inst_borrow);
                }
            }

            let inst_borrow = inst.borrow();
            if let Some(cls) = &inst_borrow.class {
                if let Some(func) = cls.methods.get(method) {
                    let func = func.clone();
                    drop(inst_borrow);

                    // FAST PATH: push/pop scope instead of env clone
                    env.push_scope();
                    env.define_new("self", obj.clone());

                    for (i, param) in func.params.iter().enumerate() {
                        let val = if i < args.len() { args[i].clone() } else { Value::Nil };
                        env.define_new(&param.name, val);
                    }

                    let result = match &func.body {
                        FuncBody::Expression(expr) => eval_expr(expr, env),
                        FuncBody::Block(stmts) => {
                            match crate::interpreter::exec::exec_block_with_value(stmts, env) {
                                Ok(val) => Ok(val),
                                Err(Signal::Return(val)) => Ok(val),
                                Err(e) => Err(e),
                            }
                        }
                    };
                    env.pop_scope();
                    return result;
                }
                // Check parent class
                let mut current = cls.parent.clone();
                while let Some(parent) = current {
                    if let Some(func) = parent.methods.get(method) {
                        let func = func.clone();
                        drop(inst_borrow);
                        let mut method_env = env.snapshot();
                        method_env.push_scope();
                        method_env.define("self", obj.clone());
                        for (i, param) in func.params.iter().enumerate() {
                            let val = if i < args.len() { args[i].clone() } else { Value::Nil };
                            method_env.define(&param.name, val);
                        }
                        return match &func.body {
                            FuncBody::Expression(expr) => eval_expr(expr, &mut method_env),
                            FuncBody::Block(stmts) => {
                                match crate::interpreter::exec::exec_block(stmts, &mut method_env) {
                                    Ok(()) => Ok(Value::Nil),
                                    Err(Signal::Return(val)) => Ok(val),
                                    Err(e) => Err(e),
                                }
                            }
                        };
                    }
                    current = parent.parent.clone();
                }

                // ── Temporal property helper methods ─────────────────────────
                // <prop>.history() / <prop>.previous() are accessed as methods
                // with naming convention __temporal_hist_<prop> / __temporal_prev_<prop>
                // The method name itself is not how they're accessed — see access_field
                // for field access. But methods like history("fieldname") are useful.
                // Actually: history and previous are called as obj.temperature.history()
                // which means the field returns a sub-object... We handle it differently:
                // We support .rollback(field) and .forward(field) on the instance itself.

                let temporal_props = cls.temporal_props.clone();
                let mutation_undoable = cls.mutation_undoable.clone();
                let faces = cls.faces.clone();
                let delegates = cls.delegates.clone();
                let class_name = cls.name.clone();
                let morph_methods = cls.morph_methods.clone();
                let capabilities = cls.capabilities.clone();
                drop(inst_borrow);

                // Temporal: rollback / forward for undoable mutation props
                match method {
                    "rollback" => {
                        let field_name = args.first()
                            .and_then(|a| if let Value::String(s) = a { Some(s.clone()) } else { None })
                            .unwrap_or_default();
                        if mutation_undoable.contains_key(&field_name) {
                            let undo_key = format!("__undo_{}", field_name);
                            let redo_key = format!("__redo_{}", field_name);
                            let undo_hist = inst.borrow().fields.get(&undo_key).cloned()
                                .unwrap_or_else(|| Value::List(Rc::new(RefCell::new(Vec::new()))));
                            if let Value::List(ref ul) = undo_hist {
                                let last = ul.borrow().last().cloned();
                                if let Some(prev_val) = last {
                                    // Move current value to redo stack
                                    let current_val = inst.borrow().fields.get(&field_name).cloned().unwrap_or(Value::Nil);
                                    let redo_hist = inst.borrow().fields.get(&redo_key).cloned()
                                        .unwrap_or_else(|| Value::List(Rc::new(RefCell::new(Vec::new()))));
                                    if let Value::List(ref rl) = redo_hist {
                                        let depth = mutation_undoable.get(&field_name).map(|(d, _)| *d).unwrap_or(10);
                                        let mut rv = rl.borrow_mut();
                                        rv.push(current_val);
                                        if rv.len() > depth { rv.remove(0); }
                                    }
                                    inst.borrow_mut().fields.insert(redo_key, redo_hist);
                                    // Pop from undo and set field
                                    ul.borrow_mut().pop();
                                    inst.borrow_mut().fields.insert(field_name.clone(), prev_val);
                                    inst.borrow_mut().fields.insert(undo_key, undo_hist);
                                    return Ok(Value::Bool(true));
                                }
                            }
                            return Ok(Value::Bool(false));
                        }
                    }
                    "forward" => {
                        let field_name = args.first()
                            .and_then(|a| if let Value::String(s) = a { Some(s.clone()) } else { None })
                            .unwrap_or_default();
                        if mutation_undoable.contains_key(&field_name) {
                            let undo_key = format!("__undo_{}", field_name);
                            let redo_key = format!("__redo_{}", field_name);
                            let redo_hist = inst.borrow().fields.get(&redo_key).cloned()
                                .unwrap_or_else(|| Value::List(Rc::new(RefCell::new(Vec::new()))));
                            if let Value::List(ref rl) = redo_hist {
                                let last = rl.borrow().last().cloned();
                                if let Some(next_val) = last {
                                    let current_val = inst.borrow().fields.get(&field_name).cloned().unwrap_or(Value::Nil);
                                    let undo_hist = inst.borrow().fields.get(&undo_key).cloned()
                                        .unwrap_or_else(|| Value::List(Rc::new(RefCell::new(Vec::new()))));
                                    if let Value::List(ref ul) = undo_hist {
                                        let depth = mutation_undoable.get(&field_name).map(|(d, _)| *d).unwrap_or(10);
                                        let mut uv = ul.borrow_mut();
                                        uv.push(current_val);
                                        if uv.len() > depth { uv.remove(0); }
                                    }
                                    inst.borrow_mut().fields.insert(undo_key, undo_hist);
                                    rl.borrow_mut().pop();
                                    inst.borrow_mut().fields.insert(field_name.clone(), next_val);
                                    inst.borrow_mut().fields.insert(redo_key, redo_hist);
                                    return Ok(Value::Bool(true));
                                }
                            }
                            return Ok(Value::Bool(false));
                        }
                    }
                    "history" => {
                        // history("field_name") returns the temporal history list
                        let field_name = args.first()
                            .and_then(|a| if let Value::String(s) = a { Some(s.clone()) } else { None })
                            .unwrap_or_default();
                        if temporal_props.contains_key(&field_name) {
                            let hist_key = format!("__temporal_hist_{}", field_name);
                            let hist = inst.borrow().fields.get(&hist_key).cloned()
                                .unwrap_or_else(|| Value::List(Rc::new(RefCell::new(Vec::new()))));
                            return Ok(hist);
                        }
                    }
                    "previous" => {
                        let field_name = args.first()
                            .and_then(|a| if let Value::String(s) = a { Some(s.clone()) } else { None })
                            .unwrap_or_default();
                        if temporal_props.contains_key(&field_name) {
                            let prev_key = format!("__temporal_prev_{}", field_name);
                            let prev = inst.borrow().fields.get(&prev_key).cloned().unwrap_or(Value::Nil);
                            return Ok(prev);
                        }
                    }
                    "mutations" => {
                        let muts = inst.borrow().fields.get("__mutations").cloned()
                            .unwrap_or_else(|| Value::List(Rc::new(RefCell::new(Vec::new()))));
                        return Ok(muts);
                    }
                    "as" => {
                        // Face projection: obj.as("face_name") or obj.as(.face_name)
                        let face_name = args.first()
                            .and_then(|a| if let Value::String(s) = a { Some(s.clone()) } else { None })
                            .unwrap_or_default();
                        if let Some(visible_fields) = faces.get(&face_name) {
                            let visible_fields = visible_fields.clone();
                            let mut projected_fields = HashMap::new();
                            let inst_fields = inst.borrow().fields.clone();
                            for f in &visible_fields {
                                if let Some(v) = inst_fields.get(f) {
                                    projected_fields.insert(f.clone(), v.clone());
                                }
                            }
                            // Return as a map representing the face view
                            return Ok(Value::Map(Rc::new(RefCell::new(projected_fields))));
                        }
                    }
                    "freeze" => {
                        inst.borrow_mut().fields.insert("__frozen".to_string(), Value::Bool(true));
                        return Ok(Value::Nil);
                    }
                    "unfreeze" => {
                        inst.borrow_mut().fields.insert("__frozen".to_string(), Value::Bool(false));
                        return Ok(Value::Nil);
                    }
                    _ => {}
                }

                // ── Morph method dispatch ─────────────────────────────────────
                // Evaluate when-clauses in order; use body of first matching clause.
                if let Some(morph_def) = morph_methods.get(method) {
                    let morph_params = morph_def.params.clone();
                    let when_clauses = morph_def.when_clauses.clone();
                    for clause in &when_clauses {
                        let mut cond_env = env.snapshot();
                        cond_env.push_scope();
                        cond_env.define("self", obj.clone());
                        for (i, param) in morph_params.iter().enumerate() {
                            let val = if i < args.len() { args[i].clone() } else { Value::Nil };
                            cond_env.define(&param.name, val);
                        }
                        let cond_val = crate::interpreter::eval::eval_expr(&clause.condition, &mut cond_env)
                            .unwrap_or(Value::Bool(false));
                        if cond_val.is_truthy() {
                            let body_stmts = clause.body.clone();
                            let mut method_env = env.snapshot();
                            method_env.push_scope();
                            method_env.define("self", obj.clone());
                            for (i, param) in morph_params.iter().enumerate() {
                                let val = if i < args.len() { args[i].clone() } else { Value::Nil };
                                method_env.define(&param.name, val);
                            }
                            return match crate::interpreter::exec::exec_block(&body_stmts, &mut method_env) {
                                Ok(()) => Ok(Value::Nil),
                                Err(Signal::Return(val)) => Ok(val),
                                Err(e) => Err(e),
                            };
                        }
                    }
                    // No clause matched — return nil
                    return Ok(Value::Nil);
                }

                // ── Capabilities check ───────────────────────────────────────
                // Restricted operations that require specific capabilities.
                const RESTRICTED_OPS: &[(&str, &str)] = &[
                    ("fs_read",      "read_file"),
                    ("fs_write",     "write_file"),
                    ("net_request",  "network"),
                    ("net_connect",  "network"),
                    ("http_get",     "network"),
                    ("http_post",    "network"),
                ];
                for (op_name, required_cap) in RESTRICTED_OPS {
                    if method == *op_name && !capabilities.contains(&required_cap.to_string()) {
                        return Err(Signal::Throw(Value::String(format!(
                            "method '{}' requires capability '{}' which is not declared on this class",
                            method, required_cap
                        ))));
                    }
                }

                // ── Extension methods ─────────────────────────────────────────
                let extensions = crate::interpreter::values::get_extensions(&class_name);
                for ext_method in &extensions {
                    if ext_method.name == method {
                        let func = ext_method.clone();
                        let mut method_env = env.snapshot();
                        method_env.push_scope();
                        method_env.define("self", obj.clone());
                        for (i, param) in func.params.iter().enumerate() {
                            let val = if i < args.len() { args[i].clone() } else { Value::Nil };
                            method_env.define(&param.name, val);
                        }
                        return match &func.body {
                            FuncBody::Expression(expr) => eval_expr(expr, &mut method_env),
                            FuncBody::Block(stmts) => {
                                match crate::interpreter::exec::exec_block(stmts, &mut method_env) {
                                    Ok(()) => Ok(Value::Nil),
                                    Err(Signal::Return(val)) => Ok(val),
                                    Err(e) => Err(e),
                                }
                            }
                        };
                    }
                }

                // ── Delegate method forwarding ────────────────────────────────
                for delegate_field in &delegates {
                    let delegate_val = inst.borrow().fields.get(delegate_field).cloned();
                    if let Some(delegate_obj) = delegate_val {
                        // Try to call the method on the delegated object
                        if let Value::Instance(_) = &delegate_obj {
                            return call_method(&delegate_obj, method, args, env);
                        }
                    }
                }

                return Err(Signal::Throw(Value::String(format!("undefined method '{}' on {}", method, obj))));
            }
            Err(Signal::Throw(Value::String(format!("undefined method '{}' on {}", method, obj))))
        }

        // Static method calls: ClassName.method()
        Value::Class(cls) => {
            if let Some(func) = cls.static_methods.get(method) {
                let func = func.clone();
                let mut method_env = env.snapshot();
                method_env.push_scope();
                for (i, param) in func.params.iter().enumerate() {
                    let val = if i < args.len() { args[i].clone() } else { Value::Nil };
                    method_env.define(&param.name, val);
                }
                return match &func.body {
                    FuncBody::Expression(expr) => eval_expr(expr, &mut method_env),
                    FuncBody::Block(stmts) => {
                        match crate::interpreter::exec::exec_block_with_value(stmts, &mut method_env) {
                            Ok(val) => Ok(val),
                            Err(Signal::Return(val)) => Ok(val),
                            Err(e) => Err(e),
                        }
                    }
                };
            }
            Err(Signal::Throw(Value::String(format!("undefined static method '{}' on {}", method, cls.name))))
        }
        // Python object method calls
        Value::PythonObject(wrapper) => {
            crate::bridge::python::python_call(wrapper, method, args)
                .map_err(|e| Signal::Throw(Value::String(e)))
        }
        _ => {
            // Extension methods on primitives
            let type_name = match obj {
                Value::Int(_) => "Int",
                Value::Float(_) => "Float",
                Value::String(_) => "Str",
                Value::Bool(_) => "Bool",
                _ => "",
            };
            if !type_name.is_empty() {
                let extensions = crate::interpreter::values::get_extensions(type_name);
                for ext_method in &extensions {
                    if ext_method.name == method {
                        let func = ext_method.clone();
                        let mut method_env = env.snapshot();
                        method_env.push_scope();
                        method_env.define("self", obj.clone());
                        for (i, param) in func.params.iter().enumerate() {
                            let val = if i < args.len() { args[i].clone() } else { Value::Nil };
                            method_env.define(&param.name, val);
                        }
                        return match &func.body {
                            FuncBody::Expression(expr) => eval_expr(expr, &mut method_env),
                            FuncBody::Block(stmts) => {
                                match crate::interpreter::exec::exec_block(stmts, &mut method_env) {
                                    Ok(()) => Ok(Value::Nil),
                                    Err(Signal::Return(val)) => Ok(val),
                                    Err(e) => Err(e),
                                }
                            }
                        };
                    }
                }
            }
            Err(Signal::Throw(Value::String(format!("undefined method '{}' on {}", method, obj))))
        }
    }
}

/// Check for built-in methods on primitive types and collections.
fn builtin_method(obj: &Value, method: &str, args: &[Value]) -> Result<Option<Value>, Signal> {
    match obj {
        Value::String(s) => {
            let result = match method {
                "len" => Value::Int(s.len() as i64),
                "upper" => Value::String(s.to_uppercase()),
                "lower" => Value::String(s.to_lowercase()),
                "trim" => Value::String(s.trim().to_string()),
                "contains" => {
                    let sub = args.first().and_then(|a| if let Value::String(s) = a { Some(s.as_str()) } else { None }).unwrap_or("");
                    Value::Bool(s.contains(sub))
                }
                "starts_with" => {
                    let sub = args.first().and_then(|a| if let Value::String(s) = a { Some(s.as_str()) } else { None }).unwrap_or("");
                    Value::Bool(s.starts_with(sub))
                }
                "ends_with" => {
                    let sub = args.first().and_then(|a| if let Value::String(s) = a { Some(s.as_str()) } else { None }).unwrap_or("");
                    Value::Bool(s.ends_with(sub))
                }
                "split" => {
                    let sep = args.first().and_then(|a| if let Value::String(s) = a { Some(s.clone()) } else { None }).unwrap_or_else(|| " ".to_string());
                    let parts: Vec<Value> = s.split(&sep).map(|p| Value::String(p.to_string())).collect();
                    Value::List(Rc::new(RefCell::new(parts)))
                }
                "replace" => {
                    let old = args.first().and_then(|a| if let Value::String(s) = a { Some(s.as_str()) } else { None }).unwrap_or("");
                    let new = args.get(1).and_then(|a| if let Value::String(s) = a { Some(s.as_str()) } else { None }).unwrap_or("");
                    Value::String(s.replace(old, new))
                }
                "chars" => {
                    let chars: Vec<Value> = s.chars().map(|c| Value::Char(c)).collect();
                    Value::List(Rc::new(RefCell::new(chars)))
                }
                "repeat" => {
                    let n = args.first().and_then(|a| a.as_int()).unwrap_or(1) as usize;
                    Value::String(s.repeat(n))
                }
                "parse_int" => {
                    match s.parse::<i64>() {
                        Ok(n) => Value::Ok(Box::new(Value::Int(n))),
                        Err(_) => Value::Err(Box::new(Value::String("parse error".into()))),
                    }
                }
                "parse_float" => {
                    match s.parse::<f64>() {
                        Ok(n) => Value::Ok(Box::new(Value::Float(n))),
                        Err(_) => Value::Err(Box::new(Value::String("parse error".into()))),
                    }
                }
                "slice" => {
                    let start = args.first().and_then(|a| a.as_int()).unwrap_or(0) as usize;
                    let end = args.get(1).and_then(|a| a.as_int()).unwrap_or(s.len() as i64) as usize;
                    Value::String(s.chars().skip(start).take(end.saturating_sub(start)).collect())
                }
                _ => return Ok(None),
            };
            Ok(Some(result))
        }
        Value::List(items) => {
            let result = match method {
                "len" => Value::Int(items.borrow().len() as i64),
                "push" => {
                    if let Some(val) = args.first() {
                        items.borrow_mut().push(val.clone());
                    }
                    Value::Nil
                }
                "pop" => items.borrow_mut().pop().unwrap_or(Value::Nil),
                "first" => items.borrow().first().cloned().unwrap_or(Value::Nil),
                "last" => items.borrow().last().cloned().unwrap_or(Value::Nil),
                "contains" => {
                    let target = args.first().unwrap_or(&Value::Nil);
                    Value::Bool(items.borrow().iter().any(|v| v.equals(target)))
                }
                "reverse" => {
                    let mut v = items.borrow().clone();
                    v.reverse();
                    Value::List(Rc::new(RefCell::new(v)))
                }
                "join" => {
                    let sep = args.first().and_then(|a| if let Value::String(s) = a { Some(s.clone()) } else { None }).unwrap_or_default();
                    let s: String = items.borrow().iter().map(|v| v.to_string()).collect::<Vec<_>>().join(&sep);
                    Value::String(s)
                }
                "map" => {
                    if let Some(func) = args.first() {
                        let mut result = Vec::new();
                        for item in items.borrow().iter() {
                            result.push(call_function(func, vec![item.clone()], &[], &mut Environment::new())?);
                        }
                        Value::List(Rc::new(RefCell::new(result)))
                    } else { Value::Nil }
                }
                "filter" => {
                    if let Some(func) = args.first() {
                        let mut result = Vec::new();
                        for item in items.borrow().iter() {
                            let keep = call_function(func, vec![item.clone()], &[], &mut Environment::new())?;
                            if keep.is_truthy() {
                                result.push(item.clone());
                            }
                        }
                        Value::List(Rc::new(RefCell::new(result)))
                    } else { Value::Nil }
                }
                "sum" => {
                    let items = items.borrow();
                    let mut total_int: i64 = 0;
                    let mut is_float = false;
                    let mut total_float: f64 = 0.0;
                    for item in items.iter() {
                        match item {
                            Value::Int(n) => { total_int += n; total_float += *n as f64; }
                            Value::Float(n) => { is_float = true; total_float += n; }
                            _ => {}
                        }
                    }
                    if is_float { Value::Float(total_float) } else { Value::Int(total_int) }
                }
                "min" => {
                    let items = items.borrow();
                    items.iter().fold(None, |acc: Option<&Value>, v| {
                        match acc {
                            None => Some(v),
                            Some(a) => {
                                let a_f = a.as_float().unwrap_or(f64::MAX);
                                let v_f = v.as_float().unwrap_or(f64::MAX);
                                if v_f < a_f { Some(v) } else { Some(a) }
                            }
                        }
                    }).cloned().unwrap_or(Value::Nil)
                }
                "max" => {
                    let items = items.borrow();
                    items.iter().fold(None, |acc: Option<&Value>, v| {
                        match acc {
                            None => Some(v),
                            Some(a) => {
                                let a_f = a.as_float().unwrap_or(f64::MIN);
                                let v_f = v.as_float().unwrap_or(f64::MIN);
                                if v_f > a_f { Some(v) } else { Some(a) }
                            }
                        }
                    }).cloned().unwrap_or(Value::Nil)
                }
                "index_of" => {
                    let target = args.first().unwrap_or(&Value::Nil);
                    let idx = items.borrow().iter().position(|v| v.equals(target));
                    match idx {
                        Some(i) => Value::Int(i as i64),
                        None => Value::Int(-1),
                    }
                }
                "remove" => {
                    let idx = args.first().and_then(|a| a.as_int()).unwrap_or(0) as usize;
                    let mut items = items.borrow_mut();
                    if idx < items.len() { items.remove(idx) } else { Value::Nil }
                }
                "insert" => {
                    let idx = args.first().and_then(|a| a.as_int()).unwrap_or(0) as usize;
                    let val = args.get(1).cloned().unwrap_or(Value::Nil);
                    items.borrow_mut().insert(idx, val);
                    Value::Nil
                }
                "sort" => {
                    let mut v = items.borrow().clone();
                    v.sort_by(|a, b| {
                        let af = a.as_float().unwrap_or(0.0);
                        let bf = b.as_float().unwrap_or(0.0);
                        af.partial_cmp(&bf).unwrap_or(std::cmp::Ordering::Equal)
                    });
                    Value::List(Rc::new(RefCell::new(v)))
                }
                "any" => {
                    if let Some(func) = args.first() {
                        let result = items.borrow().iter().any(|item| {
                            call_function(func, vec![item.clone()], &[], &mut Environment::new())
                                .map(|v| v.is_truthy()).unwrap_or(false)
                        });
                        Value::Bool(result)
                    } else { Value::Bool(false) }
                }
                "all" => {
                    if let Some(func) = args.first() {
                        let result = items.borrow().iter().all(|item| {
                            call_function(func, vec![item.clone()], &[], &mut Environment::new())
                                .map(|v| v.is_truthy()).unwrap_or(false)
                        });
                        Value::Bool(result)
                    } else { Value::Bool(true) }
                }
                "unique" => {
                    let items = items.borrow();
                    let mut seen = Vec::new();
                    for item in items.iter() {
                        if !seen.iter().any(|s: &Value| s.equals(item)) {
                            seen.push(item.clone());
                        }
                    }
                    Value::List(Rc::new(RefCell::new(seen)))
                }
                "reduce" => {
                    if args.len() >= 2 {
                        let mut acc = args[0].clone();
                        let func = &args[1];
                        for item in items.borrow().iter() {
                            acc = call_function(func, vec![acc, item.clone()], &[], &mut Environment::new())?;
                        }
                        acc
                    } else { Value::Nil }
                }
                "flat_map" => {
                    if let Some(func) = args.first() {
                        let mut result = Vec::new();
                        for item in items.borrow().iter() {
                            let mapped = call_function(func, vec![item.clone()], &[], &mut Environment::new())?;
                            if let Value::List(inner) = mapped {
                                result.extend(inner.borrow().clone());
                            } else {
                                result.push(mapped);
                            }
                        }
                        Value::List(Rc::new(RefCell::new(result)))
                    } else { Value::Nil }
                }
                "zip" => {
                    if let Some(Value::List(other)) = args.first() {
                        let other = other.borrow();
                        let items = items.borrow();
                        let result: Vec<Value> = items.iter().zip(other.iter())
                            .map(|(a, b)| Value::Tuple(vec![a.clone(), b.clone()]))
                            .collect();
                        Value::List(Rc::new(RefCell::new(result)))
                    } else { Value::Nil }
                }
                "chunks" => {
                    let n = args.first().and_then(|a| a.as_int()).unwrap_or(1) as usize;
                    let items = items.borrow();
                    let chunks: Vec<Value> = items.chunks(n)
                        .map(|c| Value::List(Rc::new(RefCell::new(c.to_vec()))))
                        .collect();
                    Value::List(Rc::new(RefCell::new(chunks)))
                }
                _ => return Ok(None),
            };
            Ok(Some(result))
        }
        Value::Map(map) => {
            let result = match method {
                "len" => Value::Int(map.borrow().len() as i64),
                "keys" => {
                    let keys: Vec<Value> = map.borrow().keys().map(|k| Value::String(k.clone())).collect();
                    Value::List(Rc::new(RefCell::new(keys)))
                }
                "values" => {
                    let vals: Vec<Value> = map.borrow().values().cloned().collect();
                    Value::List(Rc::new(RefCell::new(vals)))
                }
                "contains_key" => {
                    let key = args.first().and_then(|a| a.as_map_key()).unwrap_or_default();
                    Value::Bool(map.borrow().contains_key(&key))
                }
                "get" => {
                    let key = args.first().and_then(|a| a.as_map_key()).unwrap_or_default();
                    map.borrow().get(&key).cloned().unwrap_or(Value::Nil)
                }
                "set" => {
                    let key = args.first().and_then(|a| a.as_map_key()).unwrap_or_default();
                    let val = args.get(1).cloned().unwrap_or(Value::Nil);
                    map.borrow_mut().insert(key, val);
                    Value::Nil
                }
                "remove" => {
                    let key = args.first().and_then(|a| a.as_map_key()).unwrap_or_default();
                    map.borrow_mut().remove(&key).unwrap_or(Value::Nil)
                }
                "entries" => {
                    let entries: Vec<Value> = map.borrow().iter().map(|(k, v)| {
                        Value::Tuple(vec![Value::String(k.clone()), v.clone()])
                    }).collect();
                    Value::List(Rc::new(RefCell::new(entries)))
                }
                _ => return Ok(None),
            };
            Ok(Some(result))
        }
        Value::Set(items) => {
            let result = match method {
                "len" => Value::Int(items.borrow().len() as i64),
                "contains" => {
                    let target = args.first().unwrap_or(&Value::Nil);
                    Value::Bool(items.borrow().iter().any(|v| v.equals(target)))
                }
                "insert" => {
                    if let Some(val) = args.first() {
                        let mut items = items.borrow_mut();
                        if !items.iter().any(|v| v.equals(val)) {
                            items.push(val.clone());
                        }
                    }
                    Value::Nil
                }
                "remove" => {
                    if let Some(val) = args.first() {
                        let mut items = items.borrow_mut();
                        items.retain(|v| !v.equals(val));
                    }
                    Value::Nil
                }
                "union" => {
                    if let Some(Value::Set(other)) = args.first() {
                        let mut result = items.borrow().clone();
                        for item in other.borrow().iter() {
                            if !result.iter().any(|v| v.equals(item)) {
                                result.push(item.clone());
                            }
                        }
                        Value::Set(Rc::new(RefCell::new(result)))
                    } else { Value::Nil }
                }
                "intersect" => {
                    if let Some(Value::Set(other)) = args.first() {
                        let other = other.borrow();
                        let result: Vec<Value> = items.borrow().iter()
                            .filter(|v| other.iter().any(|o| o.equals(v)))
                            .cloned().collect();
                        Value::Set(Rc::new(RefCell::new(result)))
                    } else { Value::Nil }
                }
                "difference" => {
                    if let Some(Value::Set(other)) = args.first() {
                        let other = other.borrow();
                        let result: Vec<Value> = items.borrow().iter()
                            .filter(|v| !other.iter().any(|o| o.equals(v)))
                            .cloned().collect();
                        Value::Set(Rc::new(RefCell::new(result)))
                    } else { Value::Nil }
                }
                _ => return Ok(None),
            };
            Ok(Some(result))
        }
        _ => Ok(None),
    }
}

// ═══════════════════════════════════════════════════════════════
// Field access
// ═══════════════════════════════════════════════════════════════

fn access_field(obj: &Value, field: &str) -> Result<Value, Signal> {
    match obj {
        Value::Instance(inst) => {
            // First check computed/reactive properties (don't need to look at stored fields)
            {
                let inst_borrow = inst.borrow();
                if let Some(cls) = &inst_borrow.class {
                    // Reactive props: recompute on every access
                    if let Some(rp) = cls.reactive_props.get(field) {
                        let rp = rp.clone();
                        drop(inst_borrow);
                        let mut cp_env = Environment::new();
                        cp_env.define("self", obj.clone());
                        return eval_expr(&rp, &mut cp_env);
                    }
                    // Computed props: also recompute on every access
                    if let Some(cp) = cls.computed_props.get(field) {
                        let cp = cp.clone();
                        drop(inst_borrow);
                        let mut cp_env = Environment::new();
                        cp_env.define("self", obj.clone());
                        return eval_expr(&cp, &mut cp_env);
                    }
                }
            }

            let inst_borrow = inst.borrow();
            if let Some(val) = inst_borrow.fields.get(field) {
                return Ok(val.clone());
            }
            // Lazy property: evaluate on first access and cache
            if let Some(cls) = &inst_borrow.class {
                if let Some(lazy_expr) = cls.lazy_props.get(field) {
                    let lazy_expr = lazy_expr.clone();
                    drop(inst_borrow);
                    let mut lazy_env = Environment::new();
                    lazy_env.define("self", obj.clone());
                    let val = eval_expr(&lazy_expr, &mut lazy_env)?;
                    // Cache the result
                    inst.borrow_mut().fields.insert(field.to_string(), val.clone());
                    return Ok(val);
                }
            }
            Err(Signal::Throw(Value::String(format!("undefined field '{}' on {}", field, inst_borrow.class_name))))
        }
        Value::StructInstance(inst) => {
            let inst = inst.borrow();
            if let Some(val) = inst.fields.get(field) {
                Ok(val.clone())
            } else {
                Err(Signal::Throw(Value::String(format!("undefined field '{}' on {}", field, inst.class_name))))
            }
        }
        Value::EnumVariant { fields, .. } => {
            // Access fields by index using special field names
            match field.parse::<usize>() {
                Ok(i) => fields.get(i).cloned().ok_or_else(|| {
                    Signal::Throw(Value::String(format!("field index {} out of bounds", i)))
                }),
                Err(_) => Err(Signal::Throw(Value::String(format!("unknown field '{}' on enum variant", field)))),
            }
        }
        Value::Tuple(items) => {
            match field.parse::<usize>() {
                Ok(i) => items.get(i).cloned().ok_or_else(|| {
                    Signal::Throw(Value::String(format!("tuple index {} out of bounds", i)))
                }),
                Err(_) => Err(Signal::Throw(Value::String(format!("unknown field '{}' on tuple", field)))),
            }
        }
        // Python object attribute access
        Value::PythonObject(wrapper) => {
            crate::bridge::python::python_getattr(wrapper, field)
                .map_err(|e| Signal::Throw(Value::String(e)))
        }
        _ => Err(Signal::Throw(Value::String(format!("cannot access field '{}' on {}", field, obj)))),
    }
}

// ═══════════════════════════════════════════════════════════════
// Match evaluation
// ═══════════════════════════════════════════════════════════════

pub fn eval_match(val: &Value, arms: &[MatchArm], env: &mut Environment) -> Result<Value, Signal> {
    for arm in arms {
        env.push_scope();
        let matched = match_pattern(val, &arm.pattern, env);
        if matched {
            if let Some(guard) = &arm.guard {
                let g = eval_expr(guard, env)?;
                if !g.is_truthy() {
                    env.pop_scope();
                    continue;
                }
            }
            let result = match &arm.body {
                MatchBody::Expression(expr) => eval_expr(expr, env),
                MatchBody::Block(stmts) => {
                    match crate::interpreter::exec::exec_block(stmts, env) {
                        Ok(()) => Ok(Value::Nil),
                        Err(Signal::Return(v)) => Ok(v),
                        Err(e) => Err(e),
                    }
                }
            };
            env.pop_scope();
            return result;
        }
        env.pop_scope();
    }
    Ok(Value::Nil)
}

pub fn match_pattern(val: &Value, pattern: &Pattern, env: &mut Environment) -> bool {
    match pattern {
        Pattern::Wildcard => true,
        Pattern::Binding(name) => {
            env.define(name, val.clone());
            true
        }
        Pattern::Literal(expr) => {
            if let Ok(lit_val) = eval_expr(expr, env) {
                val.equals(&lit_val)
            } else {
                false
            }
        }
        Pattern::Range { start, end, inclusive } => {
            if let (Ok(s), Ok(e)) = (eval_expr(start, env), eval_expr(end, env)) {
                let val_f = val.as_float().unwrap_or(f64::NAN);
                let s_f = s.as_float().unwrap_or(f64::NAN);
                let e_f = e.as_float().unwrap_or(f64::NAN);
                if *inclusive {
                    val_f >= s_f && val_f <= e_f
                } else {
                    val_f >= s_f && val_f < e_f
                }
            } else {
                false
            }
        }
        Pattern::Destructure { name, fields } => {
            match val {
                Value::Ok(inner) if name == "Ok" => {
                    if fields.len() == 1 {
                        match_pattern(inner, &fields[0], env)
                    } else {
                        false
                    }
                }
                Value::Err(inner) if name == "Err" => {
                    if fields.len() == 1 {
                        match_pattern(inner, &fields[0], env)
                    } else {
                        false
                    }
                }
                Value::EnumVariant { variant, fields: vals, .. } if variant == name => {
                    if fields.len() != vals.len() { return false; }
                    fields.iter().zip(vals.iter()).all(|(p, v)| match_pattern(v, p, env))
                }
                _ => false,
            }
        }
        Pattern::EnumVariant { variant, fields } => {
            if let Value::EnumVariant { variant: v, fields: vals, .. } = val {
                if v != variant { return false; }
                if fields.len() != vals.len() { return false; }
                fields.iter().zip(vals.iter()).all(|(p, v)| match_pattern(v, p, env))
            } else {
                false
            }
        }
        Pattern::Tuple(patterns) => {
            if let Value::Tuple(vals) = val {
                if patterns.len() != vals.len() { return false; }
                patterns.iter().zip(vals.iter()).all(|(p, v)| match_pattern(v, p, env))
            } else {
                false
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════

/// Convert a value to an iterable list of values.
pub fn value_to_iter(val: &Value) -> Result<Vec<Value>, Signal> {
    match val {
        Value::List(items) => Ok(items.borrow().clone()),
        Value::Set(items) => Ok(items.borrow().clone()),
        Value::Tuple(items) => Ok(items.clone()),
        Value::String(s) => Ok(s.chars().map(|c| Value::Char(c)).collect()),
        Value::Range { start, end, inclusive, step } => {
            let mut items = Vec::new();
            let mut i = *start;
            let step = *step;
            if step > 0 {
                while if *inclusive { i <= *end } else { i < *end } {
                    items.push(Value::Int(i));
                    i += step;
                }
            } else if step < 0 {
                while if *inclusive { i >= *end } else { i > *end } {
                    items.push(Value::Int(i));
                    i += step;
                }
            }
            Ok(items)
        }
        Value::Map(map) => {
            Ok(map.borrow().iter().map(|(k, v)| {
                Value::Tuple(vec![Value::String(k.clone()), v.clone()])
            }).collect())
        }
        _ => Err(Signal::Throw(Value::String(format!("{} is not iterable", val)))),
    }
}

/// Call an operator overload method on a class instance.
fn call_operator_method(obj: &Value, op_def: &crate::parser::ast::OperatorDef, args: Vec<Value>, env: &mut Environment) -> Result<Value, Signal> {
    let mut method_env = env.snapshot();
    method_env.push_scope();
    method_env.define("self", obj.clone());
    for (i, param) in op_def.params.iter().enumerate() {
        let val = if i < args.len() { args[i].clone() } else { Value::Nil };
        method_env.define(&param.name, val);
    }
    match crate::interpreter::exec::exec_block_with_value(&op_def.body, &mut method_env) {
        Ok(val) => Ok(val),
        Err(Signal::Return(val)) => Ok(val),
        Err(e) => Err(e),
    }
}

/// Call a method on the parent class (super.method()).
fn call_super_method(self_val: &Value, method: &str, args: Vec<Value>, env: &mut Environment) -> Result<Value, Signal> {
    if let Value::Instance(inst) = self_val {
        let inst_borrow = inst.borrow();
        if let Some(cls) = &inst_borrow.class {
            if let Some(parent) = &cls.parent {
                if let Some(func) = parent.methods.get(method) {
                    let func = func.clone();
                    drop(inst_borrow);
                    let mut method_env = env.snapshot();
                    method_env.push_scope();
                    method_env.define("self", self_val.clone());
                    for (i, param) in func.params.iter().enumerate() {
                        let val = if i < args.len() { args[i].clone() } else { Value::Nil };
                        method_env.define(&param.name, val);
                    }
                    return match &func.body {
                        FuncBody::Expression(expr) => eval_expr(expr, &mut method_env),
                        FuncBody::Block(stmts) => {
                            match crate::interpreter::exec::exec_block_with_value(stmts, &mut method_env) {
                                Ok(val) => Ok(val),
                                Err(Signal::Return(val)) => Ok(val),
                                Err(e) => Err(e),
                            }
                        }
                    };
                }
            }
        }
    }
    Err(Signal::Throw(Value::String(format!("super.{}() not found", method))))
}

fn eval_cast(val: Value, target: &TypeAnnotation) -> Result<Value, Signal> {
    let type_name = match target {
        TypeAnnotation::Simple(name) => name.as_str(),
        _ => return Ok(val),
    };
    match (val, type_name) {
        (Value::Int(n), "Float") => Ok(Value::Float(n as f64)),
        (Value::Float(f), "Int") => Ok(Value::Int(f as i64)),
        (Value::Int(n), "Str") => Ok(Value::String(n.to_string())),
        (Value::Float(n), "Str") => Ok(Value::String(n.to_string())),
        (Value::Bool(b), "Str") => Ok(Value::String(b.to_string())),
        (Value::Bool(b), "Int") => Ok(Value::Int(if b { 1 } else { 0 })),
        (val, _) => Ok(val),
    }
}

// ═══════════════════════════════════════════════════════════════
// Genetic algorithm helpers
// ═══════════════════════════════════════════════════════════════

/// Tiny xorshift64 PRNG — no external crate needed.
/// Returns a new seed and a float in [0.0, 1.0).
fn rng_next(seed: &mut u64) -> f64 {
    let mut x = *seed;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *seed = x;
    // Map to [0, 1)
    (x >> 11) as f64 / (1u64 << 53) as f64
}

/// Seed from the current system time (nanoseconds as u64).
fn rng_seed_from_env() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64 + d.as_secs() * 1_000_000_000)
        .unwrap_or(12345678901234567u64);
    // Mix to avoid zero-seed issues
    let s = nanos ^ 0xdeadbeefcafe1234u64;
    if s == 0 { 1 } else { s }
}

/// Generate a random value for a single gene definition.
fn random_gene_value(
    gene: &crate::parser::ast::GeneDef,
    seed: &mut u64,
    env: &mut Environment,
) -> Result<Value, Signal> {
    // Options list takes priority: pick one uniformly at random.
    if let Some(options) = &gene.options {
        if !options.is_empty() {
            let idx = (rng_next(seed) * options.len() as f64) as usize;
            let idx = idx.min(options.len() - 1);
            return eval_expr(&options[idx], env);
        }
    }

    // Range-based gene.
    if let Some((lo_expr, hi_expr)) = &gene.range {
        let lo = eval_expr(lo_expr, env)?;
        let hi = eval_expr(hi_expr, env)?;

        // Determine step (default 1 for Int, any value for Float).
        let step_val = if let Some(step_expr) = &gene.step {
            Some(eval_expr(step_expr, env)?)
        } else {
            None
        };

        match (&lo, &hi) {
            (Value::Int(lo_i), Value::Int(hi_i)) => {
                let lo_f = *lo_i as f64;
                let hi_f = *hi_i as f64;
                let r = rng_next(seed);
                let raw = lo_f + r * (hi_f - lo_f);
                let step = step_val.and_then(|s| s.as_int()).unwrap_or(1).max(1);
                let snapped = (((raw - lo_f) / step as f64).round() as i64) * step + lo_i;
                let snapped = snapped.max(*lo_i).min(*hi_i);
                return Ok(Value::Int(snapped));
            }
            _ => {
                // Float range
                let lo_f = lo.as_float().unwrap_or(0.0);
                let hi_f = hi.as_float().unwrap_or(1.0);
                let r = rng_next(seed);
                let raw = lo_f + r * (hi_f - lo_f);
                if let Some(step_v) = step_val {
                    let step_f = step_v.as_float().unwrap_or(0.0);
                    if step_f > 0.0 {
                        let snapped = ((raw - lo_f) / step_f).round() * step_f + lo_f;
                        let snapped = snapped.max(lo_f).min(hi_f);
                        return Ok(Value::Float(snapped));
                    }
                }
                return Ok(Value::Float(raw));
            }
        }
    }

    // Default value specified.
    if let Some(default_expr) = &gene.default {
        return eval_expr(default_expr, env);
    }

    // Fallback: 0.0
    Ok(Value::Float(0.0))
}

/// Evaluate fitness for a single genetic instance.
/// Returns a float score (higher = fitter).
/// Uses a minimal environment (just a global scope + the function scope) to
/// avoid cloning the entire caller environment for every individual.
fn eval_fitness(
    instance: &Value,
    cls: &ClassValue,
    fitness_data: &Option<Value>,
    global_env: &Environment,
) -> Result<f64, Signal> {
    if let Some(fitness_fn) = &cls.fitness_fn {
        let fitness_fn = fitness_fn.clone();
        // Build a lean environment: use the closure env if set, else a fresh global
        let mut method_env = if let Some(closure) = &fitness_fn.closure_env {
            closure.borrow().clone()
        } else {
            global_env.global_only()
        };
        method_env.push_scope();
        method_env.define("self", instance.clone());
        // Optionally bind data parameter
        if let (Some(data), Some(param)) = (fitness_data, fitness_fn.params.first()) {
            method_env.define(&param.name, data.clone());
        }
        let result = match &fitness_fn.body {
            FuncBody::Expression(expr) => eval_expr(expr, &mut method_env)?,
            FuncBody::Block(stmts) => {
                match crate::interpreter::exec::exec_block_with_value(stmts, &mut method_env) {
                    Ok(val) => val,
                    Err(Signal::Return(val)) => val,
                    Err(e) => return Err(e),
                }
            }
        };
        Ok(result.as_float().unwrap_or(0.0))
    } else {
        Ok(0.0)
    }
}

/// Crossover two genetic instances, optionally applying mutation.
/// Gene values from parent_a and parent_b are alternated per gene (uniform crossover).
/// `seed` is threaded through for efficiency; pass None to create a fresh seed.
pub fn eval_crossover(
    parent_a: &Value,
    parent_b: &Value,
    mutation_rate: f64,
    env: &mut Environment,
) -> Result<Value, Signal> {
    let mut seed = rng_seed_from_env();
    eval_crossover_with_seed(parent_a, parent_b, mutation_rate, env, &mut seed)
}

fn eval_crossover_with_seed(
    parent_a: &Value,
    parent_b: &Value,
    mutation_rate: f64,
    env: &mut Environment,
    seed: &mut u64,
) -> Result<Value, Signal> {
    let (inst_a, cls) = match parent_a {
        Value::Instance(i) => {
            let borrow = i.borrow();
            let cls = borrow.class.clone().ok_or_else(|| {
                Signal::Throw(Value::String("crossover requires class instances".into()))
            })?;
            if !cls.is_genetic {
                return Err(Signal::Throw(Value::String(
                    "crossover requires genetic class instances".into()
                )));
            }
            (i.clone(), cls)
        }
        _ => return Err(Signal::Throw(Value::String("crossover: parent_a is not an instance".into()))),
    };

    let inst_b = match parent_b {
        Value::Instance(i) => i.clone(),
        _ => return Err(Signal::Throw(Value::String("crossover: parent_b is not an instance".into()))),
    };

    let mut child_fields = inst_a.borrow().fields.clone();
    let fields_b = inst_b.borrow().fields.clone();

    for chromosome in &cls.chromosomes {
        for gene in &chromosome.genes {
            // Uniform crossover: 50% chance to take from parent_b
            let take_b = rng_next(seed) < 0.5;
            if take_b {
                if let Some(val) = fields_b.get(&gene.name) {
                    child_fields.insert(gene.name.clone(), val.clone());
                }
            }

            // Apply mutation
            if mutation_rate > 0.0 && rng_next(seed) < mutation_rate {
                let mutated = random_gene_value(gene, seed, env)?;
                child_fields.insert(gene.name.clone(), mutated);
            }
        }
    }

    let child = InstanceValue {
        class_name: inst_a.borrow().class_name.clone(),
        fields: child_fields,
        class: Some(cls),
    };
    Ok(Value::Instance(Rc::new(RefCell::new(child))))
}

/// Evaluate an `evolve ClassName { ... }` block.
/// Runs a full generational genetic algorithm and returns the fittest individual.
fn eval_evolve_block(
    target: &str,
    config: &crate::parser::ast::EvolveConfig,
    env: &mut Environment,
) -> Result<Value, Signal> {
    // Resolve the class
    let class_val = env.get(target).ok_or_else(|| {
        Signal::Throw(Value::String(format!("evolve: class '{}' not found", target)))
    })?;
    let cls = match &class_val {
        Value::Class(c) => c.clone(),
        _ => return Err(Signal::Throw(Value::String(format!(
            "evolve: '{}' is not a class", target
        )))),
    };
    if !cls.is_genetic {
        return Err(Signal::Throw(Value::String(format!(
            "evolve: '{}' is not a genetic class", target
        ))));
    }

    // Evaluate config parameters
    let population_size = if let Some(pop_expr) = &config.population {
        eval_expr(pop_expr, env)?.as_int().unwrap_or(50) as usize
    } else { 50 };
    let generations = if let Some(gen_expr) = &config.generations {
        eval_expr(gen_expr, env)?.as_int().unwrap_or(100) as usize
    } else { 100 };
    let mutation_rate = if let Some(mr_expr) = &config.mutation_rate {
        eval_expr(mr_expr, env)?.as_float().unwrap_or(0.05)
    } else { 0.05 };
    let crossover_rate = if let Some(cr_expr) = &config.crossover_rate {
        eval_expr(cr_expr, env)?.as_float().unwrap_or(0.7)
    } else { 0.7 };
    let elitism_count = if let Some(el_expr) = &config.elitism {
        eval_expr(el_expr, env)?.as_int().unwrap_or(2) as usize
    } else { 2 };
    let fitness_data: Option<Value> = if let Some((_data_name, data_expr)) = &config.fitness_data {
        Some(eval_expr(data_expr, env)?)
    } else { None };

    let selection = config.selection.clone();

    let mut seed = rng_seed_from_env();

    // Create initial population by directly constructing random genetic instances
    // (avoids a full env.snapshot() per individual)
    let mut population: Vec<Value> = Vec::with_capacity(population_size);
    for _ in 0..population_size {
        let mut fields = HashMap::new();
        // Default fields from class
        for field in &cls.fields {
            if let Some(default) = &field.default {
                if let Ok(v) = eval_expr(default, env) {
                    fields.insert(field.name.clone(), v);
                }
            }
        }
        // Random gene values
        for chromosome in &cls.chromosomes {
            for gene in &chromosome.genes {
                let val = random_gene_value(gene, &mut seed, env)?;
                fields.insert(gene.name.clone(), val);
            }
        }
        fields.insert("__is_genetic".to_string(), Value::Bool(true));
        population.push(Value::Instance(Rc::new(RefCell::new(InstanceValue {
            class_name: cls.name.clone(),
            fields,
            class: Some(cls.clone()),
        }))));
    }

    // Snapshot the global env once for all fitness evaluations
    let global_env = env.global_only();

    for _gen in 0..generations {
        // Evaluate fitness for each individual
        let mut scored: Vec<(f64, Value)> = Vec::with_capacity(population.len());
        for individual in &population {
            let score = eval_fitness(individual, &cls, &fitness_data, &global_env)?;
            scored.push((score, individual.clone()));
        }

        // Sort descending by fitness (fittest first)
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        // Build next generation
        let mut next_gen: Vec<Value> = Vec::with_capacity(population_size);

        // Elitism: carry best N individuals unchanged
        let elite_n = elitism_count.min(scored.len());
        for i in 0..elite_n {
            next_gen.push(scored[i].1.clone());
        }

        // Fill the rest with crossover + mutation
        while next_gen.len() < population_size {
            let parent_a = select_parent(&scored, &selection, &mut seed);
            // Decide whether to do crossover or just clone
            let child = if rng_next(&mut seed) < crossover_rate && scored.len() >= 2 {
                let parent_b = select_parent(&scored, &selection, &mut seed);
                eval_crossover_with_seed(&parent_a, &parent_b, mutation_rate, env, &mut seed)?
            } else {
                // Mutation only: clone parent and mutate
                mutate_individual(&parent_a, &cls, mutation_rate, &mut seed, env)?
            };
            next_gen.push(child);
        }

        population = next_gen;
    }

    // Return the fittest individual from the final population
    let mut best_score = f64::NEG_INFINITY;
    let mut best: Option<Value> = None;
    for individual in &population {
        let score = eval_fitness(individual, &cls, &fitness_data, &global_env)?;
        if score > best_score {
            best_score = score;
            best = Some(individual.clone());
        }
    }

    // Store last_fitness on the best individual
    if let Some(Value::Instance(inst)) = &best {
        inst.borrow_mut().fields.insert("last_fitness".to_string(), Value::Float(best_score));
    }

    Ok(best.unwrap_or(Value::Nil))
}

/// Tournament or roulette selection of a parent from scored population.
fn select_parent(
    scored: &[(f64, Value)],
    selection: &Option<crate::parser::ast::SelectionMethod>,
    seed: &mut u64,
) -> Value {
    use crate::parser::ast::SelectionMethod;
    if scored.is_empty() {
        return Value::Nil;
    }
    match selection {
        None | Some(SelectionMethod::Tournament(_)) => {
            // Default tournament size 3
            let tournament_size = match selection {
                Some(SelectionMethod::Tournament(Some(size_expr))) => {
                    // Can't eval here — use a default of 3
                    let _ = size_expr;
                    3
                }
                _ => 3,
            };
            let mut best_idx = (rng_next(seed) * scored.len() as f64) as usize % scored.len();
            for _ in 1..tournament_size {
                let idx = (rng_next(seed) * scored.len() as f64) as usize % scored.len();
                if scored[idx].0 > scored[best_idx].0 {
                    best_idx = idx;
                }
            }
            scored[best_idx].1.clone()
        }
        Some(SelectionMethod::Roulette) => {
            // Fitness-proportionate selection (roulette wheel)
            let total: f64 = scored.iter().map(|(s, _)| s.max(0.0)).sum();
            if total <= 0.0 {
                // Fall back to uniform random
                let idx = (rng_next(seed) * scored.len() as f64) as usize % scored.len();
                return scored[idx].1.clone();
            }
            let mut pick = rng_next(seed) * total;
            for (score, individual) in scored {
                pick -= score.max(0.0);
                if pick <= 0.0 {
                    return individual.clone();
                }
            }
            scored.last().map(|(_, v)| v.clone()).unwrap_or(Value::Nil)
        }
        Some(SelectionMethod::Rank) => {
            // Rank-based: already sorted best-first, assign weights 1/rank
            let n = scored.len() as f64;
            let total = n * (n + 1.0) / 2.0;
            let mut pick = rng_next(seed) * total;
            for (rank, (_, individual)) in scored.iter().enumerate() {
                let weight = (n - rank as f64) as f64;
                pick -= weight;
                if pick <= 0.0 {
                    return individual.clone();
                }
            }
            scored.last().map(|(_, v)| v.clone()).unwrap_or(Value::Nil)
        }
    }
}

/// Mutate an individual by randomising each gene according to mutation_rate.
fn mutate_individual(
    individual: &Value,
    cls: &ClassValue,
    mutation_rate: f64,
    seed: &mut u64,
    env: &mut Environment,
) -> Result<Value, Signal> {
    if let Value::Instance(inst) = individual {
        let mut new_fields = inst.borrow().fields.clone();
        for chromosome in &cls.chromosomes {
            for gene in &chromosome.genes {
                if rng_next(seed) < mutation_rate {
                    let val = random_gene_value(gene, seed, env)?;
                    new_fields.insert(gene.name.clone(), val);
                }
            }
        }
        let child = InstanceValue {
            class_name: inst.borrow().class_name.clone(),
            fields: new_fields,
            class: Some(Rc::new(cls.clone())),
        };
        Ok(Value::Instance(Rc::new(RefCell::new(child))))
    } else {
        Ok(individual.clone())
    }
}

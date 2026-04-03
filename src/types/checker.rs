use crate::parser::ast::*;
use std::collections::HashMap;

/// Type information for the checker.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    Num,
    Bool,
    Str,
    Char,
    Nil,
    List(Box<Type>),
    Map(Box<Type>, Box<Type>),
    Set(Box<Type>),
    Tuple(Vec<Type>),
    Function(Vec<Type>, Box<Type>),
    Class(String),
    Optional(Box<Type>),
    Result(Box<Type>, Box<Type>),
    Any,
    Unknown,
}

/// A type error with location.
#[derive(Debug, Clone)]
pub struct TypeError {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub file: String,
}

/// Type checker state.
pub struct TypeChecker {
    scopes: Vec<HashMap<String, Type>>,
    errors: Vec<TypeError>,
    strict_mode: bool,
}

impl TypeChecker {
    pub fn new(strict: bool) -> Self {
        let mut global = HashMap::new();
        global.insert("print".to_string(), Type::Function(vec![Type::Any], Box::new(Type::Nil)));
        global.insert("len".to_string(), Type::Function(vec![Type::Any], Box::new(Type::Int)));
        global.insert("sqrt".to_string(), Type::Function(vec![Type::Float], Box::new(Type::Float)));
        Self {
            scopes: vec![global],
            errors: Vec::new(),
            strict_mode: strict,
        }
    }

    /// Check a program and return any type errors.
    pub fn check_program(&mut self, program: &Program) -> Vec<TypeError> {
        for stmt in &program.statements {
            self.check_stmt(stmt);
        }
        self.errors.clone()
    }

    fn push_scope(&mut self) { self.scopes.push(HashMap::new()); }
    fn pop_scope(&mut self) { if self.scopes.len() > 1 { self.scopes.pop(); } }

    fn define(&mut self, name: &str, ty: Type) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), ty);
        }
    }

    fn lookup(&self, name: &str) -> Type {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return ty.clone();
            }
        }
        Type::Unknown
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::VarDecl { name, type_ann, value, .. } => {
                let inferred = if let Some(expr) = value {
                    self.infer_expr(expr)
                } else {
                    Type::Nil
                };

                if self.strict_mode && type_ann.is_none() {
                    self.error(&format!("strict mode: variable '{}' requires a type annotation", name), &stmt.span);
                }

                let ty = if let Some(ann) = type_ann {
                    let declared = self.resolve_type_ann(ann);
                    if value.is_some() && !self.compatible(&inferred, &declared) {
                        self.error(
                            &format!("type mismatch: expected {:?}, got {:?}", declared, inferred),
                            &stmt.span,
                        );
                    }
                    declared
                } else {
                    inferred
                };
                self.define(name, ty);
            }
            StmtKind::FuncDef(fd) => {
                if self.strict_mode {
                    for p in &fd.params {
                        if p.type_ann.is_none() {
                            self.error(&format!("strict mode: parameter '{}' in '{}' requires a type annotation", p.name, fd.name), &fd.span);
                        }
                    }
                    if fd.return_type.is_none() {
                        self.error(&format!("strict mode: function '{}' requires a return type annotation", fd.name), &fd.span);
                    }
                }
                let param_types: Vec<Type> = fd.params.iter().map(|p| {
                    p.type_ann.as_ref().map(|t| self.resolve_type_ann(t)).unwrap_or(Type::Any)
                }).collect();
                let ret_type = fd.return_type.as_ref()
                    .map(|t| self.resolve_type_ann(t))
                    .unwrap_or(Type::Any);
                self.define(&fd.name, Type::Function(param_types, Box::new(ret_type)));
            }
            StmtKind::ClassDef(cd) => {
                self.define(&cd.name, Type::Class(cd.name.clone()));
            }
            StmtKind::If { condition, then_block, else_if_blocks, else_block } => {
                let cond_ty = self.infer_expr(condition);
                if !self.compatible(&cond_ty, &Type::Bool) && !matches!(cond_ty, Type::Any | Type::Unknown) {
                    // Allow truthy values — no error
                }
                self.push_scope();
                for s in then_block { self.check_stmt(s); }
                self.pop_scope();
                for (c, block) in else_if_blocks {
                    self.infer_expr(c);
                    self.push_scope();
                    for s in block { self.check_stmt(s); }
                    self.pop_scope();
                }
                if let Some(eb) = else_block {
                    self.push_scope();
                    for s in eb { self.check_stmt(s); }
                    self.pop_scope();
                }
            }
            _ => {
                // Other statements — minimal checking for now
            }
        }
    }

    fn infer_expr(&self, expr: &Expr) -> Type {
        match &expr.kind {
            ExprKind::IntLiteral(_) => Type::Int,
            ExprKind::FloatLiteral(_) => Type::Float,
            ExprKind::StringLiteral(_) | ExprKind::InterpolatedString(_) => Type::Str,
            ExprKind::BoolLiteral(_) => Type::Bool,
            ExprKind::CharLiteral(_) => Type::Char,
            ExprKind::NilLiteral => Type::Nil,
            ExprKind::Identifier(name) => self.lookup(name),
            ExprKind::Binary { left, op, right } => {
                let lt = self.infer_expr(left);
                let rt = self.infer_expr(right);
                match op {
                    BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                        if matches!((&lt, &rt), (Type::Float, _) | (_, Type::Float)) {
                            Type::Float
                        } else if matches!((&lt, &rt), (Type::Int, Type::Int)) {
                            Type::Int
                        } else if matches!(op, BinaryOp::Add) && matches!((&lt, &rt), (Type::Str, _) | (_, Type::Str)) {
                            Type::Str
                        } else {
                            Type::Any
                        }
                    }
                    BinaryOp::Eq | BinaryOp::NotEq | BinaryOp::Lt | BinaryOp::Gt
                    | BinaryOp::LtEq | BinaryOp::GtEq | BinaryOp::And | BinaryOp::Or => Type::Bool,
                    _ => Type::Any,
                }
            }
            ExprKind::Unary { op, .. } => {
                match op {
                    UnaryOp::Not => Type::Bool,
                    UnaryOp::Neg => Type::Any, // Could be Int or Float
                    UnaryOp::BitNot => Type::Int,
                }
            }
            ExprKind::Call { .. } => Type::Any,
            ExprKind::ListLiteral(_) => Type::List(Box::new(Type::Any)),
            ExprKind::MapLiteral(_) => Type::Map(Box::new(Type::Str), Box::new(Type::Any)),
            ExprKind::ResultOk(_) => Type::Result(Box::new(Type::Any), Box::new(Type::Any)),
            ExprKind::ResultErr(_) => Type::Result(Box::new(Type::Any), Box::new(Type::Any)),
            _ => Type::Any,
        }
    }

    fn resolve_type_ann(&self, ann: &TypeAnnotation) -> Type {
        match ann {
            TypeAnnotation::Simple(name) => match name.as_str() {
                "Int" => Type::Int,
                "Float" => Type::Float,
                "Num" => Type::Num,
                "Bool" => Type::Bool,
                "Str" => Type::Str,
                "Char" => Type::Char,
                name => Type::Class(name.to_string()),
            },
            TypeAnnotation::Array(inner) => Type::List(Box::new(self.resolve_type_ann(inner))),
            TypeAnnotation::Optional(inner) => Type::Optional(Box::new(self.resolve_type_ann(inner))),
            TypeAnnotation::Generic(name, args) => {
                match name.as_str() {
                    "Result" if args.len() == 2 => {
                        Type::Result(Box::new(self.resolve_type_ann(&args[0])),
                                     Box::new(self.resolve_type_ann(&args[1])))
                    }
                    _ => Type::Class(name.clone()),
                }
            }
            TypeAnnotation::SelfType => Type::Any,
            _ => Type::Any,
        }
    }

    fn compatible(&self, actual: &Type, expected: &Type) -> bool {
        if matches!(actual, Type::Any | Type::Unknown) || matches!(expected, Type::Any | Type::Unknown) {
            return true;
        }
        if actual == expected { return true; }
        // Int is compatible with Float (promotion)
        if matches!((actual, expected), (Type::Int, Type::Float) | (Type::Int, Type::Num) | (Type::Float, Type::Num)) {
            return true;
        }
        // Nil is compatible with Optional
        if matches!(actual, Type::Nil) && matches!(expected, Type::Optional(_)) {
            return true;
        }
        false
    }

    fn error(&mut self, msg: &str, span: &crate::lexer::tokens::Span) {
        self.errors.push(TypeError {
            message: msg.to_string(),
            line: span.line,
            column: span.column,
            file: span.file.clone(),
        });
    }
}

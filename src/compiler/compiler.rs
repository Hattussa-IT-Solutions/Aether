use std::collections::HashMap;

use crate::compiler::bytecode::*;
use crate::parser::ast::*;

/// Compiles AST to bytecode.
pub struct Compiler {
    chunks: Vec<Chunk>,
    current_chunk: usize,
    locals: Vec<HashMap<String, usize>>,
}

impl Compiler {
    pub fn new() -> Self {
        let main_chunk = Chunk::new("main".to_string());
        Self {
            chunks: vec![main_chunk],
            current_chunk: 0,
            locals: vec![HashMap::new()],
        }
    }

    pub fn compile_program(&mut self, program: &Program) -> Vec<Chunk> {
        for stmt in &program.statements {
            self.compile_stmt(stmt);
        }
        self.current().emit(OpCode::Halt);
        self.chunks.clone()
    }

    fn current(&mut self) -> &mut Chunk {
        &mut self.chunks[self.current_chunk]
    }

    fn compile_stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::Expression(expr) => {
                self.compile_expr(expr);
                self.current().emit(OpCode::Pop);
            }
            StmtKind::VarDecl { name, value, .. } => {
                if let Some(expr) = value {
                    self.compile_expr(expr);
                } else {
                    self.current().emit(OpCode::PushNil);
                }
                self.current().emit(OpCode::StoreGlobal(name.clone()));
            }
            StmtKind::If { condition, then_block, else_block, .. } => {
                self.compile_expr(condition);
                let jump_false = self.current().emit(OpCode::JumpIfFalse(0));
                for s in then_block { self.compile_stmt(s); }
                if let Some(else_blk) = else_block {
                    let jump_end = self.current().emit(OpCode::Jump(0));
                    self.current().patch_jump(jump_false);
                    for s in else_blk { self.compile_stmt(s); }
                    self.current().patch_jump(jump_end);
                } else {
                    self.current().patch_jump(jump_false);
                }
            }
            StmtKind::Return(val) => {
                if let Some(expr) = val {
                    self.compile_expr(expr);
                } else {
                    self.current().emit(OpCode::PushNil);
                }
                self.current().emit(OpCode::Return);
            }
            _ => {
                // Other statements — fall through for now
            }
        }
    }

    fn compile_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::IntLiteral(n) => {
                let idx = self.current().add_constant(Constant::Int(*n));
                self.current().emit(OpCode::LoadConst(idx));
            }
            ExprKind::FloatLiteral(n) => {
                let idx = self.current().add_constant(Constant::Float(*n));
                self.current().emit(OpCode::LoadConst(idx));
            }
            ExprKind::StringLiteral(s) => {
                let idx = self.current().add_constant(Constant::Str(s.clone()));
                self.current().emit(OpCode::LoadConst(idx));
            }
            ExprKind::BoolLiteral(true) => { self.current().emit(OpCode::PushTrue); }
            ExprKind::BoolLiteral(false) => { self.current().emit(OpCode::PushFalse); }
            ExprKind::NilLiteral => { self.current().emit(OpCode::PushNil); }
            ExprKind::Identifier(name) => {
                self.current().emit(OpCode::LoadGlobal(name.clone()));
            }
            ExprKind::Binary { left, op, right } => {
                self.compile_expr(left);
                self.compile_expr(right);
                let opcode = match op {
                    BinaryOp::Add => OpCode::Add,
                    BinaryOp::Sub => OpCode::Sub,
                    BinaryOp::Mul => OpCode::Mul,
                    BinaryOp::Div => OpCode::Div,
                    BinaryOp::Mod => OpCode::Mod,
                    BinaryOp::Pow => OpCode::Pow,
                    BinaryOp::Eq => OpCode::Eq,
                    BinaryOp::NotEq => OpCode::Ne,
                    BinaryOp::Lt => OpCode::Lt,
                    BinaryOp::Gt => OpCode::Gt,
                    BinaryOp::LtEq => OpCode::Le,
                    BinaryOp::GtEq => OpCode::Ge,
                    BinaryOp::And => OpCode::And,
                    BinaryOp::Or => OpCode::Or,
                    _ => return,
                };
                self.current().emit(opcode);
            }
            ExprKind::Unary { op, operand } => {
                self.compile_expr(operand);
                match op {
                    UnaryOp::Neg => { self.current().emit(OpCode::Neg); }
                    UnaryOp::Not => { self.current().emit(OpCode::Not); }
                    _ => {}
                }
            }
            ExprKind::Call { callee, args } => {
                // Optimize print() calls directly
                if let ExprKind::Identifier(name) = &callee.kind {
                    if name == "print" {
                        for arg in args {
                            self.compile_expr(&arg.value);
                        }
                        self.current().emit(OpCode::Print(args.len()));
                        return;
                    }
                }
                self.compile_expr(callee);
                for arg in args {
                    self.compile_expr(&arg.value);
                }
                self.current().emit(OpCode::Call(args.len()));
            }
            ExprKind::InterpolatedString(parts) => {
                let mut count = 0;
                for part in parts {
                    match part {
                        StringInterp::Literal(s) => {
                            let idx = self.current().add_constant(Constant::Str(s.clone()));
                            self.current().emit(OpCode::LoadConst(idx));
                            count += 1;
                        }
                        StringInterp::Expr(e) => {
                            self.compile_expr(e);
                            count += 1;
                        }
                    }
                }
                self.current().emit(OpCode::BuildString(count));
            }
            ExprKind::ListLiteral(items) => {
                for item in items {
                    self.compile_expr(item);
                }
                self.current().emit(OpCode::CreateList(items.len()));
            }
            _ => {
                self.current().emit(OpCode::PushNil);
            }
        }
    }
}

use std::collections::HashMap;

use crate::compiler::bytecode::*;
use crate::parser::ast::*;

/// Compiles AST to bytecode.
pub struct Compiler {
    chunks: Vec<Chunk>,
    current_chunk: usize,
    locals: Vec<HashMap<String, usize>>,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
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
            StmtKind::Assignment { target, op, value } => {
                if let ExprKind::Identifier(name) = &target.kind {
                    match op {
                        AssignOp::Assign => {
                            self.compile_expr(value);
                            self.current().emit(OpCode::StoreGlobal(name.clone()));
                        }
                        _ => {
                            // Compound assignment: load, compute, store
                            self.current().emit(OpCode::LoadGlobal(name.clone()));
                            self.compile_expr(value);
                            let opcode = match op {
                                AssignOp::AddAssign => OpCode::Add,
                                AssignOp::SubAssign => OpCode::Sub,
                                AssignOp::MulAssign => OpCode::Mul,
                                AssignOp::DivAssign => OpCode::Div,
                                AssignOp::ModAssign => OpCode::Mod,
                                AssignOp::PowAssign => OpCode::Pow,
                                _ => OpCode::Add,
                            };
                            self.current().emit(opcode);
                            self.current().emit(OpCode::StoreGlobal(name.clone()));
                        }
                    }
                }
            }
            StmtKind::ForLoop { pattern, iterable, body, .. } => {
                // Handle range-based for loop: for i in start..end { body }
                if let ExprKind::Range { start, end, inclusive, .. } = &iterable.kind {
                    if let ForPattern::Single(var_name) = pattern {
                        // Compile start value and store as loop variable
                        self.compile_expr(start);
                        self.current().emit(OpCode::StoreGlobal(var_name.clone()));

                        // Loop header: check condition
                        let loop_start = self.current().code.len();
                        self.current().emit(OpCode::LoadGlobal(var_name.clone()));
                        self.compile_expr(end);
                        let cmp_op = if *inclusive { OpCode::Gt } else { OpCode::Ge };
                        self.current().emit(cmp_op);
                        let exit_jump = self.current().emit(OpCode::JumpIfTrue(0));

                        // Body
                        for s in body { self.compile_stmt(s); }

                        // Increment: var += 1
                        self.current().emit(OpCode::LoadGlobal(var_name.clone()));
                        let one = self.current().add_constant(Constant::Int(1));
                        self.current().emit(OpCode::LoadConst(one));
                        self.current().emit(OpCode::Add);
                        self.current().emit(OpCode::StoreGlobal(var_name.clone()));

                        // Jump back to loop header
                        self.current().emit(OpCode::Loop(loop_start));

                        // Patch exit jump
                        self.current().patch_jump(exit_jump);
                    }
                }
            }
            StmtKind::Loop { kind, body, .. } => {
                match kind {
                    LoopKind::Times(count_expr) => {
                        // Compile count, store as __loop_counter
                        let counter = "__loop_counter".to_string();
                        let idx_var = "__loop_idx".to_string();
                        self.compile_expr(count_expr);
                        self.current().emit(OpCode::StoreGlobal(counter.clone()));
                        let zero = self.current().add_constant(Constant::Int(0));
                        self.current().emit(OpCode::LoadConst(zero));
                        self.current().emit(OpCode::StoreGlobal(idx_var.clone()));

                        let loop_start = self.current().code.len();
                        self.current().emit(OpCode::LoadGlobal(idx_var.clone()));
                        self.current().emit(OpCode::LoadGlobal(counter.clone()));
                        self.current().emit(OpCode::Ge);
                        let exit_jump = self.current().emit(OpCode::JumpIfTrue(0));

                        for s in body { self.compile_stmt(s); }

                        // idx += 1
                        self.current().emit(OpCode::LoadGlobal(idx_var.clone()));
                        let one = self.current().add_constant(Constant::Int(1));
                        self.current().emit(OpCode::LoadConst(one));
                        self.current().emit(OpCode::Add);
                        self.current().emit(OpCode::StoreGlobal(idx_var.clone()));

                        self.current().emit(OpCode::Loop(loop_start));
                        self.current().patch_jump(exit_jump);
                    }
                    LoopKind::While(cond_expr) => {
                        let loop_start = self.current().code.len();
                        self.compile_expr(cond_expr);
                        let exit_jump = self.current().emit(OpCode::JumpIfFalse(0));
                        for s in body { self.compile_stmt(s); }
                        self.current().emit(OpCode::Loop(loop_start));
                        self.current().patch_jump(exit_jump);
                    }
                    LoopKind::Infinite => {
                        let loop_start = self.current().code.len();
                        for s in body { self.compile_stmt(s); }
                        self.current().emit(OpCode::Loop(loop_start));
                    }
                }
            }
            StmtKind::FuncDef(fd) => {
                // For now, store function as a no-op global marker
                // Full function compilation would need call frames
                self.current().emit(OpCode::PushNil);
                self.current().emit(OpCode::StoreGlobal(fd.name.clone()));
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

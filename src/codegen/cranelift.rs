use cranelift_codegen::ir::types::*;
use cranelift_codegen::ir::{AbiParam, InstBuilder, Signature};
use cranelift_codegen::isa::CallConv;
use cranelift_codegen::settings::{self, Configurable};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_module::{default_libcall_names, Module};
use cranelift_jit::{JITBuilder, JITModule};

use crate::parser::ast::*;

/// JIT compile and execute a simple Aether program.
/// Currently supports: integer arithmetic, variable assignment, return value.
pub fn jit_compile_and_run(program: &Program) -> Result<i64, String> {
    let mut flag_builder = settings::builder();
    flag_builder.set("use_colocated_libcalls", "false").unwrap();
    flag_builder.set("is_pic", "false").unwrap();
    let isa_builder = cranelift_native::builder().map_err(|e| e.to_string())?;
    let isa = isa_builder.finish(settings::Flags::new(flag_builder)).map_err(|e| e.to_string())?;

    let builder = JITBuilder::with_isa(isa, default_libcall_names());
    let mut module = JITModule::new(builder);

    // Create the main function signature: () -> i64
    let mut sig = Signature::new(CallConv::SystemV);
    sig.returns.push(AbiParam::new(I64));

    let func_id = module.declare_function("main", cranelift_module::Linkage::Local, &sig)
        .map_err(|e| e.to_string())?;

    let mut ctx = module.make_context();
    ctx.func.signature = sig.clone();

    let mut fn_builder_ctx = FunctionBuilderContext::new();
    {
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut fn_builder_ctx);
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        let mut var_map = std::collections::HashMap::new();
        let mut var_idx = 0u32;
        let mut last_val = builder.ins().iconst(I64, 0);

        // Compile each statement
        for stmt in &program.statements {
            match &stmt.kind {
                StmtKind::VarDecl { name, value: Some(expr), .. } => {
                    let val = compile_cranelift_expr(expr, &mut builder, &var_map);
                    let var = Variable::from_u32(var_idx);
                    var_idx += 1;
                    builder.declare_var(var, I64);
                    builder.def_var(var, val);
                    var_map.insert(name.clone(), var);
                    last_val = val;
                }
                StmtKind::Expression(expr) => {
                    last_val = compile_cranelift_expr(expr, &mut builder, &var_map);
                }
                StmtKind::Return(Some(expr)) => {
                    let val = compile_cranelift_expr(expr, &mut builder, &var_map);
                    builder.ins().return_(&[val]);
                    // Don't continue — we returned
                    module.define_function(func_id, &mut ctx).map_err(|e| e.to_string())?;
                    module.finalize_definitions().map_err(|e| e.to_string())?;
                    let code = module.get_finalized_function(func_id);
                    let func_ptr: fn() -> i64 = unsafe { std::mem::transmute(code) };
                    return Ok(func_ptr());
                }
                _ => {}
            }
        }

        builder.ins().return_(&[last_val]);
        builder.finalize();
    }

    module.define_function(func_id, &mut ctx).map_err(|e| e.to_string())?;
    module.finalize_definitions().map_err(|e| e.to_string())?;

    let code = module.get_finalized_function(func_id);
    let func_ptr: fn() -> i64 = unsafe { std::mem::transmute(code) };
    Ok(func_ptr())
}

fn compile_cranelift_expr(
    expr: &Expr,
    builder: &mut FunctionBuilder,
    vars: &std::collections::HashMap<String, Variable>,
) -> cranelift_codegen::ir::Value {
    match &expr.kind {
        ExprKind::IntLiteral(n) => builder.ins().iconst(I64, *n),
        ExprKind::Identifier(name) => {
            if let Some(var) = vars.get(name) {
                builder.use_var(*var)
            } else {
                builder.ins().iconst(I64, 0)
            }
        }
        ExprKind::Binary { left, op, right } => {
            let l = compile_cranelift_expr(left, builder, vars);
            let r = compile_cranelift_expr(right, builder, vars);
            match op {
                BinaryOp::Add => builder.ins().iadd(l, r),
                BinaryOp::Sub => builder.ins().isub(l, r),
                BinaryOp::Mul => builder.ins().imul(l, r),
                BinaryOp::Div => builder.ins().sdiv(l, r),
                BinaryOp::Mod => builder.ins().srem(l, r),
                _ => builder.ins().iconst(I64, 0),
            }
        }
        ExprKind::Unary { op: UnaryOp::Neg, operand } => {
            let val = compile_cranelift_expr(operand, builder, vars);
            builder.ins().ineg(val)
        }
        _ => builder.ins().iconst(I64, 0),
    }
}

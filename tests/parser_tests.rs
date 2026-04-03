use aether::lexer::scanner::Scanner;
use aether::parser::ast::*;
use aether::parser::parser::Parser;

fn parse(source: &str) -> Program {
    let mut scanner = Scanner::new(source, "test.ae".to_string());
    let tokens = scanner.scan_tokens();
    let mut parser = Parser::new(tokens);
    parser.parse_program().expect("parse failed")
}

fn parse_stmt(source: &str) -> Stmt {
    let program = parse(source);
    assert!(!program.statements.is_empty(), "no statements parsed from: {}", source);
    program.statements.into_iter().next().unwrap()
}

fn parse_expr(source: &str) -> Expr {
    let stmt = parse_stmt(source);
    match stmt.kind {
        StmtKind::Expression(expr) => expr,
        StmtKind::VarDecl { value: Some(val), .. } => val,
        other => panic!("expected expression statement, got {:?}", other),
    }
}

// ═══════════════════════════════════════════════════════════════
// Literals
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_int_literal() {
    let expr = parse_expr("42");
    assert!(matches!(expr.kind, ExprKind::IntLiteral(42)));
}

#[test]
fn test_float_literal() {
    let expr = parse_expr("3.14");
    assert!(matches!(expr.kind, ExprKind::FloatLiteral(f) if (f - 3.14).abs() < 1e-10));
}

#[test]
fn test_string_literal() {
    let expr = parse_expr(r#""hello""#);
    assert!(matches!(expr.kind, ExprKind::StringLiteral(ref s) if s == "hello"));
}

#[test]
fn test_bool_literal() {
    let expr = parse_expr("true");
    assert!(matches!(expr.kind, ExprKind::BoolLiteral(true)));
}

#[test]
fn test_nil_literal() {
    let expr = parse_expr("nil");
    assert!(matches!(expr.kind, ExprKind::NilLiteral));
}

#[test]
fn test_char_literal() {
    let expr = parse_expr("'A'");
    assert!(matches!(expr.kind, ExprKind::CharLiteral('A')));
}

// ═══════════════════════════════════════════════════════════════
// Binary expressions
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_addition() {
    let expr = parse_expr("1 + 2");
    if let ExprKind::Binary { op, .. } = expr.kind {
        assert_eq!(op, BinaryOp::Add);
    } else { panic!("expected binary"); }
}

#[test]
fn test_multiplication_precedence() {
    // 1 + 2 * 3 should parse as 1 + (2 * 3)
    let expr = parse_expr("1 + 2 * 3");
    if let ExprKind::Binary { op, right, .. } = expr.kind {
        assert_eq!(op, BinaryOp::Add);
        if let ExprKind::Binary { op: inner_op, .. } = right.kind {
            assert_eq!(inner_op, BinaryOp::Mul);
        } else { panic!("expected inner binary"); }
    } else { panic!("expected binary"); }
}

#[test]
fn test_power_right_associative() {
    // 2 ** 3 ** 4 should parse as 2 ** (3 ** 4)
    let expr = parse_expr("2 ** 3 ** 4");
    if let ExprKind::Binary { op, left, right } = expr.kind {
        assert_eq!(op, BinaryOp::Pow);
        assert!(matches!(left.kind, ExprKind::IntLiteral(2)));
        if let ExprKind::Binary { op: inner_op, .. } = right.kind {
            assert_eq!(inner_op, BinaryOp::Pow);
        } else { panic!("expected inner pow"); }
    } else { panic!("expected binary"); }
}

#[test]
fn test_comparison() {
    let expr = parse_expr("a > b");
    if let ExprKind::Binary { op, .. } = expr.kind {
        assert_eq!(op, BinaryOp::Gt);
    } else { panic!("expected binary"); }
}

#[test]
fn test_logical_and_or() {
    let expr = parse_expr("a and b or c");
    // Should parse as (a and b) or c since and binds tighter than or
    if let ExprKind::Binary { op, .. } = expr.kind {
        assert_eq!(op, BinaryOp::Or);
    } else { panic!("expected binary"); }
}

#[test]
fn test_bitwise_ops() {
    let expr = parse_expr("a & b | c");
    // | binds less tightly than &, so: (a & b) | c
    if let ExprKind::Binary { op, .. } = expr.kind {
        assert_eq!(op, BinaryOp::BitOr);
    } else { panic!("expected binary"); }
}

// ═══════════════════════════════════════════════════════════════
// Unary expressions
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_unary_neg() {
    let expr = parse_expr("-42");
    if let ExprKind::Unary { op, .. } = expr.kind {
        assert_eq!(op, UnaryOp::Neg);
    } else { panic!("expected unary"); }
}

#[test]
fn test_unary_not() {
    let expr = parse_expr("!flag");
    if let ExprKind::Unary { op, .. } = expr.kind {
        assert_eq!(op, UnaryOp::Not);
    } else { panic!("expected unary"); }
}

#[test]
fn test_unary_not_keyword() {
    let expr = parse_expr("not flag");
    if let ExprKind::Unary { op, .. } = expr.kind {
        assert_eq!(op, UnaryOp::Not);
    } else { panic!("expected unary"); }
}

// ═══════════════════════════════════════════════════════════════
// Function calls
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_function_call() {
    let expr = parse_expr("foo(1, 2)");
    if let ExprKind::Call { args, .. } = &expr.kind {
        assert_eq!(args.len(), 2);
    } else { panic!("expected call"); }
}

#[test]
fn test_named_arguments() {
    let expr = parse_expr("connect(host: \"localhost\", port: 8080)");
    if let ExprKind::Call { args, .. } = &expr.kind {
        assert_eq!(args.len(), 2);
        assert_eq!(args[0].name, Some("host".to_string()));
        assert_eq!(args[1].name, Some("port".to_string()));
    } else { panic!("expected call"); }
}

#[test]
fn test_method_call() {
    let expr = parse_expr("list.push(42)");
    assert!(matches!(expr.kind, ExprKind::MethodCall { .. }));
}

// ═══════════════════════════════════════════════════════════════
// Special operators
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_pipeline() {
    let expr = parse_expr("data |> filter |> map");
    if let ExprKind::Pipeline { left, .. } = &expr.kind {
        assert!(matches!(left.kind, ExprKind::Pipeline { .. }));
    } else { panic!("expected pipeline"); }
}

#[test]
fn test_optional_chaining() {
    let expr = parse_expr("user?.name");
    assert!(matches!(expr.kind, ExprKind::OptionalChain { .. }));
}

#[test]
fn test_nil_coalescing() {
    let expr = parse_expr("x ?? 0");
    assert!(matches!(expr.kind, ExprKind::NilCoalesce { .. }));
}

#[test]
fn test_error_propagation() {
    let expr = parse_expr("read_file()?");
    assert!(matches!(expr.kind, ExprKind::ErrorPropagate(_)));
}

#[test]
fn test_range() {
    let expr = parse_expr("0..10");
    if let ExprKind::Range { inclusive, .. } = expr.kind {
        assert!(!inclusive);
    } else { panic!("expected range"); }
}

#[test]
fn test_inclusive_range() {
    let expr = parse_expr("1..=10");
    if let ExprKind::Range { inclusive, .. } = expr.kind {
        assert!(inclusive);
    } else { panic!("expected range"); }
}

#[test]
fn test_index_access() {
    let expr = parse_expr("list[0]");
    assert!(matches!(expr.kind, ExprKind::Index { .. }));
}

#[test]
fn test_field_access() {
    let expr = parse_expr("obj.field");
    assert!(matches!(expr.kind, ExprKind::FieldAccess { .. }));
}

// ═══════════════════════════════════════════════════════════════
// Lambdas
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_single_param_lambda() {
    let expr = parse_expr("x -> x * 2");
    if let ExprKind::Lambda { params, .. } = &expr.kind {
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].name, "x");
    } else { panic!("expected lambda"); }
}

#[test]
fn test_multi_param_lambda() {
    let expr = parse_expr("(a, b) -> a + b");
    if let ExprKind::Lambda { params, .. } = &expr.kind {
        assert_eq!(params.len(), 2);
    } else { panic!("expected lambda"); }
}

// ═══════════════════════════════════════════════════════════════
// Collection literals
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_list_literal() {
    let expr = parse_expr("[1, 2, 3]");
    if let ExprKind::ListLiteral(items) = &expr.kind {
        assert_eq!(items.len(), 3);
    } else { panic!("expected list"); }
}

#[test]
fn test_empty_list() {
    let expr = parse_expr("[]");
    if let ExprKind::ListLiteral(items) = &expr.kind {
        assert_eq!(items.len(), 0);
    } else { panic!("expected list"); }
}

#[test]
fn test_map_literal() {
    let expr = parse_expr(r#"{"a": 1, "b": 2}"#);
    if let ExprKind::MapLiteral(pairs) = &expr.kind {
        assert_eq!(pairs.len(), 2);
    } else { panic!("expected map, got {:?}", expr.kind); }
}

#[test]
fn test_set_literal() {
    let expr = parse_expr("{1, 2, 3}");
    if let ExprKind::SetLiteral(items) = &expr.kind {
        assert_eq!(items.len(), 3);
    } else { panic!("expected set, got {:?}", expr.kind); }
}

#[test]
fn test_tuple() {
    let expr = parse_expr("(1, 2, 3)");
    if let ExprKind::TupleLiteral(items) = &expr.kind {
        assert_eq!(items.len(), 3);
    } else { panic!("expected tuple"); }
}

// ═══════════════════════════════════════════════════════════════
// Comprehensions
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_list_comprehension() {
    let expr = parse_expr("[x * 2 for x in items]");
    assert!(matches!(expr.kind, ExprKind::Comprehension { .. }));
}

#[test]
fn test_filtered_comprehension() {
    let expr = parse_expr("[x for x in data if x > 0]");
    if let ExprKind::Comprehension { condition, .. } = &expr.kind {
        assert!(condition.is_some());
    } else { panic!("expected comprehension"); }
}

// ═══════════════════════════════════════════════════════════════
// If expression
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_if_expression() {
    // if-expression appears inside an assignment context
    let stmt = parse_stmt("val = if x > 0 then \"pos\" else \"neg\"");
    if let StmtKind::VarDecl { value: Some(val), .. } = &stmt.kind {
        assert!(matches!(val.kind, ExprKind::IfExpr { .. }));
    } else { panic!("expected var decl with if expr"); }
}

// ═══════════════════════════════════════════════════════════════
// Variable declarations
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_var_decl() {
    let stmt = parse_stmt("x = 5");
    if let StmtKind::VarDecl { name, mutable, .. } = &stmt.kind {
        assert_eq!(name, "x");
        assert!(mutable);
    } else { panic!("expected var decl"); }
}

#[test]
fn test_let_decl() {
    let stmt = parse_stmt("let x = 5");
    if let StmtKind::VarDecl { name, mutable, .. } = &stmt.kind {
        assert_eq!(name, "x");
        assert!(!mutable);
    } else { panic!("expected var decl"); }
}

#[test]
fn test_const_decl() {
    let stmt = parse_stmt("const MAX = 100");
    if let StmtKind::VarDecl { name, is_const, .. } = &stmt.kind {
        assert_eq!(name, "MAX");
        assert!(is_const);
    } else { panic!("expected var decl"); }
}

#[test]
fn test_typed_var_decl() {
    let stmt = parse_stmt("x: Int = 5");
    if let StmtKind::VarDecl { name, type_ann, .. } = &stmt.kind {
        assert_eq!(name, "x");
        assert!(type_ann.is_some());
    } else { panic!("expected var decl"); }
}

// ═══════════════════════════════════════════════════════════════
// Function definitions
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_basic_func() {
    let stmt = parse_stmt("def add(a: Int, b: Int) -> Int { return a + b }");
    if let StmtKind::FuncDef(fd) = &stmt.kind {
        assert_eq!(fd.name, "add");
        assert_eq!(fd.params.len(), 2);
        assert!(fd.return_type.is_some());
    } else { panic!("expected func def"); }
}

#[test]
fn test_expression_body_func() {
    let stmt = parse_stmt("def double(x: Num) -> Num = x * 2");
    if let StmtKind::FuncDef(fd) = &stmt.kind {
        assert_eq!(fd.name, "double");
        assert!(matches!(fd.body, FuncBody::Expression(_)));
    } else { panic!("expected func def"); }
}

#[test]
fn test_async_func() {
    let stmt = parse_stmt("async def fetch(url: Str) { }");
    if let StmtKind::FuncDef(fd) = &stmt.kind {
        assert!(fd.is_async);
    } else { panic!("expected func def"); }
}

#[test]
fn test_func_default_params() {
    let stmt = parse_stmt("def connect(host: Str, port: Int = 8080) { }");
    if let StmtKind::FuncDef(fd) = &stmt.kind {
        assert!(fd.params[1].default.is_some());
    } else { panic!("expected func def"); }
}

// ═══════════════════════════════════════════════════════════════
// Control flow
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_if_else() {
    let stmt = parse_stmt("if x > 0 {\n  y = 1\n} else {\n  y = 2\n}");
    assert!(matches!(stmt.kind, StmtKind::If { .. }));
}

#[test]
fn test_if_else_if() {
    let stmt = parse_stmt("if x > 0 {\n  a()\n} else if x < 0 {\n  b()\n} else {\n  c()\n}");
    if let StmtKind::If { else_if_blocks, else_block, .. } = &stmt.kind {
        assert_eq!(else_if_blocks.len(), 1);
        assert!(else_block.is_some());
    } else { panic!("expected if"); }
}

#[test]
fn test_match_stmt() {
    let stmt = parse_stmt("match x {\n  1 -> a()\n  2 -> b()\n  _ -> c()\n}");
    if let StmtKind::Match { arms, .. } = &stmt.kind {
        assert_eq!(arms.len(), 3);
    } else { panic!("expected match"); }
}

#[test]
fn test_match_with_guard() {
    let stmt = parse_stmt("match x {\n  n if n > 10 -> big()\n  _ -> small()\n}");
    if let StmtKind::Match { arms, .. } = &stmt.kind {
        assert!(arms[0].guard.is_some());
    } else { panic!("expected match"); }
}

#[test]
fn test_match_destructure() {
    let stmt = parse_stmt("match result {\n  Ok(val) -> val\n  Err(msg) -> msg\n}");
    if let StmtKind::Match { arms, .. } = &stmt.kind {
        assert_eq!(arms.len(), 2);
        assert!(matches!(&arms[0].pattern, Pattern::Destructure { name, .. } if name == "Ok"));
    } else { panic!("expected match"); }
}

#[test]
fn test_match_enum_variant() {
    let stmt = parse_stmt("match shape {\n  .Circle(r) -> r\n  .Rect(w, h) -> w\n}");
    if let StmtKind::Match { arms, .. } = &stmt.kind {
        assert!(matches!(&arms[0].pattern, Pattern::EnumVariant { variant, .. } if variant == "Circle"));
    } else { panic!("expected match"); }
}

// ═══════════════════════════════════════════════════════════════
// Loops
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_for_loop() {
    let stmt = parse_stmt("for item in list { }");
    if let StmtKind::ForLoop { pattern, .. } = &stmt.kind {
        assert!(matches!(pattern, ForPattern::Single(ref s) if s == "item"));
    } else { panic!("expected for loop"); }
}

#[test]
fn test_for_enumerate() {
    let stmt = parse_stmt("for i, item in list { }");
    if let StmtKind::ForLoop { pattern, .. } = &stmt.kind {
        assert!(matches!(pattern, ForPattern::Enumerate(..)));
    } else { panic!("expected for loop"); }
}

#[test]
fn test_for_range() {
    let stmt = parse_stmt("for i in 0..10 { }");
    assert!(matches!(stmt.kind, StmtKind::ForLoop { .. }));
}

#[test]
fn test_for_destructure() {
    let stmt = parse_stmt("for (name, age) in tuples { }");
    if let StmtKind::ForLoop { pattern, .. } = &stmt.kind {
        assert!(matches!(pattern, ForPattern::Destructure(..)));
    } else { panic!("expected for loop"); }
}

#[test]
fn test_loop_times() {
    let stmt = parse_stmt("loop 5 times { }");
    if let StmtKind::Loop { kind, .. } = &stmt.kind {
        assert!(matches!(kind, LoopKind::Times(_)));
    } else { panic!("expected loop"); }
}

#[test]
fn test_loop_while() {
    let stmt = parse_stmt("loop while x > 0 { }");
    if let StmtKind::Loop { kind, .. } = &stmt.kind {
        assert!(matches!(kind, LoopKind::While(_)));
    } else { panic!("expected loop"); }
}

#[test]
fn test_loop_infinite() {
    let stmt = parse_stmt("loop { break }");
    if let StmtKind::Loop { kind, .. } = &stmt.kind {
        assert!(matches!(kind, LoopKind::Infinite));
    } else { panic!("expected loop"); }
}

#[test]
fn test_break_label() {
    let stmt = parse_stmt("break:outer");
    if let StmtKind::Break { label } = &stmt.kind {
        assert_eq!(label.as_deref(), Some("outer"));
    } else { panic!("expected break"); }
}

#[test]
fn test_next_if() {
    let stmt = parse_stmt("next if x == 0");
    if let StmtKind::Next { condition, .. } = &stmt.kind {
        assert!(condition.is_some());
    } else { panic!("expected next"); }
}

// ═══════════════════════════════════════════════════════════════
// Error handling
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_try_catch() {
    let stmt = parse_stmt("try {\n  risky()\n} catch IOError as e {\n  handle(e)\n} finally {\n  cleanup()\n}");
    if let StmtKind::TryCatch { catches, finally_block, .. } = &stmt.kind {
        assert_eq!(catches.len(), 1);
        assert!(finally_block.is_some());
    } else { panic!("expected try/catch"); }
}

#[test]
fn test_return_value() {
    let stmt = parse_stmt("return 42");
    assert!(matches!(stmt.kind, StmtKind::Return(Some(_))));
}

#[test]
fn test_return_void() {
    let stmt = parse_stmt("return");
    assert!(matches!(stmt.kind, StmtKind::Return(None)));
}

#[test]
fn test_throw() {
    let stmt = parse_stmt("throw error");
    assert!(matches!(stmt.kind, StmtKind::Throw(_)));
}

// ═══════════════════════════════════════════════════════════════
// OOP: Class, Struct, Enum, Interface
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_class_basic() {
    let stmt = parse_stmt("class Dog {\n  name: Str\n  age: Int = 0\n}");
    if let StmtKind::ClassDef(cd) = &stmt.kind {
        assert_eq!(cd.name, "Dog");
        assert_eq!(cd.fields.len(), 2);
    } else { panic!("expected class"); }
}

#[test]
fn test_class_with_init() {
    let stmt = parse_stmt("class Dog {\n  name: Str\n  init(name: Str) {\n    self.name = name\n  }\n}");
    if let StmtKind::ClassDef(cd) = &stmt.kind {
        assert!(cd.init.is_some());
    } else { panic!("expected class"); }
}

#[test]
fn test_class_with_methods() {
    let stmt = parse_stmt("class Dog {\n  def bark() { }\n  def sit() { }\n}");
    if let StmtKind::ClassDef(cd) = &stmt.kind {
        assert_eq!(cd.methods.len(), 2);
    } else { panic!("expected class"); }
}

#[test]
fn test_class_inheritance() {
    let stmt = parse_stmt("class Admin : User {\n  role: Str\n}");
    if let StmtKind::ClassDef(cd) = &stmt.kind {
        assert_eq!(cd.parent.as_deref(), Some("User"));
    } else { panic!("expected class"); }
}

#[test]
fn test_class_impl_interface() {
    let stmt = parse_stmt("class User impl Serializable {\n  def to_json() -> Str { }\n}");
    if let StmtKind::ClassDef(cd) = &stmt.kind {
        assert_eq!(cd.interfaces, vec!["Serializable"]);
    } else { panic!("expected class"); }
}

#[test]
fn test_class_with_weave() {
    let stmt = parse_stmt("class Service with Logged, Cached {\n  def process() { }\n}");
    if let StmtKind::ClassDef(cd) = &stmt.kind {
        assert_eq!(cd.weaves, vec!["Logged", "Cached"]);
    } else { panic!("expected class"); }
}

#[test]
fn test_struct_def() {
    let stmt = parse_stmt("struct Point {\n  x: Float\n  y: Float\n}");
    if let StmtKind::StructDef(sd) = &stmt.kind {
        assert_eq!(sd.name, "Point");
        assert_eq!(sd.fields.len(), 2);
    } else { panic!("expected struct"); }
}

#[test]
fn test_enum_def() {
    let stmt = parse_stmt("enum Shape {\n  Circle(radius: Float)\n  Rect(width: Float, height: Float)\n}");
    if let StmtKind::EnumDef(ed) = &stmt.kind {
        assert_eq!(ed.name, "Shape");
        assert_eq!(ed.variants.len(), 2);
        assert_eq!(ed.variants[0].fields.len(), 1);
    } else { panic!("expected enum"); }
}

#[test]
fn test_interface_def() {
    let stmt = parse_stmt("interface Drawable {\n  def draw()\n  def resize(factor: Float)\n}");
    if let StmtKind::InterfaceDef(id) = &stmt.kind {
        assert_eq!(id.name, "Drawable");
        assert_eq!(id.methods.len(), 2);
    } else { panic!("expected interface"); }
}

// ═══════════════════════════════════════════════════════════════
// Parallel blocks
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_parallel_block() {
    let stmt = parse_stmt("parallel {\n  a = fetch()\n  b = load()\n}");
    if let StmtKind::Parallel { tasks, is_race, .. } = &stmt.kind {
        assert_eq!(tasks.len(), 2);
        assert!(!is_race);
    } else { panic!("expected parallel"); }
}

#[test]
fn test_parallel_race() {
    let stmt = parse_stmt("parallel.race {\n  fast = cache()\n  slow = db()\n}");
    if let StmtKind::Parallel { is_race, .. } = &stmt.kind {
        assert!(is_race);
    } else { panic!("expected parallel"); }
}

// ═══════════════════════════════════════════════════════════════
// Modules
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_use_simple() {
    let stmt = parse_stmt("use fs");
    if let StmtKind::Use { path, alias } = &stmt.kind {
        assert_eq!(path, &["fs"]);
        assert!(alias.is_none());
    } else { panic!("expected use"); }
}

#[test]
fn test_use_nested_with_alias() {
    let stmt = parse_stmt("use net.http as http");
    if let StmtKind::Use { path, alias } = &stmt.kind {
        assert_eq!(path, &["net", "http"]);
        assert_eq!(alias.as_deref(), Some("http"));
    } else { panic!("expected use"); }
}

#[test]
fn test_type_alias() {
    let stmt = parse_stmt("type UserID = Int");
    if let StmtKind::TypeAlias { name, .. } = &stmt.kind {
        assert_eq!(name, "UserID");
    } else { panic!("expected type alias"); }
}

// ═══════════════════════════════════════════════════════════════
// Type annotations
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_array_type() {
    let stmt = parse_stmt("x: Str[] = []");
    if let StmtKind::VarDecl { type_ann: Some(ty), .. } = &stmt.kind {
        assert!(matches!(ty, TypeAnnotation::Array(_)));
    } else { panic!("expected typed decl"); }
}

#[test]
fn test_optional_type() {
    let stmt = parse_stmt("x: Int? = nil");
    if let StmtKind::VarDecl { type_ann: Some(ty), .. } = &stmt.kind {
        assert!(matches!(ty, TypeAnnotation::Optional(_)));
    } else { panic!("expected typed decl"); }
}

#[test]
fn test_generic_type() {
    let stmt = parse_stmt("def f(x: Result<Int, Str>) { }");
    if let StmtKind::FuncDef(fd) = &stmt.kind {
        assert!(matches!(&fd.params[0].type_ann, Some(TypeAnnotation::Generic(name, params)) if name == "Result" && params.len() == 2));
    } else { panic!("expected func def"); }
}

// ═══════════════════════════════════════════════════════════════
// Directives
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_directive() {
    let program = parse("#strict\nx = 5");
    assert_eq!(program.directives.len(), 1);
    assert_eq!(program.directives[0].name, "strict");
}

// ═══════════════════════════════════════════════════════════════
// Decorators
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_decorator_on_func() {
    let stmt = parse_stmt("@gpu\ndef compute() { }");
    if let StmtKind::FuncDef(fd) = &stmt.kind {
        assert_eq!(fd.decorators.len(), 1);
        assert_eq!(fd.decorators[0].name, "gpu");
    } else { panic!("expected func def"); }
}

// ═══════════════════════════════════════════════════════════════
// Assignment operators
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_compound_assignment() {
    let stmt = parse_stmt("x += 1");
    if let StmtKind::Assignment { op, .. } = &stmt.kind {
        assert_eq!(*op, AssignOp::AddAssign);
    } else { panic!("expected assignment, got {:?}", stmt.kind); }
}

// ═══════════════════════════════════════════════════════════════
// Weave
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_weave_def() {
    let stmt = parse_stmt("weave Logged {\n  before {\n    log(\"start\")\n  }\n  after {\n    log(\"end\")\n  }\n}");
    if let StmtKind::WeaveDef(wd) = &stmt.kind {
        assert_eq!(wd.name, "Logged");
        assert!(wd.before.is_some());
        assert!(wd.after.is_some());
    } else { panic!("expected weave def"); }
}

// ═══════════════════════════════════════════════════════════════
// Extend
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_extend_block() {
    let stmt = parse_stmt("extend Int {\n  def is_even() -> Bool = self % 2 == 0\n}");
    if let StmtKind::ExtendBlock(eb) = &stmt.kind {
        assert_eq!(eb.methods.len(), 1);
    } else { panic!("expected extend block"); }
}

// ═══════════════════════════════════════════════════════════════
// Genetic classes
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_genetic_class() {
    let src = r#"genetic class Strategy {
    chromosome params {
        gene threshold: Float = 0.5 { range 0.0..1.0 }
        gene window: Int = 20 { range 5..200 }
    }
    fitness(data: Float) -> Float {
        return 1.0
    }
}"#;
    let stmt = parse_stmt(src);
    if let StmtKind::ClassDef(cd) = &stmt.kind {
        assert!(cd.is_genetic);
        assert_eq!(cd.chromosomes.len(), 1);
        assert_eq!(cd.chromosomes[0].genes.len(), 2);
        assert!(cd.fitness_fn.is_some());
    } else { panic!("expected class def"); }
}

// ═══════════════════════════════════════════════════════════════
// Ok/Err constructors
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_ok_constructor() {
    let expr = parse_expr("Ok(42)");
    assert!(matches!(expr.kind, ExprKind::ResultOk(_)));
}

#[test]
fn test_err_constructor() {
    let expr = parse_expr("Err(\"fail\")");
    assert!(matches!(expr.kind, ExprKind::ResultErr(_)));
}

// ═══════════════════════════════════════════════════════════════
// Enum variant expressions
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_enum_variant_expr() {
    let expr = parse_expr(".Circle(5.0)");
    if let ExprKind::EnumVariant { name, args } = &expr.kind {
        assert_eq!(name, "Circle");
        assert_eq!(args.len(), 1);
    } else { panic!("expected enum variant"); }
}

// ═══════════════════════════════════════════════════════════════
// Mutation atomic
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_mutation_atomic() {
    let stmt = parse_stmt("mutation.atomic {\n  x = 1\n}");
    assert!(matches!(stmt.kind, StmtKind::MutationAtomic { .. }));
}

// ═══════════════════════════════════════════════════════════════
// Device blocks
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_device_gpu() {
    let stmt = parse_stmt("device(.gpu) {\n  compute()\n}");
    if let StmtKind::Device { target, .. } = &stmt.kind {
        assert_eq!(*target, DeviceTarget::Gpu);
    } else { panic!("expected device"); }
}

// ═══════════════════════════════════════════════════════════════
// Interpolated strings
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_interpolated_string_parsed() {
    let expr = parse_expr(r#""Hello {name}""#);
    if let ExprKind::InterpolatedString(parts) = &expr.kind {
        assert_eq!(parts.len(), 2);
        assert!(matches!(&parts[0], StringInterp::Literal(s) if s == "Hello "));
        assert!(matches!(&parts[1], StringInterp::Expr(_)));
    } else { panic!("expected interpolated string"); }
}

// ═══════════════════════════════════════════════════════════════
// Complex programs
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_multiline_program() {
    let src = r#"
x = 10
y = 20
z = x + y
"#;
    let program = parse(src);
    assert_eq!(program.statements.len(), 3);
}

#[test]
fn test_class_with_operator_overload() {
    let src = r#"class Vec2 {
    x: Float
    y: Float
    operator +(other: Vec2) -> Vec2 {
        return Vec2(self.x + other.x, self.y + other.y)
    }
}"#;
    let stmt = parse_stmt(src);
    if let StmtKind::ClassDef(cd) = &stmt.kind {
        assert_eq!(cd.operators.len(), 1);
    } else { panic!("expected class"); }
}

#[test]
fn test_guard_statement() {
    let stmt = parse_stmt("guard let value = opt else { return }");
    assert!(matches!(stmt.kind, StmtKind::Guard { .. }));
}

#[test]
fn test_select_inheritance() {
    let stmt = parse_stmt("class Penguin : Animal select(swim, run, breathe) { }");
    if let StmtKind::ClassDef(cd) = &stmt.kind {
        assert_eq!(cd.parent.as_deref(), Some("Animal"));
        assert!(cd.select.is_some());
        assert_eq!(cd.select.as_ref().unwrap().len(), 3);
    } else { panic!("expected class"); }
}

#[test]
fn test_exclude_inheritance() {
    let stmt = parse_stmt("class Penguin : Animal exclude(fly, climb) { }");
    if let StmtKind::ClassDef(cd) = &stmt.kind {
        assert!(cd.exclude.is_some());
    } else { panic!("expected class"); }
}

#[test]
fn test_self_expr() {
    let expr = parse_expr("self");
    assert!(matches!(expr.kind, ExprKind::SelfExpr));
}

#[test]
fn test_await_expr() {
    let expr = parse_expr("await fetch()");
    assert!(matches!(expr.kind, ExprKind::Await(_)));
}

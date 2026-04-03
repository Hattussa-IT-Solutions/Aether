use crate::lexer::tokens::Span;

/// A complete Aether program (a sequence of statements/declarations).
#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Stmt>,
    pub directives: Vec<Directive>,
}

/// Top-level directive like #strict or #test.
#[derive(Debug, Clone)]
pub struct Directive {
    pub name: String,
    pub span: Span,
}

// ═══════════════════════════════════════════════════════════════
// Statements
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum StmtKind {
    /// Expression as a statement.
    Expression(Expr),

    /// Variable declaration: `x = 5`, `let x = 5`, `const X = 5`, `x: Int = 5`
    VarDecl {
        name: String,
        type_ann: Option<TypeAnnotation>,
        value: Option<Expr>,
        mutable: bool,
        is_const: bool,
    },

    /// Assignment: `x = 5`, `x += 1`, `obj.field = val`, `list[0] = val`
    Assignment {
        target: Expr,
        op: AssignOp,
        value: Expr,
    },

    /// Function definition.
    FuncDef(FuncDef),

    /// Class definition.
    ClassDef(ClassDef),

    /// Struct definition.
    StructDef(StructDef),

    /// Enum definition.
    EnumDef(EnumDef),

    /// Interface definition.
    InterfaceDef(InterfaceDef),

    /// if / else if / else.
    If {
        condition: Expr,
        then_block: Vec<Stmt>,
        else_if_blocks: Vec<(Expr, Vec<Stmt>)>,
        else_block: Option<Vec<Stmt>>,
    },

    /// guard let value = expr else { ... }
    Guard {
        pattern: Pattern,
        value: Expr,
        else_block: Vec<Stmt>,
    },

    /// if let pattern = expr { ... } else { ... }
    IfLet {
        pattern: Pattern,
        value: Expr,
        then_block: Vec<Stmt>,
        else_block: Option<Vec<Stmt>>,
    },

    /// Match statement.
    Match {
        value: Expr,
        arms: Vec<MatchArm>,
    },

    /// For loop.
    ForLoop {
        label: Option<String>,
        pattern: ForPattern,
        iterable: Expr,
        step: Option<Expr>,
        parallel: Option<Option<Expr>>, // None = not parallel, Some(None) = |parallel|, Some(Some(n)) = |parallel: n|
        body: Vec<Stmt>,
    },

    /// Loop (times, while, infinite, until).
    Loop {
        label: Option<String>,
        kind: LoopKind,
        body: Vec<Stmt>,
        until_condition: Option<Expr>,
    },

    /// break or break:label
    Break { label: Option<String> },

    /// next or next if condition
    Next {
        label: Option<String>,
        condition: Option<Expr>,
    },

    /// return (with optional value)
    Return(Option<Expr>),

    /// throw expression
    Throw(Expr),

    /// try / catch / finally
    TryCatch {
        try_block: Vec<Stmt>,
        catches: Vec<CatchClause>,
        finally_block: Option<Vec<Stmt>>,
    },

    /// parallel { } block
    Parallel {
        tasks: Vec<Stmt>,
        timeout: Option<Expr>,
        max_concurrency: Option<Expr>,
        is_race: bool,
    },

    /// after(deps) { expr } inside parallel block
    After {
        dependencies: Vec<String>,
        body: Vec<Stmt>,
    },

    /// mutation.atomic { }
    MutationAtomic {
        body: Vec<Stmt>,
    },

    /// device(.gpu) { }, device(.cpu) { }, device(.quantum) { }
    Device {
        target: DeviceTarget,
        body: Vec<Stmt>,
    },

    /// use statement
    Use {
        path: Vec<String>,
        alias: Option<String>,
    },

    /// mod block
    ModBlock {
        name: String,
        body: Vec<Stmt>,
    },

    /// type alias: type Name = Type
    TypeAlias {
        name: String,
        value: TypeAnnotation,
    },

    /// Weave definition.
    WeaveDef(WeaveDef),

    /// Extend block.
    ExtendBlock(ExtendBlock),

    /// Block of statements (for grouping).
    Block(Vec<Stmt>),
}

// ═══════════════════════════════════════════════════════════════
// Expressions
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    // Literals
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    InterpolatedString(Vec<StringInterp>),
    BoolLiteral(bool),
    CharLiteral(char),
    NilLiteral,

    // Identifiers
    Identifier(String),

    /// self
    SelfExpr,

    /// super
    SuperExpr,

    /// Binary operation: a + b, a && b, a |> b, etc.
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },

    /// Unary operation: -x, !x, ~x, not x
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },

    /// Function call: func(a, b, key: value)
    Call {
        callee: Box<Expr>,
        args: Vec<Argument>,
    },

    /// Method call: obj.method(args)
    MethodCall {
        object: Box<Expr>,
        method: String,
        args: Vec<Argument>,
    },

    /// Field access: obj.field
    FieldAccess {
        object: Box<Expr>,
        field: String,
    },

    /// Index access: list[0], map["key"]
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },

    /// Optional chaining: obj?.field
    OptionalChain {
        object: Box<Expr>,
        field: String,
    },

    /// Nil coalescing: x ?? default
    NilCoalesce {
        value: Box<Expr>,
        default: Box<Expr>,
    },

    /// Error propagation: expr?
    ErrorPropagate(Box<Expr>),

    /// Pipeline: a |> b |> c
    Pipeline {
        left: Box<Expr>,
        right: Box<Expr>,
    },

    /// Lambda: x -> x * 2, (a, b) -> a + b
    Lambda {
        params: Vec<Param>,
        body: Box<Expr>,
    },

    /// Block lambda: { stmts }
    BlockLambda {
        params: Vec<Param>,
        body: Vec<Stmt>,
    },

    /// If expression: if cond then a else b
    IfExpr {
        condition: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
    },

    /// Match expression (returns value).
    MatchExpr {
        value: Box<Expr>,
        arms: Vec<MatchArm>,
    },

    /// List literal: [1, 2, 3]
    ListLiteral(Vec<Expr>),

    /// Map literal: {"key": value}
    MapLiteral(Vec<(Expr, Expr)>),

    /// Set literal: {1, 2, 3}
    SetLiteral(Vec<Expr>),

    /// Tuple: (a, b, c)
    TupleLiteral(Vec<Expr>),

    /// Range: 0..10, 1..=10
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
        inclusive: bool,
        step: Option<Box<Expr>>,
    },

    /// Comprehension: [expr for x in iter if cond]
    Comprehension {
        expr: Box<Expr>,
        var: String,
        iterable: Box<Expr>,
        condition: Option<Box<Expr>>,
        kind: ComprehensionKind,
    },

    /// Map comprehension: {k: v for (k, v) in pairs}
    MapComprehension {
        key_expr: Box<Expr>,
        value_expr: Box<Expr>,
        key_var: String,
        value_var: String,
        iterable: Box<Expr>,
        condition: Option<Box<Expr>>,
    },

    /// Computed property access: field: Type => expr
    ComputedProperty {
        expr: Box<Expr>,
    },

    /// Await expression: await expr
    Await(Box<Expr>),

    /// Ok(value), Err(value) constructors
    ResultOk(Box<Expr>),
    ResultErr(Box<Expr>),

    /// Enum variant: .Circle(args)
    EnumVariant {
        name: String,
        args: Vec<Expr>,
    },

    /// Qualified name: module.name
    QualifiedName(Vec<String>),

    /// As cast: expr as Type
    AsCast {
        value: Box<Expr>,
        target_type: TypeAnnotation,
    },

    /// Evolve block: evolve Type { ... }
    EvolveBlock {
        target: String,
        config: EvolveConfig,
    },

    /// Crossover: crossover(a, b)
    Crossover {
        parent_a: Box<Expr>,
        parent_b: Box<Expr>,
    },

    /// Breed: breed(a, b, mutation_rate: f)
    Breed {
        parent_a: Box<Expr>,
        parent_b: Box<Expr>,
        mutation_rate: Option<Box<Expr>>,
    },
}

// ═══════════════════════════════════════════════════════════════
// Supporting types
// ═══════════════════════════════════════════════════════════════

/// String interpolation part.
#[derive(Debug, Clone)]
pub enum StringInterp {
    Literal(String),
    Expr(Expr),
}

/// Assignment operator.
#[derive(Debug, Clone, PartialEq)]
pub enum AssignOp {
    Assign,     // =
    AddAssign,  // +=
    SubAssign,  // -=
    MulAssign,  // *=
    DivAssign,  // /=
    ModAssign,  // %=
    PowAssign,  // **=
    BitAndAssign, // &=
    BitOrAssign,  // |=
    BitXorAssign, // ^=
    ShlAssign,    // <<=
    ShrAssign,    // >>=
}

/// Binary operators.
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Mod, Pow,
    Eq, NotEq, Lt, Gt, LtEq, GtEq,
    And, Or,
    BitAnd, BitOr, BitXor, Shl, Shr,
    Pipeline,
    Range, RangeInclusive,
}

/// Unary operators.
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,    // -
    Not,    // ! or not
    BitNot, // ~
}

/// Function/method parameter.
#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub type_ann: Option<TypeAnnotation>,
    pub default: Option<Expr>,
    pub variadic: bool,      // *args
    pub kw_variadic: bool,   // **kwargs
}

/// Function call argument.
#[derive(Debug, Clone)]
pub struct Argument {
    pub name: Option<String>, // None for positional, Some for named
    pub value: Expr,
}

/// Function definition.
#[derive(Debug, Clone)]
pub struct FuncDef {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<TypeAnnotation>,
    pub body: FuncBody,
    pub is_async: bool,
    pub is_static: bool,
    pub decorators: Vec<Decorator>,
    pub access: AccessModifier,
    pub span: Span,
}

/// Function body — either a block or single expression.
#[derive(Debug, Clone)]
pub enum FuncBody {
    Block(Vec<Stmt>),
    Expression(Expr),
}

/// Access modifiers.
#[derive(Debug, Clone, PartialEq)]
pub enum AccessModifier {
    Pub,
    Priv,
    Prot,
}

/// Decorator like @gpu, @test, @cached(ttl: 60).
#[derive(Debug, Clone)]
pub struct Decorator {
    pub name: String,
    pub args: Vec<Argument>,
    pub span: Span,
}

// ═══════════════════════════════════════════════════════════════
// OOP types
// ═══════════════════════════════════════════════════════════════

/// Class definition.
#[derive(Debug, Clone)]
pub struct ClassDef {
    pub name: String,
    pub parent: Option<String>,
    pub interfaces: Vec<String>,
    pub weaves: Vec<String>,
    pub select: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub capabilities: Option<Vec<String>>,
    pub fields: Vec<FieldDef>,
    pub init: Option<FuncDef>,
    pub deinit: Option<Vec<Stmt>>,
    pub methods: Vec<FuncDef>,
    pub computed_props: Vec<ComputedProp>,
    pub observed_props: Vec<ObservedProp>,
    pub lazy_props: Vec<LazyProp>,
    pub static_fields: Vec<FieldDef>,
    pub static_methods: Vec<FuncDef>,
    pub operators: Vec<OperatorDef>,
    pub bonds: Vec<BondDef>,
    pub faces: Vec<FaceDef>,
    pub delegates: Vec<DelegateDef>,
    pub morph_methods: Vec<MorphDef>,
    pub reactive_props: Vec<ReactiveProp>,
    pub temporal_props: Vec<TemporalProp>,
    pub mutation_props: Vec<MutationProp>,
    pub evolving_props: Vec<EvolvingProp>,
    pub chromosomes: Vec<ChromosomeDef>,
    pub fitness_fn: Option<FuncDef>,
    pub is_genetic: bool,
    pub decorators: Vec<Decorator>,
    pub access: AccessModifier,
    pub span: Span,
}

/// Struct definition.
#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<FieldDef>,
    pub methods: Vec<FuncDef>,
    pub computed_props: Vec<ComputedProp>,
    pub operators: Vec<OperatorDef>,
    pub decorators: Vec<Decorator>,
    pub span: Span,
}

/// Enum definition.
#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: String,
    pub variants: Vec<EnumVariantDef>,
    pub methods: Vec<FuncDef>,
    pub decorators: Vec<Decorator>,
    pub span: Span,
}

/// Interface definition.
#[derive(Debug, Clone)]
pub struct InterfaceDef {
    pub name: String,
    pub methods: Vec<InterfaceMethod>,
    pub span: Span,
}

/// A field in a class/struct.
#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: String,
    pub type_ann: Option<TypeAnnotation>,
    pub default: Option<Expr>,
    pub access: AccessModifier,
    pub is_readonly: bool,
    pub span: Span,
}

/// Computed property: `area: Float => width * height`
#[derive(Debug, Clone)]
pub struct ComputedProp {
    pub name: String,
    pub type_ann: Option<TypeAnnotation>,
    pub body: Expr,
    pub span: Span,
}

/// Observed property with did_change handler.
#[derive(Debug, Clone)]
pub struct ObservedProp {
    pub name: String,
    pub type_ann: Option<TypeAnnotation>,
    pub default: Option<Expr>,
    pub did_change: Vec<Stmt>,
    pub span: Span,
}

/// Lazy property: `lazy data: Data => expensive_load()`
#[derive(Debug, Clone)]
pub struct LazyProp {
    pub name: String,
    pub type_ann: Option<TypeAnnotation>,
    pub initializer: Expr,
    pub span: Span,
}

/// Operator overloading definition.
#[derive(Debug, Clone)]
pub struct OperatorDef {
    pub op: String,
    pub params: Vec<Param>,
    pub return_type: Option<TypeAnnotation>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

/// Enum variant definition.
#[derive(Debug, Clone)]
pub struct EnumVariantDef {
    pub name: String,
    pub fields: Vec<FieldDef>,
    pub span: Span,
}

/// Interface method signature (may have default implementation).
#[derive(Debug, Clone)]
pub struct InterfaceMethod {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<TypeAnnotation>,
    pub default_body: Option<Vec<Stmt>>,
    pub span: Span,
}

// ═══════════════════════════════════════════════════════════════
// New OOP concepts
// ═══════════════════════════════════════════════════════════════

/// Bond: bidirectional relationship.
#[derive(Debug, Clone)]
pub struct BondDef {
    pub name: String,
    pub target_type: TypeAnnotation,
    pub via: String,
    pub span: Span,
}

/// Face: object projection.
#[derive(Debug, Clone)]
pub struct FaceDef {
    pub name: String,
    pub visible_fields: Vec<String>,
    pub span: Span,
}

/// Delegate: auto-forward methods.
#[derive(Debug, Clone)]
pub struct DelegateDef {
    pub field: String,
    pub target_type: TypeAnnotation,
    pub span: Span,
}

/// Morph: compile-time conditional methods.
#[derive(Debug, Clone)]
pub struct MorphDef {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<TypeAnnotation>,
    pub when_clauses: Vec<MorphWhen>,
    pub span: Span,
}

/// A when clause in a morph method.
#[derive(Debug, Clone)]
pub struct MorphWhen {
    pub condition: Expr,
    pub body: Vec<Stmt>,
}

/// Reactive property.
#[derive(Debug, Clone)]
pub struct ReactiveProp {
    pub name: String,
    pub type_ann: Option<TypeAnnotation>,
    pub compute_expr: Expr,
    pub span: Span,
}

/// Temporal property with history ring buffer.
#[derive(Debug, Clone)]
pub struct TemporalProp {
    pub name: String,
    pub type_ann: Option<TypeAnnotation>,
    pub keep: usize,
    pub default: Option<Expr>,
    pub span: Span,
}

/// Mutation-controlled property.
#[derive(Debug, Clone)]
pub struct MutationProp {
    pub name: String,
    pub type_ann: Option<TypeAnnotation>,
    pub default: Option<Expr>,
    pub tracked: bool,
    pub undoable: Option<usize>,
    pub constrain: Option<(Expr, Expr)>,
    pub validate: Option<Expr>,
    pub transform: Option<Expr>,
    pub redact: bool,
    pub increase_rules: Vec<String>,
    pub decrease_rules: Vec<String>,
    pub span: Span,
}

/// Evolving property.
#[derive(Debug, Clone)]
pub struct EvolvingProp {
    pub name: String,
    pub type_ann: Option<TypeAnnotation>,
    pub default: Expr,
    pub eval_interval: Option<usize>,
    pub eval_body: Option<Vec<Stmt>>,
    pub span: Span,
}

/// Weave definition.
#[derive(Debug, Clone)]
pub struct WeaveDef {
    pub name: String,
    pub params: Vec<Param>,
    pub before: Option<Vec<Stmt>>,
    pub after: Option<Vec<Stmt>>,
    pub around: Option<Vec<Stmt>>,
    pub span: Span,
}

/// Extend block: add methods to existing types.
#[derive(Debug, Clone)]
pub struct ExtendBlock {
    pub target: TypeAnnotation,
    pub where_clause: Option<Vec<TypeConstraint>>,
    pub methods: Vec<FuncDef>,
    pub span: Span,
}

// ═══════════════════════════════════════════════════════════════
// Genetic classes
// ═══════════════════════════════════════════════════════════════

/// Chromosome definition inside a genetic class.
#[derive(Debug, Clone)]
pub struct ChromosomeDef {
    pub name: String,
    pub genes: Vec<GeneDef>,
    pub span: Span,
}

/// Gene definition inside a chromosome.
#[derive(Debug, Clone)]
pub struct GeneDef {
    pub name: String,
    pub type_ann: Option<TypeAnnotation>,
    pub default: Option<Expr>,
    pub range: Option<(Expr, Expr)>,
    pub options: Option<Vec<Expr>>,
    pub step: Option<Expr>,
    pub scale: Option<String>,
    pub when_condition: Option<Expr>,
    pub span: Span,
}

/// Configuration for an evolve block.
#[derive(Debug, Clone)]
pub struct EvolveConfig {
    pub population: Option<Box<Expr>>,
    pub generations: Option<Box<Expr>>,
    pub mutation_rate: Option<Box<Expr>>,
    pub crossover_rate: Option<Box<Expr>>,
    pub selection: Option<SelectionMethod>,
    pub elitism: Option<Box<Expr>>,
    pub fitness_data: Option<(String, Box<Expr>)>,
}

/// Selection method for genetic evolution.
#[derive(Debug, Clone)]
pub enum SelectionMethod {
    Tournament(Option<Box<Expr>>),
    Roulette,
    Rank,
}

// ═══════════════════════════════════════════════════════════════
// Pattern matching
// ═══════════════════════════════════════════════════════════════

/// A match arm.
#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: MatchBody,
    pub span: Span,
}

/// Match arm body — single expression or block.
#[derive(Debug, Clone)]
pub enum MatchBody {
    Expression(Expr),
    Block(Vec<Stmt>),
}

/// Pattern for matching.
#[derive(Debug, Clone)]
pub enum Pattern {
    /// Literal value: 42, "ok", true
    Literal(Expr),
    /// Range: 1..10, 90..100
    Range {
        start: Expr,
        end: Expr,
        inclusive: bool,
    },
    /// Wildcard: _
    Wildcard,
    /// Binding: captures value into variable
    Binding(String),
    /// Destructure: Ok(val), Err(msg), .Circle(r)
    Destructure {
        name: String,
        fields: Vec<Pattern>,
    },
    /// Enum variant pattern: .Circle(r)
    EnumVariant {
        variant: String,
        fields: Vec<Pattern>,
    },
    /// Tuple pattern: (a, b, c)
    Tuple(Vec<Pattern>),
}

// ═══════════════════════════════════════════════════════════════
// Loop helpers
// ═══════════════════════════════════════════════════════════════

/// For-loop variable pattern.
#[derive(Debug, Clone)]
pub enum ForPattern {
    /// Single variable: for x in ...
    Single(String),
    /// Enumerate: for i, item in ...
    Enumerate(String, String),
    /// Destructure: for (a, b) in ...
    Destructure(Vec<String>),
}

/// Loop kind.
#[derive(Debug, Clone)]
pub enum LoopKind {
    /// loop N times { }
    Times(Expr),
    /// loop while condition { }
    While(Expr),
    /// loop { } (infinite)
    Infinite,
}

/// Catch clause in try/catch.
#[derive(Debug, Clone)]
pub struct CatchClause {
    pub error_type: Option<String>,
    pub binding: Option<String>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

/// Device target.
#[derive(Debug, Clone, PartialEq)]
pub enum DeviceTarget {
    Gpu,
    Cpu,
    Quantum,
}

/// Comprehension kind.
#[derive(Debug, Clone, PartialEq)]
pub enum ComprehensionKind {
    List,
    Set,
}

// ═══════════════════════════════════════════════════════════════
// Type annotations
// ═══════════════════════════════════════════════════════════════

/// Type annotation.
#[derive(Debug, Clone)]
pub enum TypeAnnotation {
    /// Simple type: Int, Str, Float, Bool, etc.
    Simple(String),
    /// Generic: List<T>, Map<K,V>, Result<T,E>
    Generic(String, Vec<TypeAnnotation>),
    /// Array shorthand: Str[], Int[]
    Array(Box<TypeAnnotation>),
    /// Optional: T?
    Optional(Box<TypeAnnotation>),
    /// Map shorthand: {Str: Int}
    MapType(Box<TypeAnnotation>, Box<TypeAnnotation>),
    /// Function type: (Int, Str) -> Bool
    FuncType(Vec<TypeAnnotation>, Box<TypeAnnotation>),
    /// Tuple type: (Int, Str)
    TupleType(Vec<TypeAnnotation>),
    /// Self type
    SelfType,
    /// Dimensional type: Float.m, Float.s
    Dimensional(Box<TypeAnnotation>, String),
}

/// Type constraint for where clauses.
#[derive(Debug, Clone)]
pub struct TypeConstraint {
    pub type_param: String,
    pub bound: String,
}

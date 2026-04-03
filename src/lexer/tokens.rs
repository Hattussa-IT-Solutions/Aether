/// Source location for error reporting.
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(file: String, line: usize, column: usize) -> Self {
        Self { file, line, column }
    }
}

/// A token with its type, lexeme, and source location.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: String, span: Span) -> Self {
        Self { kind, lexeme, span }
    }
}

/// All token types for the Aether language.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    /// String with interpolation parts: alternating literal and expression segments.
    /// Even indices are string parts, odd indices are expression parts.
    InterpolatedString(Vec<StringPart>),
    MultilineString(String),
    RawString(String),
    CharLiteral(char),
    BoolLiteral(bool),
    NilLiteral,

    // Identifier
    Identifier(String),

    // Keywords
    Def,
    Let,
    Const,
    If,
    Else,
    Match,
    Guard,
    For,
    In,
    Loop,
    Times,
    While,
    Until,
    Step,
    Break,
    Next,
    Return,
    Class,
    Struct,
    Enum,
    Interface,
    Impl,
    SelfKw,
    Super,
    Init,
    Deinit,
    Pub,
    Priv,
    Prot,
    Readonly,
    Static,
    Lazy,
    Async,
    Await,
    Try,
    Catch,
    Finally,
    Throw,
    Use,
    Mod,
    As,
    Type,
    Parallel,
    After,
    Device,
    Model,
    Agent,
    Pipeline,
    Reactive,
    Temporal,
    Mutation,
    Evolving,
    Genetic,
    Gene,
    Chromosome,
    Fitness,
    Crossover,
    Breed,
    Evolve,
    Weave,
    Bond,
    Face,
    Extend,
    Delegate,
    Select,
    Exclude,
    Morph,
    True,
    False,
    Nil,
    And,
    Or,
    Not,
    Then,
    Operator,
    Override,
    Where,
    With,

    // Arithmetic operators
    Plus,          // +
    Minus,         // -
    Star,          // *
    Slash,         // /
    Percent,       // %
    StarStar,      // **

    // Assignment operators
    Eq,            // =
    PlusEq,        // +=
    MinusEq,       // -=
    StarEq,        // *=
    SlashEq,       // /=
    PercentEq,     // %=
    StarStarEq,    // **=
    AmpEq,         // &=
    PipeEq,        // |=
    CaretEq,       // ^=
    LtLtEq,        // <<=
    GtGtEq,        // >>=

    // Comparison operators
    EqEq,          // ==
    BangEq,        // !=
    Lt,            // <
    Gt,            // >
    LtEq,          // <=
    GtEq,          // >=

    // Logical operators
    AmpAmp,        // &&
    PipePipe,      // ||
    Bang,          // !

    // Bitwise operators
    Amp,           // &
    Pipe,          // |
    Caret,         // ^
    Tilde,         // ~
    LtLt,          // <<
    GtGt,          // >>

    // Special operators
    Arrow,         // ->
    QuestionDot,   // ?.
    QuestionQuestion, // ??
    Question,      // ?
    PipeGt,        // |>
    DotDot,        // ..
    DotDotEq,      // ..=
    FatArrow,      // =>

    // Punctuation
    Dot,           // .
    Comma,         // ,
    Colon,         // :
    At,            // @
    Hash,          // #

    // Brackets
    LParen,        // (
    RParen,        // )
    LBracket,      // [
    RBracket,      // ]
    LBrace,        // {
    RBrace,        // }

    // Special
    Newline,
    Decorator(String),  // @name
    Directive(String),  // #strict, #test

    // End of file
    Eof,
}

/// A part of an interpolated string.
#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    /// Literal text segment.
    Literal(String),
    /// Expression segment (the text between { }).
    Expression(String),
}

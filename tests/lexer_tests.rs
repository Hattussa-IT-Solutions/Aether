use aether::lexer::scanner::Scanner;
use aether::lexer::tokens::*;

fn scan(source: &str) -> Vec<Token> {
    let mut scanner = Scanner::new(source, "test.ae".to_string());
    scanner.scan_tokens()
}

fn token_kinds(source: &str) -> Vec<TokenKind> {
    scan(source)
        .into_iter()
        .filter(|t| !matches!(t.kind, TokenKind::Newline | TokenKind::Eof))
        .map(|t| t.kind)
        .collect()
}

// ═══════════════════════════════════════════════════════════════
// Keywords
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_all_keywords() {
    let keywords = vec![
        ("def", TokenKind::Def),
        ("let", TokenKind::Let),
        ("const", TokenKind::Const),
        ("if", TokenKind::If),
        ("else", TokenKind::Else),
        ("match", TokenKind::Match),
        ("guard", TokenKind::Guard),
        ("for", TokenKind::For),
        ("in", TokenKind::In),
        ("loop", TokenKind::Loop),
        ("times", TokenKind::Times),
        ("while", TokenKind::While),
        ("until", TokenKind::Until),
        ("step", TokenKind::Step),
        ("break", TokenKind::Break),
        ("next", TokenKind::Next),
        ("return", TokenKind::Return),
        ("class", TokenKind::Class),
        ("struct", TokenKind::Struct),
        ("enum", TokenKind::Enum),
        ("interface", TokenKind::Interface),
        ("impl", TokenKind::Impl),
        ("self", TokenKind::SelfKw),
        ("super", TokenKind::Super),
        ("init", TokenKind::Init),
        ("deinit", TokenKind::Deinit),
        ("pub", TokenKind::Pub),
        ("priv", TokenKind::Priv),
        ("prot", TokenKind::Prot),
        ("readonly", TokenKind::Readonly),
        ("static", TokenKind::Static),
        ("lazy", TokenKind::Lazy),
        ("async", TokenKind::Async),
        ("await", TokenKind::Await),
        ("try", TokenKind::Try),
        ("catch", TokenKind::Catch),
        ("finally", TokenKind::Finally),
        ("throw", TokenKind::Throw),
        ("use", TokenKind::Use),
        ("mod", TokenKind::Mod),
        ("as", TokenKind::As),
        ("type", TokenKind::Type),
        ("parallel", TokenKind::Parallel),
        ("after", TokenKind::After),
        ("device", TokenKind::Device),
        ("model", TokenKind::Model),
        ("agent", TokenKind::Agent),
        ("pipeline", TokenKind::Pipeline),
        ("reactive", TokenKind::Reactive),
        ("temporal", TokenKind::Temporal),
        ("mutation", TokenKind::Mutation),
        ("evolving", TokenKind::Evolving),
        ("genetic", TokenKind::Genetic),
        ("gene", TokenKind::Gene),
        ("chromosome", TokenKind::Chromosome),
        ("fitness", TokenKind::Fitness),
        ("crossover", TokenKind::Crossover),
        ("breed", TokenKind::Breed),
        ("evolve", TokenKind::Evolve),
        ("weave", TokenKind::Weave),
        ("bond", TokenKind::Bond),
        ("face", TokenKind::Face),
        ("extend", TokenKind::Extend),
        ("delegate", TokenKind::Delegate),
        ("select", TokenKind::Select),
        ("exclude", TokenKind::Exclude),
        ("morph", TokenKind::Morph),
        ("true", TokenKind::True),
        ("false", TokenKind::False),
        ("nil", TokenKind::Nil),
        ("and", TokenKind::And),
        ("or", TokenKind::Or),
        ("not", TokenKind::Not),
        ("then", TokenKind::Then),
        ("operator", TokenKind::Operator),
        ("override", TokenKind::Override),
        ("where", TokenKind::Where),
        ("with", TokenKind::With),
    ];

    for (text, expected) in keywords {
        let kinds = token_kinds(text);
        assert_eq!(kinds, vec![expected], "Failed for keyword: {}", text);
    }
}

#[test]
fn test_identifier_not_keyword() {
    let kinds = token_kinds("myVar");
    assert_eq!(kinds, vec![TokenKind::Identifier("myVar".to_string())]);
}

#[test]
fn test_identifier_with_underscore() {
    let kinds = token_kinds("_private");
    assert_eq!(kinds, vec![TokenKind::Identifier("_private".to_string())]);
}

#[test]
fn test_identifier_with_numbers() {
    let kinds = token_kinds("var123");
    assert_eq!(kinds, vec![TokenKind::Identifier("var123".to_string())]);
}

// ═══════════════════════════════════════════════════════════════
// Operators
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_arithmetic_operators() {
    let kinds = token_kinds("+ - * / % **");
    assert_eq!(kinds, vec![
        TokenKind::Plus, TokenKind::Minus, TokenKind::Star,
        TokenKind::Slash, TokenKind::Percent, TokenKind::StarStar,
    ]);
}

#[test]
fn test_comparison_operators() {
    let kinds = token_kinds("== != < > <= >=");
    assert_eq!(kinds, vec![
        TokenKind::EqEq, TokenKind::BangEq, TokenKind::Lt,
        TokenKind::Gt, TokenKind::LtEq, TokenKind::GtEq,
    ]);
}

#[test]
fn test_logical_operators() {
    let kinds = token_kinds("&& || !");
    assert_eq!(kinds, vec![TokenKind::AmpAmp, TokenKind::PipePipe, TokenKind::Bang]);
}

#[test]
fn test_bitwise_operators() {
    let kinds = token_kinds("& | ^ ~ << >>");
    assert_eq!(kinds, vec![
        TokenKind::Amp, TokenKind::Pipe, TokenKind::Caret,
        TokenKind::Tilde, TokenKind::LtLt, TokenKind::GtGt,
    ]);
}

#[test]
fn test_assignment_operators() {
    let kinds = token_kinds("= += -= *= /= %= **= &= |= ^= <<= >>=");
    assert_eq!(kinds, vec![
        TokenKind::Eq, TokenKind::PlusEq, TokenKind::MinusEq,
        TokenKind::StarEq, TokenKind::SlashEq, TokenKind::PercentEq,
        TokenKind::StarStarEq, TokenKind::AmpEq, TokenKind::PipeEq,
        TokenKind::CaretEq, TokenKind::LtLtEq, TokenKind::GtGtEq,
    ]);
}

#[test]
fn test_special_operators() {
    let kinds = token_kinds("-> ?. ?? ? |> .. ..= =>");
    assert_eq!(kinds, vec![
        TokenKind::Arrow, TokenKind::QuestionDot, TokenKind::QuestionQuestion,
        TokenKind::Question, TokenKind::PipeGt, TokenKind::DotDot,
        TokenKind::DotDotEq, TokenKind::FatArrow,
    ]);
}

#[test]
fn test_brackets() {
    let kinds = token_kinds("( ) [ ] { }");
    assert_eq!(kinds, vec![
        TokenKind::LParen, TokenKind::RParen, TokenKind::LBracket,
        TokenKind::RBracket, TokenKind::LBrace, TokenKind::RBrace,
    ]);
}

#[test]
fn test_punctuation() {
    let kinds = token_kinds(". , :");
    assert_eq!(kinds, vec![TokenKind::Dot, TokenKind::Comma, TokenKind::Colon]);
}

#[test]
fn test_chained_operators() {
    // Ensure operators don't greedily consume each other
    let kinds = token_kinds("a+b");
    assert_eq!(kinds, vec![
        TokenKind::Identifier("a".to_string()),
        TokenKind::Plus,
        TokenKind::Identifier("b".to_string()),
    ]);
}

#[test]
fn test_double_star_vs_star_star_eq() {
    let kinds = token_kinds("** **=");
    assert_eq!(kinds, vec![TokenKind::StarStar, TokenKind::StarStarEq]);
}

// ═══════════════════════════════════════════════════════════════
// Numbers
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_integer_literal() {
    let kinds = token_kinds("42");
    assert_eq!(kinds, vec![TokenKind::IntLiteral(42)]);
}

#[test]
fn test_integer_with_underscores() {
    let kinds = token_kinds("1_000_000");
    assert_eq!(kinds, vec![TokenKind::IntLiteral(1_000_000)]);
}

#[test]
fn test_float_literal() {
    let kinds = token_kinds("3.14");
    assert_eq!(kinds, vec![TokenKind::FloatLiteral(3.14)]);
}

#[test]
fn test_float_with_exponent() {
    let kinds = token_kinds("1.0e10");
    assert_eq!(kinds, vec![TokenKind::FloatLiteral(1.0e10)]);
}

#[test]
fn test_float_with_negative_exponent() {
    let kinds = token_kinds("2.5e-3");
    assert_eq!(kinds, vec![TokenKind::FloatLiteral(2.5e-3)]);
}

#[test]
fn test_hex_literal() {
    let kinds = token_kinds("0xFF");
    assert_eq!(kinds, vec![TokenKind::IntLiteral(255)]);
}

#[test]
fn test_hex_literal_uppercase() {
    let kinds = token_kinds("0XAB");
    assert_eq!(kinds, vec![TokenKind::IntLiteral(0xAB)]);
}

#[test]
fn test_binary_literal() {
    let kinds = token_kinds("0b1010");
    assert_eq!(kinds, vec![TokenKind::IntLiteral(10)]);
}

#[test]
fn test_binary_literal_uppercase() {
    let kinds = token_kinds("0B1100");
    assert_eq!(kinds, vec![TokenKind::IntLiteral(12)]);
}

#[test]
fn test_zero() {
    let kinds = token_kinds("0");
    assert_eq!(kinds, vec![TokenKind::IntLiteral(0)]);
}

#[test]
fn test_large_number() {
    let kinds = token_kinds("9999999999999");
    assert_eq!(kinds, vec![TokenKind::IntLiteral(9999999999999)]);
}

#[test]
fn test_integer_before_dotdot() {
    // 0..10 should be int(0), .., int(10)
    let kinds = token_kinds("0..10");
    assert_eq!(kinds, vec![
        TokenKind::IntLiteral(0),
        TokenKind::DotDot,
        TokenKind::IntLiteral(10),
    ]);
}

// ═══════════════════════════════════════════════════════════════
// Strings
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_simple_string() {
    let kinds = token_kinds(r#""hello""#);
    assert_eq!(kinds, vec![TokenKind::StringLiteral("hello".to_string())]);
}

#[test]
fn test_string_with_escapes() {
    let kinds = token_kinds(r#""hello\nworld""#);
    assert_eq!(kinds, vec![TokenKind::StringLiteral("hello\nworld".to_string())]);
}

#[test]
fn test_string_interpolation() {
    let kinds = token_kinds(r#""Hello {name}""#);
    assert_eq!(kinds.len(), 1);
    match &kinds[0] {
        TokenKind::InterpolatedString(parts) => {
            assert_eq!(parts.len(), 2);
            assert_eq!(parts[0], StringPart::Literal("Hello ".to_string()));
            assert_eq!(parts[1], StringPart::Expression("name".to_string()));
        }
        _ => panic!("Expected InterpolatedString"),
    }
}

#[test]
fn test_string_multiple_interpolations() {
    let kinds = token_kinds(r#""{a} and {b}""#);
    assert_eq!(kinds.len(), 1);
    match &kinds[0] {
        TokenKind::InterpolatedString(parts) => {
            assert_eq!(parts.len(), 3);
            assert_eq!(parts[0], StringPart::Expression("a".to_string()));
            assert_eq!(parts[1], StringPart::Literal(" and ".to_string()));
            assert_eq!(parts[2], StringPart::Expression("b".to_string()));
        }
        _ => panic!("Expected InterpolatedString"),
    }
}

#[test]
fn test_string_escaped_braces() {
    let kinds = token_kinds(r#""literal {{brace}}""#);
    assert_eq!(kinds, vec![TokenKind::StringLiteral("literal {brace}".to_string())]);
}

#[test]
fn test_empty_string() {
    let kinds = token_kinds(r#""""#);
    assert_eq!(kinds, vec![TokenKind::StringLiteral(String::new())]);
}

#[test]
fn test_string_with_emoji() {
    let kinds = token_kinds(r#""hello 🌍""#);
    assert_eq!(kinds, vec![TokenKind::StringLiteral("hello 🌍".to_string())]);
}

#[test]
fn test_multiline_string() {
    let source = "\"\"\"
    hello
    world
\"\"\"";
    let kinds = token_kinds(source);
    assert_eq!(kinds.len(), 1);
    match &kinds[0] {
        TokenKind::MultilineString(s) => {
            assert_eq!(s, "hello\nworld");
        }
        _ => panic!("Expected MultilineString, got {:?}", kinds[0]),
    }
}

#[test]
fn test_raw_string() {
    let kinds = token_kinds(r#"r"raw\nstring""#);
    assert_eq!(kinds, vec![TokenKind::RawString(r"raw\nstring".to_string())]);
}

// ═══════════════════════════════════════════════════════════════
// Char literals
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_char_literal() {
    let kinds = token_kinds("'A'");
    assert_eq!(kinds, vec![TokenKind::CharLiteral('A')]);
}

#[test]
fn test_char_literal_escape() {
    let kinds = token_kinds(r"'\n'");
    assert_eq!(kinds, vec![TokenKind::CharLiteral('\n')]);
}

// ═══════════════════════════════════════════════════════════════
// Comments
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_line_comment() {
    let kinds = token_kinds("x // this is a comment\ny");
    assert_eq!(kinds, vec![
        TokenKind::Identifier("x".to_string()),
        TokenKind::Identifier("y".to_string()),
    ]);
}

#[test]
fn test_block_comment() {
    let kinds = token_kinds("x /* comment */ y");
    assert_eq!(kinds, vec![
        TokenKind::Identifier("x".to_string()),
        TokenKind::Identifier("y".to_string()),
    ]);
}

#[test]
fn test_nested_block_comment() {
    let kinds = token_kinds("x /* outer /* inner */ still outer */ y");
    assert_eq!(kinds, vec![
        TokenKind::Identifier("x".to_string()),
        TokenKind::Identifier("y".to_string()),
    ]);
}

// ═══════════════════════════════════════════════════════════════
// Decorators and directives
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_decorators() {
    let kinds = token_kinds("@gpu @test @cached @experiment @deprecated");
    assert_eq!(kinds, vec![
        TokenKind::Decorator("gpu".to_string()),
        TokenKind::Decorator("test".to_string()),
        TokenKind::Decorator("cached".to_string()),
        TokenKind::Decorator("experiment".to_string()),
        TokenKind::Decorator("deprecated".to_string()),
    ]);
}

#[test]
fn test_directives() {
    let kinds = token_kinds("#strict #test");
    assert_eq!(kinds, vec![
        TokenKind::Directive("strict".to_string()),
        TokenKind::Directive("test".to_string()),
    ]);
}

// ═══════════════════════════════════════════════════════════════
// Booleans and nil
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_bool_true() {
    let kinds = token_kinds("true");
    assert_eq!(kinds, vec![TokenKind::True]);
}

#[test]
fn test_bool_false() {
    let kinds = token_kinds("false");
    assert_eq!(kinds, vec![TokenKind::False]);
}

#[test]
fn test_nil() {
    let kinds = token_kinds("nil");
    assert_eq!(kinds, vec![TokenKind::Nil]);
}

// ═══════════════════════════════════════════════════════════════
// Source location tracking
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_span_line_tracking() {
    let tokens = scan("x\ny\nz");
    // x is on line 1, y on line 2, z on line 3
    let non_special: Vec<_> = tokens
        .iter()
        .filter(|t| !matches!(t.kind, TokenKind::Newline | TokenKind::Eof))
        .collect();
    assert_eq!(non_special[0].span.line, 1);
    assert_eq!(non_special[1].span.line, 2);
    assert_eq!(non_special[2].span.line, 3);
}

#[test]
fn test_span_column_tracking() {
    let tokens = scan("abc def");
    let non_special: Vec<_> = tokens
        .iter()
        .filter(|t| !matches!(t.kind, TokenKind::Newline | TokenKind::Eof))
        .collect();
    assert_eq!(non_special[0].span.column, 1);
    assert_eq!(non_special[1].span.column, 5);
}

// ═══════════════════════════════════════════════════════════════
// Complex expressions
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_function_def_tokens() {
    let kinds = token_kinds("def add(a: Int, b: Int) -> Int");
    assert_eq!(kinds, vec![
        TokenKind::Def,
        TokenKind::Identifier("add".to_string()),
        TokenKind::LParen,
        TokenKind::Identifier("a".to_string()),
        TokenKind::Colon,
        TokenKind::Identifier("Int".to_string()),
        TokenKind::Comma,
        TokenKind::Identifier("b".to_string()),
        TokenKind::Colon,
        TokenKind::Identifier("Int".to_string()),
        TokenKind::RParen,
        TokenKind::Arrow,
        TokenKind::Identifier("Int".to_string()),
    ]);
}

#[test]
fn test_pipeline_expression() {
    let kinds = token_kinds("data |> filter |> map");
    assert_eq!(kinds, vec![
        TokenKind::Identifier("data".to_string()),
        TokenKind::PipeGt,
        TokenKind::Identifier("filter".to_string()),
        TokenKind::PipeGt,
        TokenKind::Identifier("map".to_string()),
    ]);
}

#[test]
fn test_optional_chaining_and_nil_coalesce() {
    let kinds = token_kinds("user?.name ?? \"Anonymous\"");
    assert_eq!(kinds, vec![
        TokenKind::Identifier("user".to_string()),
        TokenKind::QuestionDot,
        TokenKind::Identifier("name".to_string()),
        TokenKind::QuestionQuestion,
        TokenKind::StringLiteral("Anonymous".to_string()),
    ]);
}

#[test]
fn test_range_with_step() {
    let kinds = token_kinds("0..100 step 2");
    assert_eq!(kinds, vec![
        TokenKind::IntLiteral(0),
        TokenKind::DotDot,
        TokenKind::IntLiteral(100),
        TokenKind::Step,
        TokenKind::IntLiteral(2),
    ]);
}

#[test]
fn test_inclusive_range() {
    let kinds = token_kinds("1..=10");
    assert_eq!(kinds, vec![
        TokenKind::IntLiteral(1),
        TokenKind::DotDotEq,
        TokenKind::IntLiteral(10),
    ]);
}

#[test]
fn test_class_with_interface() {
    let kinds = token_kinds("class User impl Serializable");
    assert_eq!(kinds, vec![
        TokenKind::Class,
        TokenKind::Identifier("User".to_string()),
        TokenKind::Impl,
        TokenKind::Identifier("Serializable".to_string()),
    ]);
}

#[test]
fn test_genetic_class_tokens() {
    let kinds = token_kinds("genetic class Strategy");
    assert_eq!(kinds, vec![
        TokenKind::Genetic,
        TokenKind::Class,
        TokenKind::Identifier("Strategy".to_string()),
    ]);
}

#[test]
fn test_parallel_block_tokens() {
    let kinds = token_kinds("parallel { }");
    assert_eq!(kinds, vec![
        TokenKind::Parallel,
        TokenKind::LBrace,
        TokenKind::RBrace,
    ]);
}

#[test]
fn test_error_propagation() {
    let kinds = token_kinds("result?");
    assert_eq!(kinds, vec![
        TokenKind::Identifier("result".to_string()),
        TokenKind::Question,
    ]);
}

#[test]
fn test_lambda_arrow() {
    let kinds = token_kinds("x -> x * 2");
    assert_eq!(kinds, vec![
        TokenKind::Identifier("x".to_string()),
        TokenKind::Arrow,
        TokenKind::Identifier("x".to_string()),
        TokenKind::Star,
        TokenKind::IntLiteral(2),
    ]);
}

#[test]
fn test_computed_property() {
    let kinds = token_kinds("area: Float => width * height");
    assert_eq!(kinds, vec![
        TokenKind::Identifier("area".to_string()),
        TokenKind::Colon,
        TokenKind::Identifier("Float".to_string()),
        TokenKind::FatArrow,
        TokenKind::Identifier("width".to_string()),
        TokenKind::Star,
        TokenKind::Identifier("height".to_string()),
    ]);
}

#[test]
fn test_string_interpolation_with_expression() {
    let kinds = token_kinds(r#""result: {a + b}""#);
    assert_eq!(kinds.len(), 1);
    match &kinds[0] {
        TokenKind::InterpolatedString(parts) => {
            assert_eq!(parts.len(), 2);
            assert_eq!(parts[0], StringPart::Literal("result: ".to_string()));
            assert_eq!(parts[1], StringPart::Expression("a + b".to_string()));
        }
        _ => panic!("Expected InterpolatedString"),
    }
}

#[test]
fn test_newlines_are_produced() {
    let tokens = scan("a\nb");
    let has_newline = tokens.iter().any(|t| matches!(t.kind, TokenKind::Newline));
    assert!(has_newline);
}

#[test]
fn test_eof_always_present() {
    let tokens = scan("");
    assert!(matches!(tokens.last().unwrap().kind, TokenKind::Eof));
}

#[test]
fn test_exponent_integer() {
    // 1e5 should be a float
    let kinds = token_kinds("1e5");
    assert_eq!(kinds, vec![TokenKind::FloatLiteral(1e5)]);
}

#[test]
fn test_method_call_tokens() {
    let kinds = token_kinds("list.push(42)");
    assert_eq!(kinds, vec![
        TokenKind::Identifier("list".to_string()),
        TokenKind::Dot,
        TokenKind::Identifier("push".to_string()),
        TokenKind::LParen,
        TokenKind::IntLiteral(42),
        TokenKind::RParen,
    ]);
}

#[test]
fn test_decorator_before_def() {
    let kinds = token_kinds("@gpu\ndef compute()");
    assert_eq!(kinds, vec![
        TokenKind::Decorator("gpu".to_string()),
        TokenKind::Def,
        TokenKind::Identifier("compute".to_string()),
        TokenKind::LParen,
        TokenKind::RParen,
    ]);
}

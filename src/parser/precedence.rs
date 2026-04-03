use crate::lexer::tokens::TokenKind;
use crate::parser::ast::BinaryOp;

/// Binding power for Pratt parsing. Higher = tighter binding.
/// Returns (left_bp, right_bp). Left < right means left-associative.
/// Left > right means right-associative.
pub fn infix_binding_power(op: &TokenKind) -> Option<(u8, u8)> {
    match op {
        // Pipeline (lowest precedence among binary ops)
        TokenKind::PipeGt => Some((1, 2)),

        // Logical OR
        TokenKind::PipePipe | TokenKind::Or => Some((3, 4)),

        // Logical AND
        TokenKind::AmpAmp | TokenKind::And => Some((5, 6)),

        // Bitwise OR
        TokenKind::Pipe => Some((7, 8)),

        // Bitwise XOR
        TokenKind::Caret => Some((9, 10)),

        // Bitwise AND
        TokenKind::Amp => Some((11, 12)),

        // Equality
        TokenKind::EqEq | TokenKind::BangEq => Some((13, 14)),

        // Comparison
        TokenKind::Lt | TokenKind::Gt | TokenKind::LtEq | TokenKind::GtEq => Some((15, 16)),

        // Nil coalescing
        TokenKind::QuestionQuestion => Some((17, 18)),

        // Shift
        TokenKind::LtLt | TokenKind::GtGt => Some((19, 20)),

        // Range (non-associative, use same bp)
        TokenKind::DotDot | TokenKind::DotDotEq => Some((21, 22)),

        // Additive
        TokenKind::Plus | TokenKind::Minus => Some((23, 24)),

        // Multiplicative
        TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Some((25, 26)),

        // Power (right-associative: left_bp > right_bp swapped)
        TokenKind::StarStar => Some((28, 27)),

        // `as` cast handled separately in the parser

        _ => None,
    }
}

/// Prefix binding power for unary operators.
pub fn prefix_binding_power(op: &TokenKind) -> Option<u8> {
    match op {
        TokenKind::Minus | TokenKind::Bang | TokenKind::Not | TokenKind::Tilde => Some(29),
        _ => None,
    }
}

/// Postfix binding power for postfix operators (?, field access, index, call).
pub fn postfix_binding_power(op: &TokenKind) -> Option<u8> {
    match op {
        // Postfix ? (error propagation)
        TokenKind::Question => Some(31),
        // Field access, method call, optional chaining
        TokenKind::Dot | TokenKind::QuestionDot => Some(31),
        // Index access
        TokenKind::LBracket => Some(31),
        // Function call
        TokenKind::LParen => Some(31),
        _ => None,
    }
}

/// Convert a token kind to a binary operator.
pub fn token_to_binary_op(kind: &TokenKind) -> Option<BinaryOp> {
    match kind {
        TokenKind::Plus => Some(BinaryOp::Add),
        TokenKind::Minus => Some(BinaryOp::Sub),
        TokenKind::Star => Some(BinaryOp::Mul),
        TokenKind::Slash => Some(BinaryOp::Div),
        TokenKind::Percent => Some(BinaryOp::Mod),
        TokenKind::StarStar => Some(BinaryOp::Pow),
        TokenKind::EqEq => Some(BinaryOp::Eq),
        TokenKind::BangEq => Some(BinaryOp::NotEq),
        TokenKind::Lt => Some(BinaryOp::Lt),
        TokenKind::Gt => Some(BinaryOp::Gt),
        TokenKind::LtEq => Some(BinaryOp::LtEq),
        TokenKind::GtEq => Some(BinaryOp::GtEq),
        TokenKind::AmpAmp | TokenKind::And => Some(BinaryOp::And),
        TokenKind::PipePipe | TokenKind::Or => Some(BinaryOp::Or),
        TokenKind::Amp => Some(BinaryOp::BitAnd),
        TokenKind::Pipe => Some(BinaryOp::BitOr),
        TokenKind::Caret => Some(BinaryOp::BitXor),
        TokenKind::LtLt => Some(BinaryOp::Shl),
        TokenKind::GtGt => Some(BinaryOp::Shr),
        TokenKind::PipeGt => Some(BinaryOp::Pipeline),
        TokenKind::DotDot => Some(BinaryOp::Range),
        TokenKind::DotDotEq => Some(BinaryOp::RangeInclusive),
        TokenKind::QuestionQuestion => Some(BinaryOp::And), // handled specially in parser
        _ => None,
    }
}

use crate::lexer::tokens::*;

/// The Aether lexer/scanner. Converts source text into a stream of tokens.
pub struct Scanner {
    source: Vec<char>,
    file: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    column: usize,
    start_column: usize,
}

impl Scanner {
    pub fn new(source: &str, file: String) -> Self {
        Self {
            source: source.chars().collect(),
            file,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            column: 1,
            start_column: 1,
        }
    }

    /// Scan all tokens from the source.
    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.start_column = self.column;
            self.scan_token();
        }
        self.tokens.push(Token::new(
            TokenKind::Eof,
            String::new(),
            self.make_span(),
        ));
        self.tokens.clone()
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            // Whitespace (not newlines)
            ' ' | '\t' | '\r' => {}

            // Newlines
            '\n' => {
                self.add_token(TokenKind::Newline);
                self.line += 1;
                self.column = 1;
            }

            // Single-char tokens
            '(' => self.add_token(TokenKind::LParen),
            ')' => self.add_token(TokenKind::RParen),
            '[' => self.add_token(TokenKind::LBracket),
            ']' => self.add_token(TokenKind::RBracket),
            '{' => self.add_token(TokenKind::LBrace),
            '}' => self.add_token(TokenKind::RBrace),
            ',' => self.add_token(TokenKind::Comma),
            '~' => self.add_token(TokenKind::Tilde),
            '@' => self.scan_decorator(),
            '#' => self.scan_directive(),

            // Dot variants: . .. ..=
            '.' => {
                if self.match_char('.') {
                    if self.match_char('=') {
                        self.add_token(TokenKind::DotDotEq);
                    } else {
                        self.add_token(TokenKind::DotDot);
                    }
                } else {
                    self.add_token(TokenKind::Dot);
                }
            }

            // Colon
            ':' => self.add_token(TokenKind::Colon),

            // Plus variants: + +=
            '+' => {
                if self.match_char('=') {
                    self.add_token(TokenKind::PlusEq);
                } else {
                    self.add_token(TokenKind::Plus);
                }
            }

            // Minus variants: - -= ->
            '-' => {
                if self.match_char('=') {
                    self.add_token(TokenKind::MinusEq);
                } else if self.match_char('>') {
                    self.add_token(TokenKind::Arrow);
                } else {
                    self.add_token(TokenKind::Minus);
                }
            }

            // Star variants: * ** *= **=
            '*' => {
                if self.match_char('*') {
                    if self.match_char('=') {
                        self.add_token(TokenKind::StarStarEq);
                    } else {
                        self.add_token(TokenKind::StarStar);
                    }
                } else if self.match_char('=') {
                    self.add_token(TokenKind::StarEq);
                } else {
                    self.add_token(TokenKind::Star);
                }
            }

            // Slash variants: / /= // /* */
            '/' => {
                if self.match_char('/') {
                    // Line comment — consume until end of line
                    while !self.is_at_end() && self.peek() != '\n' {
                        self.advance();
                    }
                } else if self.match_char('*') {
                    self.scan_block_comment();
                } else if self.match_char('=') {
                    self.add_token(TokenKind::SlashEq);
                } else {
                    self.add_token(TokenKind::Slash);
                }
            }

            // Percent variants: % %=
            '%' => {
                if self.match_char('=') {
                    self.add_token(TokenKind::PercentEq);
                } else {
                    self.add_token(TokenKind::Percent);
                }
            }

            // Eq variants: = == =>
            '=' => {
                if self.match_char('=') {
                    self.add_token(TokenKind::EqEq);
                } else if self.match_char('>') {
                    self.add_token(TokenKind::FatArrow);
                } else {
                    self.add_token(TokenKind::Eq);
                }
            }

            // Bang variants: ! !=
            '!' => {
                if self.match_char('=') {
                    self.add_token(TokenKind::BangEq);
                } else {
                    self.add_token(TokenKind::Bang);
                }
            }

            // Less variants: < <= << <<=
            '<' => {
                if self.match_char('<') {
                    if self.match_char('=') {
                        self.add_token(TokenKind::LtLtEq);
                    } else {
                        self.add_token(TokenKind::LtLt);
                    }
                } else if self.match_char('=') {
                    self.add_token(TokenKind::LtEq);
                } else {
                    self.add_token(TokenKind::Lt);
                }
            }

            // Greater variants: > >= >> >>=
            '>' => {
                if self.match_char('>') {
                    if self.match_char('=') {
                        self.add_token(TokenKind::GtGtEq);
                    } else {
                        self.add_token(TokenKind::GtGt);
                    }
                } else if self.match_char('=') {
                    self.add_token(TokenKind::GtEq);
                } else {
                    self.add_token(TokenKind::Gt);
                }
            }

            // Amp variants: & && &=
            '&' => {
                if self.match_char('&') {
                    self.add_token(TokenKind::AmpAmp);
                } else if self.match_char('=') {
                    self.add_token(TokenKind::AmpEq);
                } else {
                    self.add_token(TokenKind::Amp);
                }
            }

            // Pipe variants: | || |= |>
            '|' => {
                if self.match_char('|') {
                    self.add_token(TokenKind::PipePipe);
                } else if self.match_char('=') {
                    self.add_token(TokenKind::PipeEq);
                } else if self.match_char('>') {
                    self.add_token(TokenKind::PipeGt);
                } else {
                    self.add_token(TokenKind::Pipe);
                }
            }

            // Caret variants: ^ ^=
            '^' => {
                if self.match_char('=') {
                    self.add_token(TokenKind::CaretEq);
                } else {
                    self.add_token(TokenKind::Caret);
                }
            }

            // Question variants: ? ?. ??
            '?' => {
                if self.match_char('.') {
                    self.add_token(TokenKind::QuestionDot);
                } else if self.match_char('?') {
                    self.add_token(TokenKind::QuestionQuestion);
                } else {
                    self.add_token(TokenKind::Question);
                }
            }

            // Strings
            '"' => self.scan_string(),

            // Raw strings: r"..."
            'r' if self.peek() == '"' => {
                self.advance(); // consume the opening "
                self.scan_raw_string();
            }

            // Char literals
            '\'' => self.scan_char(),

            // Numbers
            c if c.is_ascii_digit() => self.scan_number(c),

            // Identifiers and keywords
            c if c.is_alphabetic() || c == '_' => self.scan_identifier(),

            _ => {
                // Skip unknown characters — could report error
            }
        }
    }

    // ── Helpers ───────────────────────────────────────────────

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let c = self.source[self.current];
        self.current += 1;
        self.column += 1;
        c
    }

    fn peek(&self) -> char {
        if self.is_at_end() { '\0' } else { self.source[self.current] }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() { '\0' } else { self.source[self.current + 1] }
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.source[self.current] != expected {
            return false;
        }
        self.current += 1;
        self.column += 1;
        true
    }

    fn make_span(&self) -> Span {
        Span::new(self.file.clone(), self.line, self.start_column)
    }

    fn lexeme(&self) -> String {
        self.source[self.start..self.current].iter().collect()
    }

    fn add_token(&mut self, kind: TokenKind) {
        let lexeme = self.lexeme();
        let span = self.make_span();
        self.tokens.push(Token::new(kind, lexeme, span));
    }

    // ── Block comments ───────────────────────────────────────

    fn scan_block_comment(&mut self) {
        let mut depth = 1;
        while !self.is_at_end() && depth > 0 {
            if self.peek() == '/' && self.peek_next() == '*' {
                self.advance();
                self.advance();
                depth += 1;
            } else if self.peek() == '*' && self.peek_next() == '/' {
                self.advance();
                self.advance();
                depth -= 1;
            } else {
                if self.peek() == '\n' {
                    self.line += 1;
                    self.column = 0; // will be 1 after advance
                }
                self.advance();
            }
        }
    }

    // ── Strings ──────────────────────────────────────────────

    fn scan_string(&mut self) {
        // Check for multiline: """
        if self.peek() == '"' && self.peek_next() == '"' {
            self.advance(); // second "
            self.advance(); // third "
            self.scan_multiline_string();
            return;
        }

        let mut parts: Vec<StringPart> = Vec::new();
        let mut current_literal = String::new();
        let mut has_interpolation = false;

        while !self.is_at_end() && self.peek() != '"' {
            if self.peek() == '\n' {
                // Unterminated string on this line
                break;
            }
            if self.peek() == '\\' {
                self.advance(); // consume backslash
                let escaped = self.scan_escape_char();
                current_literal.push(escaped);
            } else if self.peek() == '{' {
                // Check for {{ (literal brace)
                if self.peek_next() == '{' {
                    self.advance(); // first {
                    self.advance(); // second {
                    current_literal.push('{');
                } else {
                    // Interpolation
                    has_interpolation = true;
                    self.advance(); // consume {
                    if !current_literal.is_empty() {
                        parts.push(StringPart::Literal(current_literal.clone()));
                        current_literal.clear();
                    }
                    let mut expr = String::new();
                    let mut brace_depth = 1;
                    while !self.is_at_end() && brace_depth > 0 {
                        if self.peek() == '{' {
                            brace_depth += 1;
                            expr.push(self.advance());
                        } else if self.peek() == '}' {
                            brace_depth -= 1;
                            if brace_depth > 0 {
                                expr.push(self.advance());
                            } else {
                                self.advance(); // consume closing }
                            }
                        } else {
                            expr.push(self.advance());
                        }
                    }
                    parts.push(StringPart::Expression(expr));
                }
            } else if self.peek() == '}' && self.peek_next() == '}' {
                // Literal }
                self.advance();
                self.advance();
                current_literal.push('}');
            } else {
                current_literal.push(self.advance());
            }
        }

        // Consume closing "
        if !self.is_at_end() {
            self.advance();
        }

        if has_interpolation {
            if !current_literal.is_empty() {
                parts.push(StringPart::Literal(current_literal));
            }
            self.add_token(TokenKind::InterpolatedString(parts));
        } else {
            self.add_token(TokenKind::StringLiteral(current_literal));
        }
    }

    fn scan_multiline_string(&mut self) {
        let mut value = String::new();
        while !self.is_at_end() {
            if self.peek() == '"' && self.peek_next() == '"' {
                // Check for third "
                if self.current + 2 < self.source.len() && self.source[self.current + 2] == '"' {
                    self.advance(); // "
                    self.advance(); // "
                    self.advance(); // "
                    // Trim leading/trailing newlines and common indentation
                    let trimmed = Self::trim_multiline(&value);
                    self.add_token(TokenKind::MultilineString(trimmed));
                    return;
                }
            }
            if self.peek() == '\n' {
                self.line += 1;
                self.column = 0;
            }
            value.push(self.advance());
        }
        // Unterminated multiline string
        self.add_token(TokenKind::MultilineString(value));
    }

    fn trim_multiline(s: &str) -> String {
        let mut lines: Vec<&str> = s.lines().collect();

        // Remove first line if empty
        if let Some(first) = lines.first() {
            if first.trim().is_empty() {
                lines.remove(0);
            }
        }
        // Remove last line if empty
        if let Some(last) = lines.last() {
            if last.trim().is_empty() {
                lines.pop();
            }
        }

        // Find minimum indentation
        let min_indent = lines
            .iter()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.len() - l.trim_start().len())
            .min()
            .unwrap_or(0);

        // Strip common indentation
        lines
            .iter()
            .map(|l| {
                if l.len() >= min_indent {
                    &l[min_indent..]
                } else {
                    l.trim()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn scan_raw_string(&mut self) {
        let mut value = String::new();
        while !self.is_at_end() && self.peek() != '"' {
            if self.peek() == '\n' {
                break;
            }
            value.push(self.advance());
        }
        if !self.is_at_end() {
            self.advance(); // closing "
        }
        self.add_token(TokenKind::RawString(value));
    }

    fn scan_escape_char(&mut self) -> char {
        if self.is_at_end() {
            return '\\';
        }
        let c = self.advance();
        match c {
            'n' => '\n',
            't' => '\t',
            'r' => '\r',
            '\\' => '\\',
            '"' => '"',
            '\'' => '\'',
            '0' => '\0',
            _ => c,
        }
    }

    // ── Char literals ────────────────────────────────────────

    fn scan_char(&mut self) {
        if self.is_at_end() {
            return;
        }
        let c = if self.peek() == '\\' {
            self.advance(); // consume backslash
            self.scan_escape_char()
        } else {
            self.advance()
        };
        if !self.is_at_end() && self.peek() == '\'' {
            self.advance(); // closing '
        }
        self.add_token(TokenKind::CharLiteral(c));
    }

    // ── Numbers ──────────────────────────────────────────────

    fn scan_number(&mut self, first: char) {
        // Check for hex/binary prefix
        if first == '0' {
            if self.peek() == 'x' || self.peek() == 'X' {
                self.advance(); // x
                self.scan_hex_number();
                return;
            }
            if self.peek() == 'b' || self.peek() == 'B' {
                self.advance(); // b
                self.scan_binary_number();
                return;
            }
        }

        // Decimal integer or float
        while !self.is_at_end() && (self.peek().is_ascii_digit() || self.peek() == '_') {
            self.advance();
        }

        let mut is_float = false;

        // Check for decimal point (but not .. range operator)
        if self.peek() == '.' && self.peek_next() != '.' && self.peek_next() != ')' && !self.peek_next().is_alphabetic() {
            // Could be float if next char is a digit
            if self.peek_next().is_ascii_digit() {
                is_float = true;
                self.advance(); // consume .
                while !self.is_at_end() && (self.peek().is_ascii_digit() || self.peek() == '_') {
                    self.advance();
                }
            }
        }

        // Check for exponent
        if self.peek() == 'e' || self.peek() == 'E' {
            is_float = true;
            self.advance(); // consume e
            if self.peek() == '+' || self.peek() == '-' {
                self.advance();
            }
            while !self.is_at_end() && (self.peek().is_ascii_digit() || self.peek() == '_') {
                self.advance();
            }
        }

        let text: String = self.source[self.start..self.current]
            .iter()
            .filter(|c| **c != '_')
            .collect();

        if is_float {
            let val: f64 = text.parse().unwrap_or(0.0);
            self.add_token(TokenKind::FloatLiteral(val));
        } else {
            let val: i64 = text.parse().unwrap_or(0);
            self.add_token(TokenKind::IntLiteral(val));
        }
    }

    fn scan_hex_number(&mut self) {
        while !self.is_at_end() && (self.peek().is_ascii_hexdigit() || self.peek() == '_') {
            self.advance();
        }
        let hex_str: String = self.source[self.start + 2..self.current]
            .iter()
            .filter(|c| **c != '_')
            .collect();
        let val = i64::from_str_radix(&hex_str, 16).unwrap_or(0);
        self.add_token(TokenKind::IntLiteral(val));
    }

    fn scan_binary_number(&mut self) {
        while !self.is_at_end() && (self.peek() == '0' || self.peek() == '1' || self.peek() == '_') {
            self.advance();
        }
        let bin_str: String = self.source[self.start + 2..self.current]
            .iter()
            .filter(|c| **c != '_')
            .collect();
        let val = i64::from_str_radix(&bin_str, 2).unwrap_or(0);
        self.add_token(TokenKind::IntLiteral(val));
    }

    // ── Identifiers and keywords ─────────────────────────────

    fn scan_identifier(&mut self) {
        while !self.is_at_end() && (self.peek().is_alphanumeric() || self.peek() == '_') {
            self.advance();
        }
        let text: String = self.source[self.start..self.current].iter().collect();
        let kind = Self::keyword_or_ident(&text);
        self.add_token(kind);
    }

    fn keyword_or_ident(text: &str) -> TokenKind {
        match text {
            "def" => TokenKind::Def,
            "let" => TokenKind::Let,
            "const" => TokenKind::Const,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "match" => TokenKind::Match,
            "guard" => TokenKind::Guard,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "loop" => TokenKind::Loop,
            "times" => TokenKind::Times,
            "while" => TokenKind::While,
            "until" => TokenKind::Until,
            "step" => TokenKind::Step,
            "break" => TokenKind::Break,
            "next" => TokenKind::Next,
            "return" => TokenKind::Return,
            "class" => TokenKind::Class,
            "struct" => TokenKind::Struct,
            "enum" => TokenKind::Enum,
            "interface" => TokenKind::Interface,
            "impl" => TokenKind::Impl,
            "self" => TokenKind::SelfKw,
            "super" => TokenKind::Super,
            "init" => TokenKind::Init,
            "deinit" => TokenKind::Deinit,
            "pub" => TokenKind::Pub,
            "priv" => TokenKind::Priv,
            "prot" => TokenKind::Prot,
            "readonly" => TokenKind::Readonly,
            "static" => TokenKind::Static,
            "lazy" => TokenKind::Lazy,
            "async" => TokenKind::Async,
            "await" => TokenKind::Await,
            "try" => TokenKind::Try,
            "catch" => TokenKind::Catch,
            "finally" => TokenKind::Finally,
            "throw" => TokenKind::Throw,
            "use" => TokenKind::Use,
            "mod" => TokenKind::Mod,
            "as" => TokenKind::As,
            "type" => TokenKind::Type,
            "parallel" => TokenKind::Parallel,
            "after" => TokenKind::After,
            "device" => TokenKind::Device,
            "model" => TokenKind::Model,
            "agent" => TokenKind::Agent,
            "pipeline" => TokenKind::Pipeline,
            "reactive" => TokenKind::Reactive,
            "temporal" => TokenKind::Temporal,
            "mutation" => TokenKind::Mutation,
            "evolving" => TokenKind::Evolving,
            "genetic" => TokenKind::Genetic,
            "gene" => TokenKind::Gene,
            "chromosome" => TokenKind::Chromosome,
            "fitness" => TokenKind::Fitness,
            "crossover" => TokenKind::Crossover,
            "breed" => TokenKind::Breed,
            "evolve" => TokenKind::Evolve,
            "weave" => TokenKind::Weave,
            "bond" => TokenKind::Bond,
            "face" => TokenKind::Face,
            "extend" => TokenKind::Extend,
            "delegate" => TokenKind::Delegate,
            "select" => TokenKind::Select,
            "exclude" => TokenKind::Exclude,
            "morph" => TokenKind::Morph,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "nil" => TokenKind::Nil,
            "and" => TokenKind::And,
            "or" => TokenKind::Or,
            "not" => TokenKind::Not,
            "then" => TokenKind::Then,
            "operator" => TokenKind::Operator,
            "override" => TokenKind::Override,
            "where" => TokenKind::Where,
            "with" => TokenKind::With,
            _ => TokenKind::Identifier(text.to_string()),
        }
    }

    // ── Decorators and directives ────────────────────────────

    fn scan_decorator(&mut self) {
        // @ followed by identifier
        while !self.is_at_end() && (self.peek().is_alphanumeric() || self.peek() == '_') {
            self.advance();
        }
        let name: String = self.source[self.start + 1..self.current].iter().collect();
        if name.is_empty() {
            self.add_token(TokenKind::At);
        } else {
            self.add_token(TokenKind::Decorator(name));
        }
    }

    fn scan_directive(&mut self) {
        // # followed by identifier
        while !self.is_at_end() && (self.peek().is_alphanumeric() || self.peek() == '_') {
            self.advance();
        }
        let name: String = self.source[self.start + 1..self.current].iter().collect();
        if name.is_empty() {
            self.add_token(TokenKind::Hash);
        } else {
            self.add_token(TokenKind::Directive(name));
        }
    }
}

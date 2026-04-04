use crate::lexer::tokens::*;
use crate::parser::ast::*;
use crate::parser::precedence::*;

/// Parse error with location info.
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}: {}", self.span.file, self.span.line, self.span.column, self.message)
    }
}

/// The Aether recursive descent parser.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    errors: Vec<ParseError>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0, errors: Vec::new() }
    }

    /// Parse a complete program.
    pub fn parse_program(&mut self) -> Result<Program, Vec<ParseError>> {
        let mut statements = Vec::new();
        let mut directives = Vec::new();

        self.skip_newlines();
        while !self.is_at_end() {
            // Collect directives
            if let TokenKind::Directive(name) = self.peek_kind() {
                let span = self.current_span();
                let name = name.clone();
                self.advance();
                directives.push(Directive { name, span });
                self.skip_newlines();
                continue;
            }

            // Skip stray closing braces that may remain after error recovery
            if matches!(self.peek_kind(), TokenKind::RBrace) {
                self.advance();
                self.skip_newlines();
                continue;
            }

            let pos_before = self.pos;
            match self.parse_statement() {
                Ok(stmt) => statements.push(stmt),
                Err(e) => {
                    self.errors.push(e);
                    self.synchronize();
                    // Ensure progress — if synchronize didn't advance, force advance
                    if self.pos == pos_before && !self.is_at_end() {
                        self.advance();
                    }
                }
            }
            self.skip_newlines();
        }

        if self.errors.is_empty() {
            Ok(Program { statements, directives })
        } else {
            Err(self.errors.clone())
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // Token navigation
    // ═══════════════════════════════════════════════════════════════

    fn peek(&self) -> &Token {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    fn peek_kind(&self) -> &TokenKind {
        &self.peek().kind
    }

    fn peek_ahead(&self, n: usize) -> &TokenKind {
        let mut idx = self.pos;
        let mut skipped = 0;
        while skipped < n && idx < self.tokens.len() - 1 {
            idx += 1;
            if !matches!(self.tokens[idx].kind, TokenKind::Newline) {
                skipped += 1;
            }
        }
        &self.tokens[idx.min(self.tokens.len() - 1)].kind
    }

    fn current_span(&self) -> Span {
        self.peek().span.clone()
    }

    fn advance(&mut self) -> &Token {
        let token = &self.tokens[self.pos.min(self.tokens.len() - 1)];
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        token
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::Eof)
    }

    fn skip_newlines(&mut self) {
        while matches!(self.peek_kind(), TokenKind::Newline) {
            self.advance();
        }
    }

    fn expect(&mut self, expected: &TokenKind) -> Result<Token, ParseError> {
        self.skip_newlines();
        if std::mem::discriminant(self.peek_kind()) == std::mem::discriminant(expected) {
            Ok(self.advance().clone())
        } else {
            Err(self.error(format!("expected {:?}, found {:?}", expected, self.peek_kind())))
        }
    }

    fn expect_identifier(&mut self) -> Result<String, ParseError> {
        self.skip_newlines();
        if let TokenKind::Identifier(name) = self.peek_kind().clone() {
            self.advance();
            Ok(name)
        } else {
            // Many Aether keywords can also be used as identifiers (field names, etc.)
            let name = self.keyword_as_identifier();
            if let Some(name) = name {
                self.advance();
                Ok(name)
            } else {
                Err(self.error(format!("expected identifier, found {:?}", self.peek_kind())))
            }
        }
    }

    /// Try to interpret the current keyword token as an identifier string.
    fn keyword_as_identifier(&self) -> Option<String> {
        let s = match self.peek_kind() {
            TokenKind::Breed => "breed",
            TokenKind::Model => "model",
            TokenKind::Agent => "agent",
            TokenKind::Pipeline => "pipeline",
            TokenKind::After => "after",
            TokenKind::Device => "device",
            TokenKind::Step => "step",
            TokenKind::Times => "times",
            TokenKind::Until => "until",
            TokenKind::Gene => "gene",
            TokenKind::Chromosome => "chromosome",
            TokenKind::Fitness => "fitness",
            TokenKind::Crossover => "crossover",
            TokenKind::Evolve => "evolve",
            TokenKind::Weave => "weave",
            TokenKind::Bond => "bond",
            TokenKind::Face => "face",
            TokenKind::Extend => "extend",
            TokenKind::Delegate => "delegate",
            TokenKind::Select => "select",
            TokenKind::Exclude => "exclude",
            TokenKind::Morph => "morph",
            TokenKind::Evolving => "evolving",
            TokenKind::Reactive => "reactive",
            TokenKind::Temporal => "temporal",
            TokenKind::Mutation => "mutation",
            TokenKind::Genetic => "genetic",
            TokenKind::Type => "type",
            TokenKind::As => "as",
            TokenKind::Init => "init",
            TokenKind::Override => "override",
            TokenKind::Where => "where",
            TokenKind::With => "with",
            TokenKind::Operator => "operator",
            _ => return None,
        };
        Some(s.to_string())
    }

    fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(self.peek_kind()) == std::mem::discriminant(kind)
    }

    fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn error(&self, message: String) -> ParseError {
        ParseError { message, span: self.current_span() }
    }

    /// Recover from parse error by skipping to next statement boundary.
    fn synchronize(&mut self) {
        while !self.is_at_end() {
            match self.peek_kind() {
                TokenKind::Newline => {
                    self.advance();
                    return;
                }
                TokenKind::Def | TokenKind::Class | TokenKind::Struct | TokenKind::Enum
                | TokenKind::Interface | TokenKind::If | TokenKind::For | TokenKind::Loop
                | TokenKind::Return | TokenKind::Use | TokenKind::RBrace => return,
                _ => { self.advance(); }
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // Statements
    // ═══════════════════════════════════════════════════════════════

    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        self.skip_newlines();
        let span = self.current_span();

        // Collect decorators
        let mut decorators = Vec::new();
        while let TokenKind::Decorator(name) = self.peek_kind().clone() {
            let dec_span = self.current_span();
            let name = name.clone();
            self.advance();
            let args = if self.match_token(&TokenKind::LParen) {
                let args = self.parse_arguments()?;
                self.expect(&TokenKind::RParen)?;
                args
            } else {
                Vec::new()
            };
            decorators.push(Decorator { name, args, span: dec_span });
            self.skip_newlines();
        }

        let kind = match self.peek_kind().clone() {
            TokenKind::Let => self.parse_let_decl()?,
            TokenKind::Const => self.parse_const_decl()?,
            TokenKind::Def => self.parse_func_def(false, false, decorators.clone(), AccessModifier::Pub)?,
            TokenKind::Async => {
                self.advance();
                if matches!(self.peek_kind(), TokenKind::Def) {
                    self.parse_func_def(true, false, decorators.clone(), AccessModifier::Pub)?
                } else {
                    return Err(self.error("expected 'def' after 'async'".into()));
                }
            }
            TokenKind::Class => self.parse_class_def(false, decorators.clone())?,
            TokenKind::Genetic => {
                self.advance();
                self.expect(&TokenKind::Class)?;
                self.parse_class_body(true, decorators.clone())?
            }
            TokenKind::Struct => self.parse_struct_def(decorators.clone())?,
            TokenKind::Enum => self.parse_enum_def(decorators.clone())?,
            TokenKind::Interface => self.parse_interface_def()?,
            TokenKind::If => self.parse_if_stmt()?,
            TokenKind::Guard => self.parse_guard_stmt()?,
            TokenKind::Match => self.parse_match_stmt()?,
            TokenKind::For => self.parse_for_loop()?,
            TokenKind::Loop => self.parse_loop_stmt()?,
            TokenKind::Break => self.parse_break()?,
            TokenKind::Next => self.parse_next()?,
            TokenKind::Return => self.parse_return()?,
            TokenKind::Throw => self.parse_throw()?,
            TokenKind::Try => self.parse_try_catch()?,
            TokenKind::Parallel => self.parse_parallel()?,
            TokenKind::Mutation => self.parse_mutation_stmt()?,
            TokenKind::Device => self.parse_device()?,
            TokenKind::Use => self.parse_use()?,
            TokenKind::Mod => self.parse_mod_block()?,
            TokenKind::Type => self.parse_type_alias()?,
            TokenKind::Weave => self.parse_weave_def()?,
            TokenKind::Extend => self.parse_extend_block()?,
            TokenKind::After => self.parse_after_stmt()?,
            _ => self.parse_expression_or_assignment()?,
        };

        Ok(Stmt { kind, span })
    }

    fn parse_expression_or_assignment(&mut self) -> Result<StmtKind, ParseError> {
        let expr = self.parse_expression(0)?;

        // Check for assignment operators
        let assign_op = match self.peek_kind() {
            TokenKind::Eq => {
                // Distinguish between `name = val` (var decl) and `expr = val` (assignment)
                // If expr is a bare identifier, treat as VarDecl
                if let ExprKind::Identifier(name) = &expr.kind {
                    let name = name.clone();
                    self.advance();
                    let value = self.parse_expression(0)?;
                    return Ok(StmtKind::VarDecl {
                        name,
                        type_ann: None,
                        value: Some(value),
                        mutable: true,
                        is_const: false,
                    });
                }
                Some(AssignOp::Assign)
            }
            TokenKind::PlusEq => Some(AssignOp::AddAssign),
            TokenKind::MinusEq => Some(AssignOp::SubAssign),
            TokenKind::StarEq => Some(AssignOp::MulAssign),
            TokenKind::SlashEq => Some(AssignOp::DivAssign),
            TokenKind::PercentEq => Some(AssignOp::ModAssign),
            TokenKind::StarStarEq => Some(AssignOp::PowAssign),
            TokenKind::AmpEq => Some(AssignOp::BitAndAssign),
            TokenKind::PipeEq => Some(AssignOp::BitOrAssign),
            TokenKind::CaretEq => Some(AssignOp::BitXorAssign),
            TokenKind::LtLtEq => Some(AssignOp::ShlAssign),
            TokenKind::GtGtEq => Some(AssignOp::ShrAssign),
            _ => None,
        };

        if let Some(op) = assign_op {
            self.advance();
            let value = self.parse_expression(0)?;
            Ok(StmtKind::Assignment { target: expr, op, value })
        } else {
            // Check for typed variable declaration: `name: Type = val`
            if let ExprKind::Identifier(name) = &expr.kind {
                if matches!(self.peek_kind(), TokenKind::Colon) {
                    let name = name.clone();
                    self.advance(); // :
                    let type_ann = self.parse_type_annotation()?;
                    let value = if self.match_token(&TokenKind::Eq) {
                        Some(self.parse_expression(0)?)
                    } else {
                        None
                    };
                    return Ok(StmtKind::VarDecl {
                        name,
                        type_ann: Some(type_ann),
                        value,
                        mutable: true,
                        is_const: false,
                    });
                }
            }
            Ok(StmtKind::Expression(expr))
        }
    }

    fn parse_let_decl(&mut self) -> Result<StmtKind, ParseError> {
        self.advance(); // let
        let name = self.expect_identifier()?;
        let type_ann = if self.match_token(&TokenKind::Colon) {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };
        let value = if self.match_token(&TokenKind::Eq) {
            Some(self.parse_expression(0)?)
        } else {
            None
        };
        Ok(StmtKind::VarDecl { name, type_ann, value, mutable: false, is_const: false })
    }

    fn parse_const_decl(&mut self) -> Result<StmtKind, ParseError> {
        self.advance(); // const
        let name = self.expect_identifier()?;
        let type_ann = if self.match_token(&TokenKind::Colon) {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };
        self.expect(&TokenKind::Eq)?;
        let value = self.parse_expression(0)?;
        Ok(StmtKind::VarDecl { name, type_ann, value: Some(value), mutable: false, is_const: true })
    }

    // ── Functions ────────────────────────────────────────────────

    fn parse_func_def(
        &mut self,
        is_async: bool,
        is_static: bool,
        decorators: Vec<Decorator>,
        access: AccessModifier,
    ) -> Result<StmtKind, ParseError> {
        let span = self.current_span();
        self.advance(); // def
        let name = self.expect_identifier()?;
        self.expect(&TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect(&TokenKind::RParen)?;

        let return_type = if self.match_token(&TokenKind::Arrow) {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        let body = if self.match_token(&TokenKind::Eq) {
            // Expression body: def double(x) = x * 2
            FuncBody::Expression(self.parse_expression(0)?)
        } else {
            self.skip_newlines();
            self.expect(&TokenKind::LBrace)?;
            let stmts = self.parse_block()?;
            FuncBody::Block(stmts)
        };

        Ok(StmtKind::FuncDef(FuncDef {
            name, params, return_type, body, is_async, is_static,
            decorators, access, span,
        }))
    }

    fn parse_params(&mut self) -> Result<Vec<Param>, ParseError> {
        let mut params = Vec::new();
        self.skip_newlines();
        if self.check(&TokenKind::RParen) {
            return Ok(params);
        }

        loop {
            self.skip_newlines();
            let variadic = self.match_token(&TokenKind::Star);
            let kw_variadic = if !variadic { self.match_token(&TokenKind::StarStar) } else { false };
            let name = self.expect_identifier()?;
            let type_ann = if self.match_token(&TokenKind::Colon) {
                Some(self.parse_type_annotation()?)
            } else {
                None
            };
            let default = if self.match_token(&TokenKind::Eq) {
                Some(self.parse_expression(0)?)
            } else {
                None
            };
            params.push(Param { name, type_ann, default, variadic, kw_variadic });

            self.skip_newlines();
            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }
        Ok(params)
    }

    fn parse_arguments(&mut self) -> Result<Vec<Argument>, ParseError> {
        let mut args = Vec::new();
        self.skip_newlines();
        if self.check(&TokenKind::RParen) {
            return Ok(args);
        }

        loop {
            self.skip_newlines();
            // Check for named argument: name: value
            let name = if let TokenKind::Identifier(id) = self.peek_kind().clone() {
                if matches!(self.peek_ahead(1), TokenKind::Colon) {
                    self.advance(); // identifier
                    self.advance(); // :
                    Some(id)
                } else {
                    None
                }
            } else {
                None
            };

            let value = self.parse_expression(0)?;
            args.push(Argument { name, value });

            self.skip_newlines();
            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }
        Ok(args)
    }

    // ── Classes ──────────────────────────────────────────────────

    fn parse_class_def(&mut self, is_genetic: bool, decorators: Vec<Decorator>) -> Result<StmtKind, ParseError> {
        self.advance(); // class
        self.parse_class_body(is_genetic, decorators)
    }

    fn parse_class_body(&mut self, is_genetic: bool, decorators: Vec<Decorator>) -> Result<StmtKind, ParseError> {
        let span = self.current_span();
        let name = self.expect_identifier()?;

        let mut parent = None;
        let mut interfaces = Vec::new();
        let mut weaves = Vec::new();
        let mut select = None;
        let mut exclude = None;
        let mut capabilities = None;

        // Parse class modifiers: : Parent, impl Interface, with Weave, select(...), exclude(...)
        loop {
            self.skip_newlines();
            match self.peek_kind().clone() {
                TokenKind::Colon => {
                    self.advance();
                    parent = Some(self.expect_identifier()?);
                    // select/exclude after parent
                    self.skip_newlines();
                    if matches!(self.peek_kind(), TokenKind::Select) {
                        self.advance();
                        self.expect(&TokenKind::LParen)?;
                        select = Some(self.parse_identifier_list()?);
                        self.expect(&TokenKind::RParen)?;
                    } else if matches!(self.peek_kind(), TokenKind::Exclude) {
                        self.advance();
                        self.expect(&TokenKind::LParen)?;
                        exclude = Some(self.parse_identifier_list()?);
                        self.expect(&TokenKind::RParen)?;
                    }
                }
                TokenKind::Impl => {
                    self.advance();
                    interfaces.push(self.expect_identifier()?);
                    while self.match_token(&TokenKind::Comma) {
                        interfaces.push(self.expect_identifier()?);
                    }
                }
                TokenKind::With => {
                    self.advance();
                    if matches!(self.peek_kind(), TokenKind::Identifier(ref s) if s == "capabilities") {
                        self.advance();
                        self.expect(&TokenKind::LParen)?;
                        capabilities = Some(self.parse_identifier_list()?);
                        self.expect(&TokenKind::RParen)?;
                    } else {
                        weaves.push(self.expect_identifier()?);
                        while self.match_token(&TokenKind::Comma) {
                            weaves.push(self.expect_identifier()?);
                        }
                    }
                }
                _ => break,
            }
        }

        self.skip_newlines();
        self.expect(&TokenKind::LBrace)?;

        let mut class_def = ClassDef {
            name, parent, interfaces, weaves, select, exclude, capabilities,
            fields: Vec::new(), init: None, deinit: None, methods: Vec::new(),
            computed_props: Vec::new(), observed_props: Vec::new(), lazy_props: Vec::new(),
            static_fields: Vec::new(), static_methods: Vec::new(), operators: Vec::new(),
            bonds: Vec::new(), faces: Vec::new(), delegates: Vec::new(),
            morph_methods: Vec::new(), reactive_props: Vec::new(), temporal_props: Vec::new(),
            mutation_props: Vec::new(), evolving_props: Vec::new(), chromosomes: Vec::new(),
            fitness_fn: None, is_genetic, decorators, access: AccessModifier::Pub, span,
        };

        self.parse_class_members(&mut class_def)?;
        self.expect(&TokenKind::RBrace)?;

        Ok(StmtKind::ClassDef(class_def))
    }

    fn parse_class_members(&mut self, class: &mut ClassDef) -> Result<(), ParseError> {
        loop {
            self.skip_newlines();
            if self.check(&TokenKind::RBrace) || self.is_at_end() {
                break;
            }

            // Collect decorators for member
            let mut decorators = Vec::new();
            while let TokenKind::Decorator(name) = self.peek_kind().clone() {
                let dec_span = self.current_span();
                let name = name.clone();
                self.advance();
                let args = if self.match_token(&TokenKind::LParen) {
                    let a = self.parse_arguments()?;
                    self.expect(&TokenKind::RParen)?;
                    a
                } else { Vec::new() };
                decorators.push(Decorator { name, args, span: dec_span });
                self.skip_newlines();
            }

            // Parse access modifier
            let access = self.parse_access_modifier();
            let is_readonly = self.match_token(&TokenKind::Readonly);
            let is_static = self.match_token(&TokenKind::Static);

            match self.peek_kind().clone() {
                TokenKind::Def => {
                    if let StmtKind::FuncDef(mut fd) = self.parse_func_def(false, is_static, decorators, access)? {
                        fd.is_static = is_static;
                        if is_static {
                            class.static_methods.push(fd);
                        } else {
                            class.methods.push(fd);
                        }
                    }
                }
                TokenKind::Async => {
                    self.advance();
                    if let StmtKind::FuncDef(fd) = self.parse_func_def(true, is_static, decorators, access)? {
                        class.methods.push(fd);
                    }
                }
                TokenKind::Init => {
                    let init_span = self.current_span();
                    self.advance();
                    self.expect(&TokenKind::LParen)?;
                    let params = self.parse_params()?;
                    self.expect(&TokenKind::RParen)?;
                    self.skip_newlines();
                    self.expect(&TokenKind::LBrace)?;
                    let body = self.parse_block()?;
                    class.init = Some(FuncDef {
                        name: "init".into(), params, return_type: None,
                        body: FuncBody::Block(body), is_async: false, is_static: false,
                        decorators: Vec::new(), access: AccessModifier::Pub, span: init_span,
                    });
                }
                TokenKind::Deinit => {
                    self.advance();
                    self.skip_newlines();
                    self.expect(&TokenKind::LBrace)?;
                    class.deinit = Some(self.parse_block()?);
                }
                TokenKind::Operator => {
                    let op_span = self.current_span();
                    self.advance();
                    let op_token = self.advance().clone();
                    let op = op_token.lexeme.clone();
                    self.expect(&TokenKind::LParen)?;
                    let params = self.parse_params()?;
                    self.expect(&TokenKind::RParen)?;
                    let return_type = if self.match_token(&TokenKind::Arrow) {
                        Some(self.parse_type_annotation()?)
                    } else { None };
                    self.skip_newlines();
                    self.expect(&TokenKind::LBrace)?;
                    let body = self.parse_block()?;
                    class.operators.push(OperatorDef { op, params, return_type, body, span: op_span });
                }
                TokenKind::Lazy => {
                    let lazy_span = self.current_span();
                    self.advance();
                    let name = self.expect_identifier()?;
                    let type_ann = if self.match_token(&TokenKind::Colon) {
                        Some(self.parse_type_annotation()?)
                    } else { None };
                    self.expect(&TokenKind::FatArrow)?;
                    let init = self.parse_expression(0)?;
                    class.lazy_props.push(LazyProp { name, type_ann, initializer: init, span: lazy_span });
                }
                TokenKind::Reactive => {
                    let rp_span = self.current_span();
                    self.advance();
                    let name = self.expect_identifier()?;
                    let type_ann = if self.match_token(&TokenKind::Colon) {
                        Some(self.parse_type_annotation()?)
                    } else { None };
                    self.expect(&TokenKind::Eq)?;
                    let expr = self.parse_expression(0)?;
                    class.reactive_props.push(ReactiveProp { name, type_ann, compute_expr: expr, span: rp_span });
                }
                TokenKind::Temporal => {
                    let tp_span = self.current_span();
                    self.advance();
                    // temporal(keep: N) name: Type = default
                    self.expect(&TokenKind::LParen)?;
                    self.expect_identifier()?; // "keep"
                    self.expect(&TokenKind::Colon)?;
                    let keep_expr = self.parse_expression(0)?;
                    let keep = if let ExprKind::IntLiteral(n) = keep_expr.kind { n as usize } else { 100 };
                    self.expect(&TokenKind::RParen)?;
                    let name = self.expect_identifier()?;
                    let type_ann = if self.match_token(&TokenKind::Colon) {
                        Some(self.parse_type_annotation()?)
                    } else { None };
                    let default = if self.match_token(&TokenKind::Eq) {
                        Some(self.parse_expression(0)?)
                    } else { None };
                    class.temporal_props.push(TemporalProp { name, type_ann, keep, default, span: tp_span });
                }
                TokenKind::Mutation => {
                    let mp_span = self.current_span();
                    self.advance();
                    // mutation(tracked, undoable: N) name: Type = default { ... }
                    let mut tracked = false;
                    let mut undoable = None;
                    if self.match_token(&TokenKind::LParen) {
                        loop {
                            self.skip_newlines();
                            if let TokenKind::Identifier(kw) = self.peek_kind().clone() {
                                match kw.as_str() {
                                    "tracked" => { self.advance(); tracked = true; }
                                    "undoable" => {
                                        self.advance();
                                        self.expect(&TokenKind::Colon)?;
                                        let n = self.parse_expression(0)?;
                                        if let ExprKind::IntLiteral(v) = n.kind {
                                            undoable = Some(v as usize);
                                        }
                                    }
                                    _ => { self.advance(); }
                                }
                            } else {
                                break;
                            }
                            if !self.match_token(&TokenKind::Comma) { break; }
                        }
                        self.expect(&TokenKind::RParen)?;
                    }
                    let name = self.expect_identifier()?;
                    let type_ann = if self.match_token(&TokenKind::Colon) {
                        Some(self.parse_type_annotation()?)
                    } else { None };
                    let default = if self.match_token(&TokenKind::Eq) {
                        Some(self.parse_expression(0)?)
                    } else { None };
                    // Optional block with constraints
                    let mut mp = MutationProp {
                        name, type_ann, default, tracked, undoable,
                        constrain: None, validate: None, transform: None,
                        redact: false, increase_rules: Vec::new(), decrease_rules: Vec::new(),
                        span: mp_span,
                    };
                    self.skip_newlines();
                    if self.match_token(&TokenKind::LBrace) {
                        self.parse_mutation_constraints(&mut mp)?;
                        self.expect(&TokenKind::RBrace)?;
                    }
                    class.mutation_props.push(mp);
                }
                TokenKind::Bond => {
                    let bond_span = self.current_span();
                    self.advance();
                    let name = self.expect_identifier()?;
                    self.expect(&TokenKind::Colon)?;
                    let target_type = self.parse_type_annotation()?;
                    // expect "via" as identifier
                    let via_kw = self.expect_identifier()?;
                    if via_kw != "via" {
                        return Err(self.error("expected 'via' in bond declaration".into()));
                    }
                    let via = self.expect_identifier()?;
                    class.bonds.push(BondDef { name, target_type, via, span: bond_span });
                }
                TokenKind::Face => {
                    let face_span = self.current_span();
                    self.advance();
                    let name = self.expect_identifier()?;
                    self.skip_newlines();
                    self.expect(&TokenKind::LBrace)?;
                    self.skip_newlines();
                    // Parse "show field1, field2, ..."
                    let mut fields = Vec::new();
                    if let TokenKind::Identifier(ref kw) = self.peek_kind().clone() {
                        if kw == "show" {
                            self.advance();
                            fields = self.parse_identifier_list()?;
                        }
                    }
                    self.skip_newlines();
                    self.expect(&TokenKind::RBrace)?;
                    class.faces.push(FaceDef { name, visible_fields: fields, span: face_span });
                }
                TokenKind::Delegate => {
                    let del_span = self.current_span();
                    self.advance();
                    let field = self.expect_identifier()?;
                    self.expect(&TokenKind::Colon)?;
                    let target_type = self.parse_type_annotation()?;
                    class.delegates.push(DelegateDef { field, target_type, span: del_span });
                }
                TokenKind::Morph => {
                    let morph_span = self.current_span();
                    self.advance();
                    self.expect(&TokenKind::Def)?;
                    let name = self.expect_identifier()?;
                    self.expect(&TokenKind::LParen)?;
                    let params = self.parse_params()?;
                    self.expect(&TokenKind::RParen)?;
                    let return_type = if self.match_token(&TokenKind::Arrow) {
                        Some(self.parse_type_annotation()?)
                    } else { None };
                    self.skip_newlines();
                    self.expect(&TokenKind::LBrace)?;
                    let mut when_clauses = Vec::new();
                    loop {
                        self.skip_newlines();
                        if self.check(&TokenKind::RBrace) || self.is_at_end() { break; }
                        if let TokenKind::Identifier(ref kw) = self.peek_kind().clone() {
                            if kw == "when" {
                                self.advance();
                                let condition = self.parse_expression(0)?;
                                self.skip_newlines();
                                self.expect(&TokenKind::LBrace)?;
                                let body = self.parse_block()?;
                                when_clauses.push(MorphWhen { condition, body });
                                continue;
                            }
                        }
                        break;
                    }
                    self.expect(&TokenKind::RBrace)?;
                    class.morph_methods.push(MorphDef { name, params, return_type, when_clauses, span: morph_span });
                }
                TokenKind::Chromosome => {
                    let chr_span = self.current_span();
                    self.advance();
                    let name = self.expect_identifier()?;
                    self.skip_newlines();
                    self.expect(&TokenKind::LBrace)?;
                    let mut genes = Vec::new();
                    loop {
                        self.skip_newlines();
                        if self.check(&TokenKind::RBrace) || self.is_at_end() { break; }
                        if matches!(self.peek_kind(), TokenKind::Gene) {
                            genes.push(self.parse_gene_def()?);
                        } else {
                            break;
                        }
                    }
                    self.expect(&TokenKind::RBrace)?;
                    class.chromosomes.push(ChromosomeDef { name, genes, span: chr_span });
                }
                TokenKind::Fitness => {
                    let fit_span = self.current_span();
                    self.advance();
                    self.expect(&TokenKind::LParen)?;
                    let params = self.parse_params()?;
                    self.expect(&TokenKind::RParen)?;
                    let return_type = if self.match_token(&TokenKind::Arrow) {
                        Some(self.parse_type_annotation()?)
                    } else { None };
                    self.skip_newlines();
                    self.expect(&TokenKind::LBrace)?;
                    let body = self.parse_block()?;
                    class.fitness_fn = Some(FuncDef {
                        name: "fitness".into(), params, return_type,
                        body: FuncBody::Block(body), is_async: false, is_static: false,
                        decorators: Vec::new(), access: AccessModifier::Pub, span: fit_span,
                    });
                }
                TokenKind::Evolving => {
                    let ev_span = self.current_span();
                    self.advance();
                    let name = self.expect_identifier()?;
                    let type_ann = if self.match_token(&TokenKind::Colon) {
                        Some(self.parse_type_annotation()?)
                    } else { None };
                    self.expect(&TokenKind::Eq)?;
                    let default = self.parse_expression(0)?;
                    class.evolving_props.push(EvolvingProp {
                        name, type_ann, default, eval_interval: None, eval_body: None, span: ev_span,
                    });
                }
                _ => {
                    // Field: name: Type = default
                    let field_span = self.current_span();
                    let name = self.expect_identifier()?;

                    // Check for computed property: name: Type => expr
                    if self.match_token(&TokenKind::Colon) {
                        let type_ann = self.parse_type_annotation()?;
                        if self.match_token(&TokenKind::FatArrow) {
                            let body = self.parse_expression(0)?;
                            class.computed_props.push(ComputedProp {
                                name, type_ann: Some(type_ann), body, span: field_span,
                            });
                            continue;
                        }
                        let default = if self.match_token(&TokenKind::Eq) {
                            Some(self.parse_expression(0)?)
                        } else { None };
                        // Check for observed property with did_change block
                        self.skip_newlines();
                        if self.match_token(&TokenKind::LBrace) {
                            self.skip_newlines();
                            let mut did_change = Vec::new();
                            if let TokenKind::Identifier(ref kw) = self.peek_kind().clone() {
                                if kw == "did_change" {
                                    self.advance();
                                    self.expect(&TokenKind::LParen)?;
                                    let _old = self.expect_identifier()?;
                                    self.expect(&TokenKind::Comma)?;
                                    let _new = self.expect_identifier()?;
                                    self.expect(&TokenKind::RParen)?;
                                    self.skip_newlines();
                                    self.expect(&TokenKind::LBrace)?;
                                    did_change = self.parse_block()?;
                                }
                            }
                            self.skip_newlines();
                            self.expect(&TokenKind::RBrace)?;
                            class.observed_props.push(ObservedProp {
                                name, type_ann: Some(type_ann), default, did_change, span: field_span,
                            });
                        } else {
                            let field = FieldDef {
                                name, type_ann: Some(type_ann), default,
                                access: access.clone(), is_readonly, span: field_span,
                            };
                            if is_static { class.static_fields.push(field); }
                            else { class.fields.push(field); }
                        }
                    } else if self.match_token(&TokenKind::Eq) {
                        let default = self.parse_expression(0)?;
                        let field = FieldDef {
                            name, type_ann: None, default: Some(default),
                            access: access.clone(), is_readonly, span: field_span,
                        };
                        if is_static { class.static_fields.push(field); }
                        else { class.fields.push(field); }
                    } else {
                        let field = FieldDef {
                            name, type_ann: None, default: None,
                            access: access.clone(), is_readonly, span: field_span,
                        };
                        class.fields.push(field);
                    }
                }
            }
            self.skip_newlines();
        }
        Ok(())
    }

    fn parse_access_modifier(&mut self) -> AccessModifier {
        match self.peek_kind() {
            TokenKind::Pub => { self.advance(); AccessModifier::Pub }
            TokenKind::Priv => { self.advance(); AccessModifier::Priv }
            TokenKind::Prot => { self.advance(); AccessModifier::Prot }
            _ => AccessModifier::Pub,
        }
    }

    fn parse_mutation_constraints(&mut self, mp: &mut MutationProp) -> Result<(), ParseError> {
        loop {
            self.skip_newlines();
            if self.check(&TokenKind::RBrace) || self.is_at_end() { break; }
            if let TokenKind::Identifier(kw) = self.peek_kind().clone() {
                match kw.as_str() {
                    "constrain" => {
                        self.advance();
                        let range_expr = self.parse_expression(0)?;
                        if let ExprKind::Range { start, end, .. } = range_expr.kind {
                            mp.constrain = Some((*start, *end));
                        }
                    }
                    "validate" => {
                        self.advance();
                        self.expect(&TokenKind::LBrace)?;
                        let expr = self.parse_expression(0)?;
                        self.skip_newlines();
                        self.expect(&TokenKind::RBrace)?;
                        mp.validate = Some(expr);
                    }
                    "transform" => {
                        self.advance();
                        self.expect(&TokenKind::LBrace)?;
                        let expr = self.parse_expression(0)?;
                        self.skip_newlines();
                        self.expect(&TokenKind::RBrace)?;
                        mp.transform = Some(expr);
                    }
                    "redact" => {
                        self.advance();
                        mp.redact = true;
                    }
                    "rule" => {
                        self.advance();
                        let rule_kind = self.expect_identifier()?;
                        self.expect(&TokenKind::Colon)?;
                        let method_name = self.expect_identifier()?;
                        // consume () if present
                        if self.match_token(&TokenKind::LParen) {
                            self.expect(&TokenKind::RParen)?;
                        }
                        if rule_kind == "only_increase_by" {
                            mp.increase_rules.push(method_name);
                        } else if rule_kind == "only_decrease_by" {
                            mp.decrease_rules.push(method_name);
                        }
                    }
                    _ => { self.advance(); }
                }
            } else {
                break;
            }
            self.skip_newlines();
        }
        Ok(())
    }

    fn parse_gene_def(&mut self) -> Result<GeneDef, ParseError> {
        let span = self.current_span();
        self.advance(); // gene
        let name = self.expect_identifier()?;
        self.expect(&TokenKind::Colon)?;
        let type_ann = Some(self.parse_type_annotation()?);
        let default = if self.match_token(&TokenKind::Eq) {
            Some(self.parse_expression(0)?)
        } else { None };

        let mut range = None;
        let mut options = None;
        let mut step = None;
        let mut scale = None;
        let mut when_condition = None;

        self.skip_newlines();
        if self.match_token(&TokenKind::LBrace) {
            loop {
                self.skip_newlines();
                if self.check(&TokenKind::RBrace) || self.is_at_end() { break; }
                if let TokenKind::Identifier(kw) = self.peek_kind().clone() {
                    match kw.as_str() {
                        "range" => {
                            self.advance();
                            let range_expr = self.parse_expression(0)?;
                            // Extract lo..hi from the range expression
                            if let ExprKind::Range { start, end, .. } = range_expr.kind {
                                range = Some((*start, *end));
                            } else {
                                // Fallback: treat as lo, expect .., then hi
                                return Err(self.error("expected range expression (lo..hi)".into()));
                            }
                        }
                        "options" => {
                            self.advance();
                            self.expect(&TokenKind::LBracket)?;
                            let mut opts = Vec::new();
                            loop {
                                self.skip_newlines();
                                if self.check(&TokenKind::RBracket) { break; }
                                opts.push(self.parse_expression(0)?);
                                if !self.match_token(&TokenKind::Comma) { break; }
                            }
                            self.expect(&TokenKind::RBracket)?;
                            options = Some(opts);
                        }
                        "step" => {
                            self.advance();
                            step = Some(self.parse_expression(0)?);
                        }
                        "scale" => {
                            self.advance();
                            let _ = self.match_token(&TokenKind::Dot);
                            scale = Some(self.expect_identifier()?);
                        }
                        "when" => {
                            self.advance();
                            when_condition = Some(self.parse_expression(0)?);
                        }
                        _ => { self.advance(); }
                    }
                } else { break; }
                if !self.match_token(&TokenKind::Comma) {
                    // No comma required between gene constraints
                }
            }
            self.expect(&TokenKind::RBrace)?;
        }

        Ok(GeneDef { name, type_ann, default, range, options, step, scale, when_condition, span })
    }

    // ── Struct ───────────────────────────────────────────────────

    fn parse_struct_def(&mut self, decorators: Vec<Decorator>) -> Result<StmtKind, ParseError> {
        let span = self.current_span();
        self.advance(); // struct
        let name = self.expect_identifier()?;
        self.skip_newlines();
        self.expect(&TokenKind::LBrace)?;

        let mut fields = Vec::new();
        let mut methods = Vec::new();
        let mut computed_props = Vec::new();
        let mut operators = Vec::new();

        loop {
            self.skip_newlines();
            if self.check(&TokenKind::RBrace) || self.is_at_end() { break; }

            match self.peek_kind().clone() {
                TokenKind::Def => {
                    if let StmtKind::FuncDef(fd) = self.parse_func_def(false, false, Vec::new(), AccessModifier::Pub)? {
                        methods.push(fd);
                    }
                }
                TokenKind::Operator => {
                    let op_span = self.current_span();
                    self.advance();
                    let op = self.advance().lexeme.clone();
                    self.expect(&TokenKind::LParen)?;
                    let params = self.parse_params()?;
                    self.expect(&TokenKind::RParen)?;
                    let return_type = if self.match_token(&TokenKind::Arrow) {
                        Some(self.parse_type_annotation()?)
                    } else { None };
                    self.skip_newlines();
                    self.expect(&TokenKind::LBrace)?;
                    let body = self.parse_block()?;
                    operators.push(OperatorDef { op, params, return_type, body, span: op_span });
                }
                _ => {
                    let field_span = self.current_span();
                    let name = self.expect_identifier()?;
                    self.expect(&TokenKind::Colon)?;
                    let type_ann = self.parse_type_annotation()?;
                    if self.match_token(&TokenKind::FatArrow) {
                        let body = self.parse_expression(0)?;
                        computed_props.push(ComputedProp { name, type_ann: Some(type_ann), body, span: field_span });
                    } else {
                        let default = if self.match_token(&TokenKind::Eq) {
                            Some(self.parse_expression(0)?)
                        } else { None };
                        fields.push(FieldDef {
                            name, type_ann: Some(type_ann), default,
                            access: AccessModifier::Pub, is_readonly: false, span: field_span,
                        });
                    }
                }
            }
            self.skip_newlines();
        }
        self.expect(&TokenKind::RBrace)?;

        Ok(StmtKind::StructDef(StructDef { name, fields, methods, computed_props, operators, decorators, span }))
    }

    // ── Enum ─────────────────────────────────────────────────────

    fn parse_enum_def(&mut self, decorators: Vec<Decorator>) -> Result<StmtKind, ParseError> {
        let span = self.current_span();
        self.advance(); // enum
        let name = self.expect_identifier()?;
        self.skip_newlines();
        self.expect(&TokenKind::LBrace)?;

        let mut variants = Vec::new();
        let mut methods = Vec::new();

        loop {
            self.skip_newlines();
            if self.check(&TokenKind::RBrace) || self.is_at_end() { break; }

            if matches!(self.peek_kind(), TokenKind::Def) {
                if let StmtKind::FuncDef(fd) = self.parse_func_def(false, false, Vec::new(), AccessModifier::Pub)? {
                    methods.push(fd);
                }
            } else {
                let var_span = self.current_span();
                let var_name = self.expect_identifier()?;
                let mut fields = Vec::new();
                if self.match_token(&TokenKind::LParen) {
                    loop {
                        self.skip_newlines();
                        if self.check(&TokenKind::RParen) { break; }
                        let fname = self.expect_identifier()?;
                        self.expect(&TokenKind::Colon)?;
                        let ftype = self.parse_type_annotation()?;
                        fields.push(FieldDef {
                            name: fname, type_ann: Some(ftype), default: None,
                            access: AccessModifier::Pub, is_readonly: false, span: var_span.clone(),
                        });
                        if !self.match_token(&TokenKind::Comma) { break; }
                    }
                    self.expect(&TokenKind::RParen)?;
                }
                variants.push(EnumVariantDef { name: var_name, fields, span: var_span });
            }
            self.skip_newlines();
        }
        self.expect(&TokenKind::RBrace)?;

        Ok(StmtKind::EnumDef(EnumDef { name, variants, methods, decorators, span }))
    }

    // ── Interface ────────────────────────────────────────────────

    fn parse_interface_def(&mut self) -> Result<StmtKind, ParseError> {
        let span = self.current_span();
        self.advance(); // interface
        let name = self.expect_identifier()?;
        self.skip_newlines();
        self.expect(&TokenKind::LBrace)?;

        let mut methods = Vec::new();
        loop {
            self.skip_newlines();
            if self.check(&TokenKind::RBrace) || self.is_at_end() { break; }
            let m_span = self.current_span();
            self.expect(&TokenKind::Def)?;
            let m_name = self.expect_identifier()?;
            self.expect(&TokenKind::LParen)?;
            let params = self.parse_params()?;
            self.expect(&TokenKind::RParen)?;
            let return_type = if self.match_token(&TokenKind::Arrow) {
                Some(self.parse_type_annotation()?)
            } else { None };

            let has_body = { self.skip_newlines(); self.match_token(&TokenKind::LBrace) };
            let default_body = if has_body {
                Some(self.parse_block()?)
            } else { None };

            methods.push(InterfaceMethod { name: m_name, params, return_type, default_body, span: m_span });
            self.skip_newlines();
        }
        self.expect(&TokenKind::RBrace)?;

        Ok(StmtKind::InterfaceDef(InterfaceDef { name, methods, span }))
    }

    // ── Control flow ─────────────────────────────────────────────

    fn parse_if_stmt(&mut self) -> Result<StmtKind, ParseError> {
        self.advance(); // if

        // Check for "if let"
        if matches!(self.peek_kind(), TokenKind::Let) {
            return self.parse_if_let();
        }

        let condition = self.parse_expression(0)?;
        self.skip_newlines();
        self.expect(&TokenKind::LBrace)?;
        let then_block = self.parse_block()?;

        let mut else_if_blocks = Vec::new();
        let mut else_block = None;

        loop {
            self.skip_newlines();
            if !matches!(self.peek_kind(), TokenKind::Else) { break; }
            self.advance(); // else
            self.skip_newlines();
            if matches!(self.peek_kind(), TokenKind::If) {
                self.advance(); // if
                let cond = self.parse_expression(0)?;
                self.skip_newlines();
                self.expect(&TokenKind::LBrace)?;
                let block = self.parse_block()?;
                else_if_blocks.push((cond, block));
            } else {
                self.expect(&TokenKind::LBrace)?;
                else_block = Some(self.parse_block()?);
                break;
            }
        }

        Ok(StmtKind::If { condition, then_block, else_if_blocks, else_block })
    }

    fn parse_if_let(&mut self) -> Result<StmtKind, ParseError> {
        self.advance(); // let
        let pattern = self.parse_pattern()?;
        self.expect(&TokenKind::Eq)?;
        let value = self.parse_expression(0)?;
        self.skip_newlines();
        self.expect(&TokenKind::LBrace)?;
        let then_block = self.parse_block()?;
        let has_else = { self.skip_newlines(); self.match_token(&TokenKind::Else) };
        let else_block = if has_else {
            self.skip_newlines();
            self.expect(&TokenKind::LBrace)?;
            Some(self.parse_block()?)
        } else { None };

        Ok(StmtKind::IfLet { pattern, value, then_block, else_block })
    }

    fn parse_guard_stmt(&mut self) -> Result<StmtKind, ParseError> {
        self.advance(); // guard
        self.expect(&TokenKind::Let)?;
        let pattern = self.parse_pattern()?;
        self.expect(&TokenKind::Eq)?;
        let value = self.parse_expression(0)?;
        self.skip_newlines();
        self.expect(&TokenKind::Else)?;
        self.skip_newlines();
        self.expect(&TokenKind::LBrace)?;
        let else_block = self.parse_block()?;
        Ok(StmtKind::Guard { pattern, value, else_block })
    }

    fn parse_match_stmt(&mut self) -> Result<StmtKind, ParseError> {
        self.advance(); // match
        let value = self.parse_expression(0)?;
        self.skip_newlines();
        self.expect(&TokenKind::LBrace)?;
        let arms = self.parse_match_arms()?;
        self.expect(&TokenKind::RBrace)?;
        Ok(StmtKind::Match { value, arms })
    }

    fn parse_match_arms(&mut self) -> Result<Vec<MatchArm>, ParseError> {
        let mut arms = Vec::new();
        loop {
            self.skip_newlines();
            if self.check(&TokenKind::RBrace) || self.is_at_end() { break; }
            let span = self.current_span();
            let pattern = self.parse_pattern()?;
            let guard = if matches!(self.peek_kind(), TokenKind::If) {
                self.advance();
                Some(self.parse_expression(0)?)
            } else { None };
            self.expect(&TokenKind::Arrow)?;
            self.skip_newlines();
            let body = if self.match_token(&TokenKind::LBrace) {
                MatchBody::Block(self.parse_block()?)
            } else {
                MatchBody::Expression(self.parse_expression(0)?)
            };
            arms.push(MatchArm { pattern, guard, body, span });
            self.skip_newlines();
        }
        Ok(arms)
    }

    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        self.skip_newlines();
        match self.peek_kind().clone() {
            // Wildcard
            TokenKind::Identifier(ref name) if name == "_" => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            // Enum variant: .Circle(r)
            TokenKind::Dot => {
                self.advance();
                let variant = self.expect_identifier()?;
                if self.match_token(&TokenKind::LParen) {
                    let mut fields = Vec::new();
                    loop {
                        self.skip_newlines();
                        if self.check(&TokenKind::RParen) { break; }
                        fields.push(self.parse_pattern()?);
                        if !self.match_token(&TokenKind::Comma) { break; }
                    }
                    self.expect(&TokenKind::RParen)?;
                    Ok(Pattern::EnumVariant { variant, fields })
                } else {
                    Ok(Pattern::EnumVariant { variant, fields: Vec::new() })
                }
            }
            // Tuple pattern: (a, b)
            TokenKind::LParen => {
                self.advance();
                let mut pats = Vec::new();
                loop {
                    self.skip_newlines();
                    if self.check(&TokenKind::RParen) { break; }
                    pats.push(self.parse_pattern()?);
                    if !self.match_token(&TokenKind::Comma) { break; }
                }
                self.expect(&TokenKind::RParen)?;
                Ok(Pattern::Tuple(pats))
            }
            // Identifier — could be binding or destructure like Ok(val)
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                if self.match_token(&TokenKind::LParen) {
                    let mut fields = Vec::new();
                    loop {
                        self.skip_newlines();
                        if self.check(&TokenKind::RParen) { break; }
                        fields.push(self.parse_pattern()?);
                        if !self.match_token(&TokenKind::Comma) { break; }
                    }
                    self.expect(&TokenKind::RParen)?;
                    Ok(Pattern::Destructure { name, fields })
                } else {
                    Ok(Pattern::Binding(name))
                }
            }
            // Literal patterns: numbers, strings, bools
            TokenKind::IntLiteral(n) => {
                let span = self.current_span();
                self.advance();
                let lit_expr = Expr { kind: ExprKind::IntLiteral(n), span: span.clone() };
                // Check for range pattern: 1..10
                if matches!(self.peek_kind(), TokenKind::DotDot | TokenKind::DotDotEq) {
                    let inclusive = matches!(self.peek_kind(), TokenKind::DotDotEq);
                    self.advance();
                    let end = self.parse_expression(0)?;
                    Ok(Pattern::Range { start: lit_expr, end, inclusive })
                } else {
                    Ok(Pattern::Literal(lit_expr))
                }
            }
            TokenKind::FloatLiteral(n) => {
                let span = self.current_span();
                self.advance();
                Ok(Pattern::Literal(Expr { kind: ExprKind::FloatLiteral(n), span }))
            }
            TokenKind::StringLiteral(s) => {
                let s = s.clone();
                let span = self.current_span();
                self.advance();
                Ok(Pattern::Literal(Expr { kind: ExprKind::StringLiteral(s), span }))
            }
            TokenKind::True => {
                let span = self.current_span();
                self.advance();
                Ok(Pattern::Literal(Expr { kind: ExprKind::BoolLiteral(true), span }))
            }
            TokenKind::False => {
                let span = self.current_span();
                self.advance();
                Ok(Pattern::Literal(Expr { kind: ExprKind::BoolLiteral(false), span }))
            }
            TokenKind::Nil => {
                let span = self.current_span();
                self.advance();
                Ok(Pattern::Literal(Expr { kind: ExprKind::NilLiteral, span }))
            }
            _ => Err(self.error(format!("expected pattern, found {:?}", self.peek_kind()))),
        }
    }

    // ── Loops ────────────────────────────────────────────────────

    fn parse_for_loop(&mut self) -> Result<StmtKind, ParseError> {
        self.advance(); // for

        // Check for label: for:label
        let label = if matches!(self.peek_kind(), TokenKind::Colon) {
            self.advance();
            Some(self.expect_identifier()?)
        } else { None };

        // Parse pattern
        let pattern = self.parse_for_pattern()?;
        self.expect(&TokenKind::In)?;
        let iterable = self.parse_expression(0)?;

        // Optional step
        let step = if matches!(self.peek_kind(), TokenKind::Step) {
            self.advance();
            Some(self.parse_expression(0)?)
        } else { None };

        // Optional parallel modifier: |parallel| or |parallel: N|
        let parallel = if matches!(self.peek_kind(), TokenKind::Pipe) {
            self.advance(); // |
            if matches!(self.peek_kind(), TokenKind::Parallel) {
                self.advance();
                let limit = if self.match_token(&TokenKind::Colon) {
                    Some(self.parse_expression(0)?)
                } else { None };
                self.expect(&TokenKind::Pipe)?;
                Some(limit)
            } else {
                // Not a parallel modifier, backtrack? For simplicity, treat as error.
                return Err(self.error("expected 'parallel' after '|'".into()));
            }
        } else { None };

        self.skip_newlines();
        self.expect(&TokenKind::LBrace)?;
        let body = self.parse_block()?;

        Ok(StmtKind::ForLoop { label, pattern, iterable, step, parallel, body })
    }

    fn parse_for_pattern(&mut self) -> Result<ForPattern, ParseError> {
        // (a, b) destructure
        if self.match_token(&TokenKind::LParen) {
            let vars = self.parse_identifier_list()?;
            self.expect(&TokenKind::RParen)?;
            return Ok(ForPattern::Destructure(vars));
        }

        let first = self.expect_identifier()?;
        // Check for enumerate: i, item
        if self.match_token(&TokenKind::Comma) {
            let second = self.expect_identifier()?;
            Ok(ForPattern::Enumerate(first, second))
        } else {
            Ok(ForPattern::Single(first))
        }
    }

    fn parse_loop_stmt(&mut self) -> Result<StmtKind, ParseError> {
        self.advance(); // loop

        let label = if matches!(self.peek_kind(), TokenKind::Colon) {
            self.advance();
            Some(self.expect_identifier()?)
        } else { None };

        let kind = match self.peek_kind().clone() {
            TokenKind::While => {
                self.advance();
                LoopKind::While(self.parse_expression(0)?)
            }
            TokenKind::LBrace => LoopKind::Infinite,
            _ => {
                // loop N times { }
                let count = self.parse_expression(0)?;
                if matches!(self.peek_kind(), TokenKind::Times) {
                    self.advance();
                }
                LoopKind::Times(count)
            }
        };

        self.skip_newlines();
        self.expect(&TokenKind::LBrace)?;
        let body = self.parse_block()?;

        // Check for until condition after block
        let until_condition = if matches!(self.peek_kind(), TokenKind::Until) {
            self.advance();
            Some(self.parse_expression(0)?)
        } else { None };

        Ok(StmtKind::Loop { label, kind, body, until_condition })
    }

    fn parse_break(&mut self) -> Result<StmtKind, ParseError> {
        self.advance(); // break
        let label = if matches!(self.peek_kind(), TokenKind::Colon) {
            self.advance();
            Some(self.expect_identifier()?)
        } else { None };
        Ok(StmtKind::Break { label })
    }

    fn parse_next(&mut self) -> Result<StmtKind, ParseError> {
        self.advance(); // next
        let label = if matches!(self.peek_kind(), TokenKind::Colon) {
            self.advance();
            Some(self.expect_identifier()?)
        } else { None };
        let condition = if matches!(self.peek_kind(), TokenKind::If) {
            self.advance();
            Some(self.parse_expression(0)?)
        } else { None };
        Ok(StmtKind::Next { label, condition })
    }

    fn parse_return(&mut self) -> Result<StmtKind, ParseError> {
        self.advance(); // return
        let value = if self.is_statement_end() {
            None
        } else {
            Some(self.parse_expression(0)?)
        };
        Ok(StmtKind::Return(value))
    }

    fn parse_throw(&mut self) -> Result<StmtKind, ParseError> {
        self.advance(); // throw
        let expr = self.parse_expression(0)?;
        Ok(StmtKind::Throw(expr))
    }

    fn is_statement_end(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::Newline | TokenKind::Eof | TokenKind::RBrace)
    }

    // ── Try/catch ────────────────────────────────────────────────

    fn parse_try_catch(&mut self) -> Result<StmtKind, ParseError> {
        self.advance(); // try
        self.skip_newlines();
        self.expect(&TokenKind::LBrace)?;
        let try_block = self.parse_block()?;

        let mut catches = Vec::new();
        loop {
            self.skip_newlines();
            if !matches!(self.peek_kind(), TokenKind::Catch) { break; }
            let catch_span = self.current_span();
            self.advance(); // catch
            let error_type = if let TokenKind::Identifier(name) = self.peek_kind().clone() {
                if name != "as" {
                    self.advance();
                    Some(name)
                } else { None }
            } else if matches!(self.peek_kind(), TokenKind::Identifier(_)) {
                Some(self.expect_identifier()?)
            } else { None };

            let binding = if matches!(self.peek_kind(), TokenKind::As) {
                self.advance();
                Some(self.expect_identifier()?)
            } else { None };

            self.skip_newlines();
            self.expect(&TokenKind::LBrace)?;
            let body = self.parse_block()?;
            catches.push(CatchClause { error_type, binding, body, span: catch_span });
        }

        let finally_block = if { self.skip_newlines(); matches!(self.peek_kind(), TokenKind::Finally) } {
            self.advance();
            self.skip_newlines();
            self.expect(&TokenKind::LBrace)?;
            Some(self.parse_block()?)
        } else { None };

        Ok(StmtKind::TryCatch { try_block, catches, finally_block })
    }

    // ── Parallel ─────────────────────────────────────────────────

    fn parse_parallel(&mut self) -> Result<StmtKind, ParseError> {
        self.advance(); // parallel

        let is_race = if matches!(self.peek_kind(), TokenKind::Dot) {
            self.advance();
            let method = self.expect_identifier()?;
            method == "race"
        } else { false };

        let mut timeout = None;
        let mut max_concurrency = None;

        if self.match_token(&TokenKind::LParen) {
            loop {
                self.skip_newlines();
                if self.check(&TokenKind::RParen) { break; }
                if let TokenKind::Identifier(key) = self.peek_kind().clone() {
                    self.advance();
                    self.expect(&TokenKind::Colon)?;
                    let val = self.parse_expression(0)?;
                    match key.as_str() {
                        "timeout" => timeout = Some(val),
                        "max" => max_concurrency = Some(val),
                        _ => {}
                    }
                }
                if !self.match_token(&TokenKind::Comma) { break; }
            }
            self.expect(&TokenKind::RParen)?;
        }

        self.skip_newlines();
        self.expect(&TokenKind::LBrace)?;
        let tasks = self.parse_block_stmts()?;
        self.expect(&TokenKind::RBrace)?;

        Ok(StmtKind::Parallel { tasks, timeout, max_concurrency, is_race })
    }

    fn parse_after_stmt(&mut self) -> Result<StmtKind, ParseError> {
        self.advance(); // after
        self.expect(&TokenKind::LParen)?;
        let dependencies = self.parse_identifier_list()?;
        self.expect(&TokenKind::RParen)?;
        self.skip_newlines();
        self.expect(&TokenKind::LBrace)?;
        let body = self.parse_block()?;
        Ok(StmtKind::After { dependencies, body })
    }

    fn parse_mutation_stmt(&mut self) -> Result<StmtKind, ParseError> {
        self.advance(); // mutation
        self.expect(&TokenKind::Dot)?;
        let method = self.expect_identifier()?;
        if method != "atomic" {
            return Err(self.error(format!("expected 'atomic' after 'mutation.', found '{}'", method)));
        }
        self.skip_newlines();
        self.expect(&TokenKind::LBrace)?;
        let body = self.parse_block()?;
        Ok(StmtKind::MutationAtomic { body })
    }

    fn parse_device(&mut self) -> Result<StmtKind, ParseError> {
        self.advance(); // device
        self.expect(&TokenKind::LParen)?;
        self.expect(&TokenKind::Dot)?;
        let target_name = self.expect_identifier()?;
        let target = match target_name.as_str() {
            "gpu" => DeviceTarget::Gpu,
            "cpu" => DeviceTarget::Cpu,
            "quantum" => DeviceTarget::Quantum,
            _ => return Err(self.error(format!("unknown device target: {}", target_name))),
        };
        self.expect(&TokenKind::RParen)?;
        self.skip_newlines();
        self.expect(&TokenKind::LBrace)?;
        let body = self.parse_block()?;
        Ok(StmtKind::Device { target, body })
    }

    // ── Modules ──────────────────────────────────────────────────

    fn parse_use(&mut self) -> Result<StmtKind, ParseError> {
        self.advance(); // use
        let mut path = vec![self.expect_identifier()?];
        while self.match_token(&TokenKind::Dot) {
            path.push(self.expect_identifier()?);
        }
        let alias = if matches!(self.peek_kind(), TokenKind::As) {
            self.advance();
            Some(self.expect_identifier()?)
        } else { None };
        Ok(StmtKind::Use { path, alias })
    }

    fn parse_mod_block(&mut self) -> Result<StmtKind, ParseError> {
        self.advance(); // mod
        let name = self.expect_identifier()?;
        self.skip_newlines();
        self.expect(&TokenKind::LBrace)?;
        let body = self.parse_block()?;
        Ok(StmtKind::ModBlock { name, body })
    }

    fn parse_type_alias(&mut self) -> Result<StmtKind, ParseError> {
        self.advance(); // type
        let name = self.expect_identifier()?;
        self.expect(&TokenKind::Eq)?;
        let value = self.parse_type_annotation()?;
        Ok(StmtKind::TypeAlias { name, value })
    }

    // ── Weave / Extend ───────────────────────────────────────────

    fn parse_weave_def(&mut self) -> Result<StmtKind, ParseError> {
        let span = self.current_span();
        self.advance(); // weave
        let name = self.expect_identifier()?;

        let params = if self.match_token(&TokenKind::LParen) {
            let p = self.parse_params()?;
            self.expect(&TokenKind::RParen)?;
            p
        } else { Vec::new() };

        self.skip_newlines();
        self.expect(&TokenKind::LBrace)?;

        let mut before = None;
        let mut after = None;
        let mut around = None;

        loop {
            self.skip_newlines();
            if self.check(&TokenKind::RBrace) || self.is_at_end() { break; }

            // "before", "around" are identifiers; "after" is a keyword (TokenKind::After)
            let kw = match self.peek_kind() {
                TokenKind::Identifier(s) => Some(s.clone()),
                TokenKind::After => Some("after".to_string()),
                _ => None,
            };

            if let Some(kw) = kw {
                self.advance();
                match kw.as_str() {
                    "before" | "after" | "around" => {
                        self.skip_newlines();
                        self.expect(&TokenKind::LBrace)?;
                        let block = self.parse_block()?;
                        match kw.as_str() {
                            "before" => before = Some(block),
                            "after" => after = Some(block),
                            "around" => around = Some(block),
                            _ => unreachable!(),
                        }
                    }
                    _ => { /* skip unknown identifiers */ }
                }
            } else {
                break;
            }
        }
        self.expect(&TokenKind::RBrace)?;

        Ok(StmtKind::WeaveDef(WeaveDef { name, params, before, after, around, span }))
    }

    fn parse_extend_block(&mut self) -> Result<StmtKind, ParseError> {
        let span = self.current_span();
        self.advance(); // extend
        let target = self.parse_type_annotation()?;

        let where_clause = if matches!(self.peek_kind(), TokenKind::Where) {
            self.advance();
            let mut constraints = Vec::new();
            let tp = self.expect_identifier()?;
            self.expect(&TokenKind::Colon)?;
            let bound = self.expect_identifier()?;
            constraints.push(TypeConstraint { type_param: tp, bound });
            Some(constraints)
        } else { None };

        self.skip_newlines();
        self.expect(&TokenKind::LBrace)?;

        let mut methods = Vec::new();
        loop {
            self.skip_newlines();
            if self.check(&TokenKind::RBrace) || self.is_at_end() { break; }
            if let StmtKind::FuncDef(fd) = self.parse_func_def(false, false, Vec::new(), AccessModifier::Pub)? {
                methods.push(fd);
            }
        }
        self.expect(&TokenKind::RBrace)?;

        Ok(StmtKind::ExtendBlock(ExtendBlock { target, where_clause, methods, span }))
    }

    // ═══════════════════════════════════════════════════════════════
    // Expressions (Pratt parsing)
    // ═══════════════════════════════════════════════════════════════

    /// Parse an expression with Pratt parsing. `min_bp` is the minimum binding power.
    pub fn parse_expression(&mut self, min_bp: u8) -> Result<Expr, ParseError> {
        self.skip_newlines();
        let mut lhs = self.parse_prefix()?;

        loop {
            // Don't skip newlines here — newlines terminate expressions
            // in contexts like match arms, block statements, etc.
            let kind = self.peek_kind().clone();
            if matches!(kind, TokenKind::Newline | TokenKind::Eof) {
                break;
            }

            // Check for postfix operators
            if let Some(bp) = postfix_binding_power(&kind) {
                if bp < min_bp { break; }

                match kind {
                    TokenKind::Question => {
                        self.advance();
                        lhs = Expr {
                            span: lhs.span.clone(),
                            kind: ExprKind::ErrorPropagate(Box::new(lhs)),
                        };
                        continue;
                    }
                    TokenKind::Dot => {
                        self.advance();
                        let field = self.expect_identifier()?;
                        if self.match_token(&TokenKind::LParen) {
                            let args = self.parse_arguments()?;
                            self.expect(&TokenKind::RParen)?;
                            lhs = Expr {
                                span: lhs.span.clone(),
                                kind: ExprKind::MethodCall {
                                    object: Box::new(lhs), method: field, args,
                                },
                            };
                        } else {
                            lhs = Expr {
                                span: lhs.span.clone(),
                                kind: ExprKind::FieldAccess {
                                    object: Box::new(lhs), field,
                                },
                            };
                        }
                        continue;
                    }
                    TokenKind::QuestionDot => {
                        self.advance();
                        let field = self.expect_identifier()?;
                        lhs = Expr {
                            span: lhs.span.clone(),
                            kind: ExprKind::OptionalChain {
                                object: Box::new(lhs), field,
                            },
                        };
                        continue;
                    }
                    TokenKind::LBracket => {
                        self.advance();
                        let index = self.parse_expression(0)?;
                        self.expect(&TokenKind::RBracket)?;
                        lhs = Expr {
                            span: lhs.span.clone(),
                            kind: ExprKind::Index {
                                object: Box::new(lhs), index: Box::new(index),
                            },
                        };
                        continue;
                    }
                    TokenKind::LParen => {
                        self.advance();
                        let args = self.parse_arguments()?;
                        self.expect(&TokenKind::RParen)?;
                        lhs = Expr {
                            span: lhs.span.clone(),
                            kind: ExprKind::Call {
                                callee: Box::new(lhs), args,
                            },
                        };
                        continue;
                    }
                    _ => {}
                }
            }

            // Check for `as` cast
            if matches!(kind, TokenKind::As) {
                self.advance();
                let target_type = self.parse_type_annotation()?;
                lhs = Expr {
                    span: lhs.span.clone(),
                    kind: ExprKind::AsCast { value: Box::new(lhs), target_type },
                };
                continue;
            }

            // Check for infix operators
            if let Some((l_bp, r_bp)) = infix_binding_power(&kind) {
                if l_bp < min_bp { break; }

                // Special handling for ?? (nil coalescing)
                if matches!(kind, TokenKind::QuestionQuestion) {
                    self.advance();
                    let rhs = self.parse_expression(r_bp)?;
                    lhs = Expr {
                        span: lhs.span.clone(),
                        kind: ExprKind::NilCoalesce {
                            value: Box::new(lhs), default: Box::new(rhs),
                        },
                    };
                    continue;
                }

                // Special handling for |> (pipeline)
                if matches!(kind, TokenKind::PipeGt) {
                    self.advance();
                    let rhs = self.parse_expression(r_bp)?;
                    lhs = Expr {
                        span: lhs.span.clone(),
                        kind: ExprKind::Pipeline {
                            left: Box::new(lhs), right: Box::new(rhs),
                        },
                    };
                    continue;
                }

                // Special handling for ranges
                if matches!(kind, TokenKind::DotDot | TokenKind::DotDotEq) {
                    let inclusive = matches!(kind, TokenKind::DotDotEq);
                    self.advance();
                    let rhs = self.parse_expression(r_bp)?;
                    let step = if matches!(self.peek_kind(), TokenKind::Step) {
                        self.advance();
                        Some(Box::new(self.parse_expression(r_bp)?))
                    } else { None };
                    lhs = Expr {
                        span: lhs.span.clone(),
                        kind: ExprKind::Range {
                            start: Box::new(lhs), end: Box::new(rhs), inclusive, step,
                        },
                    };
                    continue;
                }

                if let Some(op) = token_to_binary_op(&kind) {
                    self.advance();
                    let rhs = self.parse_expression(r_bp)?;
                    lhs = Expr {
                        span: lhs.span.clone(),
                        kind: ExprKind::Binary {
                            left: Box::new(lhs), op, right: Box::new(rhs),
                        },
                    };
                    continue;
                }
            }

            break;
        }

        Ok(lhs)
    }

    /// Parse a prefix expression (atoms, unary ops, literals, etc.)
    fn parse_prefix(&mut self) -> Result<Expr, ParseError> {
        let span = self.current_span();
        let kind = self.peek_kind().clone();

        match kind {
            // Unary operators
            TokenKind::Minus | TokenKind::Bang | TokenKind::Not | TokenKind::Tilde => {
                let op = match &kind {
                    TokenKind::Minus => UnaryOp::Neg,
                    TokenKind::Bang | TokenKind::Not => UnaryOp::Not,
                    TokenKind::Tilde => UnaryOp::BitNot,
                    _ => unreachable!(),
                };
                self.advance();
                let bp = prefix_binding_power(&kind).unwrap();
                let operand = self.parse_expression(bp)?;
                Ok(Expr { kind: ExprKind::Unary { op, operand: Box::new(operand) }, span })
            }

            // Await
            TokenKind::Await => {
                self.advance();
                let expr = self.parse_expression(29)?;
                Ok(Expr { kind: ExprKind::Await(Box::new(expr)), span })
            }

            // Literals
            TokenKind::IntLiteral(n) => { self.advance(); Ok(Expr { kind: ExprKind::IntLiteral(n), span }) }
            TokenKind::FloatLiteral(n) => { self.advance(); Ok(Expr { kind: ExprKind::FloatLiteral(n), span }) }
            TokenKind::StringLiteral(s) => {
                let s = s.clone(); self.advance();
                Ok(Expr { kind: ExprKind::StringLiteral(s), span })
            }
            TokenKind::MultilineString(s) => {
                let s = s.clone(); self.advance();
                Ok(Expr { kind: ExprKind::StringLiteral(s), span })
            }
            TokenKind::RawString(s) => {
                let s = s.clone(); self.advance();
                Ok(Expr { kind: ExprKind::StringLiteral(s), span })
            }
            TokenKind::InterpolatedString(parts) => {
                let parts = parts.clone();
                self.advance();
                let interp_parts: Vec<StringInterp> = parts.into_iter().map(|p| match p {
                    StringPart::Literal(s) => StringInterp::Literal(s),
                    StringPart::Expression(s) => {
                        // Re-lex and parse the expression inside {}
                        let mut scanner = crate::lexer::scanner::Scanner::new(&s, span.file.clone());
                        let tokens = scanner.scan_tokens();
                        let mut parser = Parser::new(tokens);
                        match parser.parse_expression(0) {
                            Ok(expr) => StringInterp::Expr(expr),
                            Err(_) => StringInterp::Literal(format!("{{{}}}", s)),
                        }
                    }
                }).collect();
                Ok(Expr { kind: ExprKind::InterpolatedString(interp_parts), span })
            }
            TokenKind::CharLiteral(c) => { self.advance(); Ok(Expr { kind: ExprKind::CharLiteral(c), span }) }
            TokenKind::True => { self.advance(); Ok(Expr { kind: ExprKind::BoolLiteral(true), span }) }
            TokenKind::False => { self.advance(); Ok(Expr { kind: ExprKind::BoolLiteral(false), span }) }
            TokenKind::Nil => { self.advance(); Ok(Expr { kind: ExprKind::NilLiteral, span }) }
            TokenKind::SelfKw => { self.advance(); Ok(Expr { kind: ExprKind::SelfExpr, span }) }
            TokenKind::Super => { self.advance(); Ok(Expr { kind: ExprKind::SuperExpr, span }) }

            // Identifier (or lambda if followed by ->), including Ok/Err constructors
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();

                // Ok(value) constructor
                if name == "Ok" && matches!(self.peek_kind(), TokenKind::LParen) {
                    self.advance(); // (
                    let val = self.parse_expression(0)?;
                    self.expect(&TokenKind::RParen)?;
                    return Ok(Expr { kind: ExprKind::ResultOk(Box::new(val)), span });
                }

                // Err(value) constructor
                if name == "Err" && matches!(self.peek_kind(), TokenKind::LParen) {
                    self.advance(); // (
                    let val = self.parse_expression(0)?;
                    self.expect(&TokenKind::RParen)?;
                    return Ok(Expr { kind: ExprKind::ResultErr(Box::new(val)), span });
                }

                // Check for lambda: x -> expr
                if matches!(self.peek_kind(), TokenKind::Arrow) {
                    self.advance(); // ->
                    let body = self.parse_expression(0)?;
                    return Ok(Expr {
                        kind: ExprKind::Lambda {
                            params: vec![Param { name, type_ann: None, default: None, variadic: false, kw_variadic: false }],
                            body: Box::new(body),
                        },
                        span,
                    });
                }

                Ok(Expr { kind: ExprKind::Identifier(name), span })
            }

            // Enum variant: .Name or .Name(args)
            TokenKind::Dot => {
                self.advance();
                let name = self.expect_identifier()?;
                if self.match_token(&TokenKind::LParen) {
                    let mut args = Vec::new();
                    loop {
                        self.skip_newlines();
                        if self.check(&TokenKind::RParen) { break; }
                        args.push(self.parse_expression(0)?);
                        if !self.match_token(&TokenKind::Comma) { break; }
                    }
                    self.expect(&TokenKind::RParen)?;
                    Ok(Expr { kind: ExprKind::EnumVariant { name, args }, span })
                } else {
                    Ok(Expr { kind: ExprKind::EnumVariant { name, args: Vec::new() }, span })
                }
            }

            // Parenthesized expression, tuple, or lambda params
            TokenKind::LParen => {
                self.advance();
                self.skip_newlines();

                // Empty tuple
                if self.match_token(&TokenKind::RParen) {
                    return Ok(Expr { kind: ExprKind::TupleLiteral(Vec::new()), span });
                }

                let first = self.parse_expression(0)?;
                self.skip_newlines();

                // Check if this is a tuple or grouped expression
                if self.match_token(&TokenKind::Comma) {
                    // Tuple or lambda params
                    let mut exprs = vec![first];
                    loop {
                        self.skip_newlines();
                        if self.check(&TokenKind::RParen) { break; }
                        exprs.push(self.parse_expression(0)?);
                        if !self.match_token(&TokenKind::Comma) { break; }
                    }
                    self.expect(&TokenKind::RParen)?;

                    // Check for lambda: (a, b) -> expr
                    if matches!(self.peek_kind(), TokenKind::Arrow) {
                        self.advance();
                        let params: Vec<Param> = exprs.into_iter().map(|e| {
                            let name = if let ExprKind::Identifier(n) = e.kind { n } else { "_".into() };
                            Param { name, type_ann: None, default: None, variadic: false, kw_variadic: false }
                        }).collect();
                        let body = self.parse_expression(0)?;
                        return Ok(Expr { kind: ExprKind::Lambda { params, body: Box::new(body) }, span });
                    }

                    Ok(Expr { kind: ExprKind::TupleLiteral(exprs), span })
                } else {
                    self.expect(&TokenKind::RParen)?;

                    // Check for lambda: (x) -> expr
                    if matches!(self.peek_kind(), TokenKind::Arrow) {
                        if let ExprKind::Identifier(name) = &first.kind {
                            let name = name.clone();
                            self.advance(); // ->
                            let body = self.parse_expression(0)?;
                            return Ok(Expr {
                                kind: ExprKind::Lambda {
                                    params: vec![Param { name, type_ann: None, default: None, variadic: false, kw_variadic: false }],
                                    body: Box::new(body),
                                },
                                span,
                            });
                        }
                    }

                    // Just a grouped expression
                    Ok(first)
                }
            }

            // List literal or comprehension: [...]
            TokenKind::LBracket => self.parse_list_or_comprehension(),

            // Map/Set literal or block: { ... }
            TokenKind::LBrace => self.parse_brace_expr(),

            // If expression
            TokenKind::If => {
                self.advance();
                let condition = self.parse_expression(0)?;
                self.expect(&TokenKind::Then)?;
                let then_expr = self.parse_expression(0)?;
                self.expect(&TokenKind::Else)?;
                let else_expr = self.parse_expression(0)?;
                Ok(Expr {
                    kind: ExprKind::IfExpr {
                        condition: Box::new(condition),
                        then_expr: Box::new(then_expr),
                        else_expr: Box::new(else_expr),
                    },
                    span,
                })
            }

            // Match expression
            TokenKind::Match => {
                self.advance();
                let value = self.parse_expression(0)?;
                self.skip_newlines();
                self.expect(&TokenKind::LBrace)?;
                let arms = self.parse_match_arms()?;
                self.expect(&TokenKind::RBrace)?;
                Ok(Expr { kind: ExprKind::MatchExpr { value: Box::new(value), arms }, span })
            }

            // Evolve expression
            TokenKind::Evolve => {
                self.advance();
                let target = self.expect_identifier()?;
                self.skip_newlines();
                self.expect(&TokenKind::LBrace)?;
                let config = self.parse_evolve_config()?;
                self.expect(&TokenKind::RBrace)?;
                Ok(Expr { kind: ExprKind::EvolveBlock { target, config }, span })
            }

            // Crossover
            TokenKind::Crossover => {
                self.advance();
                self.expect(&TokenKind::LParen)?;
                let a = self.parse_expression(0)?;
                self.expect(&TokenKind::Comma)?;
                let b = self.parse_expression(0)?;
                self.expect(&TokenKind::RParen)?;
                Ok(Expr { kind: ExprKind::Crossover { parent_a: Box::new(a), parent_b: Box::new(b) }, span })
            }

            // Breed
            TokenKind::Breed => {
                self.advance();
                self.expect(&TokenKind::LParen)?;
                let a = self.parse_expression(0)?;
                self.expect(&TokenKind::Comma)?;
                let b = self.parse_expression(0)?;
                let rate = if self.match_token(&TokenKind::Comma) {
                    // mutation_rate: value
                    self.expect_identifier()?; // "mutation_rate"
                    self.expect(&TokenKind::Colon)?;
                    Some(Box::new(self.parse_expression(0)?))
                } else { None };
                self.expect(&TokenKind::RParen)?;
                Ok(Expr { kind: ExprKind::Breed { parent_a: Box::new(a), parent_b: Box::new(b), mutation_rate: rate }, span })
            }

            // Keywords that can be used as identifiers in expression context
            _ => {
                if let Some(name) = self.keyword_as_identifier() {
                    self.advance();
                    // Check for lambda
                    if matches!(self.peek_kind(), TokenKind::Arrow) {
                        self.advance();
                        let body = self.parse_expression(0)?;
                        return Ok(Expr {
                            kind: ExprKind::Lambda {
                                params: vec![Param { name, type_ann: None, default: None, variadic: false, kw_variadic: false }],
                                body: Box::new(body),
                            },
                            span,
                        });
                    }
                    Ok(Expr { kind: ExprKind::Identifier(name), span })
                } else {
                    Err(self.error(format!("expected expression, found {:?}", self.peek_kind())))
                }
            }
        }
    }

    fn parse_list_or_comprehension(&mut self) -> Result<Expr, ParseError> {
        let span = self.current_span();
        self.advance(); // [
        self.skip_newlines();

        if self.match_token(&TokenKind::RBracket) {
            return Ok(Expr { kind: ExprKind::ListLiteral(Vec::new()), span });
        }

        let first = self.parse_expression(0)?;
        self.skip_newlines();

        // Check for comprehension: [expr for x in iter]
        if matches!(self.peek_kind(), TokenKind::For) {
            self.advance();
            let var = self.expect_identifier()?;
            self.expect(&TokenKind::In)?;
            let iterable = self.parse_expression(0)?;
            let condition = if matches!(self.peek_kind(), TokenKind::If) {
                self.advance();
                Some(Box::new(self.parse_expression(0)?))
            } else { None };
            self.skip_newlines();
            self.expect(&TokenKind::RBracket)?;
            return Ok(Expr {
                kind: ExprKind::Comprehension {
                    expr: Box::new(first), var, iterable: Box::new(iterable),
                    condition, kind: ComprehensionKind::List,
                },
                span,
            });
        }

        // Regular list
        let mut exprs = vec![first];
        while self.match_token(&TokenKind::Comma) {
            self.skip_newlines();
            if self.check(&TokenKind::RBracket) { break; }
            exprs.push(self.parse_expression(0)?);
            self.skip_newlines();
        }
        self.expect(&TokenKind::RBracket)?;
        Ok(Expr { kind: ExprKind::ListLiteral(exprs), span })
    }

    fn parse_brace_expr(&mut self) -> Result<Expr, ParseError> {
        let span = self.current_span();
        self.advance(); // {
        self.skip_newlines();

        // Empty map/set
        if self.match_token(&TokenKind::RBrace) {
            return Ok(Expr { kind: ExprKind::MapLiteral(Vec::new()), span });
        }

        let first = self.parse_expression(0)?;
        self.skip_newlines();

        // Map literal: {key: value, ...}
        if matches!(self.peek_kind(), TokenKind::Colon) {
            self.advance();
            let val = self.parse_expression(0)?;
            let mut pairs = vec![(first, val)];
            while self.match_token(&TokenKind::Comma) {
                self.skip_newlines();
                if self.check(&TokenKind::RBrace) { break; }
                let k = self.parse_expression(0)?;
                self.expect(&TokenKind::Colon)?;
                let v = self.parse_expression(0)?;
                pairs.push((k, v));
                self.skip_newlines();
            }
            self.skip_newlines();
            self.expect(&TokenKind::RBrace)?;
            return Ok(Expr { kind: ExprKind::MapLiteral(pairs), span });
        }

        // Set literal: {1, 2, 3}
        if matches!(self.peek_kind(), TokenKind::Comma) {
            let mut exprs = vec![first];
            while self.match_token(&TokenKind::Comma) {
                self.skip_newlines();
                if self.check(&TokenKind::RBrace) { break; }
                exprs.push(self.parse_expression(0)?);
                self.skip_newlines();
            }
            self.skip_newlines();
            self.expect(&TokenKind::RBrace)?;
            return Ok(Expr { kind: ExprKind::SetLiteral(exprs), span });
        }

        // Single-expression block/set with one element
        self.skip_newlines();
        self.expect(&TokenKind::RBrace)?;
        Ok(Expr { kind: ExprKind::SetLiteral(vec![first]), span })
    }

    fn parse_evolve_config(&mut self) -> Result<EvolveConfig, ParseError> {
        let mut config = EvolveConfig {
            population: None, generations: None, mutation_rate: None,
            crossover_rate: None, selection: None, elitism: None, fitness_data: None,
        };

        loop {
            self.skip_newlines();
            if self.check(&TokenKind::RBrace) || self.is_at_end() { break; }
            let key = self.expect_identifier()?;
            match key.as_str() {
                "population" => { self.expect(&TokenKind::Colon)?; config.population = Some(Box::new(self.parse_expression(0)?)); }
                "generations" => { self.expect(&TokenKind::Colon)?; config.generations = Some(Box::new(self.parse_expression(0)?)); }
                "mutation_rate" => { self.expect(&TokenKind::Colon)?; config.mutation_rate = Some(Box::new(self.parse_expression(0)?)); }
                "crossover_rate" => { self.expect(&TokenKind::Colon)?; config.crossover_rate = Some(Box::new(self.parse_expression(0)?)); }
                "elitism" => { self.expect(&TokenKind::Colon)?; config.elitism = Some(Box::new(self.parse_expression(0)?)); }
                "selection" => {
                    self.expect(&TokenKind::Colon)?;
                    self.expect(&TokenKind::Dot)?;
                    let method = self.expect_identifier()?;
                    config.selection = match method.as_str() {
                        "tournament" => {
                            let size = if self.match_token(&TokenKind::LParen) {
                                self.expect_identifier()?; // "size"
                                self.expect(&TokenKind::Colon)?;
                                let s = self.parse_expression(0)?;
                                self.expect(&TokenKind::RParen)?;
                                Some(Box::new(s))
                            } else { None };
                            Some(SelectionMethod::Tournament(size))
                        }
                        "roulette" => Some(SelectionMethod::Roulette),
                        "rank" => Some(SelectionMethod::Rank),
                        _ => None,
                    };
                }
                "fitness" => {
                    // fitness on data: expr
                    let on_kw = self.expect_identifier()?; // "on"
                    if on_kw != "on" {
                        return Err(self.error("expected 'on' after 'fitness'".into()));
                    }
                    let param = self.expect_identifier()?;
                    self.expect(&TokenKind::Colon)?;
                    let val = self.parse_expression(0)?;
                    config.fitness_data = Some((param, Box::new(val)));
                }
                _ => { self.advance(); } // skip unknown
            }
            self.skip_newlines();
        }
        Ok(config)
    }

    // ═══════════════════════════════════════════════════════════════
    // Type annotations
    // ═══════════════════════════════════════════════════════════════

    pub fn parse_type_annotation(&mut self) -> Result<TypeAnnotation, ParseError> {
        self.skip_newlines();

        // Map shorthand: {K: V}
        if self.match_token(&TokenKind::LBrace) {
            let key = self.parse_type_annotation()?;
            self.expect(&TokenKind::Colon)?;
            let val = self.parse_type_annotation()?;
            self.expect(&TokenKind::RBrace)?;
            return Ok(TypeAnnotation::MapType(Box::new(key), Box::new(val)));
        }

        // Tuple/function type: (A, B) or (A, B) -> C
        if self.match_token(&TokenKind::LParen) {
            let mut types = Vec::new();
            loop {
                self.skip_newlines();
                if self.check(&TokenKind::RParen) { break; }
                types.push(self.parse_type_annotation()?);
                if !self.match_token(&TokenKind::Comma) { break; }
            }
            self.expect(&TokenKind::RParen)?;
            if self.match_token(&TokenKind::Arrow) {
                let ret = self.parse_type_annotation()?;
                return Ok(TypeAnnotation::FuncType(types, Box::new(ret)));
            }
            return Ok(TypeAnnotation::TupleType(types));
        }

        // Self type
        if matches!(self.peek_kind(), TokenKind::SelfKw) {
            self.advance();
            return Ok(TypeAnnotation::SelfType);
        }

        // Named type
        let name = self.expect_identifier()?;

        // Generic: Name<T, V>
        if self.match_token(&TokenKind::Lt) {
            let mut params = Vec::new();
            loop {
                self.skip_newlines();
                if self.check(&TokenKind::Gt) { break; }
                params.push(self.parse_type_annotation()?);
                if !self.match_token(&TokenKind::Comma) { break; }
            }
            self.expect(&TokenKind::Gt)?;
            let mut ty = TypeAnnotation::Generic(name, params);
            // Check for optional ?
            if self.match_token(&TokenKind::Question) {
                ty = TypeAnnotation::Optional(Box::new(ty));
            }
            return Ok(ty);
        }

        // Array shorthand: Name[]
        if self.match_token(&TokenKind::LBracket) {
            self.expect(&TokenKind::RBracket)?;
            let mut ty = TypeAnnotation::Array(Box::new(TypeAnnotation::Simple(name)));
            if self.match_token(&TokenKind::Question) {
                ty = TypeAnnotation::Optional(Box::new(ty));
            }
            return Ok(ty);
        }

        let mut ty = TypeAnnotation::Simple(name);

        // Dimensional type: Type.unit
        if matches!(self.peek_kind(), TokenKind::Dot) {
            // Peek to see if this is a dimensional suffix (single lowercase identifier)
            if let Some(next) = self.tokens.get(self.pos + 1) {
                if let TokenKind::Identifier(ref unit) = next.kind {
                    if unit.chars().next().is_some_and(|c| c.is_lowercase()) {
                        self.advance(); // .
                        let unit = self.expect_identifier()?;
                        ty = TypeAnnotation::Dimensional(Box::new(ty), unit);
                    }
                }
            }
        }

        // Optional: Type?
        if self.match_token(&TokenKind::Question) {
            ty = TypeAnnotation::Optional(Box::new(ty));
        }

        Ok(ty)
    }

    // ═══════════════════════════════════════════════════════════════
    // Helpers
    // ═══════════════════════════════════════════════════════════════

    /// Parse a { ... } block, returning the list of statements. Consumes the closing }.
    fn parse_block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let stmts = self.parse_block_stmts()?;
        self.expect(&TokenKind::RBrace)?;
        Ok(stmts)
    }

    fn parse_block_stmts(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = Vec::new();
        loop {
            self.skip_newlines();
            if self.check(&TokenKind::RBrace) || self.is_at_end() { break; }
            stmts.push(self.parse_statement()?);
            self.skip_newlines();
        }
        Ok(stmts)
    }

    fn parse_identifier_list(&mut self) -> Result<Vec<String>, ParseError> {
        let mut names = Vec::new();
        names.push(self.expect_identifier()?);
        while self.match_token(&TokenKind::Comma) {
            self.skip_newlines();
            names.push(self.expect_identifier()?);
        }
        Ok(names)
    }
}

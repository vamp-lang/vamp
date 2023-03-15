use crate::ast::{
    BuiltIn, Expr, Import, Module, Pattern, PatternTupleMember, Statement, TupleMember,
};
use crate::lex::{tokenize, Token, TokenKind};
use crate::source::{Error, ErrorKind, Result, Span};
use crate::symbol::{Interner, Symbol};
use bumpalo::{
    collections::{String as BumpString, Vec as BumpVec},
    Bump,
};

pub struct Parser<'ast, 'src, 'sym> {
    source: &'src str,
    arena: &'ast Bump,
    interner: &'sym mut Interner,
    tokens: Vec<Token>,
    index: usize,
}

impl<'ast, 'src, 'sym> Parser<'ast, 'src, 'sym> {
    fn accept(&mut self, kind: TokenKind) -> Option<Span> {
        if self.index < self.tokens.len() && self.tokens[self.index].kind == kind {
            let span = self.tokens[self.index].span;
            self.index += 1;
            Some(span)
        } else {
            None
        }
    }

    fn accept_slice(&mut self, kind: TokenKind) -> Option<&str> {
        self.accept(kind).map(|span| &self.source[span])
    }

    fn accept_symbol(&mut self, kind: TokenKind) -> Option<Symbol> {
        self.accept_slice(kind)
            .map(|slice| self.interner.intern(slice))
    }

    fn accept_built_in(&mut self) -> Option<(BuiltIn, u8, u8)> {
        if self.index < self.tokens.len() {
            let result = match self.tokens[self.index].kind {
                TokenKind::Plus => (BuiltIn::Add, 1, 2),
                TokenKind::Minus => (BuiltIn::Sub, 1, 2),
                TokenKind::Times => (BuiltIn::Mul, 3, 4),
                TokenKind::Divide => (BuiltIn::Div, 3, 4),
                _ => return None,
            };
            self.index += 1;
            Some(result)
        } else {
            None
        }
    }

    fn invalid_token(&self) -> Error {
        Error {
            kind: ErrorKind::InvalidToken,
            span: self
                .tokens
                .get(self.index)
                .unwrap_or(&self.tokens[self.index - 1])
                .span,
        }
    }

    fn identifier(&mut self) -> Option<Symbol> {
        self.accept_symbol(TokenKind::Identifier)
    }

    fn symbol(&mut self) -> Option<Symbol> {
        self.accept_symbol(TokenKind::Symbol)
    }

    fn string(&mut self) -> Result<Option<&str>> {
        if let Some(span) = self.accept(TokenKind::String) {
            let slice = &self.source[span];
            let mut string = BumpString::with_capacity_in(slice.len(), self.arena);
            let mut chars = slice[1..slice.len() - 1].chars();
            while let Some(c) = chars.next() {
                if c == '\\' {
                    let error = Error {
                        kind: ErrorKind::StringInvalidEscapeSequence,
                        span,
                    };
                    // `unwrap()` here is safe because a string ending `\` such
                    // as `"\"` would fail with `UnterminatedString`.
                    let c = chars.next().unwrap();
                    match c {
                        '\\' => string.push('\\'),
                        '"' => string.push('"'),
                        // Bell
                        'a' => string.push('\x07'),
                        // Backspace
                        'b' => string.push('\x08'),
                        // Horizontal tab
                        't' => string.push('\t'),
                        // Vertical tab
                        'v' => string.push('\x0B'),
                        // Form feed
                        'f' => string.push('\x0C'),
                        // Newline
                        'n' => {
                            string.push('\n');
                        }
                        // Carriage return
                        'r' => {
                            string.push('\r');
                        }
                        // Nul
                        '0' => {
                            string.push('\0');
                        }
                        // Hexidecimal
                        'x' => {
                            let a = chars.next().ok_or(error)?;
                            let b = chars.next().ok_or(error)?;
                            let value =
                                16 * match a {
                                    '0'..='9' => a as u8 - b'0',
                                    'a'..='f' => 10 + a as u8 - b'a',
                                    'A'..='F' => 10 + a as u8 - b'A',
                                    _ => return Err(error),
                                } + match b {
                                    '0'..='9' => b as u8 - b'0',
                                    'a'..='f' => 10 + b as u8 - b'a',
                                    'A'..='F' => 10 + b as u8 - b'A',
                                    _ => return Err(error),
                                };
                            if value > 127 {
                                return Err(error);
                            }
                            string.push(value as char);
                        }
                        _ => return Err(error),
                    }
                } else {
                    string.push(c)
                }
            }
            Ok(Some(&string))
        } else {
            Ok(None)
        }
    }

    fn int(&mut self) -> Result<Option<i64>> {
        if let Some(integer_span) = self.accept(TokenKind::Int) {
            let mut value: i64 = 0;
            let error = Error {
                kind: ErrorKind::IntegerInvalid,
                span: integer_span,
            };
            for digit in self.source[integer_span].bytes() {
                value = value
                    .checked_mul(10)
                    .ok_or(error)?
                    .checked_add((digit - b'0') as i64)
                    .ok_or(error)?;
            }
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    fn float(&mut self) -> Result<Option<f64>> {
        if let Some(float_span) = self.accept(TokenKind::Float) {
            // TODO: Write custom float parser.
            let value = self.source[float_span].parse::<f64>().map_err(|_| Error {
                kind: ErrorKind::FloatInvalid,
                span: float_span,
            })?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    fn tuple_member(&mut self) -> Result<Option<TupleMember>> {
        if let Some(identifier) = self.identifier() {
            if self.accept(TokenKind::Colon).is_some() {
                let expr = self.expr()?.unwrap_or_else(|| Expr::Identifier(identifier));
                Ok(Some(TupleMember::Named(identifier, expr)))
            } else {
                Ok(Some(TupleMember::Positional(Expr::Identifier(identifier))))
            }
        } else if let Some(expr) = self.expr()? {
            Ok(Some(TupleMember::Positional(expr)))
        } else {
            Ok(None)
        }
    }

    fn tuple(&mut self) -> Result<Option<&[TupleMember]>> {
        let i = self.index;
        if let Some(left_parenthesis_span) = self.accept(TokenKind::LParen) {
            let mut members = BumpVec::new_in(self.arena);
            if let Some(member) = self.tuple_member()? {
                members.push(member);
                while let Some(comma_span) = self.accept(TokenKind::Comma) {
                    if let Some(member) = self.tuple_member()? {
                        members.push(member);
                    }
                }
            }
            self.accept(TokenKind::RParen).ok_or(Error {
                kind: ErrorKind::UnbalancedDelimiters,
                span: left_parenthesis_span,
            })?;
            Ok(Some(&members))
        } else {
            self.index = i;
            Ok(None)
        }
    }

    fn vector(&mut self) -> Result<Option<&[Expr]>> {
        if let Some(left_bracket_span) = self.accept(TokenKind::LBracket) {
            let mut exprs = BumpVec::new_in(self.arena);
            if let Some(expr) = self.expr()? {
                exprs.push(expr);
                while self.accept(TokenKind::Comma).is_some() {
                    if let Some(expr) = self.expr()? {
                        exprs.push(expr);
                    }
                }
            }
            self.accept(TokenKind::RBracket).ok_or(Error {
                kind: ErrorKind::UnbalancedDelimiters,
                span: left_bracket_span,
            })?;
            Ok(Some(&exprs))
        } else {
            Ok(None)
        }
    }

    fn import(&mut self) -> Result<Option<Import>> {
        if let Some(pattern) = self.pattern()? {
            if let Some(string) = self.string()? {
                return Ok(Some(Import(pattern, string)));
            }
        }
        Ok(None)
    }

    fn imports(&mut self) -> Result<&[Import]> {
        let mut imports = BumpVec::new_in(self.arena);
        if let Some(import_span) = self.accept(TokenKind::Import) {
            let left_parenthesis_span = self
                .accept(TokenKind::LParen)
                .ok_or_else(|| self.invalid_token())?;
            if let Some(import) = self.import()? {
                imports.push(import);
                while self.accept(TokenKind::Comma).is_some() {
                    if let Some(import) = self.import()? {
                        imports.push(import);
                    }
                }
            }
            self.accept(TokenKind::RParen).ok_or(Error {
                kind: ErrorKind::UnbalancedDelimiters,
                span: left_parenthesis_span,
            })?;
        }
        Ok(&imports)
    }

    fn pattern_tuple_member(&mut self) -> Result<Option<PatternTupleMember>> {
        if let Some(identifier) = self.identifier() {
            if self.accept(TokenKind::Colon).is_some() {
                let pattern = self
                    .pattern()?
                    .unwrap_or_else(|| Pattern::Identifier(identifier));
                Ok(Some(PatternTupleMember::Named(identifier, pattern)))
            } else {
                Ok(Some(PatternTupleMember::Positional(Pattern::Identifier(
                    identifier,
                ))))
            }
        } else if let Some(pattern) = self.pattern()? {
            Ok(Some(PatternTupleMember::Positional(pattern)))
        } else {
            Ok(None)
        }
    }

    fn pattern_tuple(&mut self) -> Result<Option<&[PatternTupleMember]>> {
        let i = self.index;
        if let Some(left_parenthesis_span) = self.accept(TokenKind::LParen) {
            let mut members = BumpVec::new_in(self.arena);
            if let Some(member) = self.pattern_tuple_member()? {
                members.push(member);
                while let Some(comma_span) = self.accept(TokenKind::Comma) {
                    if let Some(member) = self.pattern_tuple_member()? {
                        members.push(member);
                    }
                }
            }
            self.accept(TokenKind::RParen).ok_or(Error {
                kind: ErrorKind::UnbalancedDelimiters,
                span: left_parenthesis_span,
            })?;
            Ok(Some(&members))
        } else {
            self.index = i;
            Ok(None)
        }
    }

    fn pattern(&mut self) -> Result<Option<Pattern>> {
        if let Some(members) = self.pattern_tuple()? {
            if members.len() == 0 {
                Ok(Some(Pattern::Nil))
            } else {
                Ok(Some(Pattern::Tuple(members)))
            }
        } else if let Some(identifier) = self.identifier() {
            Ok(Some(Pattern::Identifier(identifier)))
        } else {
            Ok(None)
        }
    }

    fn statement(&mut self) -> Result<Option<Statement>> {
        if self.accept(TokenKind::Let).is_some() {
            let pattern = self.pattern()?.ok_or_else(|| self.invalid_token())?;
            let args = self.pattern_tuple()?;
            self.accept(TokenKind::Equals)
                .ok_or_else(|| self.invalid_token())?;
            let expr = self.expr()?.ok_or_else(|| self.invalid_token())?;
            if let Some(args) = args {
                Ok(Some(Statement::Let(
                    pattern,
                    self.arena.alloc(Expr::Function(
                        self.arena.alloc(args),
                        self.arena.alloc(expr),
                    )),
                )))
            } else {
                Ok(Some(Statement::Let(pattern, self.arena.alloc(expr))))
            }
        } else if let Some(expr) = self.expr()? {
            Ok(Some(Statement::Expr(expr)))
        } else {
            Ok(None)
        }
    }

    fn statements(&mut self) -> Result<&[Statement]> {
        let mut statements = BumpVec::new_in(self.arena);
        if let Some(statement) = self.statement()? {
            statements.push(statement);
            while self.accept(TokenKind::Comma).is_some() {
                if let Some(statement) = self.statement()? {
                    statements.push(statement);
                }
            }
        }
        Ok(&statements)
    }

    fn block(&mut self) -> Result<Option<Expr>> {
        if let Some(left_brace_span) = self.accept(TokenKind::LBrace) {
            let statements = self.statements()?;
            self.accept(TokenKind::RBrace).ok_or(Error {
                kind: ErrorKind::UnbalancedDelimiters,
                span: left_brace_span,
            })?;
            Ok(Some(Expr::Block(&statements)))
        } else {
            Ok(None)
        }
    }

    fn atom(&mut self) -> Result<Option<Expr>> {
        if let Some(members) = self.tuple()? {
            if members.len() == 0 {
                Ok(Some(Expr::Nil))
            } else {
                Ok(Some(Expr::Tuple(members)))
            }
        } else if let Some(vector) = self.vector()? {
            Ok(Some(Expr::Vector(vector)))
        } else if let Some(block) = self.block()? {
            Ok(Some(block))
        } else if let Some(identifier) = self.identifier() {
            Ok(Some(Expr::Identifier(identifier)))
        } else if let Some(symbol) = self.symbol() {
            Ok(Some(Expr::Symbol(symbol)))
        } else if let Some(string) = self.string()? {
            Ok(Some(Expr::String(string)))
        } else if let Some(integer) = self.int()? {
            Ok(Some(Expr::Int(integer)))
        } else if let Some(float) = self.float()? {
            Ok(Some(Expr::Float(float)))
        } else {
            Ok(None)
        }
    }

    fn function(&mut self) -> Result<Option<Expr>> {
        let i = self.index;
        if let Some(args) = self.pattern_tuple()? {
            if self.accept(TokenKind::Arrow).is_some() {
                if let Some(expr) = self.expr()? {
                    return Ok(Some(Expr::Function(args, self.arena.alloc(expr))));
                } else {
                    return Err(self.invalid_token());
                }
            }
        }
        self.index = i;
        Ok(None)
    }

    fn expr_with_precedence(&mut self, min_precedence: u8) -> Result<Option<Expr>> {
        if let Some(mut left) = self.atom()? {
            if let Some(members) = self.tuple()? {
                left = Expr::Call(self.arena.alloc(left), members);
            }
            loop {
                if let Some((built_in, left_precedence, right_precedence)) = self.accept_built_in()
                {
                    if left_precedence < min_precedence {
                        self.index -= 1;
                        break;
                    }
                    if let Some(right) = self.expr_with_precedence(right_precedence)? {
                        let args = bumpalo::vec![
                            in &self.arena;
                            TupleMember::Positional(left),
                            TupleMember::Positional(right),
                        ];
                        left = Expr::Call(self.arena.alloc(Expr::BuiltIn(built_in)), &args);
                    } else {
                        return Err(self.invalid_token());
                    }
                } else {
                    break;
                }
            }
            Ok(Some(left))
        } else {
            Ok(None)
        }
    }

    fn expr(&mut self) -> Result<Option<Expr>> {
        if let Some(function) = self.function()? {
            Ok(Some(function))
        } else if let Some(expr) = self.expr_with_precedence(0)? {
            Ok(Some(expr))
        } else {
            Ok(None)
        }
    }

    fn module(&mut self) -> Result<Module> {
        let imports = self.imports()?;
        if imports.len() > 0 {
            self.accept(TokenKind::Comma);
        }
        let statements: &[Statement] = self.statements()?;
        Ok(Module {
            imports,
            body: Expr::Block(statements),
            export: Expr::Void,
        })
    }
}

pub fn parse_module<'src, 'ast, 'sym>(
    source: &'src str,
    arena: &'ast Bump,
    interner: &'sym mut Interner,
) -> Result<Module<'ast>> {
    Parser {
        source,
        arena,
        interner,
        tokens: tokenize(source)?,
        index: 0,
    }
    .module()
}

pub fn parse_expr<'src, 'ast, 'sym>(
    source: &'src str,
    arena: &'ast Bump,
    interner: &'sym mut Interner,
) -> Result<Expr<'ast>> {
    let expr = Parser {
        source,
        arena,
        interner,
        tokens: tokenize(source)?,
        index: 0,
    }
    .expr()?
    .unwrap_or(Expr::Void);
    Ok(expr)
}

/*
#[cfg(test)]
mod tests {
    use super::*;

    fn parse(source: &str) -> Result<Expr> {
        let bump = Bump::new();
        let mut interner: Interner = Interner::new();
        super::parse(source, &bump, &mut interner)
    }

    fn parse_expr(source: &str) -> Result<Expr> {
        let bump = Bump::new();
        let mut interner: Interner = Interner::new();
        super::parse_expr(source, &bump, &mut interner)
    }

    #[test]
    fn test_parse_identifier() {
        assert_eq!(parse_expr("x"), Ok(Expr::Identifier(Symbol(0))));
    }

    #[test]
    fn test_parse_symbol() {
        assert_eq!(parse_expr("X"), Ok(Expr::Symbol(Symbol(0))));
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(parse_expr(r#""""#), Ok(Expr::String("".into())));
        assert_eq!(parse_expr(r#""\"""#), Ok(Expr::String("\"".into())));
        assert_eq!(parse_expr(r#""\\""#), Ok(Expr::String("\\".into())));
        assert_eq!(
            parse_expr(r#""\0\a\b\t\v\f\n\r""#),
            Ok(Expr::String("\0\x07\x08\t\x0B\x0C\n\r".into()))
        );
        assert_eq!(
            parse_expr(r#""\x00\x01\x02\x03\x04\x05""#),
            Ok(Expr::String("\x00\x01\x02\x03\x04\x05".into()))
        );
        assert_eq!(
            parse_expr(r#""\z""#).unwrap_err().kind,
            ErrorKind::StringInvalidEscapeSequence
        );
        assert_eq!(
            parse_expr(r#""\xFF""#).unwrap_err().kind,
            ErrorKind::StringInvalidEscapeSequence
        );
    }

    #[test]
    fn test_parse_integer() {
        assert_eq!(parse_expr("0"), Ok(Expr::Int(0)));
        //assert_eq!(parse_expr("-0"), Ok(Expr::Integer(0)));
        assert_eq!(parse_expr("7"), Ok(Expr::Int(7)));
        //assert_eq!(parse_expr("-3"), Ok(Expr::Integer(-3)));
        assert_eq!(parse_expr("123"), Ok(Expr::Int(123)));
        //assert_eq!(parse_expr("-313"), Ok(Expr::Integer(-313)));
        assert_eq!(parse_expr("000747"), Ok(Expr::Int(747)));
        //assert_eq!(parse_expr("-002200"), Ok(Expr::Integer(-2200)));
        /*
        assert_eq!(
            parse_expr("9223372036854775807"),
            Ok(Expr::Integer(9223372036854775807))
        );
        assert_eq!(
            parse_expr("9223372036854775808").unwrap_err().kind,
            ErrorKind::InvalidInteger
        );
        assert_eq!(
            parse_expr("-9223372036854775808"),
            Ok(Expr::Integer(-9223372036854775808))
        );
        assert_eq!(
            parse_expr("-9223372036854775809").unwrap_err().kind,
            ErrorKind::InvalidInteger
        );
        */
    }

    #[test]
    fn test_parse_float() {
        assert_eq!(parse_expr("0.0"), Ok(Expr::Float(0.0)));
        //assert_eq!(parse_expr("-0.0"), Ok(Expr::Float(0.0)));
        assert_eq!(parse_expr("1.0"), Ok(Expr::Float(1.0)));
        //assert_eq!(parse_expr("-1.0"), Ok(Expr::Float(-1.0)));
        assert_eq!(parse_expr("3.141592"), Ok(Expr::Float(3.141592)));
        //assert_eq!(parse_expr("-2.7800000"), Ok(Expr::Float(-2.78)));
    }

    #[test]
    fn test_parse_tuple() {
        assert_eq!(parse_expr("()"), Ok(Expr::Nil),);
        assert_eq!(
            parse_expr("(1)"),
            Ok(Expr::Tuple(Tuple {
                symbol: None,
                positional: vec![Expr::Int(1)],
                named: vec![],
            }))
        );
        assert_eq!(
            parse_expr("(1, 2, 3)"),
            Ok(Expr::Tuple(Tuple {
                symbol: None,
                positional: vec![Expr::Int(1), Expr::Int(2), Expr::Int(3)],
                named: vec![],
            }))
        );
        assert_eq!(
            parse_expr("Point(1, 2)"),
            Ok(Expr::Tuple(Tuple {
                symbol: Some(Symbol(0)),
                positional: vec![Expr::Int(1), Expr::Int(2)],
                named: vec![],
            }))
        );
        assert_eq!(
            parse_expr("(x: 1, y: 2)"),
            Ok(Expr::Tuple(Tuple {
                symbol: None,
                positional: vec![],
                named: vec![(Symbol(0), Expr::Int(1)), (Symbol(1), Expr::Int(2)),]
            }))
        );
        assert_eq!(
            parse_expr(r#"Person("id", name: "Bob", age: 49)"#),
            Ok(Expr::Tuple(Tuple {
                symbol: Some(Symbol(0)),
                positional: vec![Expr::String("id".into())],
                named: vec![
                    (Symbol(1), Expr::String("Bob".into())),
                    (Symbol(2), Expr::Int(49))
                ],
            }))
        )
    }

    #[test]
    fn test_parse_vector() {
        assert_eq!(parse_expr("[1]"), Ok(Expr::Vector(vec![Expr::Int(1)])));
        assert_eq!(
            parse_expr("[1, 2, 3]"),
            Ok(Expr::Vector(vec![Expr::Int(1), Expr::Int(2), Expr::Int(3)]))
        )
    }

    #[test]
    fn test_precedence() {
        assert_eq!(
            parse_expr("0 + 0"),
            Ok(Expr::Operator(
                OperatorKind::Add,
                vec![Expr::Int(0).into(), Expr::Int(0).into()]
            ))
        );
        assert_eq!(
            parse_expr("0 * 0"),
            Ok(Expr::Operator(
                OperatorKind::Multiply,
                vec![Expr::Int(0).into(), Expr::Int(0).into()]
            ))
        );
        assert_eq!(
            parse_expr("0 + 0 * 0"),
            Ok(Expr::Operator(
                OperatorKind::Add,
                vec![
                    Expr::Int(0),
                    Expr::Operator(OperatorKind::Multiply, vec![Expr::Int(0), Expr::Int(0)])
                ],
            ))
        );
        assert_eq!(
            parse_expr("0 * 0 + 0 / 0 - 0"),
            Ok(Expr::Operator(
                OperatorKind::Subtract,
                vec![
                    Expr::Operator(
                        OperatorKind::Add,
                        vec![
                            Expr::Operator(
                                OperatorKind::Multiply,
                                vec![Expr::Int(0), Expr::Int(0)],
                            ),
                            Expr::Operator(OperatorKind::Divide, vec![Expr::Int(0), Expr::Int(0)],),
                        ]
                    ),
                    Expr::Int(0)
                ]
            )),
        );
        assert_eq!(
            parse_expr("f x * g y + h z"),
            Ok(Expr::Operator(
                OperatorKind::Add,
                vec![
                    Expr::Operator(
                        OperatorKind::Multiply,
                        vec![
                            Expr::Call(
                                Expr::Identifier(Symbol(0)).into(),
                                vec![Expr::Identifier(Symbol(1)).into()]
                            ),
                            Expr::Call(
                                Expr::Identifier(Symbol(2)).into(),
                                vec![Expr::Identifier(Symbol(3)).into()]
                            ),
                        ]
                    ),
                    Expr::Call(
                        Expr::Identifier(Symbol(4)).into(),
                        vec![Expr::Identifier(Symbol(5))]
                    ),
                ]
            )),
        );
    }

    #[test]
    fn test_function() {
        assert_eq!(
            parse_expr("x -> x"),
            Ok(Expr::Function(
                vec![Pattern::Identifier(Symbol(0))],
                Expr::Identifier(Symbol(0)).into()
            ))
        );
        assert_eq!(
            parse_expr("x y z -> x y z"),
            Ok(Expr::Function(
                vec![
                    Pattern::Identifier(Symbol(0)),
                    Pattern::Identifier(Symbol(1)),
                    Pattern::Identifier(Symbol(2)),
                ],
                Expr::Call(
                    Expr::Identifier(Symbol(0)).into(),
                    vec![
                        Expr::Identifier(Symbol(1)).into(),
                        Expr::Identifier(Symbol(2)).into(),
                    ]
                )
                .into()
            ))
        )
    }

    #[test]
    fn test_parse_block() {
        assert_eq!(parse_expr("{}"), Ok(Expr::Void));
        assert_eq!(parse_expr("{{{{{}}}}}"), Ok(Expr::Void));
        assert_eq!(parse(""), Ok(Expr::Void));
        assert_eq!(
            parse("let x = 0, let y = 1, [x, y]"),
            Ok(Expr::Block(
                vec![],
                vec![
                    Let(Pattern::Identifier(Symbol(0)), Expr::Int(0).into()),
                    Let(Pattern::Identifier(Symbol(1)), Expr::Int(1).into())
                ],
                vec![Expr::Vector(vec![
                    Expr::Identifier(Symbol(0)),
                    Expr::Identifier(Symbol(1))
                ])]
            ))
        );
        assert_eq!(parse("{{1}}"), Ok(Expr::Int(1)));
        assert_eq!(
            parse(
                r#"
                let a = "test"
                let b = "test"
                (a, b)
                "#
            ),
            Ok(Expr::Block(
                vec![],
                vec![
                    Let(
                        Pattern::Identifier(Symbol(0)),
                        Expr::String("test".into()).into()
                    ),
                    Let(
                        Pattern::Identifier(Symbol(1)),
                        Expr::String("test".into()).into(),
                    )
                ],
                vec![Expr::Tuple(Tuple {
                    symbol: None,
                    positional: vec![Expr::Identifier(Symbol(0)), Expr::Identifier(Symbol(1))],
                    named: vec![],
                })]
            ))
        );
        assert_eq!(
            parse(
                r#"
                import (
                    x "x"
                    y "y"
                )
                let point = Point(x, y)
                point
                "#
            ),
            Ok(Expr::Block(
                vec![Import(Symbol(0), "x".into()), Import(Symbol(1), "y".into())],
                vec![Let(
                    Pattern::Identifier(Symbol(2)),
                    Expr::Tuple(Tuple {
                        symbol: Some(Symbol(3)),
                        positional: vec![Expr::Identifier(Symbol(0)), Expr::Identifier(Symbol(1))],
                        named: vec![],
                    })
                    .into()
                )],
                vec![Expr::Identifier(Symbol(2))]
            ))
        );
    }
}
*/

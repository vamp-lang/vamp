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

    fn accept_slice(&mut self, kind: TokenKind) -> Option<&'src str> {
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

    fn unescape(&mut self, span: Span) -> Result<&'ast str> {
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
                    '\'' => string.push('\''),
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
        Ok(string.into_bump_str())
    }

    fn symbol(&mut self) -> Result<Option<Symbol>> {
        if let Some(span) = self.accept(TokenKind::Symbol) {
            let unescaped = self.unescape(span)?;
            Ok(Some(self.interner.intern(unescaped)))
        } else {
            Ok(None)
        }
    }

    fn string(&mut self) -> Result<Option<&'ast str>> {
        if let Some(span) = self.accept(TokenKind::String) {
            let unescaped = self.unescape(span)?;
            Ok(Some(unescaped))
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

    fn tuple_member(&mut self) -> Result<Option<TupleMember<'ast>>> {
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

    fn tuple(&mut self) -> Result<Option<&'ast [TupleMember<'ast>]>> {
        let i = self.index;
        if let Some(left_parenthesis_span) = self.accept(TokenKind::LParen) {
            let mut members = BumpVec::new_in(self.arena);
            if let Some(member) = self.tuple_member()? {
                members.push(member);
                while let Some(_comma_span) = self.accept(TokenKind::Comma) {
                    if let Some(member) = self.tuple_member()? {
                        members.push(member);
                    }
                }
            }
            self.accept(TokenKind::RParen).ok_or_else(|| Error {
                kind: ErrorKind::UnbalancedDelimiters,
                span: left_parenthesis_span,
            })?;
            Ok(Some(members.into_bump_slice()))
        } else {
            self.index = i;
            Ok(None)
        }
    }

    fn vector(&mut self) -> Result<Option<&'ast [Expr<'ast>]>> {
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
            self.accept(TokenKind::RBracket).ok_or_else(|| Error {
                kind: ErrorKind::UnbalancedDelimiters,
                span: left_bracket_span,
            })?;
            Ok(Some(exprs.into_bump_slice()))
        } else {
            Ok(None)
        }
    }

    fn import(&mut self) -> Result<Option<Import<'ast>>> {
        if let Some(pattern) = self.pattern() {
            if let Some(string) = self.string()? {
                return Ok(Some(Import(pattern, string)));
            }
        }
        Ok(None)
    }

    fn imports(&mut self) -> Result<&'ast [Import<'ast>]> {
        let mut imports = BumpVec::new_in(self.arena);
        if let Some(_import_span) = self.accept(TokenKind::Import) {
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
            self.accept(TokenKind::RParen).ok_or_else(|| Error {
                kind: ErrorKind::UnbalancedDelimiters,
                span: left_parenthesis_span,
            })?;
        }
        Ok(imports.into_bump_slice())
    }

    fn pattern_tuple_member(&mut self) -> Option<PatternTupleMember<'ast>> {
        if let Some(identifier) = self.identifier() {
            if self.accept(TokenKind::Colon).is_some() {
                let pattern = self
                    .pattern()
                    .unwrap_or_else(|| Pattern::Identifier(identifier));
                Some(PatternTupleMember::Named(identifier, pattern))
            } else {
                Some(PatternTupleMember::Positional(Pattern::Identifier(
                    identifier,
                )))
            }
        } else if let Some(pattern) = self.pattern() {
            Some(PatternTupleMember::Positional(pattern))
        } else {
            None
        }
    }

    fn pattern_tuple(&mut self) -> Option<&'ast [PatternTupleMember<'ast>]> {
        let i = self.index;
        if let Some(left_parenthesis_span) = self.accept(TokenKind::LParen) {
            let mut members = BumpVec::new_in(self.arena);
            if let Some(member) = self.pattern_tuple_member() {
                members.push(member);
                while let Some(_comma_span) = self.accept(TokenKind::Comma) {
                    if let Some(member) = self.pattern_tuple_member() {
                        members.push(member);
                    }
                }
            }
            self.accept(TokenKind::RParen)?;
            Some(members.into_bump_slice())
        } else {
            self.index = i;
            None
        }
    }

    fn pattern(&mut self) -> Option<Pattern<'ast>> {
        if let Some(members) = self.pattern_tuple() {
            if members.len() == 0 {
                Some(Pattern::Nil)
            } else {
                Some(Pattern::Tuple(members))
            }
        } else if let Some(identifier) = self.identifier() {
            Some(Pattern::Identifier(identifier))
        } else {
            None
        }
    }

    fn statement(&mut self) -> Result<Option<Statement<'ast>>> {
        if self.accept(TokenKind::Let).is_some() {
            let pattern = self.pattern().ok_or_else(|| self.invalid_token())?;
            let args = self.pattern_tuple();
            self.accept(TokenKind::Equals)
                .ok_or_else(|| self.invalid_token())?;
            let expr = self.expr()?.ok_or_else(|| self.invalid_token())?;
            if let Some(args) = args {
                Ok(Some(Statement::Let(
                    pattern,
                    Expr::Function(self.arena.alloc(args), self.arena.alloc(expr)),
                )))
            } else {
                Ok(Some(Statement::Let(pattern, expr)))
            }
        } else if let Some(expr) = self.expr()? {
            Ok(Some(Statement::Expr(expr)))
        } else {
            Ok(None)
        }
    }

    fn statements(&mut self) -> Result<&'ast [Statement<'ast>]> {
        let mut statements = BumpVec::new_in(self.arena);
        if let Some(statement) = self.statement()? {
            statements.push(statement);
            while self.accept(TokenKind::Comma).is_some() {
                if let Some(statement) = self.statement()? {
                    statements.push(statement);
                }
            }
        }
        Ok(statements.into_bump_slice())
    }

    fn block(&mut self) -> Result<Option<Expr<'ast>>> {
        if let Some(left_brace_span) = self.accept(TokenKind::LBrace) {
            let statements = self.statements()?;
            self.accept(TokenKind::RBrace).ok_or_else(|| Error {
                kind: ErrorKind::UnbalancedDelimiters,
                span: left_brace_span,
            })?;
            if statements.len() == 0 {
                Ok(Some(Expr::Void))
            } else if let [Statement::Expr(expr)] = statements {
                Ok(Some(expr.clone()))
            } else {
                Ok(Some(Expr::Block(&statements)))
            }
        } else {
            Ok(None)
        }
    }

    fn atom(&mut self) -> Result<Option<Expr<'ast>>> {
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
        } else if let Some(symbol) = self.symbol()? {
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

    fn function_args(&mut self) -> Result<Option<&'ast [PatternTupleMember<'ast>]>> {
        if self.accept(TokenKind::Pipe).is_some() {
            let mut args = BumpVec::new_in(self.arena);
            if let Some(arg) = self.pattern_tuple_member() {
                args.push(arg);
                while self.accept(TokenKind::Comma).is_some() {
                    if let Some(arg) = self.pattern_tuple_member() {
                        args.push(arg);
                    }
                }
            }
            self.accept(TokenKind::Pipe)
                .ok_or_else(|| self.invalid_token())?;
            Ok(Some(args.into_bump_slice()))
        } else {
            Ok(None)
        }
    }

    fn function(&mut self) -> Result<Option<Expr<'ast>>> {
        if let Some(args) = self.function_args()? {
            let expr = self.expr()?.unwrap_or(Expr::Void);
            Ok(Some(Expr::Function(args, self.arena.alloc(expr))))
        } else {
            Ok(None)
        }
    }

    fn expr_with_precedence(&mut self, min_precedence: u8) -> Result<Option<Expr<'ast>>> {
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
                        left = Expr::Call(
                            self.arena.alloc(Expr::BuiltIn(built_in)),
                            args.into_bump_slice(),
                        );
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

    fn expr(&mut self) -> Result<Option<Expr<'ast>>> {
        if let Some(function) = self.function()? {
            Ok(Some(function))
        } else if let Some(expr) = self.expr_with_precedence(0)? {
            Ok(Some(expr))
        } else {
            Ok(None)
        }
    }

    fn module(&mut self) -> Result<Module<'ast>> {
        let imports = self.imports()?;
        if imports.len() > 0 {
            self.accept(TokenKind::Comma);
        }
        let statements: &[Statement] = self.statements()?;
        Ok(Module {
            imports,
            body: Expr::Block(statements),
        })
    }
}

pub fn parse_module<'ast>(
    source: &str,
    arena: &'ast Bump,
    interner: &mut Interner,
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

pub fn parse_statement<'ast>(
    source: &str,
    arena: &'ast Bump,
    interner: &mut Interner,
) -> Result<Statement<'ast>> {
    let expr = Parser {
        source,
        arena,
        interner,
        tokens: tokenize(source)?,
        index: 0,
    }
    .statement()?
    .unwrap_or(Statement::Expr(Expr::Void));
    Ok(expr)
}

pub fn parse_expr<'ast>(
    source: &str,
    arena: &'ast Bump,
    interner: &mut Interner,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_module<'ast>(source: &str, arena: &'ast Bump) -> Result<Module<'ast>> {
        let mut interner: Interner = Interner::new();
        super::parse_module(source, arena, &mut interner)
    }

    fn parse_expr<'ast>(source: &str, arena: &'ast Bump) -> Result<Expr<'ast>> {
        let mut interner: Interner = Interner::new();
        super::parse_expr(source, arena, &mut interner)
    }

    #[test]
    fn identifier() {
        let a = Bump::new();
        assert_eq!(parse_expr("x", &a), Ok(Expr::Identifier(Symbol(0))));
    }

    #[test]
    fn symbol() {
        let a = Bump::new();
        assert_eq!(parse_expr("''", &a), Ok(Expr::Symbol(Symbol(0))));
        assert_eq!(parse_expr("'\\''", &a), Ok(Expr::Symbol(Symbol(0))));
        assert_eq!(parse_expr("'x'", &a), Ok(Expr::Symbol(Symbol(0))));
    }

    #[test]
    fn string() {
        let a = Bump::new();
        assert_eq!(parse_expr(r#""""#, &a), Ok(Expr::String("")));
        assert_eq!(parse_expr(r#""\"""#, &a), Ok(Expr::String("\"")));
        assert_eq!(parse_expr(r#""\\""#, &a), Ok(Expr::String("\\")));
        assert_eq!(
            parse_expr(r#""\0\a\b\t\v\f\n\r""#, &a),
            Ok(Expr::String("\0\x07\x08\t\x0B\x0C\n\r"))
        );
        assert_eq!(
            parse_expr(r#""\x00\x01\x02\x03\x04\x05""#, &a),
            Ok(Expr::String("\x00\x01\x02\x03\x04\x05"))
        );
        assert_eq!(
            parse_expr(r#""\z""#, &a).unwrap_err().kind,
            ErrorKind::StringInvalidEscapeSequence
        );
        assert_eq!(
            parse_expr(r#""\xFF""#, &a).unwrap_err().kind,
            ErrorKind::StringInvalidEscapeSequence
        );
    }

    #[test]
    fn integer() {
        let a = Bump::new();
        assert_eq!(parse_expr("0", &a), Ok(Expr::Int(0)));
        //assert_eq!(parse_expr("-0"), Ok(Expr::Integer(0)));
        assert_eq!(parse_expr("7", &a), Ok(Expr::Int(7)));
        //assert_eq!(parse_expr("-3"), Ok(Expr::Integer(-3)));
        assert_eq!(parse_expr("123", &a), Ok(Expr::Int(123)));
        //assert_eq!(parse_expr("-313"), Ok(Expr::Integer(-313)));
        assert_eq!(parse_expr("000747", &a), Ok(Expr::Int(747)));
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
    fn float() {
        let a = Bump::new();
        assert_eq!(parse_expr("0.0", &a), Ok(Expr::Float(0.0)));
        //assert_eq!(parse_expr("-0.0"), Ok(Expr::Float(0.0)));
        assert_eq!(parse_expr("1.0", &a), Ok(Expr::Float(1.0)));
        //assert_eq!(parse_expr("-1.0"), Ok(Expr::Float(-1.0)));
        assert_eq!(parse_expr("3.141592", &a), Ok(Expr::Float(3.141592)));
        //assert_eq!(parse_expr("-2.7800000"), Ok(Expr::Float(-2.78)));
    }

    #[test]
    fn tuple() {
        let a = Bump::new();
        assert_eq!(parse_expr("()", &a), Ok(Expr::Nil));
        assert_eq!(
            parse_expr("(1)", &a),
            Ok(Expr::Tuple(&[TupleMember::Positional(Expr::Int(1))]))
        );
        assert_eq!(
            parse_expr("(1, 2, 3)", &a),
            Ok(Expr::Tuple(&[
                TupleMember::Positional(Expr::Int(1)),
                TupleMember::Positional(Expr::Int(2)),
                TupleMember::Positional(Expr::Int(3)),
            ]))
        );
        assert_eq!(
            parse_expr("(x: 1, y: 2)", &a),
            Ok(Expr::Tuple(&[
                TupleMember::Named(Symbol(0), Expr::Int(1)),
                TupleMember::Named(Symbol(1), Expr::Int(2))
            ]))
        );
        assert_eq!(
            parse_expr(r#"("id", name: "Bob", age: 49)"#, &a),
            Ok(Expr::Tuple(&[
                TupleMember::Positional(Expr::String("id")),
                TupleMember::Named(Symbol(0), Expr::String("Bob")),
                TupleMember::Named(Symbol(1), Expr::Int(49))
            ]))
        );
    }

    #[test]
    fn vector() {
        let a = Bump::new();
        assert_eq!(parse_expr("[1]", &a), Ok(Expr::Vector(&[Expr::Int(1)])));
        assert_eq!(
            parse_expr("[1, 2, 3]", &a),
            Ok(Expr::Vector(&[Expr::Int(1), Expr::Int(2), Expr::Int(3)]))
        )
    }

    #[test]
    fn expr_precedence() {
        let a = Bump::new();
        assert_eq!(
            parse_expr("0 + 0", &a),
            Ok(Expr::Call(
                &Expr::BuiltIn(BuiltIn::Add),
                &[
                    TupleMember::Positional(Expr::Int(0)),
                    TupleMember::Positional(Expr::Int(0))
                ]
            ))
        );
        assert_eq!(
            parse_expr("0 * 0", &a),
            Ok(Expr::Call(
                &Expr::BuiltIn(BuiltIn::Mul),
                &[
                    TupleMember::Positional(Expr::Int(0)),
                    TupleMember::Positional(Expr::Int(0))
                ]
            ))
        );
        assert_eq!(
            parse_expr("0 + 0 * 0", &a),
            Ok(Expr::Call(
                &Expr::BuiltIn(BuiltIn::Add),
                &[
                    TupleMember::Positional(Expr::Int(0)),
                    TupleMember::Positional(Expr::Call(
                        &Expr::BuiltIn(BuiltIn::Mul),
                        &[
                            TupleMember::Positional(Expr::Int(0)),
                            TupleMember::Positional(Expr::Int(0))
                        ],
                    )),
                ],
            ))
        );
        assert_eq!(
            parse_expr("0 * 0 + 0 / 0 - 0", &a),
            Ok(Expr::Call(
                &Expr::BuiltIn(BuiltIn::Sub),
                &[
                    TupleMember::Positional(Expr::Call(
                        &Expr::BuiltIn(BuiltIn::Add),
                        &[
                            TupleMember::Positional(Expr::Call(
                                &Expr::BuiltIn(BuiltIn::Mul),
                                &[
                                    TupleMember::Positional(Expr::Int(0)),
                                    TupleMember::Positional(Expr::Int(0))
                                ]
                            )),
                            TupleMember::Positional(Expr::Call(
                                &Expr::BuiltIn(BuiltIn::Div),
                                &[
                                    TupleMember::Positional(Expr::Int(0)),
                                    TupleMember::Positional(Expr::Int(0))
                                ]
                            ))
                        ]
                    )),
                    TupleMember::Positional(Expr::Int(0))
                ]
            )),
        );
        assert_eq!(
            parse_expr("f(x) * g(y) + h(z)", &a),
            Ok(Expr::Call(
                &Expr::BuiltIn(BuiltIn::Add),
                &[
                    TupleMember::Positional(Expr::Call(
                        &Expr::BuiltIn(BuiltIn::Mul),
                        &[
                            TupleMember::Positional(Expr::Call(
                                &Expr::Identifier(Symbol(0)),
                                &[TupleMember::Positional(Expr::Identifier(Symbol(1)))]
                            )),
                            TupleMember::Positional(Expr::Call(
                                &Expr::Identifier(Symbol(2)),
                                &[TupleMember::Positional(Expr::Identifier(Symbol(3)))]
                            )),
                        ]
                    )),
                    TupleMember::Positional(Expr::Call(
                        &Expr::Identifier(Symbol(4)),
                        &[TupleMember::Positional(Expr::Identifier(Symbol(5)))]
                    )),
                ]
            )),
        );
    }

    #[test]
    fn function() {
        let a = Bump::new();
        assert_eq!(
            parse_expr("|x| x", &a),
            Ok(Expr::Function(
                &[PatternTupleMember::Positional(Pattern::Identifier(Symbol(
                    0
                )))],
                &Expr::Identifier(Symbol(0))
            ))
        );
        assert_eq!(
            parse_expr("|x, y, z| x(y, z)", &a),
            Ok(Expr::Function(
                &[
                    PatternTupleMember::Positional(Pattern::Identifier(Symbol(0))),
                    PatternTupleMember::Positional(Pattern::Identifier(Symbol(1))),
                    PatternTupleMember::Positional(Pattern::Identifier(Symbol(2))),
                ],
                &Expr::Call(
                    &Expr::Identifier(Symbol(0)),
                    &[
                        TupleMember::Positional(Expr::Identifier(Symbol(1))),
                        TupleMember::Positional(Expr::Identifier(Symbol(2)))
                    ]
                )
            ))
        )
    }

    #[test]
    fn block() {
        let a = Bump::new();
        assert_eq!(parse_expr("{}", &a), Ok(Expr::Void));
        assert_eq!(parse_expr("{{{{{}}}}}", &a), Ok(Expr::Void));
        assert_eq!(
            parse_module("let x = 0, let y = 1, [x, y]", &a),
            Ok(Module {
                imports: &[],
                body: Expr::Block(&[
                    Statement::Let(Pattern::Identifier(Symbol(0)), Expr::Int(0)),
                    Statement::Let(Pattern::Identifier(Symbol(1)), Expr::Int(1)),
                    Statement::Expr(Expr::Vector(&[
                        Expr::Identifier(Symbol(0)),
                        Expr::Identifier(Symbol(1))
                    ])),
                ]),
            })
        );
        assert_eq!(parse_expr("{{1}}", &a), Ok(Expr::Int(1)));
        assert_eq!(
            parse_module(
                r#"
                let a = "test"
                let b = "test"
                (a, b)
                "#,
                &a
            ),
            Ok(Module {
                imports: &[],
                body: Expr::Block(&[
                    Statement::Let(Pattern::Identifier(Symbol(0)), Expr::String("test")),
                    Statement::Let(Pattern::Identifier(Symbol(1)), Expr::String("test")),
                    Statement::Expr(Expr::Tuple(&[
                        TupleMember::Positional(Expr::Identifier(Symbol(0))),
                        TupleMember::Positional(Expr::Identifier(Symbol(1))),
                    ]))
                ])
            })
        );
        assert_eq!(
            parse_module(
                r#"
                import (
                    x "x"
                    y "y"
                )
                let point = (x, y)
                point
                "#,
                &a
            ),
            Ok(Module {
                imports: &[
                    Import(Pattern::Identifier(Symbol(0)), "x"),
                    Import(Pattern::Identifier(Symbol(1)), "y"),
                ],
                body: Expr::Block(&[
                    Statement::Let(
                        Pattern::Identifier(Symbol(2)),
                        Expr::Tuple(&[
                            TupleMember::Positional(Expr::Identifier(Symbol(0))),
                            TupleMember::Positional(Expr::Identifier(Symbol(1)))
                        ])
                    ),
                    Statement::Expr(Expr::Identifier(Symbol(2)))
                ])
            })
        );
    }
}

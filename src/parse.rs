use crate::source::{Error, ErrorKind, Result, Span};
use crate::symbol::{Interner, Symbol};
use crate::tokens::{tokenize, Token, TokenKind};

#[derive(Debug, PartialEq, Clone)]
pub struct PatternTuple {
    pub tag: Option<Symbol>,
    pub positional: Vec<Pattern>,
    pub named: Vec<(Symbol, Pattern)>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Pattern {
    // TODO: Pattern matching
    //Tuple(PatternTuple),
    //Vector(Vec<Pattern>),
    Identifier(Symbol),
    //Tag(String),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Import(pub Symbol, pub String);

#[derive(Debug, PartialEq, Clone)]
pub struct Let(pub Pattern, pub Box<Expr>);

#[derive(Debug, PartialEq, Clone)]
pub struct Tuple {
    pub tag: Option<Symbol>,
    pub positional: Vec<Expr>,
    pub named: Vec<(Symbol, Expr)>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum OperatorKind {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Void,
    Nil,
    Block(Vec<Import>, Vec<Let>, Vec<Expr>),
    Function(Vec<Pattern>, Box<Expr>),
    Tuple(Tuple),
    Vector(Vec<Expr>),
    Identifier(Symbol),
    Tag(Symbol),
    String(String),
    Integer(i64),
    Float(f64),
    Operator(OperatorKind, Vec<Expr>),
    Call(Box<Expr>, Vec<Expr>),
}

pub struct Parser<'a, 'b> {
    source: &'a str,
    interner: &'b mut Interner,
    tokens: Vec<Token>,
    index: usize,
}

impl<'a, 'b> Parser<'a, 'b> {
    fn accept(&mut self, kind: TokenKind) -> Option<Span> {
        if self.index < self.tokens.len() && self.tokens[self.index].kind == kind {
            let span = self.tokens[self.index].span;
            self.index += 1;
            Some(span)
        } else {
            None
        }
    }

    fn accept_operator(&mut self) -> Option<(OperatorKind, u8, u8)> {
        if self.index < self.tokens.len() {
            let result = match self.tokens[self.index].kind {
                TokenKind::Plus => (OperatorKind::Add, 1, 2),
                TokenKind::Minus => (OperatorKind::Subtract, 1, 2),
                TokenKind::Times => (OperatorKind::Multiply, 3, 4),
                TokenKind::Divide => (OperatorKind::Divide, 3, 4),
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

    fn parse_identifier(&mut self) -> Option<String> {
        self.accept(TokenKind::Identifier)
            .map(|span| self.source[span].into())
    }

    fn parse_tag(&mut self) -> Option<Symbol> {
        self.accept(TokenKind::Tag)
            .map(|span| self.interner.intern(&self.source[span]))
    }

    fn parse_string(&mut self) -> Result<Option<String>> {
        if let Some(span) = self.accept(TokenKind::String) {
            let slice = &self.source[span];
            let mut string = String::with_capacity(slice.len());
            let mut chars = slice[1..slice.len() - 1].chars();
            while let Some(c) = chars.next() {
                if c == '\\' {
                    let error = Error {
                        kind: ErrorKind::InvalidEscapeSequence,
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
            Ok(Some(string))
        } else {
            Ok(None)
        }
    }

    fn parse_integer(&mut self) -> Result<Option<i64>> {
        if let Some(integer_span) = self.accept(TokenKind::Integer) {
            let mut value: i64 = 0;
            let error = Error {
                kind: ErrorKind::InvalidInteger,
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

    fn parse_float(&mut self) -> Result<Option<f64>> {
        if let Some(float_span) = self.accept(TokenKind::Float) {
            // TODO: Write custom float parser.
            let value = self.source[float_span].parse::<f64>().map_err(|_| Error {
                kind: ErrorKind::InvalidFloat,
                span: float_span,
            })?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    fn parse_tuple_member(&mut self) -> Result<Option<(Option<Symbol>, Expr)>> {
        if let Some(identifier) = self.parse_identifier() {
            if self.accept(TokenKind::Colon).is_some() {
                let expr = self
                    .parse_expr()?
                    .unwrap_or_else(|| Expr::Identifier(self.interner.intern(&identifier)));
                Ok(Some((Some(self.interner.intern(&identifier)), expr)))
            } else {
                Ok(Some((
                    None,
                    Expr::Identifier(self.interner.intern(&identifier)),
                )))
            }
        } else if let Some(expr) = self.parse_expr()? {
            Ok(Some((None, expr)))
        } else {
            Ok(None)
        }
    }

    fn parse_tuple(&mut self) -> Result<Option<Expr>> {
        let i = self.index;
        let tag = self.parse_tag();
        if let Some(left_parenthesis_span) = self.accept(TokenKind::LeftParenthesis) {
            let mut positional = vec![];
            let mut named = vec![];
            if let Some((key, expr)) = self.parse_tuple_member()? {
                if let Some(key) = key {
                    named.push((key, expr));
                } else {
                    positional.push(expr);
                }
                while let Some(comma_span) = self.accept(TokenKind::Comma) {
                    if let Some((key, expr)) = self.parse_tuple_member()? {
                        if let Some(key) = key {
                            named.push((key, expr));
                        } else if named.len() > 0 {
                            return Err(Error {
                                kind: ErrorKind::TuplePositionalAfterNamed,
                                span: comma_span,
                            });
                        } else {
                            positional.push(expr);
                        }
                    }
                }
            }
            self.accept(TokenKind::RightParenthesis).ok_or(Error {
                kind: ErrorKind::UnbalancedDelimiters,
                span: left_parenthesis_span,
            })?;
            if positional.len() == 0 && named.len() == 0 {
                Ok(Some(Expr::Nil))
            } else {
                Ok(Some(Expr::Tuple(Tuple {
                    tag,
                    positional,
                    named,
                })))
            }
        } else {
            self.index = i;
            Ok(None)
        }
    }

    fn parse_vector(&mut self) -> Result<Option<Expr>> {
        if let Some(left_bracket_span) = self.accept(TokenKind::LeftBracket) {
            let mut exprs = Vec::new();
            if let Some(expr) = self.parse_expr()? {
                exprs.push(expr);
                while self.accept(TokenKind::Comma).is_some() {
                    if let Some(expr) = self.parse_expr()? {
                        exprs.push(expr);
                    }
                }
            }
            self.accept(TokenKind::RightBracket).ok_or(Error {
                kind: ErrorKind::UnbalancedDelimiters,
                span: left_bracket_span,
            })?;
            Ok(Some(Expr::Vector(exprs)))
        } else {
            Ok(None)
        }
    }

    fn parse_import(&mut self) -> Result<Option<Import>> {
        if let Some(identifier) = self.parse_identifier() {
            if let Some(string) = self.parse_string()? {
                return Ok(Some(Import(self.interner.intern(&identifier), string)));
            }
        }
        return Ok(None);
    }

    fn parse_imports(&mut self) -> Result<Vec<Import>> {
        let mut imports = Vec::new();
        if let Some(import_span) = self.accept(TokenKind::Import) {
            let left_parenthesis_span = self
                .accept(TokenKind::LeftParenthesis)
                .ok_or_else(|| self.invalid_token())?;
            if let Some(import) = self.parse_import()? {
                imports.push(import);
                while self.accept(TokenKind::Comma).is_some() {
                    if let Some(import) = self.parse_import()? {
                        imports.push(import);
                    }
                }
            }
            self.accept(TokenKind::RightParenthesis).ok_or(Error {
                kind: ErrorKind::UnbalancedDelimiters,
                span: left_parenthesis_span,
            })?;
        }
        Ok(imports)
    }

    fn parse_pattern(&mut self) -> Result<Option<Pattern>> {
        if let Some(identifier) = self.parse_identifier() {
            Ok(Some(Pattern::Identifier(self.interner.intern(&identifier))))
        } else {
            Ok(None)
        }
    }

    fn parse_let(&mut self) -> Result<Option<Let>> {
        if self.accept(TokenKind::Let).is_some() {
            let pattern = self.parse_pattern()?.ok_or_else(|| self.invalid_token())?;
            let mut arg_patterns = vec![];
            while let Some(arg_pattern) = self.parse_pattern()? {
                arg_patterns.push(arg_pattern);
            }
            self.accept(TokenKind::Equals)
                .ok_or_else(|| self.invalid_token())?;
            let expr = self.parse_expr()?.ok_or_else(|| self.invalid_token())?;
            if arg_patterns.len() > 0 {
                Ok(Some(Let(
                    pattern,
                    Expr::Function(arg_patterns, expr.into()).into(),
                )))
            } else {
                Ok(Some(Let(pattern, expr.into())))
            }
        } else {
            Ok(None)
        }
    }

    fn parse_block(&mut self) -> Result<Option<Expr>> {
        if let Some(left_brace_span) = self.accept(TokenKind::LeftBrace) {
            let block = self.parse()?;
            self.accept(TokenKind::RightBrace).ok_or(Error {
                kind: ErrorKind::UnbalancedDelimiters,
                span: left_brace_span,
            })?;
            Ok(Some(block))
        } else {
            Ok(None)
        }
    }

    fn parse_atom(&mut self) -> Result<Option<Expr>> {
        if let Some(tuple) = self.parse_tuple()? {
            Ok(Some(tuple))
        } else if let Some(vector) = self.parse_vector()? {
            Ok(Some(vector))
        } else if let Some(block) = self.parse_block()? {
            Ok(Some(block))
        } else if let Some(identifier) = self.parse_identifier() {
            Ok(Some(Expr::Identifier(self.interner.intern(&identifier))))
        } else if let Some(tag) = self.parse_tag() {
            Ok(Some(Expr::Tag(tag)))
        } else if let Some(string) = self.parse_string()? {
            Ok(Some(Expr::String(string)))
        } else if let Some(integer) = self.parse_integer()? {
            Ok(Some(Expr::Integer(integer)))
        } else if let Some(float) = self.parse_float()? {
            Ok(Some(Expr::Float(float)))
        } else {
            Ok(None)
        }
    }

    fn parse_function(&mut self) -> Result<Option<Expr>> {
        let i = self.index;
        if let Some(pattern) = self.parse_pattern()? {
            let mut patterns = vec![pattern];
            while let Some(pattern) = self.parse_pattern()? {
                patterns.push(pattern);
            }
            if self.accept(TokenKind::Arrow).is_some() {
                if let Some(expr) = self.parse_expr()? {
                    return Ok(Some(Expr::Function(patterns, expr.into())));
                } else {
                    return Err(self.invalid_token());
                }
            }
        }
        self.index = i;
        Ok(None)
    }

    fn parse_expr_precedence(&mut self, min_precedence: u8) -> Result<Option<Expr>> {
        if let Some(mut left) = self.parse_atom()? {
            if let Some(arg) = self.parse_atom()? {
                let mut args = vec![arg];
                while let Some(arg) = self.parse_atom()? {
                    args.push(arg);
                }
                left = Expr::Call(left.into(), args);
            }
            loop {
                if let Some((kind, left_precedence, right_precedence)) = self.accept_operator() {
                    if left_precedence < min_precedence {
                        self.index -= 1;
                        break;
                    }
                    if let Some(right) = self.parse_expr_precedence(right_precedence)? {
                        left = Expr::Operator(kind, vec![left, right]);
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

    fn parse_expr(&mut self) -> Result<Option<Expr>> {
        if let Some(function) = self.parse_function()? {
            Ok(Some(function))
        } else if let Some(expr) = self.parse_expr_precedence(0)? {
            Ok(Some(expr))
        } else {
            Ok(None)
        }
    }

    fn parse(&mut self) -> Result<Expr> {
        let imports = self.parse_imports()?;
        if imports.len() > 0 {
            self.accept(TokenKind::Comma);
        }
        let mut lets = Vec::new();
        if let Some(let_) = self.parse_let()? {
            lets.push(let_);
            while self.accept(TokenKind::Comma).is_some() {
                if let Some(let_) = self.parse_let()? {
                    lets.push(let_);
                }
            }
        }
        let mut exprs = Vec::new();
        if let Some(expr) = self.parse_expr()? {
            exprs.push(expr);
            while self.accept(TokenKind::Comma).is_some() {
                if let Some(expr) = self.parse_expr()? {
                    exprs.push(expr);
                }
            }
        }
        if exprs.len() == 0 {
            Ok(Expr::Void)
        } else if exprs.len() == 1 && imports.len() == 0 && lets.len() == 0 {
            // Elide blocks only used for grouping.
            Ok(exprs.pop().unwrap())
        } else {
            Ok(Expr::Block(imports, lets, exprs))
        }
    }
}

pub fn parse<'a, 'b>(source: &'a str, interner: &'b mut Interner) -> Result<Expr> {
    Parser {
        source,
        interner,
        tokens: tokenize(source)?,
        index: 0,
    }
    .parse()
}

pub fn parse_expr<'a, 'b>(source: &'a str, interner: &'b mut Interner) -> Result<Expr> {
    let expr = Parser {
        source,
        interner,
        tokens: tokenize(source)?,
        index: 0,
    }
    .parse_expr()?
    .unwrap_or(Expr::Void);
    Ok(expr)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(source: &str) -> Result<Expr> {
        let mut interner: Interner = Interner::new();
        super::parse(source, &mut interner)
    }

    fn parse_expr(source: &str) -> Result<Expr> {
        let mut interner: Interner = Interner::new();
        super::parse_expr(source, &mut interner)
    }

    #[test]
    fn test_parse_identifier() {
        assert_eq!(parse_expr("x"), Ok(Expr::Identifier(Symbol(0))));
    }

    #[test]
    fn test_parse_tag() {
        assert_eq!(parse_expr("X"), Ok(Expr::Tag(Symbol(0))));
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
            ErrorKind::InvalidEscapeSequence
        );
        assert_eq!(
            parse_expr(r#""\xFF""#).unwrap_err().kind,
            ErrorKind::InvalidEscapeSequence
        );
    }

    #[test]
    fn test_parse_integer() {
        assert_eq!(parse_expr("0"), Ok(Expr::Integer(0)));
        //assert_eq!(parse_expr("-0"), Ok(Expr::Integer(0)));
        assert_eq!(parse_expr("7"), Ok(Expr::Integer(7)));
        //assert_eq!(parse_expr("-3"), Ok(Expr::Integer(-3)));
        assert_eq!(parse_expr("123"), Ok(Expr::Integer(123)));
        //assert_eq!(parse_expr("-313"), Ok(Expr::Integer(-313)));
        assert_eq!(parse_expr("000747"), Ok(Expr::Integer(747)));
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
                tag: None,
                positional: vec![Expr::Integer(1)],
                named: vec![],
            }))
        );
        assert_eq!(
            parse_expr("(1, 2, 3)"),
            Ok(Expr::Tuple(Tuple {
                tag: None,
                positional: vec![Expr::Integer(1), Expr::Integer(2), Expr::Integer(3)],
                named: vec![],
            }))
        );
        assert_eq!(
            parse_expr("Point(1, 2)"),
            Ok(Expr::Tuple(Tuple {
                tag: Some(Symbol(0)),
                positional: vec![Expr::Integer(1), Expr::Integer(2)],
                named: vec![],
            }))
        );
        assert_eq!(
            parse_expr("(x: 1, y: 2)"),
            Ok(Expr::Tuple(Tuple {
                tag: None,
                positional: vec![],
                named: vec![(Symbol(0), Expr::Integer(1)), (Symbol(1), Expr::Integer(2)),]
            }))
        );
        assert_eq!(
            parse_expr(r#"Person("id", name: "Bob", age: 49)"#),
            Ok(Expr::Tuple(Tuple {
                tag: Some(Symbol(0)),
                positional: vec![Expr::String("id".into())],
                named: vec![
                    (Symbol(1), Expr::String("Bob".into())),
                    (Symbol(2), Expr::Integer(49))
                ],
            }))
        )
    }

    #[test]
    fn test_parse_vector() {
        assert_eq!(parse_expr("[1]"), Ok(Expr::Vector(vec![Expr::Integer(1)])));
        assert_eq!(
            parse_expr("[1, 2, 3]"),
            Ok(Expr::Vector(vec![
                Expr::Integer(1),
                Expr::Integer(2),
                Expr::Integer(3)
            ]))
        )
    }

    #[test]
    fn test_precedence() {
        assert_eq!(
            parse_expr("0 + 0"),
            Ok(Expr::Operator(
                OperatorKind::Add,
                vec![Expr::Integer(0).into(), Expr::Integer(0).into()]
            ))
        );
        assert_eq!(
            parse_expr("0 * 0"),
            Ok(Expr::Operator(
                OperatorKind::Multiply,
                vec![Expr::Integer(0).into(), Expr::Integer(0).into()]
            ))
        );
        assert_eq!(
            parse_expr("0 + 0 * 0"),
            Ok(Expr::Operator(
                OperatorKind::Add,
                vec![
                    Expr::Integer(0),
                    Expr::Operator(
                        OperatorKind::Multiply,
                        vec![Expr::Integer(0), Expr::Integer(0)]
                    )
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
                                vec![Expr::Integer(0), Expr::Integer(0)],
                            ),
                            Expr::Operator(
                                OperatorKind::Divide,
                                vec![Expr::Integer(0), Expr::Integer(0)],
                            ),
                        ]
                    ),
                    Expr::Integer(0)
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
                    Let(Pattern::Identifier(Symbol(0)), Expr::Integer(0).into()),
                    Let(Pattern::Identifier(Symbol(1)), Expr::Integer(1).into())
                ],
                vec![Expr::Vector(vec![
                    Expr::Identifier(Symbol(0)),
                    Expr::Identifier(Symbol(1))
                ])]
            ))
        );
        assert_eq!(parse("{{1}}"), Ok(Expr::Integer(1)));
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
                    tag: None,
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
                        tag: Some(Symbol(3)),
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

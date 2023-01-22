use crate::source::{Error, ErrorKind, Result, Span};
use crate::tokens::{tokenize, Token, TokenKind};
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct PatternTuple {
    pub tag: Option<String>,
    pub positional: Vec<Pattern>,
    pub named: Vec<(String, Pattern)>,
}

#[derive(Debug, PartialEq)]
pub enum Pattern {
    Tuple(PatternTuple),
    Vector(Vec<Pattern>),
    Identifier(String),
    Tag(String),
}

#[derive(Debug, PartialEq)]
pub struct Import(String, String);

#[derive(Debug, PartialEq)]
pub struct Let(Pattern, Box<Expr>);

#[derive(Debug, PartialEq)]
pub struct Tuple {
    pub tag: Option<String>,
    pub positional: Vec<Expr>,
    pub named: Vec<(String, Expr)>,
}

#[derive(Debug, PartialEq)]
pub enum OperatorKind {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Void,
    Nil,
    Block(Vec<Import>, Vec<Let>, Vec<Expr>),
    Function(Pattern, Box<Expr>),
    Tuple(Tuple),
    Vector(Vec<Expr>),
    Map(Vec<(Expr, Expr)>),
    Identifier(String),
    Tag(String),
    String(String),
    Integer(i64),
    Float(f64),
    Operator(OperatorKind, Vec<Expr>),
    Call(Box<Expr>, Vec<Expr>),
}

pub struct Parser<'source> {
    source: &'source str,
    tokens: Vec<Token>,
    index: usize,
}

impl<'source> Parser<'source> {
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

    fn parse_identifier(&mut self) -> Option<String> {
        self.accept(TokenKind::Identifier)
            .map(|span| self.source[span].into())
    }

    fn parse_tag(&mut self) -> Option<String> {
        self.accept(TokenKind::Tag)
            .map(|span| self.source[span].into())
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
                        // Form feed
                        'f' => string.push('\x0A'),
                        // Vertical tab
                        'v' => string.push('\x0B'),
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
        let i = self.index;
        let minus = self.accept(TokenKind::Minus);
        if let Some(integer_span) = self.accept(TokenKind::Integer) {
            let (sign, span) = if let Some(minus_span) = minus {
                (
                    -1,
                    Span {
                        start: minus_span.start,
                        end: integer_span.end,
                    },
                )
            } else {
                (1, integer_span)
            };
            let mut value: i64 = 0;
            let error = Error {
                kind: ErrorKind::InvalidInteger,
                span,
            };
            for digit in self.source[integer_span].bytes() {
                value = value
                    .checked_mul(10)
                    .ok_or(error)?
                    .checked_add(sign * (digit - b'0') as i64)
                    .ok_or(error)?;
            }
            Ok(Some(value))
        } else {
            self.index = i;
            Ok(None)
        }
    }

    fn parse_float(&mut self) -> Result<Option<f64>> {
        let i = self.index;
        let minus = self.accept(TokenKind::Minus);
        if let Some(float_span) = self.accept(TokenKind::Float) {
            let span = if let Some(minus_span) = minus {
                Span {
                    start: minus_span.start,
                    end: float_span.end,
                }
            } else {
                float_span
            };
            // TODO: Write custom float parser.
            let value = self.source[float_span]
                .parse::<f64>()
                .map(|value| if minus.is_some() { -value } else { value })
                .map_err(|_| Error {
                    kind: ErrorKind::InvalidFloat,
                    span,
                })?;
            Ok(Some(value))
        } else {
            self.index = i;
            Ok(None)
        }
    }

    fn parse_tuple_member(&mut self) -> Result<Option<(Option<String>, Expr)>> {
        if let Some(identifier) = self.parse_identifier() {
            if self.accept(TokenKind::Colon).is_some() {
                let expr = self
                    .parse_expr()?
                    .unwrap_or_else(|| Expr::Identifier(identifier.clone()));
                Ok(Some((Some(identifier), expr)))
            } else {
                Ok(Some((None, Expr::Identifier(identifier))))
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

    fn parse_map(&mut self) -> Result<Option<Expr>> {
        let i = self.index;
        if let Some(left_brace_span) = self.accept(TokenKind::LeftBrace) {
            self.accept(TokenKind::RightBrace).ok_or(Error {
                kind: ErrorKind::UnbalancedDelimiters,
                span: left_brace_span,
            })?;
            todo!()
        } else {
            self.index = i;
            Ok(None)
        }
    }

    fn parse_import(&mut self) -> Result<Option<Import>> {
        if let Some(identifier) = self.parse_identifier() {
            if let Some(string) = self.parse_string()? {
                return Ok(Some(Import(identifier, string)));
            }
        }
        return Ok(None);
    }

    fn parse_imports(&mut self) -> Result<Option<Vec<Import>>> {
        let mut imports = Vec::new();
        if let Some(import_span) = self.accept(TokenKind::Import) {
            let left_parenthesis_span = self.accept(TokenKind::LeftParenthesis).ok_or(Error {
                kind: ErrorKind::InvalidToken,
                span: import_span,
            })?;
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
            Ok(Some(imports))
        } else {
            Ok(None)
        }
    }

    fn parse_pattern(&mut self) -> Result<Option<Pattern>> {
        if let Some(identifier) = self.parse_identifier() {
            Ok(Some(Pattern::Identifier(identifier)))
        } else {
            Ok(None)
        }
    }

    fn parse_let(&mut self) -> Result<Option<Let>> {
        if let Some(let_span) = self.accept(TokenKind::Let) {
            let pattern = self.parse_pattern()?.ok_or(Error {
                kind: ErrorKind::InvalidToken,
                span: let_span,
            })?;
            self.accept(TokenKind::Equals).ok_or(Error {
                kind: ErrorKind::InvalidToken,
                span: let_span,
            })?;
            let expr = self.parse_expr()?.ok_or(Error {
                kind: ErrorKind::InvalidToken,
                span: let_span,
            })?;
            Ok(Some(Let(pattern, expr.into())))
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
            Ok(Some(Expr::Identifier(identifier)))
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

    fn parse_expr_precedence(&mut self, min_precedence: u8) -> Result<Option<Expr>> {
        if let Some(mut left) = self.parse_atom()? {
            loop {
                if let Some((kind, left_precedence, right_precedence)) = self.accept_operator() {
                    if left_precedence < min_precedence {
                        self.index -= 1;
                        break;
                    }
                    if let Some(right) = self.parse_expr_precedence(right_precedence)? {
                        left = Expr::Operator(kind, vec![left, right]);
                    } else {
                        let span = if self.index < self.tokens.len() {
                            self.tokens[self.index].span
                        } else {
                            self.tokens[self.index - 1].span
                        };
                        return Err(Error {
                            kind: ErrorKind::InvalidToken,
                            span,
                        });
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
        self.parse_expr_precedence(0)
    }

    fn parse(&mut self) -> Result<Expr> {
        let imports = if let Some(imports) = self.parse_imports()? {
            self.accept(TokenKind::Comma);
            imports
        } else {
            Vec::new()
        };
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
        if exprs.len() > 0 {
            Ok(Expr::Block(imports, lets, exprs))
        } else {
            Ok(Expr::Void)
        }
    }
}

pub fn parse(source: &str) -> Result<Expr> {
    Parser {
        source,
        tokens: tokenize(source)?,
        index: 0,
    }
    .parse()
}

pub fn parse_expr(source: &str) -> Result<Expr> {
    let expr = Parser {
        source,
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

    #[test]
    fn test_parse_identifier() {
        assert_eq!(parse_expr("x"), Ok(Expr::Identifier("x".into())));
    }

    #[test]
    fn test_parse_tag() {
        assert_eq!(parse_expr("X"), Ok(Expr::Tag("X".into())));
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(parse_expr(r#""""#), Ok(Expr::String("".into())));
        assert_eq!(parse_expr(r#""\"""#), Ok(Expr::String("\"".into())));
        assert_eq!(parse_expr(r#""\\""#), Ok(Expr::String("\\".into())));
        assert_eq!(
            parse_expr(r#""\0\a\b\t\f\v\n\r""#),
            Ok(Expr::String("\0\x07\x08\t\x0A\x0B\n\r".into()))
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
        assert_eq!(parse_expr("-0"), Ok(Expr::Integer(0)));
        assert_eq!(parse_expr("7"), Ok(Expr::Integer(7)));
        assert_eq!(parse_expr("-3"), Ok(Expr::Integer(-3)));
        assert_eq!(parse_expr("123"), Ok(Expr::Integer(123)));
        assert_eq!(parse_expr("-313"), Ok(Expr::Integer(-313)));
        assert_eq!(parse_expr("000747"), Ok(Expr::Integer(747)));
        assert_eq!(parse_expr("-002200"), Ok(Expr::Integer(-2200)));
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
    }

    #[test]
    fn test_parse_float() {
        assert_eq!(parse_expr("0.0"), Ok(Expr::Float(0.0)));
        assert_eq!(parse_expr("-0.0"), Ok(Expr::Float(0.0)));
        assert_eq!(parse_expr("1.0"), Ok(Expr::Float(1.0)));
        assert_eq!(parse_expr("-1.0"), Ok(Expr::Float(-1.0)));
        assert_eq!(parse_expr("3.141592"), Ok(Expr::Float(3.141592)));
        assert_eq!(parse_expr("-2.7800000"), Ok(Expr::Float(-2.78)));
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
                tag: Some("Point".into()),
                positional: vec![Expr::Integer(1), Expr::Integer(2)],
                named: vec![],
            }))
        );
        assert_eq!(
            parse_expr("(x: 1, y: 2)"),
            Ok(Expr::Tuple(Tuple {
                tag: None,
                positional: vec![],
                named: vec![
                    ("x".into(), Expr::Integer(1)),
                    ("y".into(), Expr::Integer(2)),
                ]
            }))
        );
        assert_eq!(
            parse_expr(r#"Person("id", name: "Bob", age: 49)"#),
            Ok(Expr::Tuple(Tuple {
                tag: Some("Person".into()),
                positional: vec![Expr::String("id".into())],
                named: vec![
                    ("name".into(), Expr::String("Bob".into())),
                    ("age".into(), Expr::Integer(49))
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
    fn test_operators() {
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
        )
    }

    #[test]
    fn test_parse_block() {
        assert_eq!(parse_expr("{}"), Ok(Expr::Void));
        assert_eq!(parse(""), Ok(Expr::Void));
        assert_eq!(
            parse("let x = 0, let y = 1, [x, y]"),
            Ok(Expr::Block(
                vec![],
                vec![
                    Let(Pattern::Identifier("x".into()), Expr::Integer(0).into()),
                    Let(Pattern::Identifier("y".into()), Expr::Integer(1).into())
                ],
                vec![Expr::Vector(vec![
                    Expr::Identifier("x".into()),
                    Expr::Identifier("y".into())
                ])]
            ))
        );
        assert_eq!(
            parse("{{1}}"),
            Ok(Expr::Block(
                vec![],
                vec![],
                vec![Expr::Block(
                    vec![],
                    vec![],
                    vec![Expr::Block(vec![], vec![], vec![Expr::Integer(1)])]
                )]
            ))
        );
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
                        Pattern::Identifier("a".into()),
                        Expr::String("test".into()).into()
                    ),
                    Let(
                        Pattern::Identifier("b".into()),
                        Expr::String("test".into()).into(),
                    )
                ],
                vec![Expr::Tuple(Tuple {
                    tag: None,
                    positional: vec![Expr::Identifier("a".into()), Expr::Identifier("b".into())],
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
                vec![
                    Import("x".into(), "x".into()),
                    Import("y".into(), "y".into())
                ],
                vec![Let(
                    Pattern::Identifier("point".into()),
                    Expr::Tuple(Tuple {
                        tag: Some("Point".into()),
                        positional: vec![
                            Expr::Identifier("x".into()),
                            Expr::Identifier("y".into())
                        ],
                        named: vec![],
                    })
                    .into()
                )],
                vec![Expr::Identifier("point".into())]
            ))
        );
    }
}

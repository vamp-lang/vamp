use crate::source::{Error, ErrorKind, Result, Span};
use crate::tokens::{tokenize, Token, TokenKind};
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct PatternTuple {
    pub tag: Option<String>,
    pub positional: Vec<Pattern>,
    pub named: HashMap<String, Pattern>,
}

#[derive(Debug, PartialEq)]
pub enum Pattern {
    Tuple(PatternTuple),
    Vector(Vec<Pattern>),
    Identifier(String),
    Tag(String),
}

#[derive(Debug, PartialEq)]
pub struct Let(Pattern, Box<Expr>);

#[derive(Debug, PartialEq)]
pub struct Tuple {
    pub tag: Option<String>,
    pub positional: Vec<Expr>,
    pub named: HashMap<String, Expr>,
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Void,
    Nil,
    Block(Vec<Let>, Box<Expr>),
    Function(Pattern, Box<Expr>),
    Tuple(Tuple),
    Vector(Vec<Expr>),
    Map(Vec<(Expr, Expr)>),
    Identifier(String),
    Tag(String),
    String(String),
    Integer(i64),
    Float(f64),
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

    fn parse_tuple(&mut self) -> Result<Option<Expr>> {
        if let Some(left_parenthesis_span) = self.accept(TokenKind::LeftParenthesis) {
            let mut expressions = Vec::new();
            if let Some(expression) = self.parse_expression()? {
                expressions.push(expression);
                while self.accept(TokenKind::Comma).is_some() {
                    if let Some(expression) = self.parse_expression()? {
                        expressions.push(expression);
                    }
                }
            }
            self.accept(TokenKind::RightParenthesis).ok_or(Error {
                kind: ErrorKind::UnbalancedDelimiters,
                span: left_parenthesis_span,
            })?;
            let tuple = Tuple {
                tag: None,
                positional: expressions,
                named: HashMap::default(),
            };
            Ok(Some(Expr::Tuple(tuple)))
        } else {
            Ok(None)
        }
    }

    fn parse_vector(&mut self) -> Result<Option<Expr>> {
        if let Some(left_bracket_span) = self.accept(TokenKind::LeftBracket) {
            let mut expressions = Vec::new();
            if let Some(expression) = self.parse_expression()? {
                expressions.push(expression);
                while self.accept(TokenKind::Comma).is_some() {
                    if let Some(expression) = self.parse_expression()? {
                        expressions.push(expression);
                    }
                }
            }
            self.accept(TokenKind::RightBracket).ok_or(Error {
                kind: ErrorKind::UnbalancedDelimiters,
                span: left_bracket_span,
            })?;
            Ok(Some(Expr::Vector(expressions)))
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
            let expression = self.parse_expression()?.ok_or(Error {
                kind: ErrorKind::InvalidToken,
                span: let_span,
            })?;
            Ok(Some(Let(pattern, expression.into())))
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

    fn parse_expression(&mut self) -> Result<Option<Expr>> {
        if let Some(identifier) = self.parse_identifier() {
            Ok(Some(Expr::Identifier(identifier)))
        } else if let Some(tag) = self.parse_tag() {
            Ok(Some(Expr::Tag(tag)))
        } else if let Some(string) = self.parse_string()? {
            Ok(Some(Expr::String(string)))
        } else if let Some(integer) = self.parse_integer()? {
            Ok(Some(Expr::Integer(integer)))
        } else if let Some(float) = self.parse_float()? {
            Ok(Some(Expr::Float(float)))
        } else if let Some(tuple) = self.parse_tuple()? {
            Ok(Some(tuple))
        } else if let Some(vector) = self.parse_vector()? {
            Ok(Some(vector))
        } else if let Some(block) = self.parse_block()? {
            Ok(Some(block))
        } else {
            Ok(None)
        }
    }

    fn parse(&mut self) -> Result<Expr> {
        let mut lets = Vec::new();
        if let Some(let_) = self.parse_let()? {
            lets.push(let_);
            while self.accept(TokenKind::Comma).is_some() {
                if let Some(let_) = self.parse_let()? {
                    lets.push(let_);
                }
            }
        }
        let expression = self.parse_expression()?.unwrap_or(Expr::Nil);
        Ok(Expr::Block(lets, expression.into()))
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

pub fn parse_expression(source: &str) -> Result<Expr> {
    let expression = Parser {
        source,
        tokens: tokenize(source)?,
        index: 0,
    }
    .parse_expression()?
    .unwrap_or(Expr::Nil);
    Ok(expression)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_identifier() {
        assert_eq!(parse_expression("x"), Ok(Expr::Identifier("x".into())));
    }

    #[test]
    fn test_parse_tag() {
        assert_eq!(parse_expression("X"), Ok(Expr::Tag("X".into())));
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(parse_expression(r#""""#), Ok(Expr::String("".into())));
        assert_eq!(parse_expression(r#""\"""#), Ok(Expr::String("\"".into())));
        assert_eq!(parse_expression(r#""\\""#), Ok(Expr::String("\\".into())));
        assert_eq!(
            parse_expression(r#""\0\a\b\t\f\v\n\r""#),
            Ok(Expr::String("\0\x07\x08\t\x0A\x0B\n\r".into()))
        );
        assert_eq!(
            parse_expression(r#""\x00\x01\x02\x03\x04\x05""#),
            Ok(Expr::String("\x00\x01\x02\x03\x04\x05".into()))
        );
        assert_eq!(
            parse_expression(r#""\z""#).unwrap_err().kind,
            ErrorKind::InvalidEscapeSequence
        );
        assert_eq!(
            parse_expression(r#""\xFF""#).unwrap_err().kind,
            ErrorKind::InvalidEscapeSequence
        );
    }

    #[test]
    fn test_parse_integer() {
        assert_eq!(parse_expression("0"), Ok(Expr::Integer(0)));
        assert_eq!(parse_expression("-0"), Ok(Expr::Integer(0)));
        assert_eq!(parse_expression("7"), Ok(Expr::Integer(7)));
        assert_eq!(parse_expression("-3"), Ok(Expr::Integer(-3)));
        assert_eq!(parse_expression("123"), Ok(Expr::Integer(123)));
        assert_eq!(parse_expression("-313"), Ok(Expr::Integer(-313)));
        assert_eq!(parse_expression("000747"), Ok(Expr::Integer(747)));
        assert_eq!(parse_expression("-002200"), Ok(Expr::Integer(-2200)));
        assert_eq!(
            parse_expression("9223372036854775807"),
            Ok(Expr::Integer(9223372036854775807))
        );
        assert_eq!(
            parse_expression("9223372036854775808").unwrap_err().kind,
            ErrorKind::InvalidInteger
        );
        assert_eq!(
            parse_expression("-9223372036854775808"),
            Ok(Expr::Integer(-9223372036854775808))
        );
        assert_eq!(
            parse_expression("-9223372036854775809").unwrap_err().kind,
            ErrorKind::InvalidInteger
        );
    }

    #[test]
    fn test_parse_float() {
        assert_eq!(parse_expression("0.0"), Ok(Expr::Float(0.0)));
        assert_eq!(parse_expression("-0.0"), Ok(Expr::Float(0.0)));
        assert_eq!(parse_expression("1.0"), Ok(Expr::Float(1.0)));
        assert_eq!(parse_expression("-1.0"), Ok(Expr::Float(-1.0)));
        assert_eq!(parse_expression("3.141592"), Ok(Expr::Float(3.141592)));
        assert_eq!(parse_expression("-2.7800000"), Ok(Expr::Float(-2.78)));
    }

    #[test]
    fn test_parse_tuple() {
        assert_eq!(
            parse_expression("(1)"),
            Ok(Expr::Tuple(Tuple {
                tag: None,
                positional: vec![Expr::Integer(1)],
                named: HashMap::new()
            }))
        );
        assert_eq!(
            parse_expression("(1, 2, 3)"),
            Ok(Expr::Tuple(Tuple {
                tag: None,
                positional: vec![Expr::Integer(1), Expr::Integer(2), Expr::Integer(3)],
                named: HashMap::new()
            }))
        );
    }

    #[test]
    fn test_parse_vector() {
        assert_eq!(
            parse_expression("[1]"),
            Ok(Expr::Vector(vec![Expr::Integer(1)]))
        );
        assert_eq!(
            parse_expression("[1, 2, 3]"),
            Ok(Expr::Vector(vec![
                Expr::Integer(1),
                Expr::Integer(2),
                Expr::Integer(3)
            ]))
        )
    }

    #[test]
    fn test_parse_block() {
        assert_eq!(
            parse("let x = 0, let y = 1, [x, y]"),
            Ok(Expr::Block(
                vec![
                    Let(Pattern::Identifier("x".into()), Expr::Integer(0).into()),
                    Let(Pattern::Identifier("y".into()), Expr::Integer(1).into())
                ],
                Expr::Vector(vec![
                    Expr::Identifier("x".into()),
                    Expr::Identifier("y".into())
                ])
                .into()
            ))
        );
        assert_eq!(
            parse("{{1}}"),
            Ok(Expr::Block(
                vec![],
                Expr::Block(vec![], Expr::Block(vec![], Expr::Integer(1).into()).into()).into()
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
                Expr::Tuple(Tuple {
                    tag: None,
                    positional: vec![Expr::Identifier("a".into()), Expr::Identifier("b".into())],
                    named: HashMap::default()
                })
                .into()
            ))
        );
    }
}

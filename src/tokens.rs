use crate::source::{Error, ErrorKind, Position, Result, Span};

fn is_whitespace(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\n' | b'\r')
}

fn is_identifier_first(c: u8) -> bool {
    matches!(c, b'a'..=b'z' | b'_')
}

fn is_identifier_rest(c: u8) -> bool {
    matches!(c, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_')
}

fn is_symbol_first(c: u8) -> bool {
    matches!(c, b'A'..=b'Z')
}

fn is_symbol_rest(c: u8) -> bool {
    matches!(c, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_')
}

fn is_digit(c: u8) -> bool {
    matches!(c, b'0'..=b'9')
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum TokenKind {
    LeftParenthesis,
    RightParenthesis,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Comma,
    Colon,
    Equals,
    Plus,
    Minus,
    Times,
    Divide,
    Arrow,
    Identifier,
    Tag,
    Integer,
    Float,
    String,
    Let,
}

#[derive(Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

pub struct Tokens<'source> {
    source: &'source [u8],
    byte: u8,
    start: Position,
    end: Position,
    last_token_kind: Option<TokenKind>,
    auto_insert_comma: bool,
}

impl<'source> Tokens<'source> {
    pub fn new(source: &str) -> Tokens {
        let bytes = source.as_bytes();
        Tokens {
            source: bytes,
            byte: *bytes.first().unwrap_or(&b'\0'),
            start: Position {
                offset: 0,
                line: 1,
                column: 1,
            },
            end: Position {
                offset: 0,
                line: 1,
                column: 1,
            },
            last_token_kind: None,
            auto_insert_comma: false,
        }
    }

    fn span(&self) -> Span {
        Span {
            start: self.start,
            end: Position {
                offset: self.end.offset,
                // Make end column inclusive for error reporting.
                column: self.end.column - 1,
                line: self.end.line,
            },
        }
    }

    fn advance(&mut self) {
        if self.byte == b'\n' {
            self.end.line += 1;
            self.end.column = 1;
            self.auto_insert_comma = matches!(
                self.last_token_kind,
                Some(TokenKind::RightParenthesis)
                    | Some(TokenKind::RightBracket)
                    | Some(TokenKind::RightBrace)
                    | Some(TokenKind::Identifier)
                    | Some(TokenKind::Tag)
                    | Some(TokenKind::Integer)
                    | Some(TokenKind::Float)
                    | Some(TokenKind::String)
            );
        } else {
            self.end.column += 1;
        }
        self.end.offset += 1;
        self.byte = *self.source.get(self.end.offset).unwrap_or(&b'\0');
    }

    fn accept_if<P>(&mut self, p: P) -> bool
    where
        P: FnOnce(u8) -> bool,
    {
        if p(self.byte) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn accept_while<P>(&mut self, p: P)
    where
        P: Fn(u8) -> bool,
    {
        while p(self.byte) {
            self.advance();
        }
    }

    fn ok(&mut self, kind: TokenKind) -> Option<Result<Token>> {
        self.last_token_kind = Some(kind);
        Some(Ok(Token {
            kind,
            span: self.span(),
        }))
    }

    fn err(&self, kind: ErrorKind) -> Option<Result<Token>> {
        Some(Err(Error {
            kind,
            span: self.span(),
        }))
    }

    fn skip_whitespace(&mut self) {
        loop {
            self.accept_while(is_whitespace);
            if self.accept_if(|c| c == b'#') {
                self.accept_while(|c| c != b'\n');
            } else {
                break;
            }
        }
    }

    fn next_punctuation(&mut self) -> Option<Result<Token>> {
        if self.accept_if(|c| c == b'(') {
            self.ok(TokenKind::LeftParenthesis)
        } else if self.accept_if(|c| c == b')') {
            self.ok(TokenKind::RightParenthesis)
        } else if self.accept_if(|c| c == b'[') {
            self.ok(TokenKind::LeftBracket)
        } else if self.accept_if(|c| c == b']') {
            self.ok(TokenKind::RightBracket)
        } else if self.accept_if(|c| c == b'{') {
            self.ok(TokenKind::LeftBrace)
        } else if self.accept_if(|c| c == b'}') {
            self.ok(TokenKind::RightBrace)
        } else if self.accept_if(|c| c == b',') {
            self.ok(TokenKind::Comma)
        } else if self.accept_if(|c| c == b':') {
            self.ok(TokenKind::Colon)
        } else if self.accept_if(|c| c == b'=') {
            self.ok(TokenKind::Equals)
        } else if self.accept_if(|c| c == b'+') {
            self.ok(TokenKind::Plus)
        } else if self.accept_if(|c| c == b'-') {
            if self.accept_if(|c| c == b'>') {
                self.ok(TokenKind::Arrow)
            } else {
                self.ok(TokenKind::Minus)
            }
        } else if self.accept_if(|c| c == b'*') {
            self.ok(TokenKind::Times)
        } else if self.accept_if(|c| c == b'/') {
            self.ok(TokenKind::Divide)
        } else {
            None
        }
    }

    fn next_identifier(&mut self) -> Option<Result<Token>> {
        if self.accept_if(is_identifier_first) {
            self.accept_while(is_identifier_rest);
            self.ok(match &self.source[self.start.offset..self.end.offset] {
                b"let" => TokenKind::Let,
                _ => TokenKind::Identifier,
            })
        } else {
            None
        }
    }

    fn next_tag(&mut self) -> Option<Result<Token>> {
        if self.accept_if(is_symbol_first) {
            self.accept_while(is_symbol_rest);
            self.ok(TokenKind::Tag)
        } else {
            None
        }
    }

    fn next_number(&mut self) -> Option<Result<Token>> {
        if self.accept_if(is_digit) {
            self.accept_while(is_digit);
            if self.accept_if(|c| c == b'.') {
                self.accept_while(is_digit);
                self.ok(TokenKind::Float)
            } else {
                self.ok(TokenKind::Integer)
            }
        } else {
            None
        }
    }

    fn next_string(&mut self) -> Option<Result<Token>> {
        if self.accept_if(|c| c == b'"') {
            loop {
                if self.byte == b'\0' {
                    return self.err(ErrorKind::UnterminatedString);
                } else if self.accept_if(|c| c == b'\\') {
                    if !self.accept_if(|c| c != b'\0') {
                        return self.err(ErrorKind::UnterminatedString);
                    }
                } else if self.accept_if(|c| c == b'"') {
                    return self.ok(TokenKind::String);
                } else {
                    self.advance();
                }
            }
        } else {
            None
        }
    }

    fn next_error(&mut self) -> Option<Result<Token>> {
        if self.byte != b'\0' {
            self.advance();
            self.err(ErrorKind::InvalidCharacter)
        } else {
            None
        }
    }
}

impl<'source> Iterator for Tokens<'source> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();
        self.start = self.end;

        if self.auto_insert_comma {
            let comma = self.ok(TokenKind::Comma);
            self.auto_insert_comma = false;
            return comma;
        }

        self.next_punctuation()
            .or_else(|| self.next_identifier())
            .or_else(|| self.next_tag())
            .or_else(|| self.next_number())
            .or_else(|| self.next_string())
            .or_else(|| self.next_error())
    }
}

pub fn tokenize(source: &str) -> Result<Vec<Token>> {
    Tokens::new(source).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token_slices(source: &str) -> Result<Vec<(TokenKind, &str)>> {
        Tokens::new(source)
            .map(|result| {
                result
                    .map(|Token { kind, span }| (kind, &source[span.start.offset..span.end.offset]))
            })
            .collect()
    }

    #[test]
    fn test_whitespace() {
        assert_eq!(token_slices(" \t\n\r"), Ok(vec![]));
        assert_eq!(
            token_slices("# This is a comment\n# This is another comment\n"),
            Ok(vec![])
        );
    }

    #[test]
    fn test_punctuation() {
        assert_eq!(
            token_slices("( ) [ ] { } , : = + - * / ->"),
            Ok(vec![
                (TokenKind::LeftParenthesis, "("),
                (TokenKind::RightParenthesis, ")"),
                (TokenKind::LeftBracket, "["),
                (TokenKind::RightBracket, "]"),
                (TokenKind::LeftBrace, "{"),
                (TokenKind::RightBrace, "}"),
                (TokenKind::Comma, ","),
                (TokenKind::Colon, ":"),
                (TokenKind::Equals, "="),
                (TokenKind::Plus, "+"),
                (TokenKind::Minus, "-"),
                (TokenKind::Times, "*"),
                (TokenKind::Divide, "/"),
                (TokenKind::Arrow, "->"),
            ])
        );
    }

    #[test]
    fn test_auto_insert_comma() {
        assert_eq!(
            token_slices(
                "
                x
                y
                z
                "
            ),
            Ok(vec![
                (TokenKind::Identifier, "x"),
                (TokenKind::Comma, ""),
                (TokenKind::Identifier, "y"),
                (TokenKind::Comma, ""),
                (TokenKind::Identifier, "z"),
                (TokenKind::Comma, "")
            ]),
        );
    }

    #[test]
    fn test_identifiers() {
        assert_eq!(
            token_slices("_ t x1 emailAddress first_name _dateOfBirth"),
            Ok(vec![
                (TokenKind::Identifier, "_"),
                (TokenKind::Identifier, "t"),
                (TokenKind::Identifier, "x1"),
                (TokenKind::Identifier, "emailAddress"),
                (TokenKind::Identifier, "first_name"),
                (TokenKind::Identifier, "_dateOfBirth"),
            ])
        );
    }

    #[test]
    fn test_tags() {
        assert_eq!(
            token_slices("T X1 Symbol SHIFT_RIGHT"),
            Ok(vec![
                (TokenKind::Tag, "T"),
                (TokenKind::Tag, "X1"),
                (TokenKind::Tag, "Symbol"),
                (TokenKind::Tag, "SHIFT_RIGHT")
            ])
        );
    }

    #[test]
    fn test_integers() {
        assert_eq!(
            token_slices("0 12 539"),
            Ok(vec![
                (TokenKind::Integer, "0"),
                (TokenKind::Integer, "12"),
                (TokenKind::Integer, "539"),
            ])
        );
    }

    #[test]
    fn test_floats() {
        assert_eq!(
            token_slices("0. 0.5 3.14"),
            Ok(vec![
                (TokenKind::Float, "0."),
                (TokenKind::Float, "0.5"),
                (TokenKind::Float, "3.14"),
            ])
        );
    }

    #[test]
    fn test_strings() {
        assert_eq!(
            token_slices(r#""" "\\" "\\\"" "\"\"" "The quick brown fox jumps over the lazy dog.""#),
            Ok(vec![
                (TokenKind::String, r#""""#),
                (TokenKind::String, r#""\\""#),
                (TokenKind::String, r#""\\\"""#),
                (TokenKind::String, r#""\"\"""#),
                (
                    TokenKind::String,
                    r#""The quick brown fox jumps over the lazy dog.""#
                ),
            ])
        );
    }

    #[test]
    fn test_unterminated_string() {
        assert_eq!(
            token_slices("\""),
            Err(Error {
                kind: ErrorKind::UnterminatedString,
                span: Span {
                    start: Position {
                        offset: 0,
                        line: 1,
                        column: 1,
                    },
                    end: Position {
                        offset: 1,
                        line: 1,
                        column: 1,
                    },
                },
            })
        );
    }
}

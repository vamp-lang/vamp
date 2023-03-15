use crate::source::{Error, ErrorKind, Result, Span};

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum TokenKind {
    // Punctuation
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Colon,
    Equals,
    Plus,
    Minus,
    Times,
    Divide,
    Arrow,

    // Keywords
    Import,
    Export,
    Use,
    Let,
    If,
    Else,
    For,

    // Identifiers
    Identifier,

    // Literals
    Symbol,
    String,
    Int,
    Float,
}

#[derive(Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

pub struct Tokens<'a> {
    source: &'a str,
    span: Span,
    last_token_kind: Option<TokenKind>,
    auto_insert_comma: bool,
}

impl<'a> Tokens<'a> {
    pub fn new(source: &str) -> Tokens {
        Tokens {
            source,
            span: Span::default(),
            last_token_kind: None,
            auto_insert_comma: false,
        }
    }

    fn first(&self) -> u8 {
        *self.source.as_bytes().get(self.span.end).unwrap_or(&b'\0')
    }

    fn second(&self) -> u8 {
        *self
            .source
            .as_bytes()
            .get(self.span.end + 1)
            .unwrap_or(&b'\0')
    }

    fn bump(&mut self) {
        if self.first() == b'\n' {
            self.auto_insert_comma = matches!(
                self.last_token_kind,
                Some(
                    TokenKind::RParen
                        | TokenKind::RBracket
                        | TokenKind::RBrace
                        | TokenKind::Identifier
                        | TokenKind::Symbol
                        | TokenKind::Int
                        | TokenKind::Float
                        | TokenKind::String
                )
            );
        }
        self.span.end += 1;
    }

    #[inline]
    fn bump_if(&mut self, f: impl FnOnce(u8) -> bool) -> bool {
        if f(self.first()) {
            self.bump();
            true
        } else {
            false
        }
    }

    #[inline]
    fn bump_while(&mut self, f: impl Fn(u8) -> bool) {
        while f(self.first()) {
            self.bump();
        }
    }

    #[inline]
    fn ok(&mut self, kind: TokenKind) -> Option<Result<Token>> {
        self.last_token_kind = Some(kind);
        Some(Ok(Token {
            kind,
            span: self.span,
        }))
    }

    #[inline]
    fn err(&self, kind: ErrorKind) -> Option<Result<Token>> {
        Some(Err(Error {
            kind,
            span: self.span,
        }))
    }

    fn whitespace(&mut self) {
        loop {
            self.bump_while(|c| c.is_ascii_whitespace());
            if self.bump_if(|c| c == b'#') {
                self.bump_while(|c| c != b'\n');
            } else {
                break;
            }
        }
    }

    fn punctuation(&mut self) -> Option<Result<Token>> {
        if self.bump_if(|c| c == b'(') {
            self.ok(TokenKind::LParen)
        } else if self.bump_if(|c| c == b')') {
            self.ok(TokenKind::RParen)
        } else if self.bump_if(|c| c == b'[') {
            self.ok(TokenKind::LBracket)
        } else if self.bump_if(|c| c == b']') {
            self.ok(TokenKind::RBracket)
        } else if self.bump_if(|c| c == b'{') {
            self.ok(TokenKind::LBrace)
        } else if self.bump_if(|c| c == b'}') {
            self.ok(TokenKind::RBrace)
        } else if self.bump_if(|c| c == b',') {
            self.ok(TokenKind::Comma)
        } else if self.bump_if(|c| c == b':') {
            self.ok(TokenKind::Colon)
        } else if self.bump_if(|c| c == b'=') {
            self.ok(TokenKind::Equals)
        } else if self.bump_if(|c| c == b'+') {
            self.ok(TokenKind::Plus)
        } else if self.bump_if(|c| c == b'-') {
            if self.bump_if(|c| c == b'>') {
                self.ok(TokenKind::Arrow)
            } else {
                self.ok(TokenKind::Minus)
            }
        } else if self.bump_if(|c| c == b'*') {
            self.ok(TokenKind::Times)
        } else if self.bump_if(|c| c == b'/') {
            self.ok(TokenKind::Divide)
        } else {
            None
        }
    }

    fn identifier(&mut self) -> Option<Result<Token>> {
        if self.bump_if(|c| matches!(c, b'A'..=b'Z' | b'a'..=b'z' | b'_')) {
            self.bump_while(|c| matches!(c, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_'));
            self.ok(match &self.source[self.span] {
                "import" => TokenKind::Import,
                "export" => TokenKind::Export,
                "use" => TokenKind::Use,
                "let" => TokenKind::Let,
                "if" => TokenKind::If,
                "else" => TokenKind::Else,
                "for" => TokenKind::For,
                _ => TokenKind::Identifier,
            })
        } else {
            None
        }
    }

    fn symbol_or_string(&mut self) -> Option<Result<Token>> {
        if matches!(self.first(), b'\'' | b'"') {
            let delimiter = self.first();
            let kind = if delimiter == b'\'' {
                TokenKind::Symbol
            } else {
                TokenKind::String
            };
            self.bump();
            loop {
                if self.first() == b'\0' {
                    return self.err(ErrorKind::StringUnterminated);
                } else if self.bump_if(|c| c == b'\\') {
                    if !self.bump_if(|c| c != b'\0') {
                        return self.err(ErrorKind::StringUnterminated);
                    }
                } else if self.bump_if(|c| c == delimiter) {
                    return self.ok(kind);
                } else {
                    self.bump();
                }
            }
        } else {
            None
        }
    }

    fn int_or_float(&mut self) -> Option<Result<Token>> {
        if self.first() == b'0' {
            match self.second() {
                // Binary literal
                b'b' => {
                    self.bump();
                    self.bump();
                    self.bump_while(|c| matches!(c, b'0' | b'1'));
                    return self.ok(TokenKind::Int);
                }
                // Octal literal
                b'0' => {
                    self.bump();
                    self.bump();
                    self.bump_while(|c| matches!(c, b'0'..=b'7'));
                    return self.ok(TokenKind::Int);
                }
                // Hexadecimal literal
                b'x' => {
                    self.bump();
                    self.bump();
                    self.bump_while(|c| matches!(c, b'A'..=b'F' | b'a'..=b'f' | b'0'..=b'9'));
                    return self.ok(TokenKind::Int);
                }
                _ => {}
            }
        }
        if self.bump_if(|c| c.is_ascii_digit()) {
            self.bump_while(|c| c.is_ascii_digit());
            if self.bump_if(|c| c == b'.') {
                self.bump_while(|c| c.is_ascii_digit());
                if self.bump_if(|c| c == b'e') {
                    self.bump_if(|c| c == b'-');
                    self.bump_while(|c| c.is_ascii_digit());
                }
                self.ok(TokenKind::Float)
            } else if self.bump_if(|c| c == b'e') {
                self.bump_if(|c| c == b'-');
                self.bump_while(|c| c.is_ascii_digit());
                self.ok(TokenKind::Float)
            } else {
                self.ok(TokenKind::Int)
            }
        } else {
            None
        }
    }

    fn error(&mut self) -> Option<Result<Token>> {
        if self.bump_if(|c| c != b'\0') {
            self.err(ErrorKind::InvalidCharacter)
        } else {
            None
        }
    }
}

impl<'a> Iterator for Tokens<'a> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        self.whitespace();
        self.span.start = self.span.end;

        if self.auto_insert_comma {
            let comma = self.ok(TokenKind::Comma);
            self.auto_insert_comma = false;
            return comma;
        }

        self.punctuation()
            .or_else(|| self.identifier())
            .or_else(|| self.symbol_or_string())
            .or_else(|| self.int_or_float())
            .or_else(|| self.error())
    }
}

const AVERAGE_TOKEN_LEN: usize = 128;

pub fn tokenize(source: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::with_capacity(source.len() / AVERAGE_TOKEN_LEN);
    for token in Tokens::new(source) {
        tokens.push(token?)
    }
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token_slices(source: &str) -> Result<Vec<(TokenKind, &str)>> {
        let tokens = tokenize(source)?;
        Ok(tokens
            .into_iter()
            .map(|Token { kind, span }| (kind, &source[span]))
            .collect())
    }

    #[test]
    fn whitespace() {
        assert_eq!(token_slices(" \t\n\r"), Ok(vec![]));
        assert_eq!(
            token_slices("# This is a comment\n# This is another comment\n"),
            Ok(vec![])
        );
    }

    #[test]
    fn valid_tokens() {
        let cases = [
            // Punctuation
            (TokenKind::LParen, "("),
            (TokenKind::RParen, ")"),
            (TokenKind::LBracket, "["),
            (TokenKind::RBracket, "]"),
            (TokenKind::LBrace, "{"),
            (TokenKind::RBrace, "}"),
            (TokenKind::Comma, ","),
            (TokenKind::Colon, ":"),
            (TokenKind::Equals, "="),
            (TokenKind::Plus, "+"),
            (TokenKind::Minus, "-"),
            (TokenKind::Times, "*"),
            (TokenKind::Divide, "/"),
            (TokenKind::Arrow, "->"),
            // Keywords
            (TokenKind::Import, "import"),
            (TokenKind::Export, "export"),
            (TokenKind::Use, "use"),
            (TokenKind::Let, "let"),
            (TokenKind::If, "if"),
            (TokenKind::Else, "else"),
            (TokenKind::For, "for"),
            // Identifiers
            (TokenKind::Identifier, "_"),
            (TokenKind::Identifier, "t"),
            (TokenKind::Identifier, "x1"),
            (TokenKind::Identifier, "emailAddress"),
            (TokenKind::Identifier, "first_name"),
            (TokenKind::Identifier, "_dateOfBirth"),
            (TokenKind::Identifier, "T"),
            (TokenKind::Identifier, "X1"),
            (TokenKind::Identifier, "Identifier"),
            (TokenKind::Identifier, "SHIFT_RIGHT"),
            // Symbol literals
            (TokenKind::Symbol, "''"),
            (TokenKind::Symbol, "'_'"),
            (TokenKind::Symbol, r#"'\''"#),
            (TokenKind::Symbol, "'abc'"),
            // String literals
            (TokenKind::String, r#""""#),
            (TokenKind::String, r#""\\""#),
            (TokenKind::String, r#""\\\"""#),
            (TokenKind::String, r#""\"\"""#),
            (
                TokenKind::String,
                r#""The quick brown fox jumps over the lazy dog.""#,
            ),
            // Int literals
            (TokenKind::Int, "0"),
            (TokenKind::Int, "12"),
            (TokenKind::Int, "539"),
            (TokenKind::Int, "0777"),
            (TokenKind::Int, "0b1010"),
            (TokenKind::Int, "0xfAb93"),
            // Float literals
            (TokenKind::Float, "0."),
            (TokenKind::Float, "0.5"),
            (TokenKind::Float, "3.14"),
            (TokenKind::Float, "1e10"),
            (TokenKind::Float, "2.5e2"),
            (TokenKind::Float, "1e-10"),
        ];
        for (kind, slice) in cases {
            assert_eq!(token_slices(slice), Ok(vec![(kind, slice)]));
        }
    }

    #[test]
    fn auto_insert_comma() {
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
    fn string_unterminated() {
        assert!(matches!(
            token_slices("\""),
            Err(Error {
                kind: ErrorKind::StringUnterminated,
                span: _,
            })
        ));
    }
}

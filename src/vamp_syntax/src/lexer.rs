use crate::{
    error::{Error, ErrorKind, Result},
    span::Span,
};

#[cfg(test)]
mod tests;

#[derive(Debug, PartialEq, Clone, Copy)]
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
    Period,

    // Operators
    Plus,
    Minus,
    Star,
    StarStar,
    Slash,
    Percent,
    Eq,
    EqEq,
    NotEq,
    Lt,
    LtLt,
    LtEq,
    Gt,
    GtGt,
    GtEq,
    Not,
    And,
    AndAnd,
    Or,
    OrOr,
    Caret,
    Tilde,

    // Keywords
    Use,
    Let,
    If,
    Else,
    For,

    // Identifiers
    Ident,

    // Literals
    Sym,
    Str,
    Int,
    Float,
    True,
    False,
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
                        | TokenKind::Ident
                        | TokenKind::Sym
                        | TokenKind::Int
                        | TokenKind::Float
                        | TokenKind::Str
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
    fn err(&self, kind: ErrorKind, detail: Option<String>) -> Option<Result<Token>> {
        Some(Err(Error {
            kind,
            detail,
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
        } else if self.bump_if(|c| c == b'.') {
            self.ok(TokenKind::Period)
        } else if self.bump_if(|c| c == b'+') {
            self.ok(TokenKind::Plus)
        } else if self.bump_if(|c| c == b'-') {
            self.ok(TokenKind::Minus)
        } else if self.bump_if(|c| c == b'*') {
            if self.bump_if(|c| c == b'*') {
                self.ok(TokenKind::StarStar)
            } else {
                self.ok(TokenKind::Star)
            }
        } else if self.bump_if(|c| c == b'/') {
            self.ok(TokenKind::Slash)
        } else if self.bump_if(|c| c == b'%') {
            self.ok(TokenKind::Percent)
        } else if self.bump_if(|c| c == b'=') {
            if self.bump_if(|c| c == b'=') {
                self.ok(TokenKind::EqEq)
            } else {
                self.ok(TokenKind::Eq)
            }
        } else if self.bump_if(|c| c == b'!') {
            if self.bump_if(|c| c == b'=') {
                self.ok(TokenKind::NotEq)
            } else {
                self.ok(TokenKind::Not)
            }
        } else if self.bump_if(|c| c == b'>') {
            if self.bump_if(|c| c == b'>') {
                self.ok(TokenKind::GtGt)
            } else if self.bump_if(|c| c == b'=') {
                self.ok(TokenKind::GtEq)
            } else {
                self.ok(TokenKind::Gt)
            }
        } else if self.bump_if(|c| c == b'<') {
            if self.bump_if(|c| c == b'<') {
                self.ok(TokenKind::LtLt)
            } else if self.bump_if(|c| c == b'=') {
                self.ok(TokenKind::LtEq)
            } else {
                self.ok(TokenKind::Lt)
            }
        } else if self.bump_if(|c| c == b'&') {
            if self.bump_if(|c| c == b'&') {
                self.ok(TokenKind::AndAnd)
            } else {
                self.ok(TokenKind::And)
            }
        } else if self.bump_if(|c| c == b'|') {
            if self.bump_if(|c| c == b'|') {
                self.ok(TokenKind::OrOr)
            } else {
                self.ok(TokenKind::Or)
            }
        } else if self.bump_if(|c| c == b'^') {
            self.ok(TokenKind::Caret)
        } else if self.bump_if(|c| c == b'~') {
            self.ok(TokenKind::Tilde)
        } else {
            None
        }
    }

    fn identifier(&mut self) -> Option<Result<Token>> {
        if self.bump_if(|c| matches!(c, b'A'..=b'Z' | b'a'..=b'z' | b'_' | b'@')) {
            self.bump_while(|c| matches!(c, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_'));
            self.ok(match &self.source[self.span] {
                "use" => TokenKind::Use,
                "let" => TokenKind::Let,
                "if" => TokenKind::If,
                "else" => TokenKind::Else,
                "for" => TokenKind::For,
                "true" => TokenKind::True,
                "false" => TokenKind::False,
                _ => TokenKind::Ident,
            })
        } else {
            None
        }
    }

    fn symbol_or_string(&mut self) -> Option<Result<Token>> {
        if matches!(self.first(), b'\'' | b'"') {
            let delimiter = self.first();
            let kind = if delimiter == b'\'' {
                TokenKind::Sym
            } else {
                TokenKind::Str
            };
            self.bump();
            loop {
                if self.first() == b'\0' {
                    return self.err(ErrorKind::StringUnterminated, None);
                } else if self.bump_if(|c| c == b'\\') {
                    if !self.bump_if(|c| c != b'\0') {
                        return self.err(ErrorKind::StringUnterminated, None);
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
                b'o' => {
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
            self.err(ErrorKind::InvalidChar, None)
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

// Average token length used to pre-allocate the token vector based on the
// length of the source string.
const AVERAGE_TOKEN_LEN: usize = 128;

pub fn tokenize(source: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::with_capacity(source.len() / AVERAGE_TOKEN_LEN);
    for token in Tokens::new(source) {
        tokens.push(token?)
    }
    Ok(tokens)
}

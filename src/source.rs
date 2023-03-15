use std::{ops::Index, path::PathBuf};

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Index<Span> for str {
    type Output = str;

    #[inline]
    fn index(&self, span: Span) -> &str {
        &self[span.start..span.end]
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ErrorKind {
    UnbalancedDelimiters,
    InvalidCharacter,
    InvalidToken,
    StringUnterminated,
    StringInvalidEscapeSequence,
    IntegerInvalid,
    FloatInvalid,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Error {
    pub kind: ErrorKind,
    pub span: Span,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum SourceEvent {
    File(PathBuf),
    Repl(String),
    Exit,
}

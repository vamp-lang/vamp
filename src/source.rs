use std::{ops::Index, path::PathBuf};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Position {
    pub offset: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Index<Span> for str {
    type Output = str;

    #[inline(always)]
    fn index(&self, index: Span) -> &str {
        &self[index.start.offset..index.end.offset]
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ErrorKind {
    UnterminatedString,
    InvalidEscapeSequence,
    InvalidInteger,
    InvalidFloat,
    UnbalancedDelimiters,
    InvalidCharacter,
    InvalidToken,
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

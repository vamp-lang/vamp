use crate::span::Span;

/// A type of syntax error.
#[derive(Debug, PartialEq, Clone)]
pub enum ErrorKind {
    UnbalancedDelimiters,
    InvalidCharacter,
    InvalidToken,
    StringUnterminated,
    StringInvalidEscapeSequence,
    IntInvalid,
    FloatInvalid,
    NoUnboundExprAtModuleLevel,
}

/// A syntax error with both type and location.
#[derive(Debug, PartialEq, Clone)]
pub struct Error {
    /// The syntax error kind.
    pub kind: ErrorKind,
    /// Additional details about the syntax error.
    pub detail: Option<String>,
    /// The syntax error's location in the source code.
    pub span: Span,
}

pub type Result<T> = std::result::Result<T, Error>;

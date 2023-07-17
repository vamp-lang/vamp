#[derive(Debug, PartialEq)]
pub enum Error {
    Void,
    Types,
    KeyNotFound,
    Unbound,
    Mismatch,
}

pub type Result<T> = std::result::Result<T, Error>;

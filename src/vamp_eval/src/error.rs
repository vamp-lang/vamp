#[derive(Debug, PartialEq)]
pub enum Error {
    Void,
    Types,
    KeyNotFound,
    Unbound,
}

pub type Result<T> = std::result::Result<T, Error>;

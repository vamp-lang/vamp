#[derive(Debug, PartialEq)]
pub enum Error {
    Void,
    Types,
    KeyNotFound,
}

pub type Result<T> = std::result::Result<T, Error>;

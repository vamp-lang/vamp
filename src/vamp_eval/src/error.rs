#[derive(Debug, PartialEq)]
pub enum Error {
    Void,
    Types,
}

pub type Result<T> = std::result::Result<T, Error>;

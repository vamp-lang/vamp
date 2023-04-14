use vamp_ty::Ty;

#[derive(Debug, PartialEq)]
pub enum Error {
    TypeError { expected: Ty, found: Ty },
}

pub type Result<T> = std::result::Result<T, Error>;

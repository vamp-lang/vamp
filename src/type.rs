#[derive(Debug, PartialEq, Eq)]
pub enum Type {
    Void,
    Nil,
    Tuple(Tuple),
    Int,
    Float,
    Symbol,
    String,
}

use vamp_sym::Sym;
use vamp_syntax::ast::{Expr, Pat};
use vamp_tuple::Tuple;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Sym(Sym),
    Str(String),
    Int(i64),
    Float(f64),
    Tuple(Tuple<Value>),
    List(Vec<Value>),
    Fn(Tuple<Pat>, Box<Expr>),
}

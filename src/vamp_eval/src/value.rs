use vamp_sym::Sym;
use vamp_syntax::ast::{Dep, Expr, Pat};
use vamp_tuple::Tuple;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Bool(bool),
    Sym(Sym),
    Str(String),
    Int(i64),
    Float(f64),
    Tuple(Tuple<Value>),
    List(Vec<Value>),
    Fn(Tuple<Pat>, Box<Expr>),
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct Scope<'a> {
    pub parent: Option<&'a Scope<'a>>,
    pub bindings: Tuple<Value>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Mod<'a> {
    pub deps: Box<[Dep]>,
    pub scope: Scope<'a>,
}

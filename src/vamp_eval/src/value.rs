use crate::error::{Error, Result};
use std::{cell::RefCell, rc::Rc};
use vamp_sym::Sym;
use vamp_syntax::ast::{Dep, Expr, Pat};
use vamp_tuple::{Tuple, TupleEntry};

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Bool(bool),
    Sym(Sym),
    Str(String),
    Int(i64),
    Float(f64),
    Tuple(Tuple<Value>),
    List(Vec<Value>),
    Fn(Fn),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Fn {
    pub params: Tuple<Pat>,
    pub body: Box<Expr>,
    pub scope: Rc<RefCell<Scope>>,
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct Scope {
    pub parent: Option<Rc<RefCell<Scope>>>,
    pub bindings: Tuple<Value>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Mod {
    pub deps: Box<[Dep]>,
    pub scope: Rc<RefCell<Scope>>,
}

impl Scope {
    pub fn new(parent: Option<Rc<RefCell<Scope>>>) -> Self {
        Scope {
            parent,
            bindings: Default::default(),
        }
    }

    pub fn bind(&mut self, name: Sym, value: Value) {
        self.bindings.insert(name, value);
    }

    pub fn lookup(&self, name: Sym) -> Result<Value> {
        match self.bindings.get(name) {
            Some(value) => Ok(value.clone()),
            None => match &self.parent {
                Some(parent) => parent.borrow().lookup(name),
                None => Err(Error::Unbound),
            },
        }
    }
}

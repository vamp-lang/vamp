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

    pub fn bind_tuple(&mut self, pat: &Tuple<Pat>, value: Tuple<Value>) -> Result<()> {
        let mut i = 0usize;
        for entry in pat.iter() {
            match entry {
                TupleEntry::Pos(pat) => {
                    let value = value.get(i).ok_or(Error::Mismatch)?;
                    self.bind(pat, value.clone())?;
                    i += 1;
                }
                TupleEntry::Named(key, pat) => {
                    let value = value.get(key).ok_or(Error::Mismatch)?;
                    self.bind(pat, value.clone())?;
                }
            }
        }
        Ok(())
    }

    pub fn bind(&mut self, pat: &Pat, value: Value) -> Result<()> {
        match pat {
            Pat::Ident(sym) => {
                self.bindings.insert(*sym, value);
                Ok(())
            }
            Pat::Sym(sym) => match value {
                Value::Sym(value) if &value == sym => Ok(()),
                _ => Err(Error::Mismatch),
            },
            Pat::Str(str) => match value {
                Value::Str(value) if &value == str => Ok(()),
                _ => Err(Error::Mismatch),
            },
            Pat::Int(x) => match value {
                Value::Int(value) if &value == x => Ok(()),
                _ => Err(Error::Mismatch),
            },
            Pat::Float(x) => match value {
                Value::Float(value) if &value == x => Ok(()),
                _ => Err(Error::Mismatch),
            },
            Pat::Bool(x) => match value {
                Value::Bool(value) if &value == x => Ok(()),
                _ => Err(Error::Mismatch),
            },
            Pat::Tuple(tuple) => match value {
                Value::Tuple(value) => self.bind_tuple(tuple, value),
                _ => Err(Error::Mismatch),
            },
            /*
            Pat::List(items) => {
                for item in items.into_iter() {
                    self.bind(item, value);
                }
            }*/
            Pat::Wild => Ok(()),
            _ => todo!(),
        }
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

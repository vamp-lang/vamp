use crate::symbol::Symbol;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::rc::Rc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tuple<T> {
    pos: Rc<[T]>,
    named: Rc<FxHashMap<Symbol, T>>,
}

impl<T> Tuple<T> {
    pub fn new(pos: Rc<[T]>, named: Rc<FxHashMap<Symbol, T>>) -> Self {
        Tuple { pos, named }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Val {
    Nil,
    Int(i64),
    Float(f64),
    Function(Rc<[u8]>),
    Tuple(Tuple<Val>),
    Vector(Rc<[Val]>),
    Symbol(Symbol),
    String(Rc<str>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Pat {
    Ignore,
    Nil,
    Int(i64),
    Float(f64),
    Symbol(Symbol),
    String(Rc<str>),
    Identifier(Symbol),
    Tuple(Tuple<Pat>),
    Vector(Rc<[Pat]>),
    Any(Rc<[Pat]>),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Op {
    Exit,
    Noop,
    Push(Val),
    Load(Symbol),
    Store(Symbol),
    Match(Pat),
    Add,
    Sub,
    Mul,
    Div,
    Call,
}

#[derive(Debug, PartialEq)]
pub enum Error {
    BytecodeCorrupt,
    UnresolvedIdentifier,
    MatchError,
    TypeError,
}

#[derive(Default)]
pub struct Vm {
    stack: Vec<Val>,
}

#[derive(Default)]
pub struct Scope<'a> {
    parent: Option<&'a Scope<'a>>,
    bindings: FxHashMap<Symbol, Val>,
}

impl<'a> Scope<'a> {
    fn new(parent: Option<&'a Scope>) -> Self {
        Scope {
            parent,
            bindings: FxHashMap::default(),
        }
    }

    fn load(&self, symbol: Symbol) -> Option<Val> {
        self.bindings
            .get(&symbol)
            .map(|object| object.clone())
            .or_else(|| self.parent?.load(symbol))
    }

    fn store(&mut self, symbol: Symbol, val: Val) {
        self.bindings.insert(symbol, val);
    }

    fn r#match(&mut self, pattern: Pat, val: Val) -> Result<(), Error> {
        match (pattern, val) {
            (Pat::Ignore, _) => Ok(()),
            (Pat::Nil, Val::Nil) => Ok(()),
            (Pat::Int(a), Val::Int(b)) if a == b => Ok(()),
            (Pat::Float(a), Val::Float(b)) if a == b => Ok(()),
            (Pat::Symbol(a), Val::Symbol(b)) if a == b => Ok(()),
            (Pat::String(a), Val::String(b)) if a == b => Ok(()),
            (Pat::Identifier(symbol), val) => {
                self.store(symbol, val);
                Ok(())
            }
            (Pat::Any(patterns), val) => patterns
                .into_iter()
                .try_for_each(|pattern| self.r#match(pattern.clone(), val.clone())),
            _ => Err(Error::MatchError),
        }
    }
}

impl Vm {
    pub fn pop(&mut self) -> Val {
        self.stack.pop().unwrap()
    }

    pub fn push(&mut self, val: Val) {
        self.stack.push(val);
    }

    pub fn run(&mut self, bytecode: &[u8], scope: &mut Scope) -> Result<(), Error> {
        let mut cursor = Cursor::new(bytecode);
        while let Ok(op) = bincode::deserialize_from(&mut cursor) {
            match op {
                Op::Exit => break,
                Op::Noop => continue,
                Op::Push(object) => self.push(object),
                Op::Load(symbol) => {
                    if let Some(object) = scope.load(symbol) {
                        self.stack.push(object);
                    } else {
                        return Err(Error::UnresolvedIdentifier);
                    }
                }
                Op::Match(pattern) => {
                    scope.r#match(pattern, self.pop())?;
                }
                Op::Store(symbol) => scope.store(symbol, self.pop()),
                Op::Call => match self.pop() {
                    Val::Function(f) => {
                        self.run(&*f, &mut Scope::new(Some(scope)))?;
                    }
                    _ => return Err(Error::TypeError),
                },
                Op::Add => match (self.pop(), self.pop()) {
                    (Val::Int(a), Val::Int(b)) => self.push(Val::Int(a + b)),
                    (Val::Float(a), Val::Float(b)) => self.push(Val::Float(a + b)),
                    _ => return Err(Error::TypeError),
                },
                Op::Sub => match (self.pop(), self.pop()) {
                    (Val::Int(a), Val::Int(b)) => self.push(Val::Int(a - b)),
                    (Val::Float(a), Val::Float(b)) => self.push(Val::Float(a - b)),
                    _ => return Err(Error::TypeError),
                },
                Op::Mul => match (self.pop(), self.pop()) {
                    (Val::Int(a), Val::Int(b)) => self.push(Val::Int(a * b)),
                    (Val::Float(a), Val::Float(b)) => self.push(Val::Float(a * b)),
                    _ => return Err(Error::TypeError),
                },
                Op::Div => match (self.pop(), self.pop()) {
                    (Val::Int(a), Val::Int(b)) => self.stack.push(Val::Int(a / b)),
                    (Val::Float(a), Val::Float(b)) => self.stack.push(Val::Float(a / b)),
                    _ => return Err(Error::TypeError),
                },
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let bytecode = bincode::serialize(&[
            Op::Push(Val::Int(1)),
            Op::Store(Symbol(0)),
            Op::Push(Val::Int(2)),
            Op::Store(Symbol(1)),
            Op::Load(Symbol(0)),
            Op::Load(Symbol(1)),
            Op::Add,
        ])
        .unwrap();
        let mut vm = Vm::default();
        let mut scope = Scope::default();
        let result = vm.run(&bytecode, &mut scope);
        assert_eq!(result, Ok(()));
        assert_eq!(vm.pop(), Val::Int(3));
    }
}

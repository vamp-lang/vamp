use crate::symbol::Symbol;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::rc::Rc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tuple {
    positional: Rc<[Val]>,
    named: Rc<FxHashMap<Symbol, Val>>,
}

impl Tuple {
    pub fn new(positional: Rc<[Val]>, named: Rc<FxHashMap<Symbol, Val>>) -> Self {
        Tuple { positional, named }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Val {
    Nil,
    Int(i64),
    Float(f64),
    Function(Rc<[u8]>),
    Tuple(Tuple),
    Vector(Rc<[Val]>),
    Symbol(Symbol),
    String(Rc<str>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternTuple {
    positional: Rc<[Pattern]>,
    named: Rc<FxHashMap<Symbol, Pattern>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Pattern {
    Ignore,
    Nil,
    Int(i64),
    Float(f64),
    Symbol(Symbol),
    String(Rc<str>),
    Identifier(Symbol),
    Tuple(PatternTuple),
    Vector(Rc<[Pattern]>),
    Any(Rc<[Pattern]>),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Op {
    Exit,
    Noop,
    Push(Val),
    Load(Symbol),
    Store(Symbol),
    Match(Pattern),
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

    fn r#match(&mut self, pattern: Pattern, val: Val) -> Result<(), Error> {
        match (pattern, val) {
            (Pattern::Ignore, _) => Ok(()),
            (Pattern::Nil, Val::Nil) => Ok(()),
            (Pattern::Int(a), Val::Int(b)) if a == b => Ok(()),
            (Pattern::Float(a), Val::Float(b)) if a == b => Ok(()),
            (Pattern::Symbol(a), Val::Symbol(b)) if a == b => Ok(()),
            (Pattern::String(a), Val::String(b)) if a == b => Ok(()),
            (Pattern::Identifier(symbol), val) => {
                self.store(symbol, val);
                Ok(())
            }
            (Pattern::Any(patterns), val) => patterns
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

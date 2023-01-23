use crate::parse::{parse, Expr, Let, OperatorKind, Pattern};
use crate::source::Error as ParseError;
use crate::symbol::{Interner, Symbol};
use rustc_hash::FxHashMap;
use std::rc::Rc;

#[derive(Debug)]
pub enum Error {
    Void,
    UndefinedSymbol(Symbol),
    ParseError(ParseError),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Tuple {
    pub tag: Option<Symbol>,
    pub positional: Vec<Value>,
    pub named: FxHashMap<Symbol, Value>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Nil,
    Tuple(Tuple),
    Vector(Vec<Value>),
    Tag(Symbol),
    String(String),
    Integer(i64),
    Float(f64),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "()"),
            Value::Tuple(tuple) => {
                if let Some(tag) = &tuple.tag {
                    write!(f, "{:?}", tag)?;
                }
                write!(f, "(")?;
                let mut positional = tuple.positional.iter();
                if let Some(value) = positional.next() {
                    write!(f, "{}", value)?;
                    for value in positional {
                        write!(f, ", {}", value)?;
                    }
                }
                let mut named = tuple.named.iter();
                if let Some((key, value)) = named.next() {
                    if tuple.positional.len() > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{:?}: {}", key, value)?;
                    for (key, value) in named {
                        write!(f, ", {:?}: {}", key, value)?;
                    }
                }
                write!(f, ")")
            }
            Value::Vector(values) => {
                write!(f, "[")?;
                let mut iter = values.iter();
                if let Some(value) = iter.next() {
                    write!(f, "{}", value)?;
                    for value in iter {
                        write!(f, ", {}", value)?;
                    }
                }
                write!(f, "]")
            }
            // TODO: Standard tag formatting.
            Value::Tag(name) => write!(f, "{:?}", name),
            // TODO: Standard string formatting.
            Value::String(string) => write!(f, "{:?}", string),
            // TODO: Standard integer formatting.
            Value::Integer(value) => write!(f, "{}", value),
            // TODO: Standard float formatting.
            Value::Float(value) => write!(f, "{}", value),
        }
    }
}

pub struct Environment {
    interner: Interner,
}

pub struct Scope {
    parent: Option<Rc<Scope>>,
    bindings: FxHashMap<Symbol, Value>,
}

impl Scope {
    pub fn new(parent: Option<Rc<Scope>>) -> Self {
        Scope {
            parent,
            bindings: FxHashMap::default(),
        }
    }

    pub fn bind(&mut self, symbol: Symbol, value: Value) {
        self.bindings.insert(symbol, value);
    }

    pub fn lookup(&self, symbol: Symbol) -> Option<&Value> {
        self.bindings.get(&symbol)
    }
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            interner: Interner::new(),
        }
    }

    fn eval_expr(&mut self, expr: &Expr, scope: Rc<Scope>) -> Result<Value, Error> {
        match expr {
            Expr::Nil => Ok(Value::Nil),
            Expr::Void => Err(Error::Void),
            Expr::Block(imports, lets, exprs) => {
                let mut child = Rc::new(Scope::new(Some(scope)));
                for Let(pattern, right) in lets {
                    match pattern {
                        Pattern::Identifier(name) => {
                            let value = self.eval_expr(right, child.clone())?;
                            Rc::get_mut(&mut child).unwrap().bind(*name, value);
                        }
                    }
                }
                self.eval_expr(&exprs[0], child)
            }
            Expr::Function(f, args) => todo!(),
            Expr::Tuple(tuple) => {
                let mut positional = vec![];
                for expr in &tuple.positional {
                    positional.push(self.eval_expr(expr, scope.clone())?);
                }
                let mut named = FxHashMap::default();
                for (key, expr) in &tuple.named {
                    named.insert(key.clone(), self.eval_expr(expr, scope.clone())?);
                }
                Ok(Value::Tuple(Tuple {
                    tag: tuple.tag.clone(),
                    positional,
                    named,
                }))
            }
            Expr::Vector(exprs) => {
                let mut values = vec![];
                for expr in exprs {
                    values.push(self.eval_expr(expr, scope.clone())?);
                }
                Ok(Value::Vector(values))
            }
            Expr::Identifier(symbol) => match scope.lookup(*symbol) {
                Some(value) => Ok(value.clone()),
                None => Err(Error::UndefinedSymbol(*symbol)),
            },
            Expr::Tag(value) => Ok(Value::Tag(*value)),
            Expr::String(value) => Ok(Value::String(value.clone())),
            Expr::Integer(value) => Ok(Value::Integer(*value)),
            Expr::Float(value) => Ok(Value::Float(*value)),
            Expr::Operator(kind, operands) => {
                let mut values = vec![];
                for value in operands {
                    values.push(self.eval_expr(value, scope.clone())?);
                }
                match kind {
                    OperatorKind::Add => match &values[..] {
                        [Value::Integer(a), Value::Integer(b)] => Ok(Value::Integer(a + b)),
                        [Value::Float(a), Value::Float(b)] => Ok(Value::Float(a + b)),
                        [Value::String(a), Value::String(b)] => Ok(Value::String(a.clone() + b)),
                        _ => Err(Error::Void),
                    },
                    OperatorKind::Subtract => match &values[..] {
                        [Value::Integer(a), Value::Integer(b)] => Ok(Value::Integer(a - b)),
                        [Value::Float(a), Value::Float(b)] => Ok(Value::Float(a - b)),
                        _ => Err(Error::Void),
                    },
                    OperatorKind::Multiply => match &values[..] {
                        [Value::Integer(a), Value::Integer(b)] => Ok(Value::Integer(a * b)),
                        [Value::Float(a), Value::Float(b)] => Ok(Value::Float(a * b)),
                        _ => Err(Error::Void),
                    },
                    OperatorKind::Divide => match &values[..] {
                        [Value::Integer(a), Value::Integer(b)] => Ok(Value::Integer(a / b)),
                        [Value::Float(a), Value::Float(b)] => Ok(Value::Float(a / b)),
                        _ => Err(Error::Void),
                    },
                }
            }
            Expr::Call(f, args) => todo!(),
        }
    }

    pub fn eval(&mut self, source: &str) -> Result<Value, Error> {
        let expr = &parse(source, &mut self.interner).map_err(|error| Error::ParseError(error))?;
        self.eval_expr(expr, Scope::new(None).into())
    }
}

/*
use crate::ast::{Expr, Let, OperatorKind, Pattern};
use crate::parse::parse_module;
use crate::source::Error as ParseError;
use crate::symbol::{Interner, Symbol};
use bumpalo::Bump;
use rustc_hash::FxHashMap;
use std::rc::Rc;

#[derive(Debug)]
pub enum Error {
    Void,
    UndefinedSymbol(Symbol),
    LeftHandMustBeFunction,
    ParseError(ParseError),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Tuple {
    pub symbol: Option<Symbol>,
    pub positional: Vec<Value>,
    pub named: FxHashMap<Symbol, Value>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Nil,
    Tuple(Tuple),
    Vector(Vec<Value>),
    Symbol(Symbol),
    String(String),
    Integer(i64),
    Float(f64),
    Function(Vec<Pattern>, Box<Expr>),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "()"),
            Value::Tuple(tuple) => {
                if let Some(symbol) = &tuple.symbol {
                    write!(f, "{:?}", symbol)?;
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
            // TODO: Standard symbol formatting.
            Value::Symbol(name) => write!(f, "{:?}", name),
            // TODO: Standard string formatting.
            Value::String(string) => write!(f, "{:?}", string),
            // TODO: Standard integer formatting.
            Value::Integer(value) => write!(f, "{}", value),
            // TODO: Standard float formatting.
            Value::Float(value) => write!(f, "{}", value),
            // TODO: Standard function formatting.
            Value::Function(_, _) => write!(f, "[function]"),
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
        self.bindings.get(&symbol).or_else(|| {
            if let Some(parent) = &self.parent {
                parent.lookup(symbol)
            } else {
                None
            }
        })
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
                let mut child = Rc::new(Scope::new(Some(scope.clone())));
                for Let(pattern, right) in lets {
                    match pattern {
                        Pattern::Identifier(symbol) => {
                            let value = self.eval_expr(right, child.clone())?;
                            Rc::get_mut(&mut child).unwrap().bind(*symbol, value);
                        }
                    }
                }
                self.eval_expr(&exprs[0], child.clone())
            }
            Expr::Function(args, expr) => Ok(Value::Function(args.clone(), expr.clone())),
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
                    symbol: tuple.symbol.clone(),
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
            Expr::Symbol(value) => Ok(Value::Symbol(*value)),
            Expr::String(value) => Ok(Value::String(value.clone())),
            Expr::Int(value) => Ok(Value::Integer(*value)),
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
                        [Value::Vector(a), Value::Vector(b)] => Ok(Value::Vector({
                            let mut sum = a.clone();
                            sum.extend(b.clone());
                            sum
                        })),
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
            Expr::Call(function, exprs) => {
                let f = self.eval_expr(function, scope.clone())?;
                match f {
                    Value::Function(patterns, expr) => {
                        // TODO: Partial application...
                        assert!(exprs.len() == patterns.len());

                        // TODO: Other kinds of patterns...
                        let mut bound_scope = Scope::new(Some(scope.clone()));
                        for (i, Pattern::Identifier(symbol)) in patterns.iter().enumerate() {
                            bound_scope.bind(*symbol, self.eval_expr(&exprs[i], scope.clone())?);
                        }

                        self.eval_expr(&expr, bound_scope.into())
                    }
                    _ => Err(Error::LeftHandMustBeFunction),
                }
            }
        }
    }

    pub fn eval(&mut self, source: &str) -> Result<Value, Error> {
        let bump = Bump::new();
        let expr = &parse_module(source, &bump, &mut self.interner)
            .map_err(|error| Error::ParseError(error))?;
        self.eval_expr(expr, Scope::new(None).into())
    }
}
*/

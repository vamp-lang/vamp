use crate::parse::{parse, Expr, OperatorKind};
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub struct Tuple {
    pub tag: Option<String>,
    pub positional: Vec<Value>,
    pub named: HashMap<String, Value>,
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Nil,
    Tuple(Tuple),
    Vector(Vec<Value>),
    Map(HashMap<String, Value>),
    Tag(String),
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
                    write!(f, "{}", tag)?;
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
                    write!(f, "{}: {}", key, value)?;
                    for (key, value) in named {
                        write!(f, ", {}: {}", key, value)?;
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
            Value::Map(values) => todo!(),
            Value::Tag(name) => write!(f, "{}", name),
            // TODO: Standard string formatting.
            Value::String(string) => write!(f, "{:?}", string),
            // TODO: Standard integer formatting.
            Value::Integer(value) => write!(f, "{}", value),
            // TODO: Standard float formatting.
            Value::Float(value) => write!(f, "{}", value),
        }
    }
}

pub struct Environment {}

impl Environment {
    pub fn new() -> Environment {
        Environment {}
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, ()> {
        let value = match expr {
            Expr::Nil => Value::Nil,
            // TODO: This is broken
            Expr::Void => Value::Nil,
            Expr::Block(imports, lets, exprs) => self.eval_expr(&exprs[0])?,
            Expr::Function(f, args) => todo!(),
            Expr::Tuple(tuple) => {
                let mut positional = vec![];
                for expr in &tuple.positional {
                    positional.push(self.eval_expr(expr)?);
                }
                let mut named = HashMap::new();
                for (key, expr) in &tuple.named {
                    named.insert(key.clone(), self.eval_expr(expr)?);
                }
                Value::Tuple(Tuple {
                    tag: tuple.tag.clone(),
                    positional,
                    named,
                })
            }
            Expr::Vector(exprs) => {
                let mut values = vec![];
                for expr in exprs {
                    values.push(self.eval_expr(expr)?);
                }
                Value::Vector(values)
            }
            Expr::Map(entries) => todo!(),
            Expr::Identifier(name) => todo!(),
            Expr::Tag(value) => Value::Tag(value.clone()),
            Expr::String(value) => Value::String(value.clone()),
            Expr::Integer(value) => Value::Integer(*value),
            Expr::Float(value) => Value::Float(*value),
            Expr::Operator(kind, operands) => {
                let mut values = vec![];
                for value in operands {
                    values.push(self.eval_expr(value)?);
                }
                match kind {
                    OperatorKind::Add => match &values[..] {
                        [Value::Integer(a), Value::Integer(b)] => Value::Integer(a + b),
                        [Value::Integer(a), Value::Float(b)] => Value::Float(*a as f64 + b),
                        [Value::Float(a), Value::Integer(b)] => Value::Float(a + *b as f64),
                        [Value::Float(a), Value::Float(b)] => Value::Float(a + b),
                        _ => todo!(),
                    },
                    OperatorKind::Subtract => match &values[..] {
                        [Value::Integer(a), Value::Integer(b)] => Value::Integer(a - b),
                        [Value::Integer(a), Value::Float(b)] => Value::Float(*a as f64 - b),
                        [Value::Float(a), Value::Integer(b)] => Value::Float(a - *b as f64),
                        [Value::Float(a), Value::Float(b)] => Value::Float(a - b),
                        _ => todo!(),
                    },
                    OperatorKind::Multiply => match &values[..] {
                        [Value::Integer(a), Value::Integer(b)] => Value::Integer(a * b),
                        [Value::Integer(a), Value::Float(b)] => Value::Float(*a as f64 * b),
                        [Value::Float(a), Value::Integer(b)] => Value::Float(a * *b as f64),
                        [Value::Float(a), Value::Float(b)] => Value::Float(a * b),
                        _ => todo!(),
                    },
                    OperatorKind::Divide => match &values[..] {
                        [Value::Integer(a), Value::Integer(b)] => Value::Integer(a / b),
                        [Value::Integer(a), Value::Float(b)] => Value::Float(*a as f64 / b),
                        [Value::Float(a), Value::Integer(b)] => Value::Float(a / *b as f64),
                        [Value::Float(a), Value::Float(b)] => Value::Float(a / b),
                        _ => todo!(),
                    },
                }
            }
            Expr::Call(f, args) => todo!(),
        };
        Ok(value)
    }

    pub fn eval(&mut self, source: &str) -> Result<Value, ()> {
        self.eval_expr(&parse(source).map_err(|_| ())?)
    }
}

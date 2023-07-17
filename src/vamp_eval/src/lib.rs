pub mod error;
pub mod value;
pub use error::{Error, Result};
use std::convert::TryInto;
pub use value::Value;
use vamp_sym::Sym;
use vamp_syntax::ast::{BinOp, Expr, ExprKind, Pat, Stmt, UnOp};
use vamp_tuple::{Tuple, TupleEntry};

#[derive(Debug, PartialEq, Default)]
pub struct Scope<'a> {
    parent: Option<&'a Scope<'a>>,
    bindings: Tuple<Value>,
}

impl<'a> Scope<'a> {
    pub fn new(parent: Option<&'a Scope<'a>>) -> Scope<'a> {
        Scope {
            parent,
            bindings: Default::default(),
        }
    }

    fn bind_tuple(&mut self, pat: &Tuple<Pat>, value: Tuple<Value>) -> Result<()> {
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

    fn bind(&mut self, pat: &Pat, value: Value) -> Result<()> {
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

    fn lookup(&self, name: Sym) -> Result<Value> {
        self.bindings
            .get(name)
            .map(|value| value.clone())
            .ok_or(Error::Unbound)
    }
}

pub fn eval_expr(expr: &Expr, scope: &Scope) -> Result<Value> {
    match &expr.kind {
        ExprKind::Void => Err(Error::Void),
        ExprKind::Ident(sym) => scope.lookup(*sym),
        ExprKind::Sym(sym) => Ok(Value::Sym(*sym)),
        ExprKind::Str(str) => Ok(Value::Str(str.clone())),
        ExprKind::Int(value) => Ok(Value::Int(*value)),
        ExprKind::Float(value) => Ok(Value::Float(*value)),
        ExprKind::Bool(value) => Ok(Value::Bool(*value)),
        ExprKind::Tuple(tuple) => {
            // TODO: Make this transformation more efficient.
            let mut result = Tuple::new();
            for entry in tuple.iter() {
                match entry {
                    TupleEntry::Pos(expr) => {
                        let value = eval_expr(expr, scope)?;
                        result.push(value);
                    }
                    TupleEntry::Named(key, expr) => {
                        let value = eval_expr(expr, scope)?;
                        result.insert(key, value);
                    }
                }
            }
            Ok(Value::Tuple(result))
        }
        ExprKind::List(items) => {
            let mut result = Vec::with_capacity(items.len());
            for item in items.into_iter() {
                result.push(eval_expr(item, scope)?);
            }
            Ok(Value::List(result))
        }
        ExprKind::UnOp(unary_op, expr) => match unary_op {
            UnOp::Neg => match eval_expr(expr, scope)? {
                Value::Int(value) => Ok(Value::Int(-value)),
                Value::Float(value) => Ok(Value::Float(-value)),
                _ => Err(Error::Types),
            },
            UnOp::BitNot => match eval_expr(expr, scope)? {
                Value::Int(value) => Ok(Value::Int(!value)),
                _ => Err(Error::Types),
            },
            _ => todo!(),
        },
        ExprKind::BinOp(binary_op, l, r) => match binary_op {
            BinOp::Dot => match (eval_expr(l, scope)?, &r.kind) {
                (Value::Tuple(tuple), ExprKind::Ident(key)) => tuple
                    .get(*key)
                    .map(|value| value.clone())
                    .ok_or(Error::KeyNotFound),
                (Value::Tuple(tuple), ExprKind::Int(i)) => {
                    let i: usize = (*i).try_into().map_err(|_| Error::KeyNotFound)?;
                    tuple
                        .get(i)
                        .map(|value| value.clone())
                        .ok_or(Error::KeyNotFound)
                }
                _ => Err(Error::Types),
            },
            BinOp::Add => match (eval_expr(l, scope)?, eval_expr(r, scope)?) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                _ => Err(Error::Types),
            },
            BinOp::Sub => match (eval_expr(l, scope)?, eval_expr(r, scope)?) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                _ => Err(Error::Types),
            },
            BinOp::Mul => match (eval_expr(l, scope)?, eval_expr(r, scope)?) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                _ => Err(Error::Types),
            },
            BinOp::Div => match (eval_expr(l, scope)?, eval_expr(r, scope)?) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a / b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
                _ => Err(Error::Types),
            },
            BinOp::Mod => match (eval_expr(l, scope)?, eval_expr(r, scope)?) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a % b)),
                _ => Err(Error::Types),
            },
            BinOp::Exp => match (eval_expr(l, scope)?, eval_expr(r, scope)?) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.pow(b.try_into().unwrap()))),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.powf(b))),
                _ => Err(Error::Types),
            },
            BinOp::Eq => match (eval_expr(l, scope)?, eval_expr(r, scope)?) {
                (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a == b)),
                (Value::Sym(a), Value::Sym(b)) => Ok(Value::Bool(a == b)),
                (Value::Str(a), Value::Str(b)) => Ok(Value::Bool(a == b)),
                (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a == b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a == b)),
                (Value::Tuple(a), Value::Tuple(b)) => Ok(Value::Bool(a == b)),
                (Value::List(a), Value::List(b)) => Ok(Value::Bool(a == b)),
                _ => Err(Error::Types),
            },
            BinOp::NotEq => match (eval_expr(l, scope)?, eval_expr(r, scope)?) {
                (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a != b)),
                (Value::Sym(a), Value::Sym(b)) => Ok(Value::Bool(a != b)),
                (Value::Str(a), Value::Str(b)) => Ok(Value::Bool(a != b)),
                (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a != b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a != b)),
                (Value::Tuple(a), Value::Tuple(b)) => Ok(Value::Bool(a != b)),
                (Value::List(a), Value::List(b)) => Ok(Value::Bool(a != b)),
                _ => Err(Error::Types),
            },
            BinOp::Lt => match (eval_expr(l, scope)?, eval_expr(r, scope)?) {
                (Value::Str(a), Value::Str(b)) => Ok(Value::Bool(a < b)),
                (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
                _ => Err(Error::Types),
            },
            BinOp::LtEq => match (eval_expr(l, scope)?, eval_expr(r, scope)?) {
                (Value::Str(a), Value::Str(b)) => Ok(Value::Bool(a <= b)),
                (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
                _ => Err(Error::Types),
            },
            BinOp::Gt => match (eval_expr(l, scope)?, eval_expr(r, scope)?) {
                (Value::Str(a), Value::Str(b)) => Ok(Value::Bool(a > b)),
                (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
                _ => Err(Error::Types),
            },
            BinOp::GtEq => match (eval_expr(l, scope)?, eval_expr(r, scope)?) {
                (Value::Str(a), Value::Str(b)) => Ok(Value::Bool(a >= b)),
                (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),
                _ => Err(Error::Types),
            },
            BinOp::And => match eval_expr(l, scope)? {
                Value::Bool(true) => match eval_expr(r, scope)? {
                    Value::Bool(value) => Ok(Value::Bool(value)),
                    _ => Err(Error::Types),
                },
                Value::Bool(false) => Ok(Value::Bool(false)),
                _ => Err(Error::Types),
            },
            BinOp::Or => match eval_expr(l, scope)? {
                Value::Bool(true) => Ok(Value::Bool(true)),
                Value::Bool(false) => match eval_expr(r, scope)? {
                    Value::Bool(value) => Ok(Value::Bool(value)),
                    _ => Err(Error::Types),
                },
                _ => Err(Error::Types),
            },
            _ => Err(Error::Types),
        },
        ExprKind::Block(stmts) => {
            let mut block_scope = Scope::new(Some(scope));
            for stmt in stmts.iter() {
                if let Some(value) = eval_stmt(stmt, &mut block_scope)? {
                    return Ok(value);
                }
            }
            Err(Error::Void)
        }
        ExprKind::Fn(args, expr) => Ok(Value::Fn(args.clone(), expr.clone())),
        ExprKind::Call(f, args) => match eval_expr(f, scope)? {
            Value::Fn(params, body) => {
                let mut call_scope = Scope::default();
                let mut evaluated_args = Tuple::new();
                for arg in args.iter() {
                    match arg {
                        TupleEntry::Pos(expr) => {
                            let value = eval_expr(expr, scope)?;
                            evaluated_args.push(value);
                        }
                        TupleEntry::Named(key, expr) => {
                            let value = eval_expr(expr, scope)?;
                            evaluated_args.insert(key, value);
                        }
                    }
                }
                call_scope.bind_tuple(&params, evaluated_args)?;
                eval_expr(&body, &call_scope)
            }
            _ => Err(Error::Types),
        },
    }
}

pub fn eval_stmt(stmt: &Stmt, scope: &mut Scope) -> Result<Option<Value>> {
    match stmt {
        Stmt::Let(pat, expr) => {
            scope.bind(pat, eval_expr(expr, scope)?)?;
            Ok(None)
        }
        Stmt::Expr(expr) => Ok(Some(eval_expr(expr, &scope)?)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vamp_sym::{Interner, Sym};
    use vamp_syntax::parser::parse_expr;

    fn eval_string(source: &str) -> Result<Value> {
        let mut interner = Interner::new();
        let scope = Scope::default();
        let value = parse_expr(source, &mut interner).unwrap();
        eval_expr(&value, &scope)
    }

    #[test]
    fn test_void() {
        assert_eq!(eval_string("{}"), Err(Error::Void));
    }

    #[test]
    fn test_sym() {
        assert_eq!(eval_string("'abc'"), Ok(Value::Sym(Sym(0))));
    }

    #[test]
    fn test_str() {
        assert_eq!(eval_string("\"abc\""), Ok(Value::Str("abc".into())));
    }

    #[test]
    fn test_int() {
        assert_eq!(eval_string("123"), Ok(Value::Int(123)));
    }

    #[test]
    fn test_float() {
        assert_eq!(eval_string("3.14"), Ok(Value::Float(3.14)));
    }

    #[test]
    fn test_tuple() {
        assert_eq!(eval_string("()"), Ok(Value::Tuple(Tuple::new())));
        assert_eq!(
            eval_string("(1, 2, 3)"),
            Ok(Value::Tuple(Tuple::from_iter([
                TupleEntry::Pos(Value::Int(1)),
                TupleEntry::Pos(Value::Int(2)),
                TupleEntry::Pos(Value::Int(3))
            ])))
        );
        assert_eq!(eval_string("(1, {}, 3)"), Err(Error::Void));
    }

    #[test]
    fn test_list() {
        assert_eq!(eval_string("[]"), Ok(Value::List(vec![])));
        assert_eq!(
            eval_string("[1, 2, 3]"),
            Ok(Value::List(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
            ]))
        );
        assert_eq!(eval_string("[1, {}, 3]"), Err(Error::Void));
    }

    #[test]
    fn test_arithmetic() {
        assert_eq!(eval_string("2 * -1 + 10 / 2"), Ok(Value::Int(3)));
        assert_eq!(eval_string("0 * 'abc'"), Err(Error::Types));
    }
}

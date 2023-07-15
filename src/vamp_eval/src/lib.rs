pub mod error;
pub mod value;
pub use error::{Error, Result};
use std::convert::TryInto;
pub use value::Value;
use vamp_syntax::ast::{BinOp, Expr, ExprKind, UnOp};
use vamp_tuple::{Tuple, TupleEntry};

pub fn eval(expr: &Expr) -> Result<Value> {
    match &expr.kind {
        ExprKind::Void => Err(Error::Void),
        ExprKind::Sym(symbol) => Ok(Value::Sym(*symbol)),
        ExprKind::Str(string) => Ok(Value::Str(string.clone())),
        ExprKind::Int(value) => Ok(Value::Int(*value)),
        ExprKind::Float(value) => Ok(Value::Float(*value)),
        ExprKind::Tuple(tuple) => {
            // TODO: Make this transformation more efficient.
            let mut result = Tuple::new();
            for entry in tuple.iter() {
                match entry {
                    TupleEntry::Pos(expr) => {
                        let value = eval(expr)?;
                        result.push(value);
                    }
                    TupleEntry::Named(key, expr) => {
                        let value = eval(expr)?;
                        result.insert(key, value);
                    }
                }
            }
            Ok(Value::Tuple(result))
        }
        ExprKind::List(items) => {
            let mut result = Vec::with_capacity(items.len());
            for item in items.into_iter() {
                result.push(eval(item)?);
            }
            Ok(Value::List(result))
        }
        ExprKind::UnOp(unary_op, expr) => match unary_op {
            UnOp::Neg => match eval(expr)? {
                Value::Int(value) => Ok(Value::Int(-value)),
                Value::Float(value) => Ok(Value::Float(-value)),
                _ => Err(Error::Types),
            },
            UnOp::BitNot => match eval(expr)? {
                Value::Int(value) => Ok(Value::Int(!value)),
                _ => Err(Error::Types),
            },
            _ => todo!(),
        },
        ExprKind::BinOp(binary_op, l, r) => match binary_op {
            BinOp::Dot => match (eval(l)?, &r.kind) {
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
            BinOp::Add => match (eval(l)?, eval(r)?) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                _ => Err(Error::Types),
            },
            BinOp::Sub => match (eval(l)?, eval(r)?) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                _ => Err(Error::Types),
            },
            BinOp::Mul => match (eval(l)?, eval(r)?) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                _ => Err(Error::Types),
            },
            BinOp::Div => match (eval(l)?, eval(r)?) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a / b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
                _ => Err(Error::Types),
            },
            BinOp::Mod => match (eval(l)?, eval(r)?) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a % b)),
                _ => Err(Error::Types),
            },
            BinOp::Exp => match (eval(l)?, eval(r)?) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.pow(b.try_into().unwrap()))),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.powf(b))),
                _ => Err(Error::Types),
            },
            _ => Err(Error::Types),
        },
        _ => todo!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vamp_sym::{Interner, Sym};
    use vamp_syntax::parser::parse_expr;

    fn eval_string(source: &str) -> Result<Value> {
        let mut interner = Interner::new();
        let value = parse_expr(source, &mut interner).unwrap();
        eval(&value)
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

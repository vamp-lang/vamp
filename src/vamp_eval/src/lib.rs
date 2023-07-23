pub mod error;
pub mod value;
pub use error::{Error, Result};
use std::{cell::RefCell, convert::TryInto, rc::Rc};
pub use value::{Fn, Mod, Scope, Value};
use vamp_syntax::ast::{self, BinOp, Expr, ExprKind, Pat, Stmt, UnOp};
use vamp_tuple::{Tuple, TupleEntry};

pub fn eval_expr(expr: &Expr, scope: Rc<RefCell<Scope>>, ctx: Rc<RefCell<Scope>>) -> Result<Value> {
    match &expr.kind {
        ExprKind::Void => Err(Error::Void),
        ExprKind::Ident(sym) => scope.borrow().lookup(*sym).map(|value| value.clone()),
        ExprKind::CtxIdent(sym) => ctx.borrow().lookup(*sym).map(|value| value.clone()),
        ExprKind::Sym(sym) => Ok(Value::Sym(*sym)),
        ExprKind::Str(str) => Ok(Value::Str(str.clone())),
        ExprKind::Int(value) => Ok(Value::Int(*value)),
        ExprKind::Float(value) => Ok(Value::Float(*value)),
        ExprKind::Bool(value) => Ok(Value::Bool(*value)),
        ExprKind::Tuple(tuple) => {
            let mut result = Tuple::new();
            for entry in tuple.iter() {
                match entry {
                    TupleEntry::Pos(expr) => {
                        let value = eval_expr(expr, scope.clone(), ctx.clone())?;
                        result.push(value);
                    }
                    TupleEntry::Named(key, expr) => {
                        let value = eval_expr(expr, scope.clone(), ctx.clone())?;
                        result.insert(key, value);
                    }
                }
            }
            Ok(Value::Tuple(result))
        }
        ExprKind::List(items) => {
            let mut result = Vec::with_capacity(items.len());
            for item in items.into_iter() {
                result.push(eval_expr(item, scope.clone(), ctx.clone())?);
            }
            Ok(Value::List(result))
        }
        ExprKind::UnOp(unary_op, expr) => match unary_op {
            UnOp::Neg => match eval_expr(expr, scope, ctx)? {
                Value::Int(value) => Ok(Value::Int(-value)),
                Value::Float(value) => Ok(Value::Float(-value)),
                _ => Err(Error::Types),
            },
            UnOp::BitNot => match eval_expr(expr, scope, ctx)? {
                Value::Int(value) => Ok(Value::Int(!value)),
                _ => Err(Error::Types),
            },
            _ => todo!(),
        },
        ExprKind::BinOp(binary_op, l, r) => match binary_op {
            BinOp::Dot => match (eval_expr(l, scope, ctx)?, &r.kind) {
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
            BinOp::Add => match (
                eval_expr(l, scope.clone(), ctx.clone())?,
                eval_expr(r, scope, ctx)?,
            ) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                _ => Err(Error::Types),
            },
            BinOp::Sub => match (
                eval_expr(l, scope.clone(), ctx.clone())?,
                eval_expr(r, scope, ctx)?,
            ) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                _ => Err(Error::Types),
            },
            BinOp::Mul => match (
                eval_expr(l, scope.clone(), ctx.clone())?,
                eval_expr(r, scope, ctx)?,
            ) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                _ => Err(Error::Types),
            },
            BinOp::Div => match (
                eval_expr(l, scope.clone(), ctx.clone())?,
                eval_expr(r, scope, ctx)?,
            ) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a / b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
                _ => Err(Error::Types),
            },
            BinOp::Mod => match (
                eval_expr(l, scope.clone(), ctx.clone())?,
                eval_expr(r, scope, ctx)?,
            ) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a % b)),
                _ => Err(Error::Types),
            },
            BinOp::Exp => match (
                eval_expr(l, scope.clone(), ctx.clone())?,
                eval_expr(r, scope, ctx)?,
            ) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.pow(b.try_into().unwrap()))),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.powf(b))),
                _ => Err(Error::Types),
            },
            BinOp::Eq => match (
                eval_expr(l, scope.clone(), ctx.clone())?,
                eval_expr(r, scope, ctx)?,
            ) {
                (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a == b)),
                (Value::Sym(a), Value::Sym(b)) => Ok(Value::Bool(a == b)),
                (Value::Str(a), Value::Str(b)) => Ok(Value::Bool(a == b)),
                (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a == b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a == b)),
                (Value::Tuple(a), Value::Tuple(b)) => Ok(Value::Bool(a == b)),
                (Value::List(a), Value::List(b)) => Ok(Value::Bool(a == b)),
                _ => Err(Error::Types),
            },
            BinOp::NotEq => match (
                eval_expr(l, scope.clone(), ctx.clone())?,
                eval_expr(r, scope, ctx)?,
            ) {
                (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a != b)),
                (Value::Sym(a), Value::Sym(b)) => Ok(Value::Bool(a != b)),
                (Value::Str(a), Value::Str(b)) => Ok(Value::Bool(a != b)),
                (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a != b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a != b)),
                (Value::Tuple(a), Value::Tuple(b)) => Ok(Value::Bool(a != b)),
                (Value::List(a), Value::List(b)) => Ok(Value::Bool(a != b)),
                _ => Err(Error::Types),
            },
            BinOp::Lt => match (
                eval_expr(l, scope.clone(), ctx.clone())?,
                eval_expr(r, scope, ctx)?,
            ) {
                (Value::Str(a), Value::Str(b)) => Ok(Value::Bool(a < b)),
                (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
                _ => Err(Error::Types),
            },
            BinOp::LtEq => match (
                eval_expr(l, scope.clone(), ctx.clone())?,
                eval_expr(r, scope, ctx)?,
            ) {
                (Value::Str(a), Value::Str(b)) => Ok(Value::Bool(a <= b)),
                (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
                _ => Err(Error::Types),
            },
            BinOp::Gt => match (
                eval_expr(l, scope.clone(), ctx.clone())?,
                eval_expr(r, scope, ctx)?,
            ) {
                (Value::Str(a), Value::Str(b)) => Ok(Value::Bool(a > b)),
                (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
                _ => Err(Error::Types),
            },
            BinOp::GtEq => match (
                eval_expr(l, scope.clone(), ctx.clone())?,
                eval_expr(r, scope, ctx)?,
            ) {
                (Value::Str(a), Value::Str(b)) => Ok(Value::Bool(a >= b)),
                (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),
                _ => Err(Error::Types),
            },
            BinOp::And => match eval_expr(l, scope.clone(), ctx.clone())? {
                Value::Bool(true) => match eval_expr(r, scope, ctx)? {
                    Value::Bool(value) => Ok(Value::Bool(value)),
                    _ => Err(Error::Types),
                },
                Value::Bool(false) => Ok(Value::Bool(false)),
                _ => Err(Error::Types),
            },
            BinOp::Or => match eval_expr(l, scope.clone(), ctx.clone())? {
                Value::Bool(true) => Ok(Value::Bool(true)),
                Value::Bool(false) => match eval_expr(r, scope, ctx)? {
                    Value::Bool(value) => Ok(Value::Bool(value)),
                    _ => Err(Error::Types),
                },
                _ => Err(Error::Types),
            },
            _ => Err(Error::Types),
        },
        ExprKind::Block(stmts) => {
            let block_scope = Rc::new(RefCell::new(Scope::new(Some(scope))));
            for stmt in stmts.iter() {
                if let Some(value) = eval_stmt(stmt, block_scope.clone(), ctx.clone())? {
                    return Ok(value);
                }
            }
            Err(Error::Void)
        }
        ExprKind::IfElse(cond, if_expr, else_expr) => {
            match eval_expr(cond, scope.clone(), ctx.clone())? {
                Value::Bool(true) => eval_expr(if_expr, scope, ctx),
                Value::Bool(false) => eval_expr(else_expr, scope, ctx),
                _ => Err(Error::Types),
            }
        }
        ExprKind::Fn(params, body) => Ok(Value::Fn(Fn {
            params: params.clone(),
            body: body.clone(),
            scope,
        })),
        ExprKind::Call(f, args) => match eval_expr(f, scope.clone(), ctx.clone())? {
            Value::Fn(Fn {
                params,
                body,
                scope: fn_scope,
            }) => {
                let mut evaluated_args = Tuple::new();
                for arg in args.iter() {
                    match arg {
                        TupleEntry::Pos(expr) => {
                            let value = eval_expr(expr, scope.clone(), ctx.clone())?;
                            evaluated_args.push(value);
                        }
                        TupleEntry::Named(key, expr) => {
                            let value = eval_expr(expr, scope.clone(), ctx.clone())?;
                            evaluated_args.insert(key, value);
                        }
                    }
                }
                let mut call_scope = Scope::new(Some(fn_scope));
                bind_tuple(
                    &mut call_scope,
                    &mut ctx.borrow_mut(),
                    &params,
                    evaluated_args,
                )?;
                eval_expr(&body, Rc::new(RefCell::new(call_scope)), ctx)
            }
            _ => Err(Error::Types),
        },
    }
}

fn bind_tuple(
    scope: &mut Scope,
    ctx: &mut Scope,
    pat: &Tuple<Pat>,
    value: Tuple<Value>,
) -> Result<()> {
    let mut i = 0usize;
    for entry in pat.iter() {
        match entry {
            TupleEntry::Pos(pat) => {
                let value = value.get(i).ok_or(Error::Mismatch)?;
                bind(scope, ctx, pat, value.clone())?;
                i += 1;
            }
            TupleEntry::Named(key, pat) => {
                let value = value.get(key).ok_or(Error::Mismatch)?;
                bind(scope, ctx, pat, value.clone())?;
            }
        }
    }
    Ok(())
}

fn bind(scope: &mut Scope, ctx: &mut Scope, pat: &Pat, value: Value) -> Result<()> {
    match pat {
        Pat::Ident(sym) => {
            scope.bind(*sym, value);
            Ok(())
        }
        Pat::CtxIdent(sym) => {
            ctx.bind(*sym, value);
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
            Value::Tuple(value) => bind_tuple(scope, ctx, tuple, value),
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

pub fn eval_stmt(
    stmt: &Stmt,
    scope: Rc<RefCell<Scope>>,
    ctx: Rc<RefCell<Scope>>,
) -> Result<Option<Value>> {
    match stmt {
        Stmt::Let(pat, expr) => {
            let value = eval_expr(expr, scope.clone(), ctx.clone())?;
            bind(&mut scope.borrow_mut(), &mut ctx.borrow_mut(), pat, value)?;
            Ok(None)
        }
        Stmt::Expr(expr) => Ok(Some(eval_expr(expr, scope, ctx)?)),
    }
}

pub fn eval_module(
    module_ast: &ast::Mod,
    scope: Rc<RefCell<Scope>>,
    ctx: Rc<RefCell<Scope>>,
) -> Result<Mod> {
    let module = Mod {
        deps: module_ast.deps.clone(),
        scope: Default::default(),
    };
    for stmt in module_ast.defs.iter() {
        eval_stmt(stmt, scope.clone(), ctx.clone())?;
    }
    Ok(module)
}

#[cfg(test)]
mod tests {
    use super::*;
    use vamp_sym::{Interner, Sym};
    use vamp_syntax::parser::parse_expr;

    fn eval_string(source: &str) -> Result<Value> {
        let mut interner = Interner::new();
        let scope = Rc::new(RefCell::new(Scope::default()));
        let ctx = Rc::new(RefCell::new(Scope::default()));
        let value = parse_expr(source, &mut interner).unwrap();
        eval_expr(&value, scope, ctx)
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

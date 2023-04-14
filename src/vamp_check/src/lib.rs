pub mod error;

use error::{Error, Result};
use vamp_syntax::ast::{BinOp, Expr, ExprKind, Let, Mod, Stmt};
use vamp_ty::Ty;

pub fn check_expr(expr: &mut Expr) -> Result<()> {
    match &mut expr.kind {
        ExprKind::Void => {
            expr.ty = Ty::Void;
            Ok(())
        }
        ExprKind::Nil => {
            expr.ty = Ty::Nil;
            Ok(())
        }
        ExprKind::Sym(..) => {
            expr.ty = Ty::Sym;
            Ok(())
        }
        ExprKind::Str(..) => {
            expr.ty = Ty::Str;
            Ok(())
        }
        ExprKind::Int(..) => {
            expr.ty = Ty::Int;
            Ok(())
        }
        ExprKind::Float(..) => {
            expr.ty = Ty::Float;
            Ok(())
        }
        ExprKind::BinOp(bin_op, left, right) => {
            check_expr(left)?;
            check_expr(right)?;
            match bin_op {
                BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod | BinOp::Exp => {
                    if left.ty == Ty::Int && right.ty == Ty::Int {
                        // Arithmetic on integers results in integers.
                        expr.ty = Ty::Int;
                        Ok(())
                    } else if left.ty == Ty::Float && right.ty == Ty::Float {
                        // Arithmetic on floats results in floats.
                        expr.ty = Ty::Float;
                        Ok(())
                    } else {
                        Err(Error::TypeError {
                            expected: Ty::Int,
                            found: Ty::Float,
                        })
                    }
                }
                BinOp::And | BinOp::Or => {
                    if left.ty != Ty::Bool {
                        Err(Error::TypeError {
                            expected: Ty::Bool,
                            found: left.ty.clone(),
                        })
                    } else if right.ty != Ty::Bool {
                        Err(Error::TypeError {
                            expected: Ty::Bool,
                            found: right.ty.clone(),
                        })
                    } else {
                        expr.ty = Ty::Bool;
                        Ok(())
                    }
                }
                _ => todo!(),
            }
        }
        _ => todo!(),
    }
}

pub fn check_statement(statement: &mut Stmt) -> Result<()> {
    match statement {
        Stmt::Let(Let(_, expr)) => {
            check_expr(expr)?;
            Ok(())
        }
        Stmt::Expr(expr) => {
            check_expr(expr)?;
            Ok(())
        }
    }
}

pub fn check_module(module: &mut Mod) -> Result<()> {
    for statement in module.definitions.iter_mut() {
        check_statement(statement)?;
    }
    Ok(())
}

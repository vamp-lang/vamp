use crate::ast::{BinOp, Expr};
use crate::source::Error;
use crate::vm::{Op, Val};
use bincode;
use std::io::Write;

struct Compiler<W: Write> {
    writer: W,
}

impl<W: Write> Compiler<W> {
    #[inline]
    fn new(writer: W) -> Self {
        Compiler { writer }
    }

    #[inline]
    fn write(&mut self, op: &Op) {
        bincode::serialize_into(&mut self.writer, op).unwrap();
    }

    fn compile(&mut self, ast: &Expr) -> Result<(), Error> {
        match *ast {
            Expr::Void => self.write(&Op::Exit),
            Expr::Nil => self.write(&Op::Push(Val::Nil)),
            Expr::Int(a) => self.write(&Op::Push(Val::Int(a))),
            Expr::Float(a) => self.write(&Op::Push(Val::Float(a))),
            Expr::Symbol(s) => self.write(&Op::Push(Val::Symbol(s))),
            Expr::String(s) => self.write(&Op::Push(Val::String(s.into()))),
            Expr::BinOp(bin_op, l, r) => {
                self.compile(l)?;
                self.compile(r)?;
                self.write(&match bin_op mut {
                    BinOp::Add => Op::Add,
                    BinOp::Sub => Op::Sub,
                    BinOp::Mul => Op::Mul,
                    BinOp::Div => Op::Div,
                });
            }
            _ => todo!(),
        }
        Ok(())
    }
}

pub fn compile(ast: &Expr) -> Result<Vec<u8>, Error> {
    let mut writer = vec![];
    Compiler::new(&mut writer).compile(ast)?;
    Ok(writer)
}

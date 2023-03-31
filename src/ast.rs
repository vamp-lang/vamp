use crate::symbol::Symbol;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TupMember<T> {
    Pos(T),
    Named(Symbol, T),
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Pat<'ast> {
    Nil,
    Tuple(&'ast [TupMember<Pat<'ast>>]),
    Vector(&'ast [Pat<'ast>]),
    Identifier(Symbol),
    Symbol(Symbol),
    String(&'ast str),
    Int(i64),
    Float(f64),
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Stmt<'ast> {
    Use(Pat<'ast>, Expr<'ast>),
    Let(Pat<'ast>, Expr<'ast>),
    Expr(Expr<'ast>),
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Expr<'ast> {
    Void,
    Nil,
    Block(&'ast [Stmt<'ast>]),
    Tuple(&'ast [TupMember<Expr<'ast>>]),
    Vector(&'ast [Expr<'ast>]),
    Call(&'ast Expr<'ast>, &'ast [TupMember<Expr<'ast>>]),
    Function(&'ast [TupMember<Expr<'ast>>], &'ast Expr<'ast>),
    Identifier(Symbol),
    Symbol(Symbol),
    String(&'ast str),
    Int(i64),
    Float(f64),
    BinOp(BinOp, &'ast Expr<'ast>, &'ast Expr<'ast>),
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Import<'ast>(pub Pat<'ast>, pub &'ast str);

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Module<'ast> {
    pub imports: &'ast [Import<'ast>],
    pub body: Expr<'ast>,
}

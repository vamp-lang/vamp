use crate::symbol::Symbol;

#[derive(Debug, PartialEq, Clone)]
pub enum PatternTupleMember<'ast> {
    Positional(Pattern<'ast>),
    Named(Symbol, Pattern<'ast>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Pattern<'ast> {
    Nil,
    Tuple(&'ast [PatternTupleMember<'ast>]),
    Vector(&'ast [Pattern<'ast>]),
    Identifier(Symbol),
    Symbol(Symbol),
    String(&'ast str),
    Int(i64),
    Float(f64),
}

#[derive(Debug, PartialEq, Clone)]
pub enum TupleMember<'ast> {
    Positional(Expr<'ast>),
    Named(Symbol, Expr<'ast>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement<'ast> {
    Use(Pattern<'ast>, Expr<'ast>),
    Let(Pattern<'ast>, Expr<'ast>),
    Expr(Expr<'ast>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum BuiltIn {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Exp,
    Index,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr<'ast> {
    Void,
    Nil,
    Block(&'ast [Statement<'ast>]),
    If(&'ast Expr<'ast>, &'ast [Statement<'ast>]),
    For(&'ast Expr<'ast>, &'ast [Statement<'ast>]),
    Tuple(&'ast [TupleMember<'ast>]),
    Vector(&'ast [Expr<'ast>]),
    Call(&'ast Expr<'ast>, &'ast [TupleMember<'ast>]),
    Function(&'ast [PatternTupleMember<'ast>], &'ast Expr<'ast>),
    Identifier(Symbol),
    Symbol(Symbol),
    String(&'ast str),
    Int(i64),
    Float(f64),
    BuiltIn(BuiltIn),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Import<'ast>(pub Pattern<'ast>, pub &'ast str);

#[derive(Debug, PartialEq, Clone)]
pub struct Module<'ast> {
    pub imports: &'ast [Import<'ast>],
    pub body: Expr<'ast>,
}

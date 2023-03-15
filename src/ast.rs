use crate::symbol::Symbol;

pub enum PatternTupleMember<'ast> {
    Positional(Pattern<'ast>),
    Named(Symbol, Pattern<'ast>),
}

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

pub enum TupleMember<'ast> {
    Positional(Expr<'ast>),
    Named(Symbol, Expr<'ast>),
}

pub enum Statement<'ast> {
    Use(Pattern<'ast>, &'ast Expr<'ast>),
    Let(Pattern<'ast>, &'ast Expr<'ast>),
    Expr(Expr<'ast>),
}

pub enum BuiltIn {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Exp,
    Index,
}

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

pub struct Import<'ast>(pub Pattern<'ast>, pub &'ast str);

pub struct Module<'ast> {
    pub imports: &'ast [Import<'ast>],
    pub body: Expr<'ast>,
    pub export: Expr<'ast>,
}

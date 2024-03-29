use vamp_sym::Sym;
use vamp_tuple::Tuple;
use vamp_ty::Ty;

#[derive(Debug, PartialEq, Clone)]
pub enum Pat {
    Tuple(Tuple<Pat>),
    List(Box<[Pat]>),
    Ident(Sym),
    CtxIdent(Sym),
    Sym(Sym),
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Wild,
}

/// A block statement.
#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    /// A let binding `let y = f(x)`.
    Let(Pat, Expr),
    /// An unbound expression `f(x)`.
    Expr(Expr),
}

// Unary operators.
#[derive(Debug, PartialEq, Clone)]
pub enum UnOp {
    /// Negation `(-)`
    Neg,
    /// Logical not `(!)`
    Not,
    /// Bitwise not `(~)`
    BitNot,
}

/// Binary operators.
#[derive(Debug, PartialEq, Clone)]
pub enum BinOp {
    // Property lookup
    /// Dot `(.)`
    Dot,

    // Mathematical
    /// Addition `(+)`
    Add,
    /// Subtraction `(-)`
    Sub,
    /// Multiplication `(*)`
    Mul,
    /// Division `(/)`
    Div,
    /// Modulo `(%)`
    Mod,
    /// Exponentiation `(**)`
    Exp,

    // Logical
    /// Equality `(==)`
    Eq,
    /// Inequality `(!=)`
    NotEq,
    /// Less than `(<)`
    Lt,
    /// Less than or equal to `(<=)`
    LtEq,
    /// Greater than `(>)`
    Gt,
    /// Greater than or equal to `(>=)`
    GtEq,
    /// Logical and `(&&)`
    And,
    /// Logical or `(||)`
    Or,

    // Bitwise
    /// Bitwise and `(&)`
    BitAnd,
    /// Bitwise or `(|)`
    BitOr,
    /// Bitwise xor `(^)`
    Xor,
    /// Bitwise left shift `(<<)`
    ShiftL,
    /// Bitwise right shift `(>>)`
    ShiftR,
}

/// An expression. Except for a `Module`, which has no value, everything in Vamp
/// builds and composes from `Expr`.
#[derive(Debug, PartialEq, Clone)]
pub enum ExprKind {
    /// An empty sequence of statements `{}`.
    Void,
    /// A nonempty sequence of statements `{...}`.
    Block(Box<[Stmt]>),
    /// A tuple `(...)`.
    Tuple(Tuple<Expr>),
    /// A list literal `[...]`.
    List(Box<[Expr]>),
    /// A function call/application.
    Call(Box<Expr>, Tuple<Expr>),
    /// A function abstraction.
    Fn(Tuple<Pat>, Box<Expr>),
    /// An identifier.
    Ident(Sym),
    /// A context identifier.
    CtxIdent(Sym),
    /// A symbol literal `'abc'`.
    Sym(Sym),
    /// A string literal `"abc"`.
    Str(String),
    /// An integer literal `1`.
    Int(i64),
    /// A floating point literal `1.2`.
    Float(f64),
    /// A boolean literal `true`.
    Bool(bool),
    /// A unary operator applied to a single operand expression.
    UnOp(UnOp, Box<Expr>),
    /// A binary operator applied to two operand expressions.
    BinOp(BinOp, Box<Expr>, Box<Expr>),
    /// An if-else expression.
    IfElse(Box<Expr>, Box<Expr>, Box<Expr>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Expr {
    pub ty: Ty,
    pub kind: ExprKind,
}

impl Expr {
    pub fn unknown(kind: ExprKind) -> Self {
        Self {
            ty: Ty::Unknown,
            kind,
        }
    }
}

/// A module's location.
#[derive(Debug, PartialEq, Clone)]
pub struct ModPath {
    /// Whether or not the module is local to the curent package.
    pub local: bool,
    /// The module path's segments split by `"."`.
    pub segments: Box<[Sym]>,
}

/// Represents a dependency on a single module.
#[derive(Debug, PartialEq, Clone)]
pub struct Dep {
    /// The location of the module being depended on.
    pub path: ModPath,
    /// A map of symbols to bind in the form `[(source, destination), ...]`.
    pub bindings: Box<[(Sym, Sym)]>,
}

/// The top-level type for Vamp files/modules.
#[derive(Debug, PartialEq, Clone)]
pub struct Mod {
    /// A module's dependencies.
    pub deps: Box<[Dep]>,
    /// A module's definitions.
    pub defs: Box<[Stmt]>,
}

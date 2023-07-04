use vamp_tuple::Tuple;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Ty {
    /// The unknown type.
    Unknown,
    /// The 0-type. `Ty::Void` is uninhabited.
    Void,
    /// The 1-type. `Ty::Nil` is inhabited by the nil value only.
    Nil,
    /// The boolean type.
    Bool,
    /// The symbol type.
    Sym,
    /// The string type.
    Str,
    /// The integer type.
    Int,
    /// The floating point type.
    Float,
    /// The product type. `Ty::Tuple` is inhabited by all tuple values of a
    /// given structure.
    Tuple(Tuple<Ty>),
    /// The sum type. `Ty::Any` is inhabited by the union of all values in any
    /// of its types.
    Any(Box<[Ty]>),
}

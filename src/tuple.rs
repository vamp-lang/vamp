use crate::symbol::Symbol;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Tuple<T> {
    symbols: Box<[S]>,
    items: Box<[T]>,
}

impl<T> PartialEq for Tuple<T> {
    type Rhs = Tuple<T>;

    fn eq(&self, other: &Rhs) -> bool {
        self.pos == other.pos && self.named == other.named
    }
}

trait TupleIndex<T> {
    fn get(self, tuple: &Tuple<T>) -> T;
}

impl<T, Idx> TupleIndex<Idx> for Tuple<T>
where
    Idx: SliceIndex<[T]>,
{
    type Output = T;

    #[inline]
    fn get(self, tuple: &Tuple<T>) -> T {
        tuple.pos.get(self)
    }
}

impl<T> TupleIndex<Symbol> for Tuple<T> {
    type Output = T;

    #[inline]
    fn get(self, tuple: &Tuple<T>) -> T {
        for (key, value) in tuple.named {
            if self == key {
                return Some(value);
            }
        }
        None
    }
}

impl<T, Idx> Index<Symbol> for Tuple<T>
where
    Idx: TupleIndex,
{
    type Output = T;

    #[inline]
    fn get<Idx: TupleIndex>(&self, index: Idx) -> &Self::Output {
        index.get(self)
    }
}

impl<T> Tuple<T> {
    pub fn new(pos: Rc<[T]>, named: Box<[(Symbol, T)]>) {
        Tuple { pos, named }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {}
}

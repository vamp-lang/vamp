use vamp_sym::Sym;

/// Represents a single positional or named entry in a tuple.
pub enum TupleEntry<T> {
    /// A positional tuple entry.
    Pos(T),
    /// A named tuple entry.
    Named(Sym, T),
}

impl<T: std::fmt::Debug> std::fmt::Debug for TupleEntry<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TupleEntry::Pos(value) => f.debug_tuple("Pos").field(&value).finish(),
            TupleEntry::Named(symbol, value) => {
                f.debug_tuple("Named").field(&symbol).field(&value).finish()
            }
        }
    }
}

impl<T: PartialEq> PartialEq for TupleEntry<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TupleEntry::Pos(a), TupleEntry::Pos(b)) => a.eq(b),
            (TupleEntry::Named(s1, a), TupleEntry::Named(s2, b)) => s1 == s2 && a.eq(b),
            _ => false,
        }
    }
}

impl<T: Eq> Eq for TupleEntry<T> {}

impl<T: std::fmt::Debug> std::fmt::Debug for Tuple<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Tuple")
            .field("keys", &self.keys)
            .field("data", &self.data)
            .finish()
    }
}

/// Represents a combination of positional and named members.
pub struct Tuple<T> {
    /// Sorted list of keys.
    pub(crate) keys: Vec<Sym>,
    /// Tuple data, represented as `keys.len() - `data.len()` positional items,
    /// followed by `keys.len()` named items corresponding to `keys`.
    pub(crate) data: Vec<T>,
}

impl<T> Default for Tuple<T> {
    fn default() -> Self {
        Tuple {
            keys: vec![],
            data: vec![],
        }
    }
}

impl<T: PartialEq> PartialEq for Tuple<T> {
    fn eq(&self, other: &Self) -> bool {
        self.keys == other.keys && self.data == other.data
    }
}

impl<T: Eq> Eq for Tuple<T> {}

pub trait TupleIndex<T> {
    fn index(self, tuple: &Tuple<T>) -> Option<&T>;
}

impl<T> TupleIndex<T> for usize {
    fn index(self, tuple: &Tuple<T>) -> Option<&T> {
        tuple.data.get(self)
    }
}

impl<T> TupleIndex<T> for Sym {
    fn index(self, tuple: &Tuple<T>) -> Option<&T> {
        tuple.data.get(tuple.key_position(self)?)
    }
}

impl<T> std::ops::Index<usize> for Tuple<T> {
    type Output = T;

    fn index(&self, i: usize) -> &T {
        &self.data[i]
    }
}

impl<T> std::ops::IndexMut<usize> for Tuple<T> {
    fn index_mut(&mut self, i: usize) -> &mut T {
        &mut self.data[i]
    }
}

impl<T> std::ops::Index<Sym> for Tuple<T> {
    type Output = T;

    fn index(&self, key: Sym) -> &T {
        &self.data[self.key_position(key).unwrap()]
    }
}

impl<T: Clone> Clone for Tuple<T> {
    fn clone(&self) -> Self {
        Tuple {
            keys: self.keys.clone(),
            data: self.data.clone(),
        }
    }
}

impl<T> Tuple<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn keys_len(&self) -> usize {
        self.keys.len()
    }

    fn named_offset(&self) -> usize {
        self.data.len() - self.keys.len()
    }

    fn key_position(&self, key: Sym) -> Option<usize> {
        self.keys
            .binary_search(&key)
            .ok()
            .map(|i| self.named_offset() + i)
    }

    pub fn get<Idx: TupleIndex<T>>(&self, i: Idx) -> Option<&T> {
        i.index(&self)
    }

    pub fn push(&mut self, value: T) {
        self.data.insert(self.named_offset(), value);
    }

    pub fn insert(&mut self, key: Sym, value: T) -> Option<T> {
        let offset = self.named_offset();
        match self.keys.binary_search(&key) {
            Ok(i) => Some(std::mem::replace(&mut self.data[offset + i], value)),
            Err(i) => {
                self.keys.insert(i, key);
                self.data.insert(offset + i, value);
                None
            }
        }
    }

    pub fn iter(&self) -> Iter<T> {
        Iter {
            tuple: self,
            index: 0,
        }
    }
}

#[derive(Debug)]
pub struct Iter<'a, T> {
    tuple: &'a Tuple<T>,
    index: usize,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = TupleEntry<&'a T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.tuple.len() {
            return None;
        }
        let named_offset = self.tuple.named_offset();
        let entry = if self.index < named_offset {
            TupleEntry::Pos(&self.tuple.data[self.index])
        } else {
            TupleEntry::Named(
                self.tuple.keys[self.index - named_offset],
                &self.tuple.data[self.index],
            )
        };
        self.index += 1;
        Some(entry)
    }
}

impl<T> FromIterator<TupleEntry<T>> for Tuple<T> {
    fn from_iter<I>(entries: I) -> Self
    where
        I: IntoIterator<Item = TupleEntry<T>>,
    {
        let mut tuple = Tuple::new();
        for entry in entries.into_iter() {
            match entry {
                TupleEntry::Pos(value) => tuple.push(value),
                TupleEntry::Named(key, value) => {
                    tuple.insert(key, value);
                }
            }
        }
        tuple
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positional() {
        let mut tuple = Tuple::new();
        tuple.push(0);
        tuple.push(1);
        tuple.push(2);
        assert_eq!(tuple.len(), 3);
        assert_eq!(tuple.keys_len(), 0);
        assert_eq!(tuple[0], 0);
        assert_eq!(tuple[1], 1);
        assert_eq!(tuple[2], 2);
    }

    #[test]
    fn named() {
        let mut tuple = Tuple::new();
        tuple.insert(Sym(2), "a");
        tuple.insert(Sym(1), "b");
        tuple.insert(Sym(0), "c");
        assert_eq!(tuple.len(), 3);
        assert_eq!(tuple.keys_len(), 3);
        assert_eq!(tuple.key_position(Sym(2)), Some(2));
        assert_eq!(tuple.key_position(Sym(1)), Some(1));
        assert_eq!(tuple.key_position(Sym(0)), Some(0));
        assert_eq!(tuple[Sym(2)], "a");
        assert_eq!(tuple[Sym(1)], "b");
        assert_eq!(tuple[Sym(0)], "c");
    }

    #[test]
    fn collect() {
        let tuple: Tuple<_> = [
            TupleEntry::Pos("a"),
            TupleEntry::Pos("b"),
            TupleEntry::Named(Sym(0), "c"),
            TupleEntry::Pos("d"),
            TupleEntry::Named(Sym(1), "e"),
        ]
        .into_iter()
        .collect();
        assert_eq!(tuple[0], "a");
        assert_eq!(tuple[1], "b");
        assert_eq!(tuple[2], "d");
        assert_eq!(tuple[Sym(0)], "c");
        assert_eq!(tuple[Sym(1)], "e");
    }
}

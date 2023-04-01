use rustc_hash::FxHashMap;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Sym(pub u32);

#[derive(Default)]
pub struct Interner {
    map: FxHashMap<String, Sym>,
    vector: Vec<String>,
}

impl Interner {
    /// Constructs an empty `SymTable`.
    pub fn new() -> Self {
        Interner::default()
    }

    /// Interns `string` and returns a `Sym`.
    pub fn intern(&mut self, string: &str) -> Sym {
        if let Some(&symbol) = self.map.get(string) {
            return symbol;
        }
        let symbol = Sym(self.vector.len() as u32);
        self.map.insert(string.into(), symbol);
        self.vector.push(string.into());
        symbol
    }

    /// Generates a private symbol.
    pub fn private(&mut self) -> Sym {
        let n = self.vector.len() as u32;
        let symbol = Sym(n);
        let string = format!("#{}", n);
        self.map.insert(string.clone(), symbol);
        self.vector.push(string);
        symbol
    }

    /// Looks up the string value of `symbol`.
    pub fn lookup(&self, symbol: Sym) -> &str {
        &self.vector[symbol.0 as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intern() {
        let mut table = Interner::new();
        assert_eq!(table.intern("".into()), Sym(0));
        assert_eq!(table.intern("".into()), table.intern("".into()));
        assert_eq!(table.intern("abc".into()), Sym(1));
        assert_eq!(table.intern("abc".into()), table.intern("abc".into()));
    }

    #[test]
    fn lookup() {
        let mut table = Interner::new();
        let strings = ["", "x0", "@self", "d013397b-f874-49e0-9f38-01fa235caabc"];
        let symbols: Vec<_> = strings.iter().map(|&s| table.intern(s.into())).collect();
        for i in 0..symbols.len() {
            assert_eq!(table.lookup(symbols[i]), strings[i]);
        }
    }
}

use rustc_hash::FxHashMap;

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub struct Symbol(pub u32);

pub struct Interner {
    map: FxHashMap<String, Symbol>,
    vector: Vec<String>,
}

impl Interner {
    pub fn new() -> Interner {
        Interner {
            map: FxHashMap::default(),
            vector: Vec::new(),
        }
    }

    pub fn intern(&mut self, name: &str) -> Symbol {
        if let Some(&symbol) = self.map.get(name) {
            return symbol;
        }
        let symbol = Symbol(self.vector.len() as u32);
        self.map.insert(name.into(), symbol);
        self.vector.push(name.into());
        debug_assert!(self.lookup(symbol) == name);
        debug_assert!(self.intern(name) == symbol);
        symbol
    }

    pub fn lookup(&self, symbol: Symbol) -> &str {
        &self.vector[symbol.0 as usize]
    }
}

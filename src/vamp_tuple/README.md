# vamp_tuple

This crate contains a generic "tuple" data structure that combines positional and symbol-based access. This data structure is used throughout compiler phases to represent tuple values, types, etc. It is optimized to perform well for a small number of tuple elements (<100) since this is the most common case in source code. Internally, `Tuple<T>` is implemented as a `Vec<T>` with a binary heap `Vec<Sym>` that defines the positioning of symbolic members. Positional lookup is `O(1)` and symbolic lookup is `O(log(n))`.

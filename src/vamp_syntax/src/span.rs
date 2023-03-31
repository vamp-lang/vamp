use std::ops::Index;

/// A span of characters in source code.
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct Span {
    /// The inclusive start byte-offset in the source code.
    pub start: usize,
    /// The exclusive end byte-offset in the source code.
    pub end: usize,
}

impl Index<Span> for str {
    type Output = str;

    #[inline]
    fn index(&self, span: Span) -> &str {
        &self[span.start..span.end]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index() {
        let slice = "Slice me up.";
        let span = Span { start: 6, end: 8 };
        assert_eq!(&slice[span], "me");
    }
}

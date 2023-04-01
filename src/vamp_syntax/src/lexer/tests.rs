use super::*;

fn token_slices(source: &str) -> Result<Vec<(TokenKind, &str)>> {
    let tokens = tokenize(source)?;
    Ok(tokens
        .into_iter()
        .map(|Token { kind, span }| (kind, &source[span]))
        .collect())
}

#[test]
fn whitespace() {
    assert_eq!(token_slices(" \t\n\r"), Ok(vec![]));
    assert_eq!(
        token_slices("# This is a comment\n# This is another comment\n"),
        Ok(vec![])
    );
}

#[test]
fn valid_tokens() {
    let cases = [
        // Punctuation
        (TokenKind::LParen, "("),
        (TokenKind::RParen, ")"),
        (TokenKind::LBracket, "["),
        (TokenKind::RBracket, "]"),
        (TokenKind::LBrace, "{"),
        (TokenKind::RBrace, "}"),
        (TokenKind::Comma, ","),
        (TokenKind::Colon, ":"),
        (TokenKind::Period, "."),
        // Operators
        (TokenKind::Plus, "+"),
        (TokenKind::Minus, "-"),
        (TokenKind::Star, "*"),
        (TokenKind::StarStar, "**"),
        (TokenKind::Slash, "/"),
        (TokenKind::Percent, "%"),
        (TokenKind::Eq, "="),
        (TokenKind::EqEq, "=="),
        (TokenKind::NotEq, "!="),
        (TokenKind::Lt, "<"),
        (TokenKind::LtLt, "<<"),
        (TokenKind::LtEq, "<="),
        (TokenKind::Gt, ">"),
        (TokenKind::GtGt, ">>"),
        (TokenKind::GtEq, ">="),
        (TokenKind::Not, "!"),
        (TokenKind::And, "&"),
        (TokenKind::AndAnd, "&&"),
        (TokenKind::Or, "|"),
        (TokenKind::OrOr, "||"),
        (TokenKind::Caret, "^"),
        (TokenKind::Tilde, "~"),
        // Keywords
        (TokenKind::Use, "use"),
        (TokenKind::Let, "let"),
        (TokenKind::Type, "type"),
        (TokenKind::If, "if"),
        (TokenKind::Else, "else"),
        (TokenKind::For, "for"),
        // Identifiers
        (TokenKind::Ident, "_"),
        (TokenKind::Ident, "t"),
        (TokenKind::Ident, "x1"),
        (TokenKind::Ident, "emailAddress"),
        (TokenKind::Ident, "first_name"),
        (TokenKind::Ident, "_dateOfBirth"),
        (TokenKind::Ident, "T"),
        (TokenKind::Ident, "X1"),
        (TokenKind::Ident, "Identifier"),
        (TokenKind::Ident, "SHIFT_RIGHT"),
        (TokenKind::Ident, "@"),
        (TokenKind::Ident, "@self"),
        // Symbol literals
        (TokenKind::Sym, "''"),
        (TokenKind::Sym, "'_'"),
        (TokenKind::Sym, r#"'\''"#),
        (TokenKind::Sym, "'abc'"),
        // String literals
        (TokenKind::Str, r#""""#),
        (TokenKind::Str, r#""\\""#),
        (TokenKind::Str, r#""\\\"""#),
        (TokenKind::Str, r#""\"\"""#),
        (
            TokenKind::Str,
            r#""The quick brown fox jumps over the lazy dog.""#,
        ),
        // Int literals
        (TokenKind::Int, "0"),
        (TokenKind::Int, "12"),
        (TokenKind::Int, "539"),
        (TokenKind::Int, "0777"),
        (TokenKind::Int, "0b1010"),
        (TokenKind::Int, "0xfAb93"),
        // Float literals
        (TokenKind::Float, "0."),
        (TokenKind::Float, "0.5"),
        (TokenKind::Float, "3.14"),
        (TokenKind::Float, "1e10"),
        (TokenKind::Float, "2.5e2"),
        (TokenKind::Float, "1e-10"),
    ];
    for (kind, slice) in cases {
        assert_eq!(token_slices(slice), Ok(vec![(kind, slice)]));
    }
}

#[test]
fn auto_insert_comma() {
    assert_eq!(
        token_slices(
            "
            x
            y
            z
            "
        ),
        Ok(vec![
            (TokenKind::Ident, "x"),
            (TokenKind::Comma, ""),
            (TokenKind::Ident, "y"),
            (TokenKind::Comma, ""),
            (TokenKind::Ident, "z"),
            (TokenKind::Comma, "")
        ]),
    );
}

#[test]
fn string_unterminated() {
    assert!(matches!(
        token_slices("\""),
        Err(Error {
            kind: ErrorKind::StringUnterminated,
            detail: None,
            span: _,
        })
    ));
}

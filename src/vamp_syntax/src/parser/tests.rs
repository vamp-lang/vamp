use super::*;

#[test]
fn identifiers() {
    let mut interner = Interner::new();
    let x = interner.intern("x");
    let y2 = interner.intern("y2");
    let at_0 = interner.intern("@0");
    let at_self = interner.intern("@self");
    let lower_snake_case = interner.intern("lower_snake_case");
    let upper_snake_case = interner.intern("UPPER_SNAKE_CASE");
    let lower_camel_case = interner.intern("lowerCamelCase");
    let upper_camel_case = interner.intern("UpperCamelCase");
    assert_eq!(parse_expr("x", &mut interner), Ok(Expr::Ident(x)));
    assert_eq!(parse_expr("y2", &mut interner), Ok(Expr::Ident(y2)));
    assert_eq!(parse_expr("@0", &mut interner), Ok(Expr::Ident(at_0)));
    assert_eq!(parse_expr("@self", &mut interner), Ok(Expr::Ident(at_self)));
    assert_eq!(
        parse_expr("lower_snake_case", &mut interner),
        Ok(Expr::Ident(lower_snake_case))
    );
    assert_eq!(
        parse_expr("UPPER_SNAKE_CASE", &mut interner),
        Ok(Expr::Ident(upper_snake_case))
    );
    assert_eq!(
        parse_expr("lowerCamelCase", &mut interner),
        Ok(Expr::Ident(lower_camel_case))
    );
    assert_eq!(
        parse_expr("UpperCamelCase", &mut interner),
        Ok(Expr::Ident(upper_camel_case))
    );
}

#[test]
fn symbol() {
    let mut interner = Interner::new();
    let s1 = interner.intern("");
    let s2 = interner.intern(r#"\"#);
    let s3 = interner.intern("x");
    assert_eq!(parse_expr("''", &mut interner), Ok(Expr::Sym(s1)));
    assert_eq!(parse_expr(r#"'\\'"#, &mut interner), Ok(Expr::Sym(s2)));
    assert_eq!(parse_expr("'x'", &mut interner), Ok(Expr::Sym(s3)));
}

#[test]
fn string() {
    let mut interner = Interner::new();
    assert_eq!(
        parse_expr(r#""""#, &mut interner),
        Ok(Expr::Str(String::from("")))
    );
    assert_eq!(
        parse_expr(r#""\"""#, &mut interner),
        Ok(Expr::Str(String::from("\"")))
    );
    assert_eq!(
        parse_expr(r#""\\""#, &mut interner),
        Ok(Expr::Str(String::from("\\")))
    );
    assert_eq!(
        parse_expr(r#""\0\a\b\t\v\f\n\r""#, &mut interner),
        Ok(Expr::Str(String::from("\0\x07\x08\t\x0B\x0C\n\r")))
    );
    assert_eq!(
        parse_expr(r#""\x00\x01\x02\x03\x04\x05""#, &mut interner),
        Ok(Expr::Str(String::from("\x00\x01\x02\x03\x04\x05")))
    );
}

#[test]
fn string_invalid_escape_sequence() {
    let mut interner = Interner::new();
    assert_eq!(
        parse_expr(r#""\z""#, &mut interner).unwrap_err().kind,
        ErrorKind::StringEscSeqInvalid
    );
    assert_eq!(
        parse_expr(r#""\xFF""#, &mut interner).unwrap_err().kind,
        ErrorKind::StringEscSeqInvalid
    );
}

#[test]
fn integer() {
    let mut interner = Interner::new();
    assert_eq!(parse_expr("0", &mut interner), Ok(Expr::Int(0)));
    //assert_eq!(parse_expr("-0"), Ok(Expr::Integer(0)));
    assert_eq!(parse_expr("7", &mut interner), Ok(Expr::Int(7)));
    //assert_eq!(parse_expr("-3"), Ok(Expr::Integer(-3)));
    assert_eq!(parse_expr("123", &mut interner), Ok(Expr::Int(123)));
    //assert_eq!(parse_expr("-313"), Ok(Expr::Integer(-313)));
    assert_eq!(parse_expr("0o747", &mut interner), Ok(Expr::Int(0o747)));
    //assert_eq!(parse_expr("-002200"), Ok(Expr::Integer(-2200)));
    /*
    assert_eq!(
        parse_expr("9223372036854775807"),
        Ok(Expr::Integer(9223372036854775807))
    );
    assert_eq!(
        parse_expr("9223372036854775808").unwrap_err().kind,
        ErrorKind::InvalidInteger
    );
    assert_eq!(
        parse_expr("-9223372036854775808"),
        Ok(Expr::Integer(-9223372036854775808))
    );
    assert_eq!(
        parse_expr("-9223372036854775809").unwrap_err().kind,
        ErrorKind::InvalidInteger
    );
    */
}

#[test]
fn float() {
    let mut interner = Interner::new();
    assert_eq!(parse_expr("0.0", &mut interner), Ok(Expr::Float(0.0)));
    //assert_eq!(parse_expr("-0.0"), Ok(Expr::Float(0.0)));
    assert_eq!(parse_expr("1.0", &mut interner), Ok(Expr::Float(1.0)));
    //assert_eq!(parse_expr("-1.0"), Ok(Expr::Float(-1.0)));
    assert_eq!(
        parse_expr("3.141592", &mut interner),
        Ok(Expr::Float(3.141592))
    );
    //assert_eq!(parse_expr("-2.7800000"), Ok(Expr::Float(-2.78)));
}

#[test]
fn tuple() {
    let mut interner = Interner::new();
    let x = interner.intern("x");
    let y = interner.intern("y");
    let name = interner.intern("name");
    let age = interner.intern("age");
    assert_eq!(parse_expr("()", &mut interner), Ok(Expr::Nil));
    assert_eq!(
        parse_expr("(1)", &mut interner),
        Ok(Expr::Tuple(Tuple::from_iter([TupleEntry::Pos(Expr::Int(
            1
        ))])))
    );
    assert_eq!(
        parse_expr("(1, 2, 3)", &mut interner),
        Ok(Expr::Tuple(Tuple::from_iter([
            TupleEntry::Pos(Expr::Int(1)),
            TupleEntry::Pos(Expr::Int(2)),
            TupleEntry::Pos(Expr::Int(3)),
        ])))
    );
    assert_eq!(
        parse_expr("(x: 1, y: 2)", &mut interner),
        Ok(Expr::Tuple(Tuple::from_iter([
            TupleEntry::Named(x, Expr::Int(1)),
            TupleEntry::Named(y, Expr::Int(2))
        ])))
    );
    assert_eq!(
        parse_expr(r#"("id", name: "Bob", age: 49)"#, &mut interner),
        Ok(Expr::Tuple(Tuple::from_iter([
            TupleEntry::Pos(Expr::Str(String::from("id"))),
            TupleEntry::Named(name, Expr::Str(String::from("Bob"))),
            TupleEntry::Named(age, Expr::Int(49))
        ])))
    );
}

#[test]
fn vector() {
    let mut interner = Interner::new();
    assert_eq!(parse_expr("[]", &mut interner), Ok(Expr::List([].into())));
    assert_eq!(
        parse_expr("[1]", &mut interner),
        Ok(Expr::List([Expr::Int(1)].into()))
    );
    assert_eq!(
        parse_expr("[1, 2, 3]", &mut interner),
        Ok(Expr::List(
            [Expr::Int(1), Expr::Int(2), Expr::Int(3)].into()
        ))
    )
}

#[test]
fn precedence() {
    let mut interner = Interner::new();
    assert_eq!(
        parse_expr("0 + 0", &mut interner),
        Ok(Expr::BinOp(
            BinOp::Add,
            Expr::Int(0).into(),
            Expr::Int(0).into()
        ))
    );
    assert_eq!(
        parse_expr("0 * 0", &mut interner),
        Ok(Expr::BinOp(
            BinOp::Mul,
            Expr::Int(0).into(),
            Expr::Int(0).into()
        ))
    );
    assert_eq!(
        parse_expr("0 + 0 * 0", &mut interner),
        Ok(Expr::BinOp(
            BinOp::Add,
            Expr::Int(0).into(),
            Expr::BinOp(BinOp::Mul, Expr::Int(0).into(), Expr::Int(0).into()).into()
        ))
    );
    assert_eq!(
        parse_expr("0 * 0 + 0 / 0 - 0", &mut interner),
        Ok(Expr::BinOp(
            BinOp::Sub,
            Expr::BinOp(
                BinOp::Add,
                Expr::BinOp(BinOp::Mul, Expr::Int(0).into(), Expr::Int(0).into()).into(),
                Expr::BinOp(BinOp::Div, Expr::Int(0).into(), Expr::Int(0).into()).into(),
            )
            .into(),
            Expr::Int(0).into(),
        ))
    );
    let f = interner.intern("f");
    let g = interner.intern("g");
    let h = interner.intern("h");
    let x = interner.intern("x");
    let y = interner.intern("y");
    let z = interner.intern("z");
    assert_eq!(
        parse_expr("f(x) * g(y) + h(z)", &mut interner),
        Ok(Expr::BinOp(
            BinOp::Add,
            Expr::BinOp(
                BinOp::Mul,
                Expr::Call(
                    Expr::Ident(f).into(),
                    Tuple::from_iter([TupleEntry::Pos(Expr::Ident(x))]),
                )
                .into(),
                Expr::Call(
                    Expr::Ident(g).into(),
                    Tuple::from_iter([TupleEntry::Pos(Expr::Ident(y))]),
                )
                .into()
            )
            .into(),
            Expr::Call(
                Expr::Ident(h).into(),
                Tuple::from_iter([TupleEntry::Pos(Expr::Ident(z))]),
            )
            .into()
        )),
    );
}

#[test]
fn function() {
    let mut interner = Interner::new();
    let x = interner.intern("x");
    let y = interner.intern("y");
    let z = interner.intern("z");
    assert_eq!(
        parse_expr("|x| x", &mut interner),
        Ok(Expr::Fn(
            Tuple::from_iter([TupleEntry::Pos(Pat::Ident(x))]),
            Expr::Ident(x).into()
        ))
    );
    assert_eq!(
        parse_expr("|x, y, z| x(y, z)", &mut interner),
        Ok(Expr::Fn(
            Tuple::from_iter([
                TupleEntry::Pos(Pat::Ident(x)),
                TupleEntry::Pos(Pat::Ident(y)),
                TupleEntry::Pos(Pat::Ident(z)),
            ]),
            Expr::Call(
                Expr::Ident(x).into(),
                Tuple::from_iter([
                    TupleEntry::Pos(Expr::Ident(y)),
                    TupleEntry::Pos(Expr::Ident(z))
                ])
            )
            .into()
        ))
    )
}

#[test]
fn block() {
    let mut interner = Interner::new();
    let x = interner.intern("x");
    let y = interner.intern("y");
    assert_eq!(parse_expr("{}", &mut interner), Ok(Expr::Void));
    assert_eq!(parse_expr("{{{{{}}}}}", &mut interner), Ok(Expr::Void));
    assert_eq!(
        parse_expr("{ let x = 0, let y = 1, [x, y] }", &mut interner),
        Ok(Expr::Block(
            [
                Stmt::Let(Let(Pat::Ident(x), Expr::Int(0))),
                Stmt::Let(Let(Pat::Ident(y), Expr::Int(1))),
                Stmt::Expr(Expr::List([Expr::Ident(x), Expr::Ident(y)].into())),
            ]
            .into()
        ))
    );
    assert_eq!(parse_expr("{{1}}", &mut interner), Ok(Expr::Int(1)));
}

#[test]
fn module() {
    let mut interner = Interner::new();
    let x = interner.intern("x");
    let y = interner.intern("y");
    let z = interner.intern("z");
    let w = interner.intern("w");
    let q = interner.intern("q");
    assert_eq!(
        parse_module(
            "
            use {
                x.y.z (w)
            }
            let q = w
            ",
            &mut interner
        ),
        Ok(Mod {
            dependencies: [Dep {
                path: ModPath {
                    local: false,
                    segments: [x, y, z].into(),
                },
                bindings: [(w, w)].into()
            }]
            .into(),
            definitions: [Let(Pat::Ident(q), Expr::Ident(w))].into(),
        })
    );
}

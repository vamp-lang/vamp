use crate::{
    ast::{BinOp, Dep, Expr, ExprKind, Mod, ModPath, Pat, Stmt, UnOp},
    error::{Error, ErrorKind, Result},
    lexer::{tokenize, Token, TokenKind},
    span::Span,
};
use vamp_sym::{Interner, Sym};
use vamp_tuple::{Tuple, TupleEntry};

#[cfg(test)]
mod tests;

pub struct Parser<'src, 'sym> {
    source: &'src str,
    tokens: Vec<Token>,
    index: usize,
    interner: &'sym mut Interner,
}

impl<'src, 'sym> Parser<'src, 'sym> {
    fn new(source: &'src str, tokens: Vec<Token>, interner: &'sym mut Interner) -> Self {
        Self {
            source,
            tokens,
            index: 0,
            interner,
        }
    }

    fn accept(&mut self, kind: TokenKind) -> Option<Span> {
        if self.index < self.tokens.len() && self.tokens[self.index].kind == kind {
            let span = self.tokens[self.index].span;
            self.index += 1;
            Some(span)
        } else {
            None
        }
    }

    fn accept_slice(&mut self, kind: TokenKind) -> Option<&'src str> {
        self.accept(kind).map(|span| &self.source[span])
    }

    fn accept_sym(&mut self, kind: TokenKind) -> Option<Sym> {
        self.accept_slice(kind)
            .map(|slice| self.interner.intern(slice.into()))
    }

    fn accept_un_op(&mut self) -> Option<(UnOp, u8)> {
        if self.index < self.tokens.len() {
            let result = match self.tokens[self.index].kind {
                TokenKind::Minus => (UnOp::Neg, 20),
                TokenKind::Not => (UnOp::Not, 20),
                TokenKind::Tilde => (UnOp::BitNot, 20),
                _ => return None,
            };
            self.index += 1;
            Some(result)
        } else {
            None
        }
    }

    fn accept_bin_op(&mut self) -> Option<(BinOp, u8, u8)> {
        if self.index < self.tokens.len() {
            let result = match self.tokens[self.index].kind {
                TokenKind::OrOr => (BinOp::Or, 0, 1),
                TokenKind::AndAnd => (BinOp::And, 2, 3),
                TokenKind::EqEq => (BinOp::Eq, 4, 5),
                TokenKind::NotEq => (BinOp::NotEq, 4, 5),
                TokenKind::Lt => (BinOp::Lt, 4, 5),
                TokenKind::LtEq => (BinOp::LtEq, 4, 5),
                TokenKind::Gt => (BinOp::Gt, 4, 5),
                TokenKind::GtEq => (BinOp::GtEq, 4, 5),
                TokenKind::Or => (BinOp::BitOr, 6, 7),
                TokenKind::Caret => (BinOp::Xor, 8, 9),
                TokenKind::And => (BinOp::BitAnd, 10, 11),
                TokenKind::LtLt => (BinOp::ShiftL, 12, 13),
                TokenKind::GtGt => (BinOp::ShiftR, 12, 13),
                TokenKind::Plus => (BinOp::Add, 14, 15),
                TokenKind::Minus => (BinOp::Sub, 14, 15),
                TokenKind::Star => (BinOp::Mul, 16, 17),
                TokenKind::Slash => (BinOp::Div, 16, 17),
                TokenKind::Percent => (BinOp::Mod, 16, 17),
                TokenKind::StarStar => (BinOp::Exp, 18, 19),
                TokenKind::Period => (BinOp::Dot, 20, 21),
                _ => return None,
            };
            self.index += 1;
            Some(result)
        } else {
            None
        }
    }

    fn invalid_token(&self) -> Error {
        Error {
            kind: ErrorKind::InvalidToken,
            detail: None,
            span: self
                .tokens
                .get(self.index)
                .unwrap_or(&self.tokens[self.index - 1])
                .span,
        }
    }

    fn identifier(&mut self) -> Option<Sym> {
        self.accept_sym(TokenKind::Ident)
    }

    fn unescape(&mut self, span: Span) -> Result<String> {
        let slice = &self.source[span];
        let mut string = String::with_capacity(slice.len());
        let mut chars = slice[1..slice.len() - 1].chars();
        while let Some(c) = chars.next() {
            if c == '\\' {
                let invalid_escape_sequence = || Error {
                    kind: ErrorKind::StringEscSeqInvalid,
                    detail: None,
                    span,
                };
                // `unwrap()` here is safe because a string ending `\` such
                // as `"\"` would fail with `UnterminatedString`.
                let c = chars.next().unwrap();
                match c {
                    '\\' => string.push('\\'),
                    '"' => string.push('"'),
                    '\'' => string.push('\''),
                    // Bell
                    'a' => string.push('\x07'),
                    // Backspace
                    'b' => string.push('\x08'),
                    // Horizontal tab
                    't' => string.push('\t'),
                    // Vertical tab
                    'v' => string.push('\x0B'),
                    // Form feed
                    'f' => string.push('\x0C'),
                    // Newline
                    'n' => {
                        string.push('\n');
                    }
                    // Carriage return
                    'r' => {
                        string.push('\r');
                    }
                    // Nul
                    '0' => {
                        string.push('\0');
                    }
                    // Hexidecimal
                    'x' => {
                        let a = chars.next().ok_or_else(invalid_escape_sequence)?;
                        let b = chars.next().ok_or_else(invalid_escape_sequence)?;
                        let value =
                            16 * match a {
                                '0'..='9' => a as u8 - b'0',
                                'a'..='f' => 10 + a as u8 - b'a',
                                'A'..='F' => 10 + a as u8 - b'A',
                                _ => return Err(invalid_escape_sequence()),
                            } + match b {
                                '0'..='9' => b as u8 - b'0',
                                'a'..='f' => 10 + b as u8 - b'a',
                                'A'..='F' => 10 + b as u8 - b'A',
                                _ => return Err(invalid_escape_sequence()),
                            };
                        if value > 127 {
                            return Err(invalid_escape_sequence());
                        }
                        string.push(value as char);
                    }
                    _ => return Err(invalid_escape_sequence()),
                }
            } else {
                string.push(c)
            }
        }
        Ok(string)
    }

    fn symbol(&mut self) -> Result<Option<Sym>> {
        if let Some(span) = self.accept(TokenKind::Sym) {
            let unescaped = self.unescape(span)?;
            Ok(Some(self.interner.intern(&unescaped)))
        } else {
            Ok(None)
        }
    }

    fn string(&mut self) -> Result<Option<String>> {
        if let Some(span) = self.accept(TokenKind::Str) {
            let unescaped = self.unescape(span)?;
            Ok(Some(unescaped))
        } else {
            Ok(None)
        }
    }

    fn int(&mut self) -> Result<Option<i64>> {
        if let Some(int_span) = self.accept(TokenKind::Int) {
            let int_invalid = || Error {
                kind: ErrorKind::IntInvalid,
                detail: None,
                span: int_span,
            };
            let mut value = 0i64;
            let slice = &self.source[int_span];
            if slice.starts_with("0b") {
                // Binary literal
                // TODO: Optimize to use bit twiddling.
                for digit in slice[2..].bytes() {
                    value = value
                        .checked_mul(2)
                        .ok_or_else(int_invalid)?
                        .checked_add((digit - b'0') as i64)
                        .ok_or_else(int_invalid)?;
                }
            } else if slice.starts_with("0o") {
                // Octal literal
                // TODO: Optimize to use bit twiddling.
                for digit in slice[2..].bytes() {
                    value = value
                        .checked_mul(8)
                        .ok_or_else(int_invalid)?
                        .checked_add((digit - b'0') as i64)
                        .ok_or_else(int_invalid)?;
                }
            } else if slice.starts_with("0x") {
                // Hexadecimal literal
                // TODO: Optimize to use bit twiddling.
                for digit in slice[2..].bytes() {
                    let n = if matches!(digit, b'0'..=b'9') {
                        (digit - b'0') as i64
                    } else if matches!(digit, b'a'..=b'f') {
                        10 + (digit - b'a') as i64
                    } else {
                        10 + (digit - b'A') as i64
                    };
                    value = value
                        .checked_mul(16)
                        .ok_or_else(int_invalid)?
                        .checked_add(n)
                        .ok_or_else(int_invalid)?;
                }
            } else {
                // Decimal literal
                for digit in slice.bytes() {
                    value = value
                        .checked_mul(10)
                        .ok_or_else(int_invalid)?
                        .checked_add((digit - b'0') as i64)
                        .ok_or_else(int_invalid)?;
                }
            }
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    fn float(&mut self) -> Result<Option<f64>> {
        if let Some(float_span) = self.accept(TokenKind::Float) {
            // TODO: Write custom float parser.
            let value = self.source[float_span].parse::<f64>().map_err(|_| Error {
                kind: ErrorKind::FloatInvalid,
                detail: None,
                span: float_span,
            })?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    fn bool(&mut self) -> Result<Option<bool>> {
        if self.accept(TokenKind::True).is_some() {
            Ok(Some(true))
        } else if self.accept(TokenKind::False).is_some() {
            Ok(Some(false))
        } else {
            Ok(None)
        }
    }

    fn tuple_entry(&mut self) -> Result<Option<TupleEntry<Expr>>> {
        if let Some(identifier) = self.identifier() {
            if self.accept(TokenKind::Colon).is_some() {
                let expr = self
                    .expr()?
                    .unwrap_or_else(|| Expr::unknown(ExprKind::Ident(identifier)));
                Ok(Some(TupleEntry::Named(identifier, expr)))
            } else {
                Ok(Some(TupleEntry::Pos(Expr::unknown(ExprKind::Ident(
                    identifier,
                )))))
            }
        } else if let Some(expr) = self.expr()? {
            Ok(Some(TupleEntry::Pos(expr)))
        } else {
            Ok(None)
        }
    }

    fn tuple(&mut self) -> Result<Option<Tuple<Expr>>> {
        let i = self.index;
        if let Some(lparen_span) = self.accept(TokenKind::LParen) {
            let mut entries = vec![];
            if let Some(entry) = self.tuple_entry()? {
                entries.push(entry);
                while self.accept(TokenKind::Comma).is_some() {
                    if let Some(entry) = self.tuple_entry()? {
                        entries.push(entry);
                    }
                }
            }
            self.accept(TokenKind::RParen).ok_or_else(|| Error {
                kind: ErrorKind::Delimiters,
                detail: None,
                span: lparen_span,
            })?;
            Ok(Some(Tuple::from_iter(entries)))
        } else {
            self.index = i;
            Ok(None)
        }
    }

    fn list(&mut self) -> Result<Option<Box<[Expr]>>> {
        if let Some(left_bracket_span) = self.accept(TokenKind::LBracket) {
            let mut exprs = vec![];
            if let Some(expr) = self.expr()? {
                exprs.push(expr);
                while self.accept(TokenKind::Comma).is_some() {
                    if let Some(expr) = self.expr()? {
                        exprs.push(expr);
                    }
                }
            }
            self.accept(TokenKind::RBracket).ok_or_else(|| Error {
                kind: ErrorKind::Delimiters,
                detail: None,
                span: left_bracket_span,
            })?;
            Ok(Some(exprs.into()))
        } else {
            Ok(None)
        }
    }

    fn pat_tuple_entry(&mut self) -> Option<TupleEntry<Pat>> {
        if let Some(identifier) = self.identifier() {
            if self.accept(TokenKind::Colon).is_some() {
                let pattern = self.pat().unwrap_or_else(|| Pat::Ident(identifier));
                Some(TupleEntry::Named(identifier, pattern))
            } else {
                Some(TupleEntry::Pos(Pat::Ident(identifier)))
            }
        } else if let Some(pattern) = self.pat() {
            Some(TupleEntry::Pos(pattern))
        } else {
            None
        }
    }

    fn pat_tuple(&mut self) -> Option<Tuple<Pat>> {
        let i = self.index;
        if self.accept(TokenKind::LParen).is_some() {
            let mut entries = vec![];
            if let Some(entry) = self.pat_tuple_entry() {
                entries.push(entry);
                while self.accept(TokenKind::Comma).is_some() {
                    if let Some(entry) = self.pat_tuple_entry() {
                        entries.push(entry);
                    }
                }
            }
            self.accept(TokenKind::RParen)?;
            Some(Tuple::from_iter(entries))
        } else {
            self.index = i;
            None
        }
    }

    fn pat(&mut self) -> Option<Pat> {
        if let Some(members) = self.pat_tuple() {
            Some(Pat::Tuple(members))
        } else if let Some(identifier) = self.identifier() {
            Some(Pat::Ident(identifier))
        } else {
            None
        }
    }

    fn stmt(&mut self) -> Result<Option<Stmt>> {
        if self.accept(TokenKind::Let).is_some() {
            let pattern = self.pat().ok_or_else(|| self.invalid_token())?;
            let args = self.pat_tuple();
            self.accept(TokenKind::Eq)
                .ok_or_else(|| self.invalid_token())?;
            let expr = self.expr()?.ok_or_else(|| self.invalid_token())?;
            if let Some(args) = args {
                Ok(Some(Stmt::Let(
                    pattern,
                    Expr::unknown(ExprKind::Fn(args, expr.into())),
                )))
            } else {
                Ok(Some(Stmt::Let(pattern, expr.into())))
            }
        } else if let Some(expr) = self.expr()? {
            Ok(Some(Stmt::Expr(expr)))
        } else {
            Ok(None)
        }
    }

    fn stmts(&mut self) -> Result<Box<[Stmt]>> {
        let mut statements = vec![];
        if let Some(statement) = self.stmt()? {
            statements.push(statement);
            while self.accept(TokenKind::Comma).is_some() {
                if let Some(statement) = self.stmt()? {
                    statements.push(statement);
                }
            }
        }
        Ok(statements.into())
    }

    fn block(&mut self) -> Result<Option<Expr>> {
        if let Some(left_brace_span) = self.accept(TokenKind::LBrace) {
            let statements = self.stmts()?;
            self.accept(TokenKind::RBrace).ok_or_else(|| Error {
                kind: ErrorKind::Delimiters,
                detail: None,
                span: left_brace_span,
            })?;
            if statements.len() == 0 {
                Ok(Some(Expr::unknown(ExprKind::Void)))
            } else if let [Stmt::Expr(expr)] = statements.as_ref() {
                Ok(Some(expr.clone()))
            } else {
                Ok(Some(Expr::unknown(ExprKind::Block(statements))))
            }
        } else {
            Ok(None)
        }
    }

    fn atom(&mut self) -> Result<Option<Expr>> {
        if let Some(tuple) = self.tuple()? {
            Ok(Some(Expr::unknown(ExprKind::Tuple(tuple))))
        } else if let Some(list) = self.list()? {
            Ok(Some(Expr::unknown(ExprKind::List(list))))
        } else if let Some(block) = self.block()? {
            Ok(Some(block))
        } else if let Some(identifier) = self.identifier() {
            Ok(Some(Expr::unknown(ExprKind::Ident(identifier))))
        } else if let Some(symbol) = self.symbol()? {
            Ok(Some(Expr::unknown(ExprKind::Sym(symbol))))
        } else if let Some(string) = self.string()? {
            Ok(Some(Expr::unknown(ExprKind::Str(string))))
        } else if let Some(int) = self.int()? {
            Ok(Some(Expr::unknown(ExprKind::Int(int))))
        } else if let Some(float) = self.float()? {
            Ok(Some(Expr::unknown(ExprKind::Float(float))))
        } else if let Some(bool) = self.bool()? {
            Ok(Some(Expr::unknown(ExprKind::Bool(bool))))
        } else {
            Ok(None)
        }
    }

    fn function_args(&mut self) -> Result<Option<Tuple<Pat>>> {
        if self.accept(TokenKind::Or).is_some() {
            let mut args = Vec::new();
            if let Some(arg) = self.pat_tuple_entry() {
                args.push(arg);
                while self.accept(TokenKind::Comma).is_some() {
                    if let Some(arg) = self.pat_tuple_entry() {
                        args.push(arg);
                    }
                }
            }
            self.accept(TokenKind::Or)
                .ok_or_else(|| self.invalid_token())?;
            Ok(Some(Tuple::from_iter(args)))
        } else {
            Ok(None)
        }
    }

    fn function(&mut self) -> Result<Option<Expr>> {
        if let Some(args) = self.function_args()? {
            let expr = self
                .expr()?
                .unwrap_or_else(|| Expr::unknown(ExprKind::Void));
            Ok(Some(Expr::unknown(ExprKind::Fn(args, expr.into()))))
        } else {
            Ok(None)
        }
    }

    fn expr_with_precedence(&mut self, min_prec: u8) -> Result<Option<Expr>> {
        // Handle unary operators.
        let left = if let Some((un_op, r_prec)) = self.accept_un_op() {
            if let Some(right) = self.expr_with_precedence(r_prec)? {
                Some(Expr::unknown(ExprKind::UnOp(un_op, right.into())))
            } else {
                return Err(self.invalid_token());
            }
        } else if let Some(mut left) = self.atom()? {
            // Handle function calls.
            while let Some(tuple) = self.tuple()? {
                left = Expr::unknown(ExprKind::Call(left.into(), tuple));
            }
            Some(left)
        } else {
            None
        };

        // Handle binary operators.
        if let Some(mut left) = left {
            loop {
                if let Some((bin_op, l_prec, r_prec)) = self.accept_bin_op() {
                    if l_prec < min_prec {
                        self.index -= 1;
                        break;
                    }
                    if let Some(right) = self.expr_with_precedence(r_prec)? {
                        left = Expr::unknown(ExprKind::BinOp(bin_op, left.into(), right.into()));
                    } else {
                        return Err(self.invalid_token());
                    }
                } else {
                    break;
                }
            }

            Ok(Some(left))
        } else {
            Ok(None)
        }
    }

    fn expr(&mut self) -> Result<Option<Expr>> {
        if let Some(function) = self.function()? {
            Ok(Some(function))
        } else if let Some(expr) = self.expr_with_precedence(0)? {
            Ok(Some(expr))
        } else {
            Ok(None)
        }
    }

    fn module_path(&mut self) -> Result<Option<ModPath>> {
        let i = self.index;
        let local = self.accept(TokenKind::Period).is_some();
        if let Some(segment) = self.accept_sym(TokenKind::Ident) {
            let mut segments = vec![segment];
            while self.accept(TokenKind::Period).is_some() {
                if let Some(segment) = self.accept_sym(TokenKind::Ident) {
                    segments.push(segment);
                } else {
                    break;
                }
            }
            Ok(Some(ModPath {
                local,
                segments: segments.into(),
            }))
        } else {
            self.index = i;
            Ok(None)
        }
    }

    fn bindings(&mut self) -> Result<Option<Box<[(Sym, Sym)]>>> {
        if self.accept(TokenKind::LParen).is_some() {
            let mut bindings = vec![];
            if let Some(binding) = self.accept_sym(TokenKind::Ident) {
                // TODO: Allow renames.
                bindings.push((binding, binding));
                while self.accept(TokenKind::Comma).is_some() {
                    if let Some(binding) = self.accept_sym(TokenKind::Ident) {
                        bindings.push((binding, binding));
                    }
                }
            }
            self.accept(TokenKind::RParen)
                .ok_or_else(|| self.invalid_token())?;
            Ok(Some(bindings.into()))
        } else {
            Ok(None)
        }
    }

    fn dep(&mut self) -> Result<Option<Dep>> {
        if let Some(path) = self.module_path()? {
            let bindings = self.bindings()?.ok_or_else(|| self.invalid_token())?;
            Ok(Some(Dep { path, bindings }))
        } else {
            Ok(None)
        }
    }

    fn deps(&mut self) -> Result<Box<[Dep]>> {
        let mut deps = vec![];
        if self.accept(TokenKind::Use).is_some() {
            self.accept(TokenKind::LBrace)
                .ok_or_else(|| self.invalid_token())?;
            if let Some(dep) = self.dep()? {
                deps.push(dep);
                while self.accept(TokenKind::Comma).is_some() {
                    if let Some(dep) = self.dep()? {
                        deps.push(dep);
                    }
                }
            }
            self.accept(TokenKind::RBrace)
                .ok_or_else(|| self.invalid_token())?;
        }
        Ok(deps.into())
    }

    fn defs(&mut self) -> Result<Box<[Stmt]>> {
        let def = self.stmts()?;
        for definition in def.iter() {
            if let Stmt::Expr(_) = definition {
                return Err(self.invalid_token());
            }
        }
        Ok(def.into())
    }

    fn module(&mut self) -> Result<Mod> {
        let deps = self.deps()?;
        self.accept(TokenKind::Comma);
        let defs = self.defs()?;
        Ok(Mod { deps, defs })
    }
}

pub fn parse_expr(source: &str, interner: &mut Interner) -> Result<Expr> {
    let expr = Parser::new(source, tokenize(source)?, interner)
        .expr()?
        .unwrap_or(Expr::unknown(ExprKind::Void));
    Ok(expr)
}

pub fn parse_stmt(source: &str, interner: &mut Interner) -> Result<Stmt> {
    let stmt = Parser::new(source, tokenize(source)?, interner)
        .stmt()?
        .unwrap_or(Stmt::Expr(Expr::unknown(ExprKind::Void)));
    Ok(stmt)
}

pub fn parse_module(source: &str, interner: &mut Interner) -> Result<Mod> {
    Parser::new(source, tokenize(source)?, interner).module()
}

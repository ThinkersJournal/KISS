//! Reference recursive-descent evaluator for the KISS-Contract §6.6-0006 Dispatch
//! **expression grammar**: arithmetic over launch-scalar symbols, `ceil_div`, and
//! the element-subscript operator `sym[k]` (the R1.1 grammar fix, issue #43).
//!
//! Grammar (exactly as pinned by §6.6-0006):
//! ```text
//! expr   := term (('+' | '-') term)*
//! term   := factor (('*' | '/') factor)*
//! factor := int | 'ceil_div' '(' expr ',' expr ')' | symbol ('[' int ']')? | '(' expr ')'
//! ```
//! Left-associative; the subscript binds tighter than `* /`, which bind tighter
//! than `+ -`.
//!
//! The launch-scalar vocabulary (§6.5-0004a) splits into:
//!   * **array** symbols (`array_len_kind = rank`, §6.11-0006): `extents{i}`,
//!     `strides{i}`, `idx_extents{i}` — spelled with the operand index appended
//!     (`extents0`, `strides0`, `idx_extents0`, …) — REQUIRE a `[k]` subscript.
//!   * **scalar** symbols: `n`, `ws_bytes` (no operand index) and `off{i}`,
//!     `param{i}` (an appended operand/param index, but no `[k]` subscript).
//!
//! A Domain-mode field (`invocation_domain` / `workgroup_sizing` / `count_to_grid`)
//! MUST evaluate to a non-negative integer; a Structural-mode field
//! (`thread_mapping` / `addressing_rule`) is exempt (class-2 strides may be
//! negative). Any malformed or out-of-vocabulary expression is a typed
//! [`Decline`], never a panic (KISS-Conform §6.7).

// ---- launch scalars (Contract §6.5-0004a) -------------------------------------

/// The concrete launch-scalar values a Dispatch expression is evaluated against.
#[derive(Debug, Clone)]
pub struct LaunchScalars {
    /// The Interface `rank` — the axis count for every `extents{i}`/`strides{i}`.
    pub rank: u32,
    /// The element count `n`.
    pub n: i64,
    /// `extents[operand][axis]`.
    pub extents: Vec<Vec<i64>>,
    /// `strides[operand][axis]` (may be negative — class-2 signed strides).
    pub strides: Vec<Vec<i64>>,
    /// `idx_extents[index-operand][axis]`; its own rank is that index operand's
    /// rank, which may differ from `rank`.
    pub idx_extents: Vec<Vec<i64>>,
    /// `off[operand]` — a scalar base offset per operand (class-4).
    pub off: Vec<i64>,
    /// The shared-memory/workspace byte count.
    pub ws_bytes: i64,
    /// `param[field]` — declared op params.
    pub param: Vec<i64>,
}

/// Whether the §6.6-0006 non-negative post-condition applies: Domain fields
/// (`invocation_domain` / `workgroup_sizing` / `count_to_grid`) enforce it;
/// Structural fields (`thread_mapping` / `addressing_rule`) do not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvalMode {
    Domain,
    Structural,
}

/// A typed decline: an evaluator MUST refuse a malformed or out-of-vocabulary
/// expression with one of these, never a panic (KISS-Conform §6.7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decline {
    /// An identifier that names no symbol in the launch-scalar vocabulary.
    UnknownSymbol(String),
    /// A `[k]` subscript applied to a scalar symbol (`n`, `off{i}`, `ws_bytes`,
    /// `param{i}`).
    SubscriptOfScalar(String),
    /// A rank-length array symbol (`extents{i}`/`strides{i}`/`idx_extents{i}`)
    /// used with no `[k]` subscript.
    BareArraySymbol(String),
    /// `sym[k]` with `k >= rank` for that symbol's operand.
    SubscriptOutOfBounds { sym: String, k: u32, rank: u32 },
    /// Malformed input: an unexpected character, unexpected token, or
    /// unparseable arithmetic (e.g. a zero/non-positive divisor).
    ParseError(String),
    /// A Domain-mode expression (or one of its sub-expressions) evaluated
    /// negative.
    NegativeInDomain(i64),
}

// ---- lexer ---------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
enum Tok {
    Int(i64),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Comma,
    End,
}

/// Tokenize a Dispatch expression: integer literals, `[a-z][a-z0-9_]*`
/// identifiers (greedily capturing a trailing operand-index digit run as part
/// of the symbol spelling), the operators `+ - * /`, `( ) [ ] ,`. Any other
/// character is a typed decline, never a panic.
fn lex(src: &str) -> Result<Vec<Tok>, Decline> {
    let mut toks = Vec::new();
    let b = src.as_bytes();
    let mut i = 0;
    while i < b.len() {
        let c = b[i] as char;
        if c.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        match c {
            '+' => {
                toks.push(Tok::Plus);
                i += 1;
            }
            '-' => {
                toks.push(Tok::Minus);
                i += 1;
            }
            '*' => {
                toks.push(Tok::Star);
                i += 1;
            }
            '/' => {
                toks.push(Tok::Slash);
                i += 1;
            }
            '(' => {
                toks.push(Tok::LParen);
                i += 1;
            }
            ')' => {
                toks.push(Tok::RParen);
                i += 1;
            }
            '[' => {
                toks.push(Tok::LBracket);
                i += 1;
            }
            ']' => {
                toks.push(Tok::RBracket);
                i += 1;
            }
            ',' => {
                toks.push(Tok::Comma);
                i += 1;
            }
            '0'..='9' => {
                let start = i;
                while i < b.len() && (b[i] as char).is_ascii_digit() {
                    i += 1;
                }
                let s = &src[start..i];
                let v: i64 = s
                    .parse()
                    .map_err(|_| Decline::ParseError(format!("bad integer literal `{s}`")))?;
                toks.push(Tok::Int(v));
            }
            'a'..='z' => {
                let start = i;
                while i < b.len() && {
                    let ch = b[i] as char;
                    ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_'
                } {
                    i += 1;
                }
                toks.push(Tok::Ident(src[start..i].to_string()));
            }
            other => {
                return Err(Decline::ParseError(format!("unexpected character `{other}`")));
            }
        }
    }
    toks.push(Tok::End);
    Ok(toks)
}

// ---- AST -------------------------------------------------------------------

#[derive(Debug, Clone)]
enum Ast {
    Int(i64),
    /// A scalar symbol reference (no subscript in source).
    Symbol(String),
    /// A `sym[k]` element-subscript reference.
    Subscript(String, u32),
    Add(Box<Ast>, Box<Ast>),
    Sub(Box<Ast>, Box<Ast>),
    Mul(Box<Ast>, Box<Ast>),
    Div(Box<Ast>, Box<Ast>),
    CeilDiv(Box<Ast>, Box<Ast>),
}

// ---- parser (exactly: expr := term (('+'|'-') term)*;
//              term := factor (('*'|'/') factor)*;
//              factor := int | ceil_div(expr,expr) | symbol ('[' int ']')? | '(' expr ')') ----

struct Parser<'a> {
    toks: &'a [Tok],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> &Tok {
        &self.toks[self.pos]
    }

    fn bump(&mut self) -> Tok {
        let t = self.toks[self.pos].clone();
        if self.toks[self.pos] != Tok::End {
            self.pos += 1;
        }
        t
    }

    fn expect(&mut self, want: &Tok) -> Result<(), Decline> {
        if self.peek() == want {
            self.bump();
            Ok(())
        } else {
            Err(Decline::ParseError(format!(
                "expected {want:?}, found {:?}",
                self.peek()
            )))
        }
    }

    fn parse_expr(&mut self) -> Result<Ast, Decline> {
        let mut lhs = self.parse_term()?;
        loop {
            match self.peek() {
                Tok::Plus => {
                    self.bump();
                    let rhs = self.parse_term()?;
                    lhs = Ast::Add(Box::new(lhs), Box::new(rhs));
                }
                Tok::Minus => {
                    self.bump();
                    let rhs = self.parse_term()?;
                    lhs = Ast::Sub(Box::new(lhs), Box::new(rhs));
                }
                _ => break,
            }
        }
        Ok(lhs)
    }

    fn parse_term(&mut self) -> Result<Ast, Decline> {
        let mut lhs = self.parse_factor()?;
        loop {
            match self.peek() {
                Tok::Star => {
                    self.bump();
                    let rhs = self.parse_factor()?;
                    lhs = Ast::Mul(Box::new(lhs), Box::new(rhs));
                }
                Tok::Slash => {
                    self.bump();
                    let rhs = self.parse_factor()?;
                    lhs = Ast::Div(Box::new(lhs), Box::new(rhs));
                }
                _ => break,
            }
        }
        Ok(lhs)
    }

    fn parse_factor(&mut self) -> Result<Ast, Decline> {
        match self.peek().clone() {
            Tok::Int(v) => {
                self.bump();
                Ok(Ast::Int(v))
            }
            Tok::LParen => {
                self.bump();
                let e = self.parse_expr()?;
                self.expect(&Tok::RParen)?;
                Ok(e)
            }
            Tok::Ident(name) => {
                self.bump();
                if name == "ceil_div" {
                    self.expect(&Tok::LParen)?;
                    let a = self.parse_expr()?;
                    self.expect(&Tok::Comma)?;
                    let b = self.parse_expr()?;
                    self.expect(&Tok::RParen)?;
                    Ok(Ast::CeilDiv(Box::new(a), Box::new(b)))
                } else if matches!(self.peek(), Tok::LBracket) {
                    self.bump();
                    let k = match self.peek().clone() {
                        Tok::Int(v) if v >= 0 => {
                            self.bump();
                            v as u32
                        }
                        other => {
                            return Err(Decline::ParseError(format!(
                                "expected a non-negative integer subscript after `{name}[`, found {other:?}"
                            )));
                        }
                    };
                    self.expect(&Tok::RBracket)?;
                    Ok(Ast::Subscript(name, k))
                } else {
                    Ok(Ast::Symbol(name))
                }
            }
            other => Err(Decline::ParseError(format!("unexpected token {other:?}"))),
        }
    }
}

// ---- symbol resolution -------------------------------------------------------

/// Split a symbol spelling into its base name and trailing operand-index digit
/// run, e.g. `"extents0"` -> `("extents", Some(0))`, `"n"` -> `("n", None)`.
fn split_ident(name: &str) -> (&str, Option<u32>) {
    let digit_start = name
        .rfind(|c: char| !c.is_ascii_digit())
        .map(|i| i + 1)
        .unwrap_or(0);
    if digit_start < name.len() {
        (&name[..digit_start], name[digit_start..].parse().ok())
    } else {
        (name, None)
    }
}

/// Resolve a bare (non-subscripted) scalar symbol reference.
fn resolve_scalar(name: &str, s: &LaunchScalars) -> Result<i64, Decline> {
    let (base, idx) = split_ident(name);
    match (base, idx) {
        ("n", None) => Ok(s.n),
        ("ws_bytes", None) => Ok(s.ws_bytes),
        ("off", Some(i)) => s
            .off
            .get(i as usize)
            .copied()
            .ok_or_else(|| Decline::UnknownSymbol(name.to_string())),
        ("param", Some(i)) => s
            .param
            .get(i as usize)
            .copied()
            .ok_or_else(|| Decline::UnknownSymbol(name.to_string())),
        ("extents", Some(_)) | ("strides", Some(_)) | ("idx_extents", Some(_)) => {
            Err(Decline::BareArraySymbol(name.to_string()))
        }
        _ => Err(Decline::UnknownSymbol(name.to_string())),
    }
}

/// Resolve one array row (`extents[i]` or `strides[i]`) against the shared
/// Interface `rank`, then subscript it at axis `k`.
fn resolve_rank_array(
    name: &str,
    rows: &[Vec<i64>],
    i: u32,
    k: u32,
    rank: u32,
) -> Result<i64, Decline> {
    let row = rows
        .get(i as usize)
        .ok_or_else(|| Decline::UnknownSymbol(name.to_string()))?;
    if k >= rank {
        return Err(Decline::SubscriptOutOfBounds { sym: name.to_string(), k, rank });
    }
    row.get(k as usize)
        .copied()
        .ok_or_else(|| Decline::SubscriptOutOfBounds { sym: name.to_string(), k, rank })
}

/// Resolve a `sym[k]` element-subscript reference.
fn resolve_subscript(name: &str, k: u32, s: &LaunchScalars) -> Result<i64, Decline> {
    let (base, idx) = split_ident(name);
    match (base, idx) {
        ("n", None) | ("ws_bytes", None) | ("off", Some(_)) | ("param", Some(_)) => {
            Err(Decline::SubscriptOfScalar(name.to_string()))
        }
        ("extents", Some(i)) => resolve_rank_array(name, &s.extents, i, k, s.rank),
        ("strides", Some(i)) => resolve_rank_array(name, &s.strides, i, k, s.rank),
        ("idx_extents", Some(i)) => {
            let row = s
                .idx_extents
                .get(i as usize)
                .ok_or_else(|| Decline::UnknownSymbol(name.to_string()))?;
            let rank = row.len() as u32;
            if k >= rank {
                return Err(Decline::SubscriptOutOfBounds { sym: name.to_string(), k, rank });
            }
            Ok(row[k as usize])
        }
        _ => Err(Decline::UnknownSymbol(name.to_string())),
    }
}

// ---- evaluation (§6.6-0006 non-negative Domain post-condition) ---------------

/// Ceiling division `ceil_div(a, b) = ceil(a / b)`, defined for a positive
/// divisor. A non-positive divisor is a decline (design choice: the grammar
/// gives no meaning to `ceil_div` by zero or a negative divisor, so it is
/// treated as malformed input rather than silently picking a rounding rule).
fn ceil_div(a: i64, b: i64) -> Result<i64, Decline> {
    if b <= 0 {
        return Err(Decline::ParseError(format!(
            "ceil_div divisor must be positive, got {b}"
        )));
    }
    Ok((a + b - 1).div_euclid(b))
}

fn eval_ast(node: &Ast, s: &LaunchScalars, mode: EvalMode) -> Result<i64, Decline> {
    let v = match node {
        Ast::Int(v) => *v,
        Ast::Symbol(name) => resolve_scalar(name, s)?,
        Ast::Subscript(name, k) => resolve_subscript(name, *k, s)?,
        Ast::Add(a, b) => eval_ast(a, s, mode)? + eval_ast(b, s, mode)?,
        Ast::Sub(a, b) => eval_ast(a, s, mode)? - eval_ast(b, s, mode)?,
        Ast::Mul(a, b) => eval_ast(a, s, mode)? * eval_ast(b, s, mode)?,
        Ast::Div(a, b) => {
            let (av, bv) = (eval_ast(a, s, mode)?, eval_ast(b, s, mode)?);
            if bv == 0 {
                return Err(Decline::ParseError("division by zero".to_string()));
            }
            av / bv
        }
        Ast::CeilDiv(a, b) => {
            let (av, bv) = (eval_ast(a, s, mode)?, eval_ast(b, s, mode)?);
            ceil_div(av, bv)?
        }
    };
    if mode == EvalMode::Domain && v < 0 {
        return Err(Decline::NegativeInDomain(v));
    }
    Ok(v)
}

/// Evaluate a Dispatch §6.6-0006 expression against concrete launch-scalar
/// values. `mode` selects whether the non-negative post-condition applies
/// (Domain fields) or not (Structural fields, which may carry signed strides).
/// A malformed expression, an out-of-vocabulary symbol reference, or (in
/// Domain mode) a negative result is a typed [`Decline`] — this function never
/// panics on any `&str` input.
pub fn eval(expr: &str, s: &LaunchScalars, mode: EvalMode) -> Result<i64, Decline> {
    let toks = lex(expr)?;
    let mut p = Parser { toks: &toks, pos: 0 };
    let ast = p.parse_expr()?;
    if p.peek() != &Tok::End {
        return Err(Decline::ParseError(format!(
            "trailing input at token {:?}",
            p.peek()
        )));
    }
    eval_ast(&ast, s, mode)
}

//! Reference-decomposition body grammar (KISS-Ops §6.13-0006) and the structured-op
//! classification (§6.13-0009).
//!
//! §6.13-0006 requires that a **scalar-expression** decomposition body parse as a tree
//! over KISS-Ops ops and §6.12 scalar-source leaves, optionally with SSA `name = expr;`
//! let-bindings. A **structured op** (§6.13-0009) — `matmul`, `argmax`/`argmin`, the
//! pools, the gather/scatter family, `im2col` — is *not* a scalar-expression body: its
//! decomposition is a §6.11 structural-op reference parameterized by an attribute record
//! and iteration space, and MUST NOT be required to parse as this grammar. Before this
//! split (#42's sibling defect #41), §6.13-0006 demanded every row parse as a scalar
//! tree, which ~9 structured rows (prose over an iteration space) cannot satisfy.
//!
//! This module provides (a) [`parse_scalar_body`], a mechanical resolver for the scalar
//! grammar, and (b) the [`STRUCTURED_OPS`] table binding each structured op to the §6.11
//! structural op its decomposition references.

/// A structured op (§6.13-0009) paired with the §6.11 structural op (or `matmul`
/// contraction) its reference decomposition names. These bodies describe an iteration
/// space / index mapping in prose and are deliberately NOT scalar-expression trees.
pub const STRUCTURED_OPS: &[(&str, &str)] = &[
    ("matmul", "matmul"),      // reduce(sum, K) of element_map(mul) over the M/N/K space
    ("argmax", "sort_network"), // index at rank 0 of sort_network(desc, keys=x)
    ("argmin", "sort_network"),
    ("avg_pool", "reduce"),    // reduce_mean over the window view
    ("max_pool", "reduce"),    // reduce(max) over the window view
    ("index_select", "gather"),
    ("embedding", "gather"),
    ("scatter_add", "scatter"),
    ("im2col", "gather"),
];

/// The §6.11 structural ops a structured op's decomposition may reference (the `matmul`
/// contraction included).
pub const STRUCTURAL_OPS: &[&str] =
    &["reduce", "prefix_scan", "gather", "scatter", "sort_network", "matmul"];

/// Parse a scalar-expression decomposition body per §6.13-0006. Returns `Ok(())` when the
/// body is a well-formed scalar-expression tree or SSA let-binding sequence, and `Err`
/// otherwise — including on any prose token a mechanical resolver could not consume, which
/// is exactly how a structured op's body is (correctly) rejected by this grammar.
///
/// Scope: this validates the grammatical *structure* (call trees, `const(...)` and scalar
/// leaves) and the full SSA discipline of §6.13-0006 — each let name bound exactly once,
/// and no reference to itself or to a not-yet-bound name. A bare identifier that names no
/// let-binding is a scalar-source leaf (its vocabulary validity is a separate §6.12 concern).
pub fn parse_scalar_body(body: &str) -> Result<(), String> {
    let toks = tokenize(body)?;
    let binding_names = collect_binding_names(&toks);
    let mut p = Parser { toks: &toks, pos: 0, binding_names, bound: Vec::new() };
    p.parse_body()?;
    if p.pos != p.toks.len() {
        return Err(format!(
            "leftover tokens from position {} — not a scalar-expression body",
            p.pos
        ));
    }
    Ok(())
}

/// The SSA binding names: every identifier that leads a statement (depth 0, at a statement
/// start) and is immediately followed by `=`. A `key=value` attribute *inside* a call
/// (depth > 0, e.g. `monoid=sum`) is not a binding. Knowing the full set lets the parser
/// tell a forward/self reference (a binding name used before it is bound) from a leaf.
fn collect_binding_names(toks: &[Tok]) -> Vec<String> {
    let mut names = Vec::new();
    let mut depth = 0i32;
    let mut at_stmt_start = true;
    for (i, t) in toks.iter().enumerate() {
        match t {
            Tok::LParen => {
                depth += 1;
                at_stmt_start = false;
            }
            Tok::RParen => {
                depth -= 1;
                at_stmt_start = false;
            }
            Tok::Semi if depth == 0 => at_stmt_start = true,
            Tok::Ident(n) if depth == 0 && at_stmt_start => {
                if matches!(toks.get(i + 1), Some(Tok::Eq)) {
                    names.push(n.clone());
                }
                at_stmt_start = false;
            }
            _ => at_stmt_start = false,
        }
    }
    names
}

#[derive(PartialEq, Debug, Clone)]
enum Tok {
    Ident(String),
    Num,
    Const, // a `const(...)` leaf, inner content consumed opaquely (§6.12-0003)
    LParen,
    RParen,
    Comma,
    Eq,
    Semi,
}

fn tokenize(s: &str) -> Result<Vec<Tok>, String> {
    let b = s.as_bytes();
    let mut i = 0;
    let mut out = Vec::new();
    while i < b.len() {
        let c = b[i] as char;
        if c.is_whitespace() {
            i += 1;
        } else if c == '(' {
            out.push(Tok::LParen);
            i += 1;
        } else if c == ')' {
            out.push(Tok::RParen);
            i += 1;
        } else if c == ',' {
            out.push(Tok::Comma);
            i += 1;
        } else if c == '=' {
            out.push(Tok::Eq);
            i += 1;
        } else if c == ';' {
            out.push(Tok::Semi);
            i += 1;
        } else if c.is_ascii_digit() || (c == '-' && matches!(b.get(i + 1), Some(d) if (*d as char).is_ascii_digit())) {
            i += 1;
            while i < b.len() && matches!(b[i] as char, '0'..='9' | '.') {
                i += 1;
            }
            out.push(Tok::Num);
        } else if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            while i < b.len() && (matches!(b[i] as char, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_')) {
                i += 1;
            }
            let word = &s[start..i];
            if word == "const" {
                while i < b.len() && (b[i] as char).is_whitespace() {
                    i += 1;
                }
                if b.get(i).map(|&c| c as char) != Some('(') {
                    return Err("`const` not followed by `(`".into());
                }
                let mut depth = 0usize;
                while i < b.len() {
                    match b[i] as char {
                        '(' => depth += 1,
                        ')' => {
                            depth -= 1;
                            if depth == 0 {
                                i += 1;
                                break;
                            }
                        }
                        _ => {}
                    }
                    i += 1;
                }
                if depth != 0 {
                    return Err("unbalanced parentheses in `const(...)`".into());
                }
                out.push(Tok::Const);
            } else {
                out.push(Tok::Ident(word.to_string()));
            }
        } else {
            return Err(format!(
                "unexpected character `{c}` at {i} — prose, not a scalar-expression body"
            ));
        }
    }
    Ok(out)
}

struct Parser<'a> {
    toks: &'a [Tok],
    pos: usize,
    /// Every SSA binding name in the body (from the pre-pass).
    binding_names: Vec<String>,
    /// Binding names bound so far, in order.
    bound: Vec<String>,
}

impl Parser<'_> {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
    }

    /// §6.13-0006: a reference to a let name MUST be to an already-bound one — not itself,
    /// not a later binding. A bare identifier that names no binding is a scalar-source leaf.
    fn check_ref(&self, name: &str) -> Result<(), String> {
        if self.binding_names.iter().any(|b| b == name) && !self.bound.iter().any(|b| b == name)
        {
            Err(format!(
                "`{name}` references itself or a not-yet-bound name — violates §6.13-0006 SSA"
            ))
        } else {
            Ok(())
        }
    }

    /// body := statement (`;` statement)*   (last statement has no trailing `;`)
    fn parse_body(&mut self) -> Result<(), String> {
        if self.toks.is_empty() {
            return Err("empty body".into());
        }
        loop {
            self.parse_statement()?;
            match self.peek() {
                Some(Tok::Semi) => {
                    self.pos += 1;
                }
                _ => break,
            }
        }
        Ok(())
    }

    /// statement := (Ident `=` expr) | expr    — an `=` form is an SSA let-binding.
    fn parse_statement(&mut self) -> Result<(), String> {
        if let (Some(Tok::Ident(name)), Some(Tok::Eq)) =
            (self.toks.get(self.pos), self.toks.get(self.pos + 1))
        {
            let name = name.clone();
            self.pos += 2; // consume `name =`
            self.parse_expr()?; // the RHS FIRST, so a self-reference `a = a` is caught
            if self.bound.contains(&name) {
                return Err(format!("`{name}` bound more than once — not SSA"));
            }
            self.bound.push(name); // bind only AFTER its defining expression
            Ok(())
        } else {
            self.parse_expr()
        }
    }

    /// expr := `const(...)` | Num | Ident (`(` arglist `)`)?
    fn parse_expr(&mut self) -> Result<(), String> {
        match self.peek() {
            Some(Tok::Const) | Some(Tok::Num) => {
                self.pos += 1;
                Ok(())
            }
            Some(Tok::Ident(name)) => {
                let name = name.clone();
                self.pos += 1;
                if let Some(Tok::LParen) = self.peek() {
                    self.pos += 1;
                    self.parse_arglist()?;
                    match self.peek() {
                        Some(Tok::RParen) => {
                            self.pos += 1;
                            Ok(())
                        }
                        other => Err(format!("expected `)`, found {other:?}")),
                    }
                } else {
                    // bare name atom: a scalar-source leaf, or an SSA-bound name reference
                    // that MUST already be bound (§6.13-0006).
                    self.check_ref(&name)
                }
            }
            other => Err(format!("expected an expression, found {other:?}")),
        }
    }

    /// arglist := (arg (`,` arg)*)?
    fn parse_arglist(&mut self) -> Result<(), String> {
        if let Some(Tok::RParen) = self.peek() {
            return Ok(());
        }
        loop {
            self.parse_arg()?;
            match self.peek() {
                Some(Tok::Comma) => self.pos += 1,
                _ => break,
            }
        }
        Ok(())
    }

    /// arg := (Ident `=` (Ident|Num|const)) | expr   — the first form is a keyword
    /// attribute like `monoid=sum`, `oob=skip`, `axis=K`, `keys=x`.
    fn parse_arg(&mut self) -> Result<(), String> {
        if let (Some(Tok::Ident(_)), Some(Tok::Eq)) =
            (self.toks.get(self.pos), self.toks.get(self.pos + 1))
        {
            self.pos += 2;
            match self.peek() {
                Some(Tok::Ident(v)) => {
                    let v = v.clone();
                    self.pos += 1;
                    self.check_ref(&v) // a keyword value may reference a bound name
                }
                Some(Tok::Num) | Some(Tok::Const) => {
                    self.pos += 1;
                    Ok(())
                }
                other => Err(format!("keyword-arg value expected, found {other:?}")),
            }
        } else {
            self.parse_expr()
        }
    }
}

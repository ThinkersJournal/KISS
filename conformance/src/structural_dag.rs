// SPDX-License-Identifier: MIT OR Apache-2.0
//! Structural op-DAG equality (Conform §6.9) — the comparator for the tier-1 round-trip.
//!
//! Two KISS-Ops op DAGs are equal iff, after resolving every non-primitive node to the
//! primitive floor, placing nodes and edges in canonical order and normalizing commutative
//! operand order, their **node sets, edge sets and per-node OpAttrs byte channels** are
//! identical (§6.9-0001). It compares STRUCTURE: it never looks at emitted source bytes
//! (§6.9-0002) and never at numeric values (§6.9-0003, not the tier-2 comparator).
//!
//! Resolution to the floor is driven by a [`Decompositions`] table. [`Decompositions::from_ops_spec`]
//! builds one from the KISS-Ops §6.13 reference-decomposition rows, so the harness resolves
//! through the owner's own text, not a table it authored. A non-primitive op whose body this
//! module cannot read (a let-bound or prose body) is recorded in `unreadable` and resolving
//! it is a typed [`DagError::Unresolvable`], never a silent treatment as a primitive.
//!
//! Stated limits (do not over-credit):
//! * Normalization is commutative operand order, by [`crate::grammar::is_commutative`]
//!   (`add`, `mul`). KISS-Ops declares no per-op associativity property, so nothing is
//!   re-bracketed (§6.9-0001 normalizes associative operand *order*, which for a binary
//!   commutative op is the same step).
//! * Only nodes reachable from the root are compared.
//! * When two commutative operands have IDENTICAL canonical subtrees but different sharing
//!   with the rest of the DAG, the canonical order between them is not distinguished; such a
//!   pair compares unequal at worst (a false-red on a conformant lift), never equal wrongly.

use crate::grammar::is_commutative;
use std::collections::{BTreeMap, BTreeSet};

/// One op node. `args` index EARLIER nodes (children precede parents). `attrs` is the node's
/// OpAttrs byte channel (the literal of a `const`/`input`/`param` leaf lives here).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub op: String,
    pub attrs: Vec<u8>,
    pub args: Vec<usize>,
}

/// An op DAG with an explicit root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dag {
    pub nodes: Vec<Node>,
    pub root: usize,
}

/// Why a DAG could not be compared (typed, never a panic).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DagError {
    /// Expression text did not parse.
    Parse(String),
    /// A node index points forward or out of range (not a children-first DAG).
    BadEdge { node: usize, arg: usize },
    /// The root index is out of range.
    BadRoot,
    /// A non-primitive op whose reference body this module cannot read.
    Unresolvable(String),
    /// A node's argument count differs from its decomposition's parameter count.
    Arity { op: String, expected: usize, got: usize },
    /// Resolution did not terminate within the depth bound (a cyclic or runaway table).
    TooDeep(String),
}

const SCALAR_SOURCES: &[&str] = &["input", "const", "param", "coord", "reduced", "extent"];
const MAX_DEPTH: usize = 64;

// ---------------------------------------------------------------------------
// Expression text -> Dag (the "lift" a test needs; also the body reader).
// ---------------------------------------------------------------------------

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
    nodes: Vec<Node>,
    inputs: &'a [String],
}

impl<'a> Parser<'a> {
    fn ws(&mut self) {
        while self.i < self.s.len() && self.s[self.i].is_ascii_whitespace() {
            self.i += 1;
        }
    }
    fn ident(&mut self) -> Option<String> {
        self.ws();
        let st = self.i;
        while self.i < self.s.len()
            && (self.s[self.i].is_ascii_alphanumeric() || self.s[self.i] == b'_')
        {
            self.i += 1;
        }
        (self.i > st).then(|| String::from_utf8_lossy(&self.s[st..self.i]).into_owned())
    }
    fn push(&mut self, op: &str, attrs: Vec<u8>, args: Vec<usize>) -> usize {
        self.nodes.push(Node { op: op.into(), attrs, args });
        self.nodes.len() - 1
    }
    /// `name(<raw literal>)` for a scalar-source leaf (`const(0.5)`, `const(pi/2)`, `input(0)`):
    /// the literal runs to the matching `)` and rides as the node's attrs.
    fn literal(&mut self, name: &str) -> Result<usize, DagError> {
        let st = self.i;
        let mut depth = 1;
        while self.i < self.s.len() && depth > 0 {
            match self.s[self.i] {
                b'(' => depth += 1,
                b')' => depth -= 1,
                _ => {}
            }
            self.i += 1;
        }
        if depth != 0 {
            return Err(DagError::Parse("unterminated scalar-source literal".into()));
        }
        let lit = self.s[st..self.i - 1].to_vec();
        Ok(self.push(name, lit, vec![]))
    }

    /// `name(arg, ...)` after the opening `(`.
    fn call(&mut self, name: &str) -> Result<usize, DagError> {
        let mut args = Vec::new();
        self.ws();
        if self.s.get(self.i) == Some(&b')') {
            self.i += 1;
            return Ok(self.push(name, vec![], args));
        }
        loop {
            args.push(self.expr()?);
            self.ws();
            match self.s.get(self.i) {
                Some(b',') => self.i += 1,
                Some(b')') => {
                    self.i += 1;
                    return Ok(self.push(name, vec![], args));
                }
                _ => return Err(DagError::Parse(format!("expected , or ) at byte {}", self.i))),
            }
        }
    }

    /// A bare name: a formal input is ONE node however often it is named (the textual form's
    /// occurrences are uses of the same operand, not distinct nodes); any other name is a symbol
    /// leaf (a monoid name such as `sum`).
    fn leaf(&mut self, name: String) -> usize {
        match self.inputs.iter().position(|n| *n == name) {
            Some(k) => {
                let attrs = k.to_string().into_bytes();
                match self.nodes.iter().position(|n| n.op == "input" && n.attrs == attrs) {
                    Some(i) => i,
                    None => self.push("input", attrs, vec![]),
                }
            }
            None => self.push("sym", name.into_bytes(), vec![]),
        }
    }

    fn expr(&mut self) -> Result<usize, DagError> {
        let name = self.ident().ok_or_else(|| DagError::Parse(format!("expected a name at byte {}", self.i)))?;
        self.ws();
        if self.s.get(self.i) != Some(&b'(') {
            return Ok(self.leaf(name));
        }
        self.i += 1;
        if SCALAR_SOURCES.contains(&name.as_str()) {
            self.literal(&name)
        } else {
            self.call(&name)
        }
    }
}

impl Dag {
    /// Parse `op(arg, ...)` text; every name in `inputs` is the formal input of that index.
    pub fn parse_expr(text: &str, inputs: &[&str]) -> Result<Dag, DagError> {
        let inputs: Vec<String> = inputs.iter().map(|s| s.to_string()).collect();
        let mut p = Parser { s: text.as_bytes(), i: 0, nodes: Vec::new(), inputs: &inputs };
        let root = p.expr()?;
        p.ws();
        if p.i != p.s.len() {
            return Err(DagError::Parse(format!("trailing text at byte {}", p.i)));
        }
        Ok(Dag { nodes: p.nodes, root })
    }

    fn validate(&self) -> Result<(), DagError> {
        if self.root >= self.nodes.len() {
            return Err(DagError::BadRoot);
        }
        for (i, n) in self.nodes.iter().enumerate() {
            for &a in &n.args {
                if a >= i {
                    return Err(DagError::BadEdge { node: i, arg: a });
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Decompositions: op -> template over formal inputs.
// ---------------------------------------------------------------------------

/// Reference decompositions read from KISS-Ops §6.13, plus the ops it could not read.
#[derive(Debug, Default, Clone)]
pub struct Decompositions {
    pub templates: BTreeMap<String, (usize, Dag)>,
    pub unreadable: BTreeSet<String>,
}

impl Decompositions {
    /// Read every `| `op` | family | refinement | `body` ... |` row of the KISS-Ops spec text.
    /// Formal parameters are the single-letter names of the body, taken in alphabetical order
    /// (the §6.13 table writes `x`, or `a`,`b`); a call whose argument count disagrees is a
    /// typed [`DagError::Arity`], so a wrong inference fails loudly.
    pub fn from_ops_spec(markdown: &str) -> Decompositions {
        let mut d = Decompositions::default();
        for line in markdown.replace("\r\n", "\n").lines() {
            let cells: Vec<&str> = line.split('|').map(str::trim).collect();
            // | `op` | family | refinement | body... |  => cells = ["", "`op`", fam, ref, "`body`..."]
            if cells.len() < 5 || !cells[0].is_empty() {
                continue;
            }
            let op = match cells[1].strip_prefix('`').and_then(|s| s.strip_suffix('`')) {
                Some(o) if o.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') => o,
                _ => continue,
            };
            if !(cells[3] == "—" || cells[3] == "✓") {
                continue; // not a §6.13 decomposition row
            }
            let body_cell = cells[4];
            let Some(open) = body_cell.find('`') else { continue };
            let Some(len) = body_cell[open + 1..].find('`') else { continue };
            let body = &body_cell[open + 1..open + 1 + len];
            if body.contains(';') || body.contains('=') {
                d.unreadable.insert(op.to_string()); // let-bound: not a single expression
                continue;
            }
            let mut params: Vec<String> = Vec::new();
            for tok in body.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')) {
                if tok.len() == 1
                    && tok.as_bytes()[0].is_ascii_lowercase()
                    && !params.iter().any(|p| p == tok)
                {
                    params.push(tok.to_string());
                }
            }
            params.sort();
            let refs: Vec<&str> = params.iter().map(String::as_str).collect();
            match Dag::parse_expr(body, &refs) {
                Ok(dag) => {
                    d.templates.insert(op.to_string(), (params.len(), dag));
                }
                Err(_) => {
                    d.unreadable.insert(op.to_string());
                }
            }
        }
        d
    }
}

/// Fully expand `dag` through `table` into an equivalent DAG that contains only primitives.
fn resolve(dag: &Dag, table: &Decompositions) -> Result<Dag, DagError> {
    dag.validate()?;
    let mut out: Vec<Node> = Vec::new();
    let mut map: Vec<usize> = Vec::with_capacity(dag.nodes.len());
    for n in &dag.nodes {
        let args: Vec<usize> = n.args.iter().map(|&a| map[a]).collect();
        let idx = instantiate(&n.op, &n.attrs, &args, table, &mut out, 0)?;
        map.push(idx);
    }
    Ok(Dag { nodes: out, root: map[dag.root] })
}

fn instantiate(
    op: &str,
    attrs: &[u8],
    args: &[usize],
    table: &Decompositions,
    out: &mut Vec<Node>,
    depth: usize,
) -> Result<usize, DagError> {
    if depth > MAX_DEPTH {
        return Err(DagError::TooDeep(op.to_string()));
    }
    if table.unreadable.contains(op) {
        return Err(DagError::Unresolvable(op.to_string()));
    }
    let Some((arity, tpl)) = table.templates.get(op) else {
        out.push(Node { op: op.into(), attrs: attrs.to_vec(), args: args.to_vec() });
        return Ok(out.len() - 1);
    };
    if *arity != args.len() {
        return Err(DagError::Arity { op: op.into(), expected: *arity, got: args.len() });
    }
    let mut map: Vec<usize> = Vec::with_capacity(tpl.nodes.len());
    for n in &tpl.nodes {
        let idx = if n.op == "input" {
            let k: usize = String::from_utf8_lossy(&n.attrs).parse().unwrap_or(usize::MAX);
            *args.get(k).ok_or(DagError::Arity { op: op.into(), expected: *arity, got: args.len() })?
        } else {
            let a: Vec<usize> = n.args.iter().map(|&x| map[x]).collect();
            instantiate(&n.op, &n.attrs, &a, table, out, depth + 1)?
        };
        map.push(idx);
    }
    Ok(map[tpl.root])
}

// ---------------------------------------------------------------------------
// Canonical form and equality.
// ---------------------------------------------------------------------------

/// The canonical form: nodes reachable from the root, children first, commutative operands
/// ordered by their canonical subtree label. Equal forms <=> structurally equal DAGs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Canonical(pub Vec<(String, Vec<u8>, Vec<usize>)>);

fn label(dag: &Dag, i: usize, memo: &mut Vec<Option<String>>) -> String {
    if let Some(l) = &memo[i] {
        return l.clone();
    }
    let n = &dag.nodes[i];
    let mut kids: Vec<String> = n.args.iter().map(|&a| label(dag, a, memo)).collect();
    if is_commutative(&n.op) {
        kids.sort();
    }
    let hex: String = n.attrs.iter().map(|b| format!("{b:02x}")).collect();
    let l = format!("{}[{}]({})", n.op, hex, kids.join(","));
    memo[i] = Some(l.clone());
    l
}

fn emit(
    dag: &Dag,
    i: usize,
    labels: &mut Vec<Option<String>>,
    seen: &mut BTreeMap<usize, usize>,
    out: &mut Vec<(String, Vec<u8>, Vec<usize>)>,
) -> usize {
    if let Some(&k) = seen.get(&i) {
        return k;
    }
    let n = &dag.nodes[i];
    let mut order: Vec<usize> = n.args.clone();
    if is_commutative(&n.op) {
        order.sort_by_key(|&a| label(dag, a, labels));
    }
    let ids: Vec<usize> = order.iter().map(|&a| emit(dag, a, labels, seen, out)).collect();
    out.push((n.op.clone(), n.attrs.clone(), ids));
    seen.insert(i, out.len() - 1);
    out.len() - 1
}

/// Canonicalize `dag` (resolving through `table` first).
pub fn canonical_form(dag: &Dag, table: &Decompositions) -> Result<Canonical, DagError> {
    let r = resolve(dag, table)?;
    let mut labels = vec![None; r.nodes.len()];
    let mut seen = BTreeMap::new();
    let mut out = Vec::new();
    emit(&r, r.root, &mut labels, &mut seen, &mut out);
    Ok(Canonical(out))
}

/// §6.9-0001 structural op-DAG equality. `Err` means a DAG could not be compared (typed
/// decline), which is NOT "unequal".
pub fn structurally_equal(a: &Dag, b: &Dag, table: &Decompositions) -> Result<bool, DagError> {
    Ok(canonical_form(a, table)? == canonical_form(b, table)?)
}

// SPDX-License-Identifier: MIT OR Apache-2.0
//! The reference resolver for KISS-CONTRACT-6.4-0005 / 6.4-0010 over `corpus/op_decomposition.json`
//! (generated from `spec/ops.md` by `tools/kiss_op_decomp.py`).
//!
//! It walks the `direct` edges only: the table's `level` and `floor` columns are the answer it must reproduce,
//! so a generator that mis-derived them disagrees with this walk instead of being believed.

use crate::json::{self, Json};
use std::collections::BTreeSet;

const TABLE: &str = include_str!("../corpus/op_decomposition.json");

/// What a resolver answers for one op node (§6.4-0010).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Resolution {
    /// Every chain reaches the primitive floor with every needed contract available.
    Resolved,
    /// A contract the chain needs is unavailable: the smallest such op name (§6.4-0010 b).
    Deferred(String),
    /// No decomposition reaches the floor: the name is not a KISS-Ops op (§6.4-0010 a).
    Decline,
}

fn ops() -> &'static [(String, Json)] {
    use std::sync::OnceLock;
    static T: OnceLock<Vec<(String, Json)>> = OnceLock::new();
    T.get_or_init(|| {
        let doc = json::parse(TABLE).expect("op_decomposition.json is valid JSON");
        match doc.get("ops") {
            Some(Json::Obj(o)) => o.clone(),
            _ => panic!("op_decomposition.json has an `ops` object"),
        }
    })
}

fn entry(op: &str) -> Option<&'static Json> {
    ops().iter().find(|(n, _)| n == op).map(|(_, e)| e)
}

fn direct(e: &Json) -> Vec<&str> {
    e.get("direct").and_then(Json::as_arr).unwrap_or(&[]).iter().filter_map(Json::as_str).collect()
}

/// Every op name in the table, in table order.
pub fn op_names() -> Vec<&'static str> {
    ops().iter().map(|(n, _)| n.as_str()).collect()
}

/// Walk `op`'s `direct` edges to the floor: `(level, floor ops)`, or `None` for a name outside the table.
/// A floor op has level 0 and is its own floor; any other op is `1 +` its highest child level.
pub fn walk(op: &str) -> Option<(u64, BTreeSet<String>)> {
    let e = entry(op)?;
    if e.get("kind").and_then(Json::as_str) == Some("primitive") {
        return Some((0, BTreeSet::from([op.to_string()])));
    }
    let (mut level, mut floor) = (0, BTreeSet::new());
    for c in direct(e) {
        let (l, f) = walk(c)?;
        level = level.max(l);
        floor.extend(f);
    }
    Some((level + 1, floor))
}

/// The recorded `(level, floor)` of `op`, read from the table's own columns.
pub fn recorded(op: &str) -> Option<(u64, BTreeSet<String>)> {
    let e = entry(op)?;
    let floor = e.get("floor")?.as_arr()?.iter().filter_map(Json::as_str).map(str::to_string).collect();
    Some((e.get("level")?.as_u64()?, floor))
}

fn needed(op: &str, out: &mut BTreeSet<String>) {
    let Some(e) = entry(op) else { return };
    if e.get("kind").and_then(Json::as_str) == Some("primitive") || !out.insert(op.to_string()) {
        return;
    }
    for c in direct(e) {
        needed(c, out);
    }
}

/// Resolve `op` when the contracts of `unavailable` cannot be acquired (a floor op needs no contract).
pub fn resolve(op: &str, unavailable: &[String]) -> Resolution {
    if entry(op).is_none() {
        return Resolution::Decline;
    }
    let mut need = BTreeSet::new();
    needed(op, &mut need);
    match need.into_iter().find(|n| unavailable.contains(n)) {
        Some(missing) => Resolution::Deferred(missing),
        None => Resolution::Resolved,
    }
}

/// The published outcome vectors: `(op, unavailable, expected)`.
pub fn outcome_vectors() -> Vec<(String, Vec<String>, Resolution)> {
    let doc = json::parse(TABLE).expect("op_decomposition.json is valid JSON");
    let rows = doc.get("outcomes").and_then(Json::as_arr).expect("outcomes array");
    rows.iter()
        .map(|r| {
            let s = |k: &str| r.get(k).and_then(Json::as_str).unwrap_or("").to_string();
            let un = r.get("unavailable").and_then(Json::as_arr).unwrap_or(&[]);
            let un = un.iter().filter_map(Json::as_str).map(str::to_string).collect();
            let want = match s("outcome").as_str() {
                "resolved" => Resolution::Resolved,
                "deferred" => Resolution::Deferred(s("missing")),
                "decline" => Resolution::Decline,
                other => panic!("unknown outcome {other:?}"),
            };
            (s("op"), un, want)
        })
        .collect()
}

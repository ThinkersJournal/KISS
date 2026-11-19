// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-Kernel's reference rule (KISS-KERNEL-6.1-0001; board 450 option A): the KISS-Ops "Kernel-governed ops" table
//! (§6.22), the `*Kernel-governed:*` pointers in the Ops clauses and KISS-Kernel's own section map (§2.2) agree. The same
//! check runs as a document lint (`tools/kiss_kernel_refs.py`); this is its independent second implementation.

use std::collections::{BTreeMap, BTreeSet};

const OPS: &str = include_str!("../../spec/ops.md");
const KERNEL: &str = include_str!("../../spec/kernel.md");

/// `6.N` section numbers of kernel.md §2.2 (rows `| 6.N | ...`).
fn kernel_sections(kernel: &str) -> BTreeSet<String> {
    let start = kernel.find("### 2.2").expect("kernel.md has §2.2");
    let rest = &kernel[start..];
    let end = rest[4..].find("\n##").map(|i| i + 4).unwrap_or(rest.len());
    rest[..end]
        .lines()
        .filter_map(|l| l.strip_prefix("| "))
        .filter_map(|l| l.split(" |").next())
        .filter(|c| c.starts_with("6.") && c[2..].chars().all(|d| d.is_ascii_digit()) && c.len() > 2)
        .map(str::to_string)
        .collect()
}

/// `KISS-KERNEL-6.N` section tokens in a text run.
fn kernel_tokens(s: &str) -> Vec<String> {
    s.split("KISS-KERNEL-").skip(1).filter_map(|t| {
        let n: String = t.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
        let n = n.trim_end_matches('.');
        n.contains('.').then(|| n.to_string())
    }).collect()
}

/// op -> (sections, clauses) from ops.md §6.22.
fn governed_table(ops: &str) -> BTreeMap<String, (Vec<String>, Vec<String>)> {
    let start = ops.find("### 6.22 Kernel-governed ops").expect("ops.md has §6.22");
    let rest = &ops[start..];
    let end = rest.find("\n## ").expect("§6.22 ends at the next top-level section");
    let mut out = BTreeMap::new();
    for l in rest[..end].lines() {
        let cells: Vec<&str> = l.trim().trim_matches('|').split(" | ").map(str::trim).collect();
        if cells.len() == 3 && cells[0].starts_with('`') && cells[0].ends_with('`') && cells[1].chars().next().is_some_and(|c| c.is_ascii_digit()) {
            let clauses = cells[2].split(", ").filter(|c| c.starts_with("KISS-OPS-")).map(str::to_string).collect();
            out.insert(cells[0].trim_matches('`').to_string(), (cells[1].split(", ").map(str::to_string).collect(), clauses));
        }
    }
    out
}

/// Ops clause id -> sections of its `*Kernel-governed:*` pointer(s), by the nearest preceding `- **KISS-OPS-...**`.
fn pointers(ops: &str) -> BTreeMap<String, BTreeSet<String>> {
    let mut out: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut current: Option<String> = None;
    for l in ops.lines() {
        if let Some(r) = l.strip_prefix("- **KISS-OPS-") {
            current = r.split("**").next().map(|id| format!("KISS-OPS-{id}"));
        }
        let mut rest = l;
        while let Some(i) = rest.find("*Kernel-governed:*") {
            let tail = &rest[i + "*Kernel-governed:*".len()..];
            // the pointer is the run of `KISS-KERNEL-n.m` items separated by `, `
            let mut end = 0;
            let mut t = tail.trim_start();
            let lead = tail.len() - t.len();
            while t.starts_with("KISS-KERNEL-") {
                let n = t["KISS-KERNEL-".len()..].chars().take_while(|c| c.is_ascii_digit() || *c == '.').count();
                end += "KISS-KERNEL-".len() + n;
                t = &t["KISS-KERNEL-".len() + n..];
                match t.strip_prefix(", ") { Some(x) => { end += 2; t = x } None => break }
            }
            if let Some(c) = &current {
                out.entry(c.clone()).or_default().extend(kernel_tokens(&tail[lead..lead + end]));
            }
            rest = &tail[lead + end..];
        }
    }
    out
}

/// The violations of the reference rule over these two documents; empty when they agree.
fn violations(ops: &str, kernel: &str, all_ops: &[&str]) -> Vec<String> {
    let known = kernel_sections(kernel);
    let table = governed_table(ops);
    let ptr = pointers(ops);
    let mut v = Vec::new();
    if known.is_empty() || table.is_empty() {
        v.push("kernel.md §2.2 or ops.md §6.22 parsed empty".to_string());
    }
    let mut listed = BTreeSet::new();
    for (op, (secs, clauses)) in &table {
        if !all_ops.contains(&op.as_str()) {
            v.push(format!("{op} is not a KISS-Ops op"));
        }
        for s in secs.iter().filter(|s| !known.contains(*s)) {
            v.push(format!("{op}: section {s} absent from kernel.md §2.2"));
        }
        for c in clauses {
            listed.insert(c.clone());
            match ptr.get(c) {
                None => v.push(format!("{c}: governed op {op} but no pointer")),
                Some(have) => {
                    for s in secs.iter().filter(|s| !have.contains(*s)) {
                        v.push(format!("{c}: pointer omits {s} for {op}"));
                    }
                }
            }
        }
    }
    for (c, secs) in &ptr {
        if !listed.contains(c) {
            v.push(format!("{c}: pointer with no §6.22 row"));
        }
        for s in secs.iter().filter(|s| !known.contains(*s)) {
            v.push(format!("{c}: pointer names section {s} absent from kernel.md §2.2"));
        }
    }
    v
}

/// The shipped documents agree, and each way they can disagree is caught: a governed op whose Ops clause lost its pointer, a
/// pointer no table row lists, a table row naming a section KISS-Kernel does not list, and a table op that is not an op.
/// Enforces[enforced]: KISS-KERNEL-6.1-0001 — the §6.22 table, the Ops pointers and kernel.md §2.2 agree
// Proven: KISS-KERNEL-6.1-0001 (subject: impl; ref: kernel_refs.rs)
#[test]
fn test_kernel_governed_ops_have_pointers() {
    let ops_names = ["scatter", "scatter_add", "add"];
    assert_eq!(violations(OPS, KERNEL, &ops_names), Vec::<String>::new());
    let table = governed_table(OPS);
    assert_eq!(table.keys().map(String::as_str).collect::<Vec<_>>(), ["scatter", "scatter_add"]);

    // MUTATIONS, each asserted to apply exactly once and to be caught.
    let strip = OPS.replacen("(*Kernel-governed:* KISS-KERNEL-6.4, KISS-KERNEL-6.5; §6.22)", "", 1);
    assert_ne!(strip, OPS, "mutation 1 did not apply");
    assert!(violations(&strip, KERNEL, &ops_names).iter().any(|m| m.contains("no pointer")), "a governed op with no pointer is caught");

    let orphan = OPS.replacen("| `scatter_add` | 6.4, 6.5 | KISS-OPS-6.11-0006, KISS-OPS-6.11-0024 |\n", "", 1);
    assert_ne!(orphan, OPS, "mutation 2 did not apply");
    assert!(violations(&orphan, KERNEL, &ops_names).is_empty(), "scatter still lists 0006 and 0024, so dropping the scatter_add row alone is not a violation");
    let orphan = orphan.replacen("| `scatter` | 6.4, 6.5 | KISS-OPS-6.11-0006, KISS-OPS-6.11-0010, KISS-OPS-6.11-0024 |", "| `scatter` | 6.4, 6.5 | KISS-OPS-6.11-0006, KISS-OPS-6.11-0024 |", 1);
    assert!(violations(&orphan, KERNEL, &ops_names).iter().any(|m| m.contains("pointer with no")), "a pointer no table row lists is caught");

    let reserved = KERNEL.replacen("| 6.5 |", "| 6.9 |", 1);
    assert_ne!(reserved, KERNEL, "mutation 3 did not apply");
    assert!(violations(OPS, &reserved, &ops_names).iter().any(|m| m.contains("absent from kernel.md")), "a section kernel.md does not list is caught");

    assert!(violations(OPS, KERNEL, &["add"]).iter().any(|m| m.contains("is not a KISS-Ops op")), "a table op that is not an op is caught");
}

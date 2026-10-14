// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-Conform §6.13-0003 — the KISS-Classify tier: the `structure_key` token codec and the
//! `target_capability` grammar are compared under the **exact-byte** comparator, the
//! namespace-registry snapshot is **bundled** with the suite, and golden token vectors are
//! supplied.
//!
//! Three obligations, one test, each with its own tooth:
//!   * COMPARATOR — every bundled positive vector's token survives parse -> re-emit and the
//!     result is compared with `compare(ExactByte, ..)`. A one-byte drift in the re-emitted
//!     token (the control below) MUST be rejected, so the loop cannot pass vacuously.
//!   * BUNDLE — the registry snapshot compiled into the suite is versioned, machine-readable,
//!     and holds a row for every namespace the golden vectors name. A vector namespace with no
//!     row (the control) is detected.
//!   * VECTORS — the golden file is non-empty and names at least one target namespace.
//!
//! Whether an offline reader can reconstruct the *whole* suite from the bundle (matrix, oracle
//! vectors, negative vectors, op signature set) is §6.3-0003's scope, not this clause's.

use kiss_conformance::json::{self, Json};
use kiss_conformance::structure_key::from_token;
use kiss_conformance::{compare, DeterminismClass};

const VECTORS: &str = include_str!("../corpus/structure_key_vectors.json");
const REGISTRY: &str = include_str!("../registry/namespaces.json");

fn arr<'a>(j: &'a Json, key: &str) -> &'a [Json] {
    j.get(key).and_then(Json::as_arr).unwrap_or_else(|| panic!("missing array `{key}`"))
}

fn registry_namespaces(reg: &Json) -> Vec<String> {
    arr(reg, "namespaces")
        .iter()
        .map(|r| r.get("namespace").and_then(Json::as_str).expect("row namespace").to_string())
        .collect()
}

/// The first `:`-delimited component of a `target_capability`, the registered namespace.
fn namespace_of(target: &str) -> &str {
    target.split(':').next().unwrap_or("")
}

/// KISS-CONFORM-6.13-0003 — exact-byte codec comparison, bundled registry, golden vectors.
#[test]
fn test_conform_classify_exact_byte_and_registry_bundle() {
    let vectors = json::parse(VECTORS).expect("golden vectors parse");
    let reg = json::parse(REGISTRY).expect("registry snapshot parses");

    // VECTORS: supplied and non-empty.
    let positives = arr(&vectors, "positive_vectors");
    assert!(
        !positives.is_empty(),
        "KISS-CONFORM-6.13-0003: the bundle supplies no golden token vectors"
    );

    // BUNDLE: the snapshot is versioned and machine-readable.
    assert!(
        reg.get("registry_version").and_then(Json::as_u64).is_some(),
        "KISS-CONFORM-6.13-0003: the bundled registry snapshot carries no registry_version"
    );
    let registered = registry_namespaces(&reg);
    assert!(!registered.is_empty(), "KISS-CONFORM-6.13-0003: the registry holds no namespace");

    // COMPARATOR + BUNDLE: every golden token re-emits byte-identically, and its target
    // namespace has a registry row.
    for v in positives {
        let name = v.get("name").and_then(Json::as_str).expect("vector name");
        let token = v.get("token").and_then(Json::as_str).expect("vector token");
        let key = from_token(token)
            .unwrap_or_else(|e| panic!("KISS-CONFORM-6.13-0003: golden `{name}` declined: {e:?}"));
        let re = key.to_token();
        if let Err(e) = compare(DeterminismClass::ExactByte, re.as_bytes(), token.as_bytes()) {
            panic!("KISS-CONFORM-6.13-0003: golden `{name}` re-emit differs: {e}");
        }
        let ns = namespace_of(v.get("target").and_then(Json::as_str).expect("vector target"));
        assert!(
            registered.iter().any(|r| r == ns),
            "KISS-CONFORM-6.13-0003: golden `{name}` names namespace `{ns}` with no registry row"
        );
    }

    // TEETH. A one-byte drift in a re-emitted token MUST fail the comparator ...
    let first = positives[0].get("token").and_then(Json::as_str).unwrap();
    let mut drifted = first.as_bytes().to_vec();
    let last = drifted.len() - 1;
    drifted[last] ^= 0x01;
    assert!(
        compare(DeterminismClass::ExactByte, &drifted, first.as_bytes()).is_err(),
        "KISS-CONFORM-6.13-0003: the exact-byte comparator accepted a one-byte drift"
    );
    // ... and a namespace outside the snapshot MUST be seen as unregistered.
    assert!(
        !registered.iter().any(|r| r == namespace_of("unregistered_ns:x")),
        "KISS-CONFORM-6.13-0003: the registry membership check cannot reject a stranger"
    );
}

//! KISS-Conform teeth for the §6.1 traceability matrix and the §6.2 build-fail
//! gate — the clauses describing the suite's OWN build gate.
//!
//! The gate is implemented by `tools/kiss_trace.py`, which accepts `--spec-dir`
//! and `--conformance-dir`. Each test below builds a synthetic spec + harness
//! fixture exhibiting exactly one defect and asserts the gate REJECTS it, plus a
//! clean control asserting it ACCEPTS a well-formed suite. The control is what
//! stops the negative assertions passing vacuously: a gate that rejects
//! everything is not a gate.
//!
//! CITATION DISCIPLINE: each test cites ONLY its own clause id; cross-references
//! use the `§<sec>-<nnnn>` short form, which does not match the citation grammar.
//!
//! Python is NOT an optional toolchain here — `kiss_trace.py` IS the §6.2 gate,
//! so a missing interpreter FAILS these tests rather than silently skipping
//! them. A skip would report `ok` while verifying nothing.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The ten spec stems the gate requires; a missing file is itself a violation.
const STEMS: &[&str] = &[
    "umbrella", "announce", "classify", "ops", "grammar", "contract", "synth",
    "consume", "emit", "conform",
];

/// Fixture clause IDs, assembled at run time.
///
/// Written as source literals these match the clause-ID grammar, get scanned out
/// of this file by the very gate under test, and are reported as dangling
/// citations against the real suite. Building them from parts keeps the fixture
/// invisible to the scanner while remaining exact in the generated spec.
fn fid(ordinal: &str) -> String {
    format!("{}-{}-{}-{}", "KISS", "OPS", "6.99", ordinal)
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
}

/// Build a fixture suite: `spec/` with `ops_body` written into `ops.md` (all
/// other stems empty), and `conformance/` containing `harness` as Rust source.
fn fixture(tag: &str, ops_body: &str, harness: &str) -> PathBuf {
    // Unique per process: two concurrent `cargo test` runs would otherwise share
    // one fixture root and clobber each other mid-run.
    let root = std::env::temp_dir()
        .join(format!("kiss_tracegate_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let spec = root.join("spec");
    let conf = root.join("conformance");
    std::fs::create_dir_all(&spec).expect("mkdir spec");
    std::fs::create_dir_all(&conf).expect("mkdir conformance");
    for s in STEMS {
        let content = if *s == "ops" { ops_body } else { "" };
        std::fs::write(spec.join(format!("{s}.md")), content).expect("write spec stem");
    }
    std::fs::write(conf.join("fixture_tests.rs"), harness).expect("write harness");
    root
}

/// Run the gate over a fixture; returns (exited_zero, combined output).
fn run_gate(root: &Path) -> (bool, String) {
    let tool = repo_root().join("tools").join("kiss_trace.py");
    let out = Command::new("python")
        .arg(&tool)
        .arg("--spec-dir")
        .arg(root.join("spec"))
        .arg("--conformance-dir")
        .arg(root.join("conformance"))
        .output()
        .unwrap_or_else(|e| {
            panic!(
                "could not run the build gate ({}): {e}. Python is a REQUIRED \
                 dependency of this suite — kiss_trace.py IS the gate — so this \
                 is a failure, not a skip.",
                tool.display()
            )
        });
    let mut s = String::from_utf8_lossy(&out.stdout).to_string();
    s.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.success(), s)
}

/// A well-formed one-clause spec: body definition + §9 matrix row.
fn clean_spec() -> String {
    let mut s = String::new();
    s.push_str("## 6.0 Fixture\n\n");
    s.push_str(&format!("- **{}** — A fixture clause. An implementation MUST do ", fid("0042")));
    s.push_str("the fixture thing. *Test:* `test_ops_fixture_probe`.\n\n");
    s.push_str("## 9. Traceability\n\n");
    s.push_str("| Clause | Test |\n|---|---|\n");
    s.push_str(&format!("| {} | `test_ops_fixture_probe` |\n", fid("0042")));
    s
}

fn clean_harness() -> &'static str {
    "#[test]\nfn test_ops_fixture_probe() { assert!(true); }\n"
}

// ---- the discrimination control ---------------------------------------------

/// Assert the gate ACCEPTS a well-formed suite. A helper, deliberately not a
/// `#[test]`.
///
/// It is not evidence for any clause — it is what makes the negative assertions
/// mean anything, since a gate that rejects unconditionally satisfies every one
/// of them. Standing alone as a test it would cite no clause and count as an
/// orphan; citing one would credit coverage to a test whose entire content is
/// "a valid suite passes", which is the tautology refused for §6.1-0001.
///
/// Each clause-bound test calls this FIRST, so every negative assertion is paired
/// with its positive control in the same test — stronger than a standalone
/// control, because the pairing cannot drift apart.
fn assert_gate_accepts_a_wellformed_suite(tag: &str) {
    let root = fixture(tag, &clean_spec(), clean_harness());
    let (ok, out) = run_gate(&root);
    assert!(
        ok,
        "control: the gate MUST accept a well-formed suite, otherwise the negative \
         assertion in this test proves nothing — a gate that rejects everything \
         satisfies it trivially. The gate rejected:\n{out}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

// ---- KISS-CONFORM-6.2-0002 — dangling cite ---------------------------------

/// KISS-CONFORM-6.2-0002 — the build MUST fail on a test citing a clause ID that
/// exists in no current clause source. Catches a gate that validates only the
/// clause-to-test direction: the cite below names a clause defined nowhere, and
/// a one-directional gate reports the suite clean.
#[test]
fn test_conform_build_fails_dangling_cite() {
    assert_gate_accepts_a_wellformed_suite("ctl_dangling");
    // A bare-comment reference to a clause defined nowhere: the dangling gate is
    // REFERENCE hygiene (§3.3 burns retired ids), so it flags a stale reference even in a
    // comment that backs nothing — #187 narrowed COVERAGE credit to real backings but the
    // dangling scan still reads every citation (`cited_raw`), or a burned id in a comment
    // would be caught by nothing. This fixture is the teeth on that: a mention, not a backing.
    let harness = format!(
        "// this fixture test cites {}, which is defined nowhere\n#[test]\nfn \
         test_ops_fixture_probe() {{ assert!(true); }}\n",
        fid("9999")
    );
    let root = fixture("dangling", &clean_spec(), &harness);
    let (ok, out) = run_gate(&root);
    assert!(
        !ok,
        "KISS-CONFORM-6.2-0002: a test citing a clause ID that exists in no spec \
         MUST fail the build; the gate accepted it:\n{out}"
    );
    assert!(
        out.contains("DANGLING"),
        "KISS-CONFORM-6.2-0002: the failure MUST identify the dangling citation so \
         it can be triaged; output was:\n{out}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

// ---- KISS-CONFORM-6.2-0001 — an untested MUST fails the BUILD --------------

/// KISS-CONFORM-6.2-0001 — the build MUST fail hard and non-zero on a normative
/// clause that no test cites, independent of whether any mapped test passes.
/// Catches a gate that treats an untested MUST as advisory: the clause here is
/// well-formed and its named test simply does not exist in the harness.
#[test]
fn test_conform_build_fails_untested_must() {
    assert_gate_accepts_a_wellformed_suite("ctl_untested");
    // The harness contains a test, but NOT the one the clause names.
    let harness = "#[test]\nfn test_ops_something_else() { assert!(true); }\n";
    let root = fixture("untested", &clean_spec(), harness);
    let (ok, out) = run_gate(&root);
    assert!(
        !ok,
        "KISS-CONFORM-6.2-0001: a normative clause whose named test does not exist \
         MUST fail the build hard and non-zero; the gate accepted it:\n{out}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

// ---- #247 subsumption proof: forward-existence subsumes the `test_`-prefix check ----

/// The `test_`-prefix convention (`kiss_trace.py`, doc-consistency check 5) was removed
/// in #247 because its ONLY real service — *"the matrix names something that is actually a
/// test"* — is delivered STRICTLY BETTER by the forward-existence check above: existence
/// also catches a bogus name that happens to start with `test_`, which a prefix match never
/// could. This proves the subsumption on the case the architect named as load-bearing (#247):
/// a §9 entry naming a REAL symbol that is not a `#[test]`. The name IS `test_`-prefixed, so
/// the prefix check is NEUTRAL — any failure here is the existence check alone. If this ever
/// passes, the existence check does NOT subsume the prefix check and it must not be removed.
/// Convention 9: the seed asserts it applied.
#[test]
fn test_conform_existence_subsumes_the_prefix_check() {
    assert_gate_accepts_a_wellformed_suite("ctl_subsume");
    // `clean_spec()` names `test_ops_fixture_probe` (test_-prefixed). The harness defines that
    // EXACT symbol but NOT as a `#[test]` — a real fn, not a test. `discover_tests` finds only
    // `#[test] fn`s, so existence must still redden.
    let harness = "fn test_ops_fixture_probe() { assert!(true); } // a plain fn, no test attribute\n";
    assert!(
        harness.contains("fn test_ops_fixture_probe") && !harness.contains("#[test]"),
        "SEED NOT APPLIED (convention 9): the fixture must define the named symbol as a \
         non-#[test] fn, or this proves nothing"
    );
    eprintln!("SEED APPLIED: §9 names `test_ops_fixture_probe`; harness defines it as a non-#[test] fn");
    let root = fixture("subsume", &clean_spec(), harness);
    let (ok, out) = run_gate(&root);
    assert!(
        !ok,
        "#247 SUBSUMPTION INCOMPLETE: a §9 matrix entry naming a real symbol that is not a \
         `#[test]` (name test_-prefixed, so the prefix check is neutral) MUST fail via \
         forward-existence. If it does not, `discover_tests`' `#[test]`-only scan is not \
         catching a non-test symbol, the prefix check WAS doing something, and it must NOT be \
         removed. Output:\n{out}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

// ---- KISS-CONFORM-6.1-0002 — direction-1 totality (clause -> test) ---------

/// KISS-CONFORM-6.1-0002 — the matrix MUST be total in direction 1: every
/// normative clause resolves to at least one mapped named test. This is the
/// STRUCTURAL mapping-existence property, distinct from §6.2-0001's build-fail
/// consequence asserted above by exit status. Catches a matrix that silently
/// drops a clause: the second clause is defined in the body with no matrix row,
/// so a gate harvesting only the matrix never sees it.
#[test]
fn test_conform_every_clause_has_test() {
    assert_gate_accepts_a_wellformed_suite("ctl_totality");
    let mut spec = String::new();
    spec.push_str("## 6.0 Fixture\n\n");
    spec.push_str(&format!("- **{}** — A fixture clause. An implementation MUST do ", fid("0042")));
    spec.push_str("the fixture thing. *Test:* `test_ops_fixture_probe`.\n");
    spec.push_str(&format!("- **{}** — A second clause the matrix OMITS. An ", fid("0043")));
    spec.push_str("implementation MUST also do this. *Test:* `test_ops_second_probe`.\n\n");
    spec.push_str("## 9. Traceability\n\n");
    spec.push_str("| Clause | Test |\n|---|---|\n");
    spec.push_str(&format!("| {} | `test_ops_fixture_probe` |\n", fid("0042")));
    let harness = concat!(
        "#[test]\nfn test_ops_fixture_probe() { assert!(true); }\n",
        "#[test]\nfn test_ops_second_probe() { assert!(true); }\n"
    );
    let root = fixture("totality", &spec, harness);
    let (ok, out) = run_gate(&root);
    assert!(
        !ok,
        "KISS-CONFORM-6.1-0002: a normative clause defined in the body but absent \
         from the traceability matrix breaks direction-1 totality and MUST be \
         reported; the gate accepted it:\n{out}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

// ---- KISS-CONFORM-6.1-0005 — coverage measured on normative clauses only ---

/// KISS-CONFORM-6.1-0005 — coverage MUST be measured against normative clauses
/// only (MUST / MUST NOT / SHALL); SHOULD / MAY governance clauses MUST be
/// catalogued separately and MUST NOT be build-gating.
///
/// Asserted against the live suite rather than a fixture, because the property
/// is about WHICH clauses enter the measured set. Catches the plausible drift: a
/// SHOULD-bodied clause given a clause ID, which would silently become
/// build-gating and inflate the denominator with a non-normative row.
#[test]
fn test_conform_coverage_normative_only() {
    let defs = clause_definitions(&repo_root().join("spec"))
        .expect("the clause scan MUST complete; an unreadable spec file is an \n                error, not a silently smaller scan");
    assert!(
        !defs.is_empty(),
        "KISS-CONFORM-6.1-0005: the clause scan found nothing — an empty scan \
         would satisfy the assertion below vacuously"
    );
    let offenders: Vec<String> = defs
        .into_iter()
        .filter(|(_, body)| {
            let normative = body.contains("MUST") || body.contains("SHALL");
            let governance = body.contains("SHOULD") || body.contains("MAY");
            !normative && governance
        })
        .map(|(cid, _)| cid)
        .collect();
    assert!(
        offenders.is_empty(),
        "KISS-CONFORM-6.1-0005: coverage is measured against normative clauses \
         only, but these clause IDs carry a SHOULD/MAY operative verb with no \
         MUST/SHALL, so they would be build-gated as if normative: {offenders:?}"
    );
}

/// Scan clause definitions: `- **<ID>** — <body>`, up to the next definition or
/// heading. Stdlib only, per the harness's no-dependency rule.
///
/// An unreadable stem is an ERROR, never a skip. A `continue` here would produce a
/// truncated scan indistinguishable from a complete one, and the caller asserts on
/// what the scan FOUND — so a spec file that could not be read would silently
/// remove its clauses from examination and the assertion would pass over them.
/// That is exactly the degradation §6.5-0016 forbids, and it is the fourth
/// instance of this idiom in this repository; see #142.
fn clause_definitions(spec_dir: &Path) -> std::io::Result<Vec<(String, String)>> {
    let mut out = Vec::new();
    for s in STEMS {
        let p = spec_dir.join(format!("{s}.md"));
        let text = std::fs::read_to_string(&p).map_err(|e| {
            std::io::Error::new(e.kind(), format!("read_to_string({}): {e}", p.display()))
        })?;
        let lines: Vec<&str> = text.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim_start();
            let Some(rest) = line.strip_prefix("- **KISS-") else {
                i += 1;
                continue;
            };
            let Some(end) = rest.find("**") else {
                i += 1;
                continue;
            };
            let cid = format!("KISS-{}", &rest[..end]);
            if !is_clause_id(&cid) {
                // A document name (`- **KISS-Grammar** — ...`), not a clause id.
                i += 1;
                continue;
            }
            let mut body = String::new();
            let mut j = i;
            while j < lines.len() {
                let l = lines[j].trim_start();
                if j > i && (l.starts_with("- **KISS-") || l.starts_with('#')) {
                    break;
                }
                body.push_str(lines[j]);
                body.push('\n');
                j += 1;
            }
            out.push((cid, body));
            i = j;
        }
    }
    Ok(out)
}

/// Does `cid` match the clause-ID grammar `KISS-<SUB>-<sec>-<nnnn>[a]`?
///
/// Prose contains bold list items naming DOCUMENTS (`- **KISS-Grammar** — ...`).
/// Scanning those as clauses is how the first version of this test reported four
/// phantom offenders; the shape check is what makes the scan mean what it says.
fn is_clause_id(cid: &str) -> bool {
    let parts: Vec<&str> = cid.split('-').collect();
    if parts.len() != 4 || parts[0] != "KISS" {
        return false;
    }
    let sub_ok = !parts[1].is_empty()
        && parts[1].bytes().all(|b| b.is_ascii_uppercase());
    let sec_ok = !parts[2].is_empty()
        && parts[2].bytes().all(|b| b.is_ascii_digit() || b == b'.');
    let ord = parts[3].trim_end_matches(|c: char| c.is_ascii_lowercase());
    let ord_ok = ord.len() == 4 && ord.bytes().all(|b| b.is_ascii_digit());
    sub_ok && sec_ok && ord_ok
}

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
    let root = std::env::temp_dir().join(format!("kiss_tracegate_{tag}"));
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

// ---- the control: a well-formed suite MUST be accepted ----------------------

#[test]
fn trace_gate_accepts_a_wellformed_suite() {
    // Discrimination control. Without it, every negative assertion below is
    // satisfiable by a gate that rejects unconditionally.
    let root = fixture("clean", &clean_spec(), clean_harness());
    let (ok, out) = run_gate(&root);
    assert!(ok, "the gate MUST accept a well-formed suite; it rejected:\n{out}");
    let _ = std::fs::remove_dir_all(&root);
}

// ---- KISS-CONFORM-6.2-0002 — dangling cite ---------------------------------

/// KISS-CONFORM-6.2-0002 — the build MUST fail on a test citing a clause ID that
/// exists in no current clause source. Catches a gate that validates only the
/// clause-to-test direction: the cite below names a clause defined nowhere, and
/// a one-directional gate reports the suite clean.
#[test]
fn test_conform_build_fails_dangling_cite() {
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

// ---- KISS-CONFORM-6.1-0002 — direction-1 totality (clause -> test) ---------

/// KISS-CONFORM-6.1-0002 — the matrix MUST be total in direction 1: every
/// normative clause resolves to at least one mapped named test. This is the
/// STRUCTURAL mapping-existence property, distinct from §6.2-0001's build-fail
/// consequence asserted above by exit status. Catches a matrix that silently
/// drops a clause: the second clause is defined in the body with no matrix row,
/// so a gate harvesting only the matrix never sees it.
#[test]
fn test_conform_every_clause_has_test() {
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
    let defs = clause_definitions(&repo_root().join("spec"));
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
fn clause_definitions(spec_dir: &Path) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for s in STEMS {
        let Ok(text) = std::fs::read_to_string(spec_dir.join(format!("{s}.md"))) else {
            continue;
        };
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
    out
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

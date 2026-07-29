//! Shared integration-test helper: compile a C fixture to a DLL and resolve its
//! `kiss_add` entry as a `BinaryKernel`. (Rust integration-test files are
//! separate crates; `tests/common/mod.rs` is the idiomatic single home.)

use kiss_conformance::harness::abi::BinaryKernel;
use kiss_conformance::harness::loader::Artifact;
use kiss_conformance::harness::msvc;
use std::path::PathBuf;

/// Compile `tests/harness_fixtures/<name>.c` to a DLL and resolve `kiss_add`.
/// Returns `None` (skip) if no toolchain. Leaks the `Artifact` so the resolved
/// fn pointer stays valid for the test's lifetime.
pub fn compile_and_load(name: &str) -> Option<BinaryKernel> {
    let m = msvc::find_msvc()?;
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/harness_fixtures")
        .join(format!("{name}.c"));
    let out = std::env::temp_dir().join(format!("kiss_harness_{name}"));
    std::fs::create_dir_all(&out).unwrap();
    let dll = msvc::compile_c_to_dll(&m, &src, &out).expect("compile fixture");
    let art = Box::leak(Box::new(Artifact::load(&dll).expect("load fixture")));
    let sym = art.symbol("kiss_add").expect("kiss_add export");
    // SAFETY: every fixture exports exactly the BinaryKernel C signature.
    Some(unsafe { std::mem::transmute::<*const core::ffi::c_void, BinaryKernel>(sym) })
}

//! Shared integration-test helper: compile a C fixture to a DLL and resolve its
//! `kiss_add` entry as a `BinaryKernel`. (Rust integration-test files are
//! separate crates; `tests/common/mod.rs` is the idiomatic single home.)
//!
//! Each integration-test binary includes this module via `mod common;` but only
//! calls a subset of its helpers (e.g. the reduce smoke test never calls
//! `compile_and_load`, the binary-op tests never call `compile_and_load_reduce`).
//! Since each integration test compiles as its own crate, `dead_code` would
//! otherwise fire per-binary on whichever helper that binary doesn't use.

#![allow(dead_code)]

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
    // Per-process dir: Cargo runs integration-test binaries in parallel, and more
    // than one compiles the same fixture — a shared deterministic dir would race on
    // the linker output / DLL file lock. `process::id()` separates the binaries.
    let out = std::env::temp_dir().join(format!("kiss_harness_{}_{name}", std::process::id()));
    std::fs::create_dir_all(&out).unwrap();
    let dll = msvc::compile_c_to_dll(&m, &src, &out).expect("compile fixture");
    let art = Box::leak(Box::new(Artifact::load(&dll).expect("load fixture")));
    let sym = art.symbol("kiss_add").expect("kiss_add export");
    // SAFETY: every fixture exports exactly the BinaryKernel C signature.
    Some(unsafe { std::mem::transmute::<*const core::ffi::c_void, BinaryKernel>(sym) })
}

use kiss_conformance::harness::abi::ReduceKernel;

/// Compile `tests/harness_fixtures/<name>.c` and resolve `kiss_reduce_mean` as a
/// `ReduceKernel`. `None` (skip) if no toolchain; leaks the `Artifact` so the fn
/// pointer stays valid.
pub fn compile_and_load_reduce(name: &str) -> Option<ReduceKernel> {
    let m = msvc::find_msvc()?;
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/harness_fixtures")
        .join(format!("{name}.c"));
    let out = std::env::temp_dir().join(format!("kiss_harness_{}_{name}", std::process::id()));
    std::fs::create_dir_all(&out).unwrap();
    let dll = msvc::compile_c_to_dll(&m, &src, &out).expect("compile fixture");
    let art = Box::leak(Box::new(Artifact::load(&dll).expect("load fixture")));
    let sym = art.symbol("kiss_reduce_mean").expect("kiss_reduce_mean export");
    // SAFETY: every reduce fixture exports exactly the ReduceKernel C signature.
    Some(unsafe { std::mem::transmute::<*const core::ffi::c_void, ReduceKernel>(sym) })
}

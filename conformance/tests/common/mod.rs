// Shared test helper for compiling and loading C fixtures.

use kiss_conformance::harness::abi::BinaryKernel;
use kiss_conformance::harness::{loader::Artifact, msvc};
use std::path::Path;

/// Compile a fixture C file and load its `kiss_add` symbol.
///
/// Returns `None` if the MSVC toolchain is unavailable, or if compilation/loading fails.
/// The returned kernel pointer is valid for the lifetime of the process (the `Artifact`
/// is `Box::leak`ed to prevent unloading).
pub fn compile_and_load(fixture_name: &str) -> Option<BinaryKernel> {
    let Some(msvc) = msvc::find_msvc() else { return None; };

    // Locate the fixture source relative to this test file.
    let fixture_path = Path::new(file!())
        .parent()?
        .join("harness_fixtures")
        .join(format!("{fixture_name}.c"));

    let out_dir = std::env::temp_dir().join("kiss_harness_smoke_test");
    std::fs::create_dir_all(&out_dir).ok()?;

    // Compile the fixture to a DLL.
    let dll = msvc::compile_c_to_dll(&msvc, &fixture_path, &out_dir).ok()?;

    // Load the DLL.
    let artifact = Artifact::load(&dll).ok()?;

    // Resolve the `kiss_add` symbol.
    let sym = artifact.symbol("kiss_add").ok()?;

    // SAFETY: the fixture exports the §6.5 elementwise-binary C signature.
    let kernel: BinaryKernel = unsafe { std::mem::transmute(sym) };

    // Box::leak the artifact to keep it alive for the kernel pointer's lifetime.
    Box::leak(Box::new(artifact));

    Some(kernel)
}

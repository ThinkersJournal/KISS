# Differential-Conformance Harness Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a runnable differential-conformance harness that differences a foreign C `add` kernel — invoked live through the KISS-Contract §6.5 positional C-ABI — against the from-scratch CPU oracle, and proves it catches a wrong kernel (realizing the ≥2-dissimilar-impls freeze-gate, KISS-CONFORM-6.13-0006).

**Architecture:** A new `harness` module tree in the `kiss-conformance` crate. It (1) discovers the MSVC toolchain and compiles C fixture kernels to DLLs at test time (mirroring the existing `cuda`/`nvcc` runtime-shell-out pattern), (2) loads a DLL and resolves its entry symbol via raw Win32 FFI, (3) marshals a §6.5 elementwise-binary invocation and calls it, and (4) differences the kernel's outputs against `semantics::add` over a provenance-tagged deterministic corpus, emitting a verdict. Reuses `differential.rs` (corpus + `agree`) and `semantics.rs` (oracle).

**Tech Stack:** Rust (edition 2021, stdlib only), raw Win32 FFI (`kernel32`: `LoadLibraryW`/`GetProcAddress`/`FreeLibrary`), MSVC `cl.exe` for fixture compilation, C for the kernel fixtures.

## Global Constraints

- **Stdlib only** — no crate dependencies (dev or runtime). Raw FFI, not `libloading`; direct `cl.exe` invocation, not the `cc` crate. (KISS-Conform §6.5: the harness shares no lowering code with any impl under test.)
- **No `build.rs`** — fixtures compile at *test* time via a runtime shell-out, gated on toolchain availability, exactly like the `cuda` feature. Default `cargo test` must pass whether or not the C toolchain is reachable.
- **All `unsafe` isolated** in `loader.rs` and the single call-through-fn-pointer site, each with a documented `// SAFETY:` contract.
- **Edition 2021, rust-version ≥ 1.77.** Crate name in code: `kiss_conformance`.
- **Verified MSVC invocation (2026-07-29, this host):** cl at `C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\<latest>\bin\Hostx64\x64\cl.exe`; set `INCLUDE` = MSVC `include` + Windows Kits `Include\<sdk>\{ucrt,um,shared}`; set `LIB` = MSVC `lib\x64` + Windows Kits `Lib\<sdk>\{ucrt,um}\x64`; args `/nologo /LD /O2 <src.c> /Fe:<out.dll>`. `<latest>` = newest dir under MSVC Tools; `<sdk>` = newest dir under `Windows Kits\10\Include`.
- **Commit trailers** on every commit:
  ```
  Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
  Claude-Session: https://claude.ai/code/session_01KCqNyYxCai7zELrXNnX5XX
  ```
- **Worktree:** all work happens in `C:/Projects/kiss-harness` on branch `feat/differential-harness`. Run `cargo` from `C:/Projects/kiss-harness/conformance`.

---

## File Structure

| File | Responsibility |
|---|---|
| `conformance/src/harness/mod.rs` | Module root; `HarnessError`; re-exports. |
| `conformance/src/harness/msvc.rs` | MSVC toolchain discovery + `compile_c_to_dll`. |
| `conformance/src/harness/loader.rs` | Raw Win32 FFI: load a DLL, resolve a symbol. All `unsafe` here. |
| `conformance/src/harness/abi.rs` | The §6.5 elementwise-binary kernel type + `invoke_binary` marshaller. |
| `conformance/src/harness/corpus.rs` | Provenance-tagged deterministic corpus of `(a, b, expected)` vectors. |
| `conformance/src/harness/differ.rs` | Class-aware comparison + `Divergence`; `run_binary` verdict. |
| `conformance/src/lib.rs` | Add `pub mod harness;` (after line 32, near `differential`). |
| `conformance/tests/harness_fixtures/add_a.c` | Correct C `add` kernel #1. |
| `conformance/tests/harness_fixtures/add_b.c` | Correct C `add` kernel #2 (independent source). |
| `conformance/tests/harness_fixtures/add_wrong.c` | Deliberately-wrong C `add` kernel. |
| `conformance/tests/harness_differential.rs` | The freeze-gate integration test. |

---

### Task 1: MSVC discovery + `compile_c_to_dll`

**Files:**
- Create: `conformance/src/harness/mod.rs`
- Create: `conformance/src/harness/msvc.rs`
- Modify: `conformance/src/lib.rs` (add `pub mod harness;` after `pub mod differential;`)
- Test: inline `#[cfg(test)] mod tests` in `msvc.rs`

**Interfaces:**
- Produces: `harness::HarnessError` (enum: `NoToolchain`, `Compile(String)`, `Load(String)`, `Symbol(String)`); `harness::msvc::find_msvc() -> Option<Msvc>`; `harness::msvc::Msvc` (fields `cl: PathBuf`, `include: String`, `lib: String`); `harness::msvc::compile_c_to_dll(msvc: &Msvc, src: &Path, out_dir: &Path) -> Result<PathBuf, HarnessError>`.

- [ ] **Step 1: Create the module root**

Create `conformance/src/harness/mod.rs`:
```rust
//! A live differential-conformance harness (KISS-Conform §6.5 / §6.13-0006).
//!
//! Differences a foreign C op kernel — invoked through the KISS-Contract §6.5
//! positional C-ABI — against the from-scratch [`crate::semantics`] oracle over a
//! deterministic corpus. Increment 1: one elementwise binary op (`add`).

pub mod abi;
pub mod corpus;
pub mod differ;
pub mod loader;
pub mod msvc;

/// Every way the harness can fail *without* a divergence (a divergence is data,
/// not an error). A bad artifact is a typed error, never a panic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HarnessError {
    /// No C toolchain found — the differential slice is skipped, not failed.
    NoToolchain,
    /// `cl.exe` ran but compilation/link failed (captured stderr).
    Compile(String),
    /// The DLL could not be loaded (Win32 last-error rendered).
    Load(String),
    /// The entry symbol was absent from the DLL.
    Symbol(String),
}
```

- [ ] **Step 2: Add the module to the crate**

In `conformance/src/lib.rs`, add after the `pub mod differential;` line (line 32):
```rust
pub mod harness;
```

- [ ] **Step 3: Write the failing test**

Create `conformance/src/harness/msvc.rs` with ONLY this test (no impl yet), so it fails to compile:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn compiles_a_trivial_c_kernel_to_a_dll() {
        let Some(msvc) = find_msvc() else {
            eprintln!("SKIP: no MSVC toolchain found");
            return; // graceful skip, like the cuda feature
        };
        let dir = std::env::temp_dir().join("kiss_harness_msvc_test");
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join("t.c");
        let mut f = std::fs::File::create(&src).unwrap();
        f.write_all(b"__declspec(dllexport) void k(const float* a,const float* b,float* o,long long n){for(long long i=0;i<n;++i)o[i]=a[i]+b[i];}").unwrap();
        drop(f);

        let dll = compile_c_to_dll(&msvc, &src, &dir).expect("compile should succeed");
        assert!(dll.exists(), "expected a .dll at {dll:?}");
    }
}
```

- [ ] **Step 4: Run it, verify it fails to compile**

Run: `cargo test -p kiss-conformance --lib harness::msvc 2>&1 | tail -20`
Expected: FAIL — `cannot find function find_msvc` / `compile_c_to_dll`.

- [ ] **Step 5: Implement discovery + compile**

Prepend to `conformance/src/harness/msvc.rs` (above the test module):
```rust
//! MSVC toolchain discovery + C→DLL compilation, mirroring the `cuda`/`nvcc`
//! runtime-shell-out pattern. Dependency-free: globs the install dirs and calls
//! `cl.exe` directly with explicit INCLUDE/LIB (no `vcvars`, which is slow/blocking).

use super::HarnessError;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A located MSVC toolchain: the `cl.exe` path and the semicolon-joined
/// INCLUDE / LIB search paths a direct (no-`vcvars`) invocation needs.
pub struct Msvc {
    pub cl: PathBuf,
    pub include: String,
    pub lib: String,
}

/// Newest immediate subdirectory of `root` (lexicographically greatest name),
/// or `None` if `root` has no subdirectories.
fn newest_subdir(root: &Path) -> Option<PathBuf> {
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(root)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    dirs.pop()
}

/// Discover an MSVC toolchain by globbing the default VS install roots. Returns
/// `None` if none is present (the differential slice then skips gracefully).
pub fn find_msvc() -> Option<Msvc> {
    // VS install roots to probe (Community/Professional/Enterprise/BuildTools).
    let vs_roots = [
        r"C:\Program Files\Microsoft Visual Studio",
        r"C:\Program Files (x86)\Microsoft Visual Studio",
    ];
    for root in vs_roots {
        let root = Path::new(root);
        if !root.exists() {
            continue;
        }
        // <root>\<edition-year>\<Community|...>\VC\Tools\MSVC\<ver>\bin\Hostx64\x64\cl.exe
        for year in std::fs::read_dir(root).ok()?.filter_map(|e| e.ok()).map(|e| e.path()) {
            for edition in std::fs::read_dir(&year).ok().into_iter().flatten().filter_map(|e| e.ok()).map(|e| e.path()) {
                let msvc_root = edition.join(r"VC\Tools\MSVC");
                let Some(ver) = newest_subdir(&msvc_root) else { continue };
                let cl = ver.join(r"bin\Hostx64\x64\cl.exe");
                if !cl.exists() {
                    continue;
                }
                // Windows SDK (Include/Lib live under Windows Kits\10).
                let kit = Path::new(r"C:\Program Files (x86)\Windows Kits\10");
                let sdk = newest_subdir(&kit.join("Include"))?;
                let sdk_name = sdk.file_name()?.to_string_lossy().into_owned();
                let inc = ver.join("include");
                let s = |p: PathBuf| p.to_string_lossy().into_owned();
                let include = format!(
                    "{};{};{};{}",
                    s(inc),
                    s(kit.join(format!(r"Include\{sdk_name}\ucrt"))),
                    s(kit.join(format!(r"Include\{sdk_name}\um"))),
                    s(kit.join(format!(r"Include\{sdk_name}\shared"))),
                );
                let lib = format!(
                    "{};{};{}",
                    s(ver.join(r"lib\x64")),
                    s(kit.join(format!(r"Lib\{sdk_name}\ucrt\x64"))),
                    s(kit.join(format!(r"Lib\{sdk_name}\um\x64"))),
                );
                return Some(Msvc { cl, include, lib });
            }
        }
    }
    None
}

/// Compile `src` (a C source with a `__declspec(dllexport)` entry) to a DLL in
/// `out_dir`, returning the DLL path. Calls `cl.exe` directly with explicit
/// INCLUDE/LIB — Rust's `Command` passes `/flags` verbatim.
pub fn compile_c_to_dll(msvc: &Msvc, src: &Path, out_dir: &Path) -> Result<PathBuf, HarnessError> {
    let stem = src.file_stem().and_then(|s| s.to_str()).unwrap_or("out");
    let dll = out_dir.join(format!("{stem}.dll"));
    let out = Command::new(&msvc.cl)
        .current_dir(out_dir)
        .env("INCLUDE", &msvc.include)
        .env("LIB", &msvc.lib)
        .args([
            "/nologo",
            "/LD",
            "/O2",
            &src.to_string_lossy(),
            &format!("/Fe:{}", dll.to_string_lossy()),
        ])
        .output()
        .map_err(|e| HarnessError::Compile(format!("spawn cl.exe: {e}")))?;
    if !out.status.success() {
        return Err(HarnessError::Compile(format!(
            "cl.exe failed: {}\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr),
        )));
    }
    Ok(dll)
}
```

- [ ] **Step 6: Run the test, verify it passes**

Run: `cargo test -p kiss-conformance --lib harness::msvc 2>&1 | tail -12`
Expected: PASS (`compiles_a_trivial_c_kernel_to_a_dll ... ok`). If the toolchain is absent it prints SKIP and still passes.

- [ ] **Step 7: Commit**

```bash
cd C:/Projects/kiss-harness
git add conformance/src/harness/mod.rs conformance/src/harness/msvc.rs conformance/src/lib.rs
git commit  # message: "harness: MSVC discovery + compile_c_to_dll (§6.5 fixture toolchain)"
```

---

### Task 2: Raw Win32 FFI loader

**Files:**
- Create: `conformance/src/harness/loader.rs`
- Test: inline `#[cfg(test)] mod tests` in `loader.rs`

**Interfaces:**
- Consumes: `harness::HarnessError`; `harness::msvc::{find_msvc, compile_c_to_dll}`.
- Produces: `harness::loader::Artifact` (owns the module handle; `Drop` calls `FreeLibrary`); `Artifact::load(path: &Path) -> Result<Artifact, HarnessError>`; `Artifact::symbol(&self, name: &str) -> Result<*const core::ffi::c_void, HarnessError>`.

- [ ] **Step 1: Write the failing test**

Create `conformance/src/harness/loader.rs` with ONLY this test:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::msvc;
    use std::io::Write;

    #[test]
    fn loads_a_dll_and_calls_its_export() {
        let Some(m) = msvc::find_msvc() else { eprintln!("SKIP: no MSVC"); return; };
        let dir = std::env::temp_dir().join("kiss_harness_loader_test");
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join("addk.c");
        std::fs::File::create(&src).unwrap().write_all(
            b"__declspec(dllexport) void kiss_add(const float* a,const float* b,float* o,long long n){for(long long i=0;i<n;++i)o[i]=a[i]+b[i];}"
        ).unwrap();
        let dll = msvc::compile_c_to_dll(&m, &src, &dir).unwrap();

        let art = Artifact::load(&dll).expect("load");
        let sym = art.symbol("kiss_add").expect("symbol");
        // SAFETY: the fixture exports exactly this C signature.
        let k: unsafe extern "C" fn(*const f32, *const f32, *mut f32, i64) =
            unsafe { std::mem::transmute(sym) };
        let (a, b) = ([1.0f32, 2.0, 3.0], [10.0f32, 20.0, 30.0]);
        let mut o = [0.0f32; 3];
        unsafe { k(a.as_ptr(), b.as_ptr(), o.as_mut_ptr(), 3) };
        assert_eq!(o, [11.0, 22.0, 33.0]);
    }

    #[test]
    fn missing_symbol_is_a_typed_error_not_a_panic() {
        let Some(m) = msvc::find_msvc() else { eprintln!("SKIP: no MSVC"); return; };
        let dir = std::env::temp_dir().join("kiss_harness_loader_test2");
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join("empty.c");
        std::fs::File::create(&src).unwrap().write_all(
            b"__declspec(dllexport) void present(void){}"
        ).unwrap();
        let dll = msvc::compile_c_to_dll(&m, &src, &dir).unwrap();
        let art = Artifact::load(&dll).unwrap();
        assert_eq!(art.symbol("absent"), Err(HarnessError::Symbol("absent".into())));
    }
}
```

- [ ] **Step 2: Run it, verify it fails**

Run: `cargo test -p kiss-conformance --lib harness::loader 2>&1 | tail -20`
Expected: FAIL — `cannot find type Artifact`.

- [ ] **Step 3: Implement the raw-FFI loader**

Prepend to `conformance/src/harness/loader.rs`:
```rust
//! Raw Win32 dynamic-loading (dependency-free — no `libloading`). All `unsafe`
//! in the harness is confined to this file behind a safe `Artifact` wrapper.

use super::HarnessError;
use core::ffi::c_void;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

// Opaque module handle. `HMODULE` is a pointer-sized handle.
type Hmodule = *mut c_void;

#[link(name = "kernel32")]
extern "system" {
    fn LoadLibraryW(lp_lib_file_name: *const u16) -> Hmodule;
    fn GetProcAddress(h_module: Hmodule, lp_proc_name: *const u8) -> *const c_void;
    fn FreeLibrary(h_module: Hmodule) -> i32;
    fn GetLastError() -> u32;
}

/// A loaded shared library. Frees the module on drop.
pub struct Artifact {
    handle: Hmodule,
}

impl Artifact {
    /// Load a DLL by path. Errors (not panics) if the OS refuses to load it.
    pub fn load(path: &Path) -> Result<Artifact, HarnessError> {
        // LoadLibraryW wants a NUL-terminated UTF-16 string.
        let wide: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
        // SAFETY: `wide` is a valid NUL-terminated UTF-16 buffer that outlives the call.
        let handle = unsafe { LoadLibraryW(wide.as_ptr()) };
        if handle.is_null() {
            // SAFETY: GetLastError reads thread-local error state, always sound.
            let code = unsafe { GetLastError() };
            return Err(HarnessError::Load(format!("LoadLibraryW failed (GetLastError={code}) for {path:?}")));
        }
        Ok(Artifact { handle })
    }

    /// Resolve an exported symbol to an opaque pointer, or `Err(Symbol)` if absent.
    pub fn symbol(&self, name: &str) -> Result<*const c_void, HarnessError> {
        // GetProcAddress wants a NUL-terminated ANSI C string.
        let cname: Vec<u8> = name.bytes().chain(std::iter::once(0)).collect();
        // SAFETY: `self.handle` is a live module; `cname` is NUL-terminated.
        let p = unsafe { GetProcAddress(self.handle, cname.as_ptr()) };
        if p.is_null() {
            return Err(HarnessError::Symbol(name.to_string()));
        }
        Ok(p)
    }
}

impl Drop for Artifact {
    fn drop(&mut self) {
        // SAFETY: `self.handle` was returned by LoadLibraryW and not yet freed.
        unsafe { FreeLibrary(self.handle) };
    }
}
```

- [ ] **Step 4: Run the tests, verify they pass**

Run: `cargo test -p kiss-conformance --lib harness::loader 2>&1 | tail -12`
Expected: PASS (both tests; or SKIP if no toolchain).

- [ ] **Step 5: Commit**

```bash
git add conformance/src/harness/loader.rs
git commit  # message: "harness: raw Win32 FFI DLL loader (unsafe isolated)"
```

---

### Task 3: §6.5 elementwise-binary marshaller

**Files:**
- Create: `conformance/src/harness/abi.rs`
- Test: inline `#[cfg(test)] mod tests` in `abi.rs`

**Interfaces:**
- Consumes: nothing from prior tasks (pure).
- Produces: `harness::abi::BinaryKernel` (type alias `unsafe extern "C" fn(*const f32, *const f32, *mut f32, i64)`); `harness::abi::invoke_binary(kernel: BinaryKernel, a: &[f32], b: &[f32]) -> Vec<f32>`.

**Note on §6.5 fidelity:** For a rank-1, fully-packed elementwise-binary cell, the §6.5 positional signature reduces to the present launch-scalar classes: operand pointers (`in0`, `in1`, then the output) followed by the class-3 iteration count `n` (class-1 extents collapse to `n`; class-2 strides absent because packed; classes 4–8 absent). Multi-rank/strided signatures are a later increment.

- [ ] **Step 1: Write the failing test**

Create `conformance/src/harness/abi.rs` with ONLY this test:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    // A Rust-side kernel with the §6.5 elementwise-binary C signature, used to
    // test the marshaller without FFI.
    unsafe extern "C" fn rust_add(a: *const f32, b: *const f32, o: *mut f32, n: i64) {
        for i in 0..n as isize {
            // SAFETY: the marshaller guarantees a/b have n readable, o has n writable.
            unsafe { *o.offset(i) = *a.offset(i) + *b.offset(i) };
        }
    }

    #[test]
    fn marshals_and_invokes_over_slices() {
        let a = [1.0f32, -2.0, 0.5];
        let b = [10.0f32, 20.0, 0.25];
        let out = invoke_binary(rust_add, &a, &b);
        assert_eq!(out, vec![11.0, 18.0, 0.75]);
    }

    #[test]
    fn empty_input_yields_empty_output() {
        let out = invoke_binary(rust_add, &[], &[]);
        assert!(out.is_empty());
    }
}
```

- [ ] **Step 2: Run it, verify it fails**

Run: `cargo test -p kiss-conformance --lib harness::abi 2>&1 | tail -20`
Expected: FAIL — `cannot find function invoke_binary`.

- [ ] **Step 3: Implement the marshaller**

Prepend to `conformance/src/harness/abi.rs`:
```rust
//! The KISS-Contract §6.5 positional ABI for a rank-1 packed elementwise-binary
//! op: `(in0*, in1*, out*, n)`. The marshaller sizes the output from `n`, lays
//! the operands out in canonical order, and calls the entry point.

/// A §6.5 elementwise-binary kernel entry point (C ABI).
pub type BinaryKernel = unsafe extern "C" fn(*const f32, *const f32, *mut f32, i64);

/// Invoke `kernel` over equal-length inputs `a`, `b`; returns the `n`-element
/// output. Panics only on the caller's contract violation (mismatched lengths).
pub fn invoke_binary(kernel: BinaryKernel, a: &[f32], b: &[f32]) -> Vec<f32> {
    assert_eq!(a.len(), b.len(), "elementwise-binary inputs must be equal length");
    let n = a.len();
    let mut out = vec![0.0f32; n];
    // SAFETY: `a` and `b` each expose `n` readable f32; `out` exposes `n`
    // writable f32; the pointers are non-overlapping (distinct allocations);
    // `n` fits i64. The kernel writes exactly `out[0..n]`.
    unsafe { kernel(a.as_ptr(), b.as_ptr(), out.as_mut_ptr(), n as i64) };
    out
}
```

- [ ] **Step 4: Run the tests, verify they pass**

Run: `cargo test -p kiss-conformance --lib harness::abi 2>&1 | tail -12`
Expected: PASS (both tests).

- [ ] **Step 5: Commit**

```bash
git add conformance/src/harness/abi.rs
git commit  # message: "harness: §6.5 elementwise-binary marshaller (invoke_binary)"
```

---

### Task 4: C fixtures + smoke test

**Files:**
- Create: `conformance/tests/harness_fixtures/add_a.c`
- Create: `conformance/tests/harness_fixtures/add_b.c`
- Create: `conformance/tests/harness_fixtures/add_wrong.c`
- Create: `conformance/tests/harness_smoke.rs`

**Interfaces:**
- Consumes: `kiss_conformance::harness::msvc::{find_msvc, compile_c_to_dll}`, `harness::loader::Artifact`, `harness::abi::{BinaryKernel, invoke_binary}`.
- Produces: (test-only) a reusable helper `compile_and_load(name) -> Option<(Artifact, BinaryKernel)>` local to the test crate.

- [ ] **Step 1: Write the three fixtures**

Create `conformance/tests/harness_fixtures/add_a.c`:
```c
/* Correct elementwise add, straightforward loop. Exposes the §6.5 rank-1
   packed elementwise-binary entry: (in0, in1, out, n). */
__declspec(dllexport) void kiss_add(const float* in0, const float* in1, float* out, long long n) {
    for (long long i = 0; i < n; ++i) out[i] = in0[i] + in1[i];
}
```

Create `conformance/tests/harness_fixtures/add_b.c` (independent source, bit-identical result — IEEE add is commutative and exact):
```c
/* Correct elementwise add, independent implementation: pointer-walk + commuted
   operand order. `b + a == a + b` bit-for-bit in IEEE-754. */
__declspec(dllexport) void kiss_add(const float* in0, const float* in1, float* out, long long n) {
    const float* p = in0; const float* q = in1; float* r = out;
    for (long long i = 0; i < n; ++i) { *r = *q + *p; ++p; ++q; ++r; }
}
```

Create `conformance/tests/harness_fixtures/add_wrong.c` (deliberately wrong — subtraction):
```c
/* WRONG on purpose: computes a - b. The harness must catch this. */
__declspec(dllexport) void kiss_add(const float* in0, const float* in1, float* out, long long n) {
    for (long long i = 0; i < n; ++i) out[i] = in0[i] - in1[i];
}
```

- [ ] **Step 2: Write the smoke test**

Create `conformance/tests/harness_smoke.rs`:
```rust
//! Each C fixture compiles, loads, and computes on a trivial input.

use kiss_conformance::harness::abi::{invoke_binary, BinaryKernel};
use kiss_conformance::harness::loader::Artifact;
use kiss_conformance::harness::msvc;
use std::path::PathBuf;

/// Compile `tests/harness_fixtures/<name>.c` to a DLL and resolve `kiss_add`.
/// Returns `None` (skip) if no toolchain. Leaks the `Artifact` so the resolved
/// fn pointer stays valid for the test's lifetime.
fn compile_and_load(name: &str) -> Option<BinaryKernel> {
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

#[test]
fn all_three_fixtures_compile_load_and_run() {
    let Some(add_a) = compile_and_load("add_a") else { eprintln!("SKIP: no MSVC"); return; };
    let add_b = compile_and_load("add_b").unwrap();
    let add_wrong = compile_and_load("add_wrong").unwrap();

    let (a, b) = ([2.0f32, 5.0], [3.0f32, 1.0]);
    assert_eq!(invoke_binary(add_a, &a, &b), vec![5.0, 6.0]);
    assert_eq!(invoke_binary(add_b, &a, &b), vec![5.0, 6.0]);
    assert_eq!(invoke_binary(add_wrong, &a, &b), vec![-1.0, 4.0]); // a - b
}
```

- [ ] **Step 3: Run the smoke test, verify it passes**

Run: `cargo test -p kiss-conformance --test harness_smoke 2>&1 | tail -12`
Expected: PASS (or SKIP if no toolchain). Confirms fixtures + toolchain + loader + marshaller compose.

- [ ] **Step 4: Commit**

```bash
git add conformance/tests/harness_fixtures conformance/tests/harness_smoke.rs
git commit  # message: "harness: C add fixtures (2 correct + 1 wrong) + smoke test"
```

---

### Task 5: Provenance-tagged corpus + class-aware differ

**Files:**
- Create: `conformance/src/harness/corpus.rs`
- Create: `conformance/src/harness/differ.rs`
- Test: inline `#[cfg(test)] mod tests` in each

**Interfaces:**
- Consumes: `crate::differential::{edge_f32, SplitMix64}`, `crate::semantics::add`.
- Produces:
  - `harness::corpus::Vector` (`{ a: f32, b: f32, expected: f32, provenance: &'static str }`);
  - `harness::corpus::tagged_corpus(seed: u64, n: usize) -> Vec<Vector>`;
  - `harness::differ::Divergence` (`{ index: usize, a: f32, b: f32, expected: f32, actual: f32 }`);
  - `harness::differ::run_binary(outputs: &[f32], corpus: &[Vector]) -> Vec<Divergence>`.

- [ ] **Step 1: Write the failing corpus test**

Create `conformance/src/harness/corpus.rs` with ONLY this test:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_vector_is_tagged_and_expected_matches_the_oracle() {
        let c = tagged_corpus(0xC0FFEE, 64);
        assert!(c.len() >= 64, "edge cases + 64 random");
        for v in &c {
            assert!(!v.provenance.is_empty(), "vector missing provenance tag (§6.5-0003)");
            // The expected value is the oracle's, bit-for-bit (NaN-relaxed).
            let o = crate::semantics::add(v.a, v.b);
            assert!(o.to_bits() == v.expected.to_bits() || (o.is_nan() && v.expected.is_nan()));
        }
    }

    #[test]
    fn corpus_is_deterministic() {
        assert_eq!(
            tagged_corpus(7, 32).iter().map(|v| (v.a.to_bits(), v.b.to_bits())).collect::<Vec<_>>(),
            tagged_corpus(7, 32).iter().map(|v| (v.a.to_bits(), v.b.to_bits())).collect::<Vec<_>>(),
        );
    }
}
```

- [ ] **Step 2: Run it, verify it fails**

Run: `cargo test -p kiss-conformance --lib harness::corpus 2>&1 | tail -20`
Expected: FAIL — `cannot find function tagged_corpus`.

- [ ] **Step 3: Implement the tagged corpus**

Prepend to `conformance/src/harness/corpus.rs`:
```rust
//! A deterministic, provenance-tagged corpus of `add` invocations. Each vector
//! carries a tag naming the source of its expected value (KISS-CONFORM-6.5-0003)
//! — here, the from-scratch oracle. Reuses the `differential` edge set + PRNG.

use crate::differential::{edge_f32, SplitMix64};

/// One conformance vector: inputs, the oracle's expected output, and a
/// derivation-provenance tag (§6.5-0003).
#[derive(Debug, Clone, Copy)]
pub struct Vector {
    pub a: f32,
    pub b: f32,
    pub expected: f32,
    pub provenance: &'static str,
}

/// The derivation source tag for increment 1: the `add` reference decomposition
/// is the primitive-floor op itself, evaluated by `semantics::add`.
const PROVENANCE: &str = "oracle:KISS-OPS-6.4-0001/semantics::add";

/// The edge f32 × edge f32 grid, then `n` seeded-random pairs. Deterministic:
/// same seed → same vectors.
pub fn tagged_corpus(seed: u64, n: usize) -> Vec<Vector> {
    let make = |a: f32, b: f32| Vector { a, b, expected: crate::semantics::add(a, b), provenance: PROVENANCE };
    let edges = edge_f32();
    let mut v = Vec::new();
    for &a in &edges {
        for &b in &edges {
            v.push(make(a, b));
        }
    }
    let mut rng = SplitMix64::new(seed);
    for _ in 0..n {
        let a = f32::from_bits(rng.next_u64() as u32);
        let b = f32::from_bits(rng.next_u64() as u32);
        v.push(make(a, b));
    }
    v
}
```

- [ ] **Step 4: Run the corpus tests, verify they pass**

Run: `cargo test -p kiss-conformance --lib harness::corpus 2>&1 | tail -12`
Expected: PASS (both tests).

- [ ] **Step 5: Write the failing differ test**

Create `conformance/src/harness/differ.rs` with ONLY this test:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::corpus::tagged_corpus;

    #[test]
    fn identical_outputs_have_no_divergences() {
        let c = tagged_corpus(1, 16);
        let outs: Vec<f32> = c.iter().map(|v| v.expected).collect();
        assert!(run_binary(&outs, &c).is_empty());
    }

    #[test]
    fn a_single_wrong_output_is_caught_with_its_index() {
        let c = tagged_corpus(1, 16);
        let mut outs: Vec<f32> = c.iter().map(|v| v.expected).collect();
        outs[3] = outs[3] + 1.0; // perturb one (finite edge cases guarantee this differs)
        let d = run_binary(&outs, &c);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].index, 3);
    }
}
```

- [ ] **Step 6: Run it, verify it fails**

Run: `cargo test -p kiss-conformance --lib harness::differ 2>&1 | tail -20`
Expected: FAIL — `cannot find function run_binary`.

- [ ] **Step 7: Implement the differ**

Prepend to `conformance/src/harness/differ.rs`:
```rust
//! Class-aware comparison of a candidate's outputs against the oracle-tagged
//! corpus. For `add` (a bit-stable exact-byte op) the comparator is the
//! NaN-relaxed exact-byte `agree` from `differential`. Each divergence is data.

use crate::differential::agree;
use crate::harness::corpus::Vector;

/// One caught divergence, reproducible by `index` into the corpus.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Divergence {
    pub index: usize,
    pub a: f32,
    pub b: f32,
    pub expected: f32,
    pub actual: f32,
}

/// Difference a candidate's `outputs` (aligned 1:1 with `corpus`) against each
/// vector's oracle `expected`. Returns every divergence (empty ⇒ conformant).
pub fn run_binary(outputs: &[f32], corpus: &[Vector]) -> Vec<Divergence> {
    assert_eq!(outputs.len(), corpus.len(), "one output per corpus vector");
    let mut out = Vec::new();
    for (i, (v, &actual)) in corpus.iter().zip(outputs).enumerate() {
        if !agree(v.expected, actual) {
            out.push(Divergence { index: i, a: v.a, b: v.b, expected: v.expected, actual });
        }
    }
    out
}
```

- [ ] **Step 8: Run the differ tests, verify they pass**

Run: `cargo test -p kiss-conformance --lib harness::differ 2>&1 | tail -12`
Expected: PASS (both tests).

- [ ] **Step 9: Commit**

```bash
git add conformance/src/harness/corpus.rs conformance/src/harness/differ.rs
git commit  # message: "harness: provenance-tagged corpus (§6.5-0003) + class-aware differ"
```

---

### Task 6: The freeze-gate integration test (the deliverable)

**Files:**
- Create: `conformance/tests/harness_differential.rs`

**Interfaces:**
- Consumes: `harness::{msvc, loader::Artifact, abi::{invoke_binary, BinaryKernel}, corpus::tagged_corpus, differ::run_binary}`.
- Produces: nothing (top-level integration test).

- [ ] **Step 1: Write the freeze-gate test**

Create `conformance/tests/harness_differential.rs`:
```rust
//! KISS-CONFORM-6.13-0006 (increment 1): the differential harness differences a
//! foreign C `add` kernel against the from-scratch oracle, and TWO dissimilar
//! correct kernels agree on floor semantics while a wrong one is CAUGHT.
//!
//! Scope: `add` is a primitive-floor op, so this exercises the ≥2-dissimilar-
//! impls-agree + harness-catches-divergence obligations of 6.13-0006. The
//! decomposition-resolution obligation (a non-primitive op → floor) is a later
//! increment and is NOT claimed here.

use kiss_conformance::harness::abi::{invoke_binary, BinaryKernel};
use kiss_conformance::harness::corpus::tagged_corpus;
use kiss_conformance::harness::differ::run_binary;
use kiss_conformance::harness::loader::Artifact;
use kiss_conformance::harness::msvc;
use std::path::PathBuf;

fn compile_and_load(name: &str) -> Option<BinaryKernel> {
    let m = msvc::find_msvc()?;
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/harness_fixtures")
        .join(format!("{name}.c"));
    let out = std::env::temp_dir().join(format!("kiss_harness_diff_{name}"));
    std::fs::create_dir_all(&out).unwrap();
    let dll = msvc::compile_c_to_dll(&m, &src, &out).expect("compile fixture");
    let art = Box::leak(Box::new(Artifact::load(&dll).expect("load fixture")));
    let sym = art.symbol("kiss_add").expect("kiss_add export");
    // SAFETY: each fixture exports exactly the BinaryKernel C signature.
    Some(unsafe { std::mem::transmute::<*const core::ffi::c_void, BinaryKernel>(sym) })
}

/// Run a kernel over the corpus and return its outputs (1:1 with the corpus).
fn outputs_of(k: BinaryKernel, corpus: &[kiss_conformance::harness::corpus::Vector]) -> Vec<f32> {
    let a: Vec<f32> = corpus.iter().map(|v| v.a).collect();
    let b: Vec<f32> = corpus.iter().map(|v| v.b).collect();
    invoke_binary(k, &a, &b)
}

#[test]
fn test_conform_ops_oracle_and_freeze_gate() {
    // KISS-CONFORM-6.13-0006 — the differential harness + ≥2-dissimilar-impls gate.
    let Some(add_a) = compile_and_load("add_a") else {
        eprintln!("SKIP: no MSVC toolchain — the C differential slice needs cl.exe");
        return;
    };
    let add_b = compile_and_load("add_b").unwrap();
    let add_wrong = compile_and_load("add_wrong").unwrap();

    let corpus = tagged_corpus(0x5EED_1234, 256);
    let out_a = outputs_of(add_a, &corpus);
    let out_b = outputs_of(add_b, &corpus);
    let out_w = outputs_of(add_wrong, &corpus);

    // (1) Both dissimilar correct impls agree with the oracle: zero divergences.
    assert!(run_binary(&out_a, &corpus).is_empty(),
        "KISS-CONFORM-6.13-0006: add_a diverged from the oracle");
    assert!(run_binary(&out_b, &corpus).is_empty(),
        "KISS-CONFORM-6.13-0006: add_b diverged from the oracle");

    // (2) The two dissimilar impls agree with EACH OTHER, bit-for-bit (NaN-relaxed).
    for (i, (&x, &y)) in out_a.iter().zip(&out_b).enumerate() {
        assert!(x.to_bits() == y.to_bits() || (x.is_nan() && y.is_nan()),
            "KISS-CONFORM-6.13-0006: the two dissimilar impls disagree at index {i}");
    }

    // (3) TEETH: a wrong kernel is CAUGHT — ≥1 divergence, reproducible by index.
    let caught = run_binary(&out_w, &corpus);
    assert!(!caught.is_empty(),
        "KISS-CONFORM-6.13-0006: the harness FAILED to catch a wrong kernel");
    let d = caught[0];
    // Re-running the single caught invocation reproduces the divergence.
    let repro = invoke_binary(add_wrong, &[d.a], &[d.b]);
    assert_eq!(repro[0].to_bits(), d.actual.to_bits(),
        "a caught divergence must reproduce from its recorded inputs");
}
```

- [ ] **Step 2: Run the freeze-gate test, verify it passes**

Run: `cargo test -p kiss-conformance --test harness_differential 2>&1 | tail -15`
Expected: PASS (`test_conform_ops_oracle_and_freeze_gate ... ok`), or SKIP if no toolchain.

- [ ] **Step 3: Run the whole suite to confirm no regression**

Run: `cd C:/Projects/kiss-harness/conformance && cargo test 2>&1 | grep -E 'test result|error|warning:' | tail -20`
Expected: all suites `ok`, no warnings.

- [ ] **Step 4: Commit**

```bash
git add conformance/tests/harness_differential.rs
git commit  # message: "harness: freeze-gate differential test — 2 impls agree, wrong one caught (6.13-0006 inc.1)"
```

---

### Task 7: Honest ledger reconciliation

**Files:**
- Modify: `conformance/UNBACKED.tsv` (regenerated)
- Possibly modify: `spec/conform.md` (only if a `*Test:*` tag must be aligned — see step 2)

**Interfaces:** none (bookkeeping).

- [ ] **Step 1: Regenerate the ledger and inspect what dropped**

Run:
```bash
cd C:/Projects/kiss-harness
python tools/kiss_trace.py --update-ledger
git diff conformance/UNBACKED.tsv | grep '^-KISS' || echo "no clauses newly backed"
```

- [ ] **Step 2: Decide honestly what increment 1 backs**

The test fn is named `test_conform_ops_oracle_and_freeze_gate` (6.13-0006's `*Test:*` tag) and cites `KISS-CONFORM-6.13-0006`. **Honesty check:** 6.13-0006 also requires *resolving a non-primitive op's decomposition to the floor*. Increment 1 differences only the primitive `add`, so that obligation is unexercised.
  - If the ledger now shows `6.13-0006` as backed: **this is a partial/over-credit** unless you judge the ≥2-impls-agree core sufficient. Per the burndown honesty bar, prefer NOT to claim it yet — rename the test's cited anchor to a comment (keep the fn name for future binding) OR add a doc-comment stating the decomposition obligation is deferred, and revert `6.13-0006` to untested in the ledger.
  - The provenance-tag obligation (`KISS-CONFORM-6.5-0003`) IS fully satisfied by `tagged_corpus`; if a `*Test:*`-named binding for it is cheap and honest, keep that one.
  - Record the decision in the commit message. When in doubt, under-claim (the deliverable is the working tool, not the clause count).

- [ ] **Step 3: Regenerate + verify the ledger matches the decision**

Run:
```bash
python tools/kiss_trace.py --update-ledger
python tools/kiss_trace.py --report 2>&1 | grep -i conform
```
Expected: the ledger reflects exactly what was decided in Step 2 (no over-claim).

- [ ] **Step 4: Commit**

```bash
git add conformance/UNBACKED.tsv spec/conform.md 2>/dev/null; git add conformance/UNBACKED.tsv
git commit  # message: "harness: honest ledger reconciliation for increment 1 (capability first; no over-claim)"
```

---

## Self-Review

**1. Spec coverage** (against `2026-07-29-differential-conformance-harness-design.md`):
- §3 built pieces — `abi` (Task 3), `loader` (Task 2), `differ` (Task 5), `runner`/verdict (the `run_binary` + integration test, Tasks 5–6). ✓
- §4 components — oracle (reused, Task 5), corpus w/ provenance (Task 5), abi (Task 3), loader (Task 2), differ (Task 5), fixtures (Task 4). ✓
- §5 error handling — `HarnessError` typed, no panic on bad artifact (Tasks 1–2 tests), divergence-is-data (Task 5). ✓
- §6 teeth — agreement (Task 6 assert 1–2) + catch-the-wrong-one (Task 6 assert 3). ✓
- §7 honest clause scope — Task 7 explicitly guards against over-claiming 6.13-0006's decomposition obligation. ✓
- §2 decision "two C artifacts" — Task 4 (`add_a.c`, `add_b.c` independent) + wrong (`add_wrong.c`). ✓
- Global constraint "stdlib only / no build.rs" — raw FFI (Task 2), runtime `cl.exe` shell-out (Task 1), no `Cargo.toml` dependency edits anywhere. ✓

**2. Placeholder scan:** No TBD/TODO; every code step has complete code; commands have expected output. ✓

**3. Type consistency:** `BinaryKernel = unsafe extern "C" fn(*const f32, *const f32, *mut f32, i64)` is defined in Task 3 and consumed identically in Tasks 4 and 6. `Vector { a, b, expected, provenance }` defined in Task 5 corpus, consumed in Task 5 differ and Task 6. `Divergence { index, a, b, expected, actual }` defined + consumed in Task 5, consumed in Task 6. `Artifact::{load, symbol}`, `msvc::{find_msvc, compile_c_to_dll}`, `Msvc { cl, include, lib }`, `HarnessError::{NoToolchain, Compile, Load, Symbol}` consistent across Tasks 1–6. ✓

*Note for the implementer:* every test skips gracefully (`return`) when `find_msvc()` is `None`, so `cargo test` is green with or without the toolchain — but on this host the toolchain is present, so the C differential slice actually runs.

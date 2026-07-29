//! MSVC toolchain discovery + C→DLL compilation, mirroring the `cuda`/`nvcc`
//! runtime-shell-out pattern. Dependency-free: globs the install dirs and calls
//! `cl.exe` directly with explicit INCLUDE/LIB (no `vcvars`, which is slow/blocking).

use super::HarnessError;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A located MSVC toolchain: the `cl.exe` path and the semicolon-joined
/// INCLUDE / LIB search paths a direct (no-`vcvars`) invocation needs.
#[derive(Debug)]
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
        for year in std::fs::read_dir(root).ok().into_iter().flatten().filter_map(|e| e.ok()).map(|e| e.path()) {
            for edition in std::fs::read_dir(&year).ok().into_iter().flatten().filter_map(|e| e.ok()).map(|e| e.path()) {
                let msvc_root = edition.join(r"VC\Tools\MSVC");
                let Some(ver) = newest_subdir(&msvc_root) else { continue };
                let cl = ver.join(r"bin\Hostx64\x64\cl.exe");
                if !cl.exists() {
                    continue;
                }
                // Windows SDK (Include/Lib live under Windows Kits\10). Usually
                // under Program Files (x86), but not on every host/image — probe
                // both roots. A missing kit/subdir means this candidate can't be
                // completed; try the next edition/root rather than aborting.
                let kit_roots = [
                    Path::new(r"C:\Program Files (x86)\Windows Kits\10"),
                    Path::new(r"C:\Program Files\Windows Kits\10"),
                ];
                let Some((kit, sdk)) = kit_roots
                    .iter()
                    .find_map(|k| newest_subdir(&k.join("Include")).map(|sdk| (*k, sdk)))
                else {
                    continue;
                };
                let Some(sdk_name) = sdk.file_name() else { continue };
                let sdk_name = sdk_name.to_string_lossy().into_owned();
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

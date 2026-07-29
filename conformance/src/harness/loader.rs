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

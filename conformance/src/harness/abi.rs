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

/// A §6.5 rank-1 full-reduction kernel entry point (C ABI): reads `n` inputs,
/// writes one scalar output.
pub type ReduceKernel = unsafe extern "C" fn(*const f32, *mut f32, i64);

/// Invoke a reduction `kernel` over `xs`; returns the scalar output. `xs` must be
/// non-empty (a mean over zero elements is undefined — the caller's contract).
pub fn invoke_reduce(kernel: ReduceKernel, xs: &[f32]) -> f32 {
    assert!(!xs.is_empty(), "reduce over an empty slice is undefined");
    let mut out = 0.0f32;
    // SAFETY: `xs` exposes `xs.len()` readable f32; `out` is one writable f32; the
    // pointers are non-overlapping (distinct allocations); len fits i64. The kernel
    // writes exactly `out`.
    unsafe { kernel(xs.as_ptr(), &mut out as *mut f32, xs.len() as i64) };
    out
}

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

    unsafe extern "C" fn rust_reduce_mean(x: *const f32, o: *mut f32, n: i64) {
        let mut s = 0.0f32;
        for i in 0..n as isize {
            // SAFETY: the marshaller guarantees x has n readable f32.
            unsafe { s += *x.offset(i) };
        }
        // SAFETY: o points to one writable f32.
        unsafe { *o = s / n as f32 };
    }

    #[test]
    fn marshals_and_invokes_a_reduction() {
        let xs = [1.0f32, 2.0, 3.0, 4.0];
        assert_eq!(invoke_reduce(rust_reduce_mean, &xs), 2.5);
    }
}

#![cfg(windows)]
//! Each reduce fixture compiles, loads, and computes on a trivial input.
mod common;
use common::compile_and_load_reduce;
use kiss_conformance::harness::abi::invoke_reduce;

#[test]
fn reduce_fixtures_compile_load_and_run() {
    let Some(a) = compile_and_load_reduce("mean_a") else { eprintln!("SKIP: no MSVC"); return; };
    let b = compile_and_load_reduce("mean_b").unwrap();
    let w = compile_and_load_reduce("mean_wrong").unwrap();
    let xs = [2.0f32, 4.0, 6.0, 8.0]; // mean 5.0; wrong (÷3) = 6.666...
    assert_eq!(invoke_reduce(a, &xs), 5.0);
    assert_eq!(invoke_reduce(b, &xs), 5.0);
    assert!((invoke_reduce(w, &xs) - 20.0 / 3.0).abs() < 1e-4);
}

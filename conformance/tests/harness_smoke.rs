#![cfg(windows)]
//! Each C fixture compiles, loads, and computes on a trivial input.

mod common;
use common::compile_and_load;
use kiss_conformance::harness::abi::invoke_binary;

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

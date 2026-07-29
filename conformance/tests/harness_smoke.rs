// Smoke test for the differential-conformance harness: compile, load, and run
// three C fixtures (two correct, one deliberately wrong) on trivial input.

mod common;

use kiss_conformance::harness::abi::invoke_binary;

#[test]
fn smoke_test_fixtures() {
    // Compile and load the three fixtures.
    let add_a = common::compile_and_load("add_a");
    let add_b = common::compile_and_load("add_b");
    let add_wrong = common::compile_and_load("add_wrong");

    if add_a.is_none() || add_b.is_none() || add_wrong.is_none() {
        eprintln!("SKIP: MSVC toolchain or compilation failed");
        return;
    }

    // Trivial test input.
    let a = [1.0f32, 2.0, 3.0];
    let b = [10.0f32, 20.0, 30.0];

    // Expected results for add (a + b).
    let expected_add = [11.0f32, 22.0, 33.0];

    // Expected results for subtract (a - b).
    let expected_subtract = [-9.0f32, -18.0, -27.0];

    // Run both correct implementations and verify they give identical results.
    let result_a = invoke_binary(add_a.unwrap(), &a, &b);
    let result_b = invoke_binary(add_b.unwrap(), &a, &b);
    assert_eq!(result_a, expected_add, "add_a result mismatch");
    assert_eq!(result_b, expected_add, "add_b result mismatch");
    assert_eq!(result_a, result_b, "add_a and add_b should give identical results");

    // Run the wrong implementation and verify it gives a - b.
    let result_wrong = invoke_binary(add_wrong.unwrap(), &a, &b);
    assert_eq!(
        result_wrong, expected_subtract,
        "add_wrong should compute a - b, not a + b"
    );
}

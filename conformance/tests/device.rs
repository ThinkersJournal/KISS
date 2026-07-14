//! KISS-Conform §6.6 — ON-DEVICE real-kernel differential slice (integration).
//!
//! Non-normative. This test carries the Conform §6.5 differential oracle onto real
//! hardware: it compiles `cuda/fmax_ieee.cu` with `nvcc` and runs it on the local
//! GPU, where a hand-written kernel implementing the KISS-Ops §6.15 `fmax_ieee`
//! (NaN-**suppressing** maximum, IEEE-754 maxNum) is differenced against a
//! from-scratch CPU reference decomposition over the SAME corpus the Rust oracle
//! uses (`differential::corpus_f32` — the 16 edge cases + the splitmix64 stream at
//! seed `0x1234_5678`, 4096 draws). The device program exits 0 (PASS), 1 (a
//! divergence — FAIL), or 2 (a CUDA/environment error, so a real semantics
//! divergence is never confused with a broken toolchain). Where this and a spec
//! clause disagree, the clause governs (KISS-Conform §0).
//!
//! Gating — this test costs nothing on a machine without CUDA and never runs on the
//! default `cargo test`:
//!   * **Compile-time**: the whole file is behind `#![cfg(feature = "cuda")]`, so
//!     `cargo test` (no feature) does not build it. Opt in with
//!     `cargo test --features cuda`.
//!   * **Run-time**: even with the feature on, if `nvcc` is not on `PATH` each test
//!     prints a skip notice and returns success — CI without a GPU stays green.
//!
//! What it proves (Conform §6.5: the point is that a *wrong* impl is caught, not
//! merely that a correct one passes):
//!   * The correct kernel matches the oracle bit-exactly over all `4112²`
//!     (= 16,908,544) ordered pairs, INCLUDING the two NaN corpus entries
//!     (suppression: `fmax_ieee(NaN,b)=b`, opposite of `max_prop`) and the `{±0}`
//!     pair whose result sign §6.15 pins order-dependently (`cmp_ge(+0,-0)` is
//!     TRUE, so the tie returns the FIRST operand: `fmax_ieee(+0,-0)=+0`,
//!     `fmax_ieee(-0,+0)=-0`).
//!   * A **negative control** built with `-DUSE_FMAXF_INTRINSIC` (which lowers to
//!     CUDA's `fmaxf`, which scrubs the ±0 sign — returning `+0` for the `{-0,+0}`
//!     pair regardless of operand order) MUST diverge — asserting the on-device
//!     slice has teeth, not just green paint.

#![cfg(feature = "cuda")]

use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;

/// The exit code the device program returns for a CUDA API / launch / environment
/// error (as opposed to a semantics divergence). Kept distinct so a toolchain
/// failure is never misread as a spec violation.
const EXIT_ENV_ERROR: i32 = 2;

/// Serializes the build+run across the two `#[test]` functions. cargo runs tests in
/// one binary on parallel threads; the drivers compile to a single temp exe whose
/// name is only clock-random (cmd's `%RANDOM%` is seeded from the wall clock, so two
/// near-simultaneous invocations collide → an `LNK1104: cannot open file`), and two
/// concurrent GPU launches needlessly contend. Taking this lock for the whole
/// compile→run→cleanup window makes each slice a clean, sequential on-device run.
static ONDEVICE_LOCK: Mutex<()> = Mutex::new(());

/// `cuda/` alongside this crate's `Cargo.toml`.
fn cuda_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("cuda")
}

/// True iff `nvcc` answers `--version` — the run-time gate. The Conform harness
/// stays dependency-free (Conform §6.5): we *detect* the toolchain and shell out,
/// we never link a CUDA crate.
fn nvcc_present() -> bool {
    Command::new("nvcc")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Build+run the on-device program via the platform driver script, forwarding any
/// extra args straight to `nvcc`. Returns `(exit_code, combined stdout+stderr)`.
fn run_ondevice(extra_nvcc_args: &[&str]) -> (i32, String) {
    // Serialize the whole compile→run→cleanup window (see ONDEVICE_LOCK). Recover a
    // poisoned lock: a panic in the *other* test already fails that test, and this
    // one still wants its own clean sequential run.
    let _guard = ONDEVICE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let dir = cuda_dir();
    let mut cmd;
    if cfg!(windows) {
        // cmd /c build_and_run.bat [args...] — the .bat pins a CUDA-13.3-compatible
        // MSVC host toolset (the newest STL is rejected by the CUDA headers) then
        // invokes nvcc.
        cmd = Command::new("cmd");
        cmd.arg("/c").arg(dir.join("build_and_run.bat"));
    } else {
        cmd = Command::new("bash");
        cmd.arg(dir.join("build_and_run.sh"));
    }
    for a in extra_nvcc_args {
        cmd.arg(a);
    }
    cmd.current_dir(&dir);
    let out = cmd.output().expect("failed to spawn on-device build+run driver");
    let mut s = String::from_utf8_lossy(&out.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.code().unwrap_or(-1), s)
}

/// The kernel matches the from-scratch oracle on real hardware over the full corpus
/// of ordered pairs, bit-exactly (KISS-Ops §6.15 `fmax_ieee`; Conform §6.5/§6.6).
#[test]
fn fmax_ieee_matches_oracle_on_device() {
    if !nvcc_present() {
        eprintln!("SKIP: nvcc not on PATH — the on-device §6.6 slice needs a CUDA toolkit");
        return;
    }
    let (code, log) = run_ondevice(&[]);
    assert_ne!(
        code, EXIT_ENV_ERROR,
        "on-device slice hit a CUDA/environment error (exit 2), not a semantics result:\n{log}"
    );
    assert!(
        code == 0 && log.contains("PASS"),
        "on-device fmax_ieee kernel diverged from the §6.15 oracle (exit {code}):\n{log}"
    );
}

/// Negative control: a kernel lowered to CUDA's `fmaxf` (which does not preserve the
/// order-dependent sign of ±0 that §6.15 pins) MUST be caught. This asserts the
/// differential slice has teeth (Conform §6.5), not merely that the correct kernel
/// happens to pass.
#[test]
fn fmaxf_intrinsic_is_caught_on_device() {
    if !nvcc_present() {
        eprintln!("SKIP: nvcc not on PATH — the on-device §6.6 slice needs a CUDA toolkit");
        return;
    }
    let (code, log) = run_ondevice(&["-DUSE_FMAXF_INTRINSIC"]);
    assert_ne!(
        code, EXIT_ENV_ERROR,
        "negative control hit a CUDA/environment error (exit 2):\n{log}"
    );
    assert!(
        code == 1 && log.contains("FAIL"),
        "negative control (fmaxf) was NOT caught — the on-device slice lacks teeth (exit {code}):\n{log}"
    );
}

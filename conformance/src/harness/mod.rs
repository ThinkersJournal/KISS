//! A live differential-conformance harness (KISS-Conform §6.5 / §6.13-0006).
//!
//! Differences a foreign C op kernel — invoked through the KISS-Contract §6.5
//! positional C-ABI — against the from-scratch [`crate::semantics`] oracle over a
//! deterministic corpus. Increment 1: one elementwise binary op (`add`).

pub mod abi;
pub mod corpus;
pub mod differ;
#[cfg(windows)]
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

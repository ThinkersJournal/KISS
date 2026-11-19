kind: cargo-deny in CI
---
Added `conformance/deny.toml` (copied from fishstringer; `all-features = true`) and a `cargo-deny` job in `.github/workflows/conformance.yml` (pm-kernels TASK). `cargo deny --manifest-path conformance/Cargo.toml check` run locally at origin/main 4ac8803: advisories ok, bans ok, licenses ok, sources ok (license-not-encountered warnings only). No spec, corpus or wire byte moves.

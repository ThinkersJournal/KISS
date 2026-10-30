# Security policy

## Supported versions

The latest release line receives security fixes. An older line receives a backport when the backport is small. When it is not, we do not attempt it: the advisory says that line is not fixed and recommends upgrading. No support period is promised.

### Which spec versions kiss-ref validates

The specification itself (`spec/`) is a pre-freeze draft and carries no support promise. `kiss-ref` validates against the KISS conformance corpus it vendors, recorded file by file in its `crates/kiss-ref-conformance/fixtures/kiss-corpus/PROVENANCE.md`. Measured at `kiss-ref` `origin/main` (0.4.6, 2026-10-29): the corpus copied from KISS 0.2.14 (`structure_key` sk5, `contract_version` 3), plus the integer, bool, atan2, sqrt and nextafter bundles added through KISS 0.2.26. It does not yet validate the current line (`contract_version` 4, sk6).

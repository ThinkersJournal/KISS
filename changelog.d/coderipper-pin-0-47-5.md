kind: CI: CodeRipper portfolio workflow pinned to 0.47.5
---
`.github/workflows/conformance.yml` calls the shared `portfolio.yml` at coderipper `c72a6e03479d914de8ea8a73609ba73e93a0de92` (0.47.5, was `d14a22b6` 0.9.14). The repo carries no `.coderipper.toml` and no `check <rule>` or `--skip-rule` text, so `migrate-rule-ids.py` has nothing to rewrite here. The job stays advisory, not a required check.

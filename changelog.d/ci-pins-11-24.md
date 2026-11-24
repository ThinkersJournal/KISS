kind: CI pins: portfolio.yml and cargo-deny-action
---
`.github/workflows/conformance.yml`: the CodeRipper `portfolio.yml` pin moves from `c72a6e03` to `dc45e3632ab9f471292458e5ccba5f375084a70a` (installs CodeRipper 0.47.15; coderipper main is 0.47.21), and `EmbarkStudios/cargo-deny-action` moves from 2.1.1 to 2.1.3 (`17aeb3c839e42274e2b8adf721c8c65edfdc74dd`, folded from dependabot #844). No spec, corpus or wire change; no #843 follow-ups owed.

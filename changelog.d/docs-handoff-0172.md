kind: docs: HANDOFF rewritten for the 0.17.2 state
---
Docs only. `HANDOFF.md` is rewritten (a rewrite, not an append) for the state after the 0.15.9 to 0.17.2 batch: what landed, the stamp procedure, the unruled items (board 440, the sk8 and backward RFCs, kernel-body scope), FireLine's five contract-vector requests as the first task, and the three WAIT timers. Verified: `python tools/kiss_trace.py --ratchet --base-ref origin/main` and `python tools/kiss_readme_coverage.py` clean.

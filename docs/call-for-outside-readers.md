# A call for outside readers of KISS

**Written to be posted as a community post. Informative. Nothing in it changes a clause.**
Status of the thing being read: KISS is a **pre-1.0 draft**. Nothing is frozen. That is the point of this
post, not an obstacle to it.

## What KISS is, in three sentences

KISS is a set of free, open standards for the seams between ML and compute libraries and the software
that provides compute kernels: how a kernel is **described, discovered, requested and verified**. It
does not say how a kernel is compiled or run. Source and specs: <https://github.com/ThinkersJournal/KISS>
(start at `README.md`, then `spec/umbrella.md`).

## Why we are asking

KISS is a verification standard: a freeze is only meaningful if someone who did **not** write it can read
the text and reproduce the bytes. So far every implementation we have (Baracuda, Fuel, Unpopped, Vulkane,
and our own reference code) traces back to one reading of the spec. The project owner's ruling is that
those are a partial mitigation and **not** a clearing; the roadmap (`ROADMAP.md`, R4.1) lists a foreign
reader as outstanding. We would rather learn what is wrong now, while it is cheap to change, than after a
freeze. **You do not need the freeze to help, and your report is useful even if you stop early.**

## Who we want

- People from **other ML-framework backgrounds**: JAX/XLA, PyTorch, ONNX, TVM, MLIR, ggml, tinygrad,
  Triton, and anyone who has written glue between a framework and a vendor kernel library.
- **Implementers in other languages**: if you write C, C++, Go, Zig, Python, Java, Julia or anything not
  Rust, you are exactly the reader we lack.
- **Genuinely foreign readers**: people who have never seen our code or our discussions, and who will
  read the documents the way a stranger would.

We are not asking for endorsement. A reader who finds the text ambiguous is more useful than one who
agrees with it.

## What you would need

Everything below is in the repository; you should not need to ask us for anything.

- **The documents:** `spec/` has the umbrella and nine sub-standards. Read the one your task touches.
- **The manifests:** `conformance/corpus/dtype_manifest.json` and `conformance/corpus/op_manifest.json`
  (the dtype and operation vocabularies), and `conformance/registry/namespaces.json`.
- **The vectors:** `conformance/corpus/` holds pinned examples with expected outputs, for example
  `structure_key_vectors.json` (key tokens), `announce_vectors.json` (Announce frames),
  `contract_vectors.json` (Contract documents), `grammar_vectors.json`, the `opattrs_*` files, and the
  numeric `ops-*.json` files. `conformance/corpus/PROVENANCE.md` says where each came from.
- **A worked example of what a reproduction looks like:** Baracuda reproduced the 12 pinned `vulkan:`
  token vectors from a single manifest file without reading its source. The write-up and the script are in
  `docs/measurements/kiss-6.8-0017-vulkan-reproduction/` in the public repository
  <https://github.com/ciresnave/baracuda> (read at commit `037ec07f`, `README.md` and `produce.py`). Its
  result, in its own words, is that 12 of 12 pinned vectors reproduce byte-identically, with **eight
  things that had to be supplied from outside the manifest**, and that the corpus cannot contain a
  saturating shape. To re-run it: `python produce.py` in that directory (standard library only). That
  record also lists its own mistakes; please read it as a template for how to report, not as a verdict
  on the namespace.
- **Our own suite:** `cd conformance && cargo test` runs the reference harness (needs a Rust toolchain;
  the GPU-only slice is behind `--features cuda` and is not needed for any of the tasks below).

## What we ask you to do

Pick **one**, at whatever depth suits you.

1. **Reproduce a token.** From `spec/classify.md`, `dtype_manifest.json` and `structure_key_vectors.json`
   alone, write a producer in your language and check it against the pinned vectors. Do not read our Rust.
2. **Read a wire format.** Parse the Announce frames or the Contract document in your language, from
   `spec/announce.md` / `spec/contract.md` and the golden vectors, and say whether the bytes match.
3. **Read cold.** Take one sub-standard (the smallest is a good start), read it as a stranger, and list
   every sentence that admits two readings.

Whatever you pick, **keep a ledger of every input you had to supply from outside the documents**: a
constant you looked up, a rule you inferred from the vectors, an ordering you guessed. A guess that turned
out right still goes in the ledger. The ledger is the result: it tells us what the text fails to
determine. Also record what you read, so we know which parts you came to blind.

## How to respond

Publicly, on GitHub, so the record stays visible:

- Open an issue at <https://github.com/ThinkersJournal/KISS/issues/new/choose> using the template
  **"Outside reader report"**. It asks for the task, your language, the ledger, and the clauses involved.
- A clause that is ambiguous or self-contradictory can also use **"Spec defect / interoperability
  failure"**; a proposed change uses **"RFC"**. The process is in `CONTRIBUTING.md`.
- A partial report, a single unclear sentence, or "I got stuck here" is welcome. We will answer an issue
  from an outsider within three days.

There is no private address for this, on purpose: what you find belongs in the open record.

## What we will do with it

Every finding is answered on the issue, and a finding that changes the text is fixed there, not
quietly. A reproduction that needs a guess is recorded as a defect in the document, not as a failure of
the reader.

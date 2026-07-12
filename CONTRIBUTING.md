# Contributing to KISS

KISS (Kernel Interface Standards Suite) is developed in the open and stewarded by
[ThinkersJournal](https://github.com/ThinkersJournal). Comment and proposals are welcome
now, while every document is still a pre-1.0 draft and nothing is frozen.

This file is a short operational guide. The authoritative governance and legal terms live
in the umbrella specification: [`spec/umbrella.md`](spec/umbrella.md) §7 (Governance) and
§9 (Legal).

## How to participate

- **Comment or ask a question** — open an issue. Point at the document and clause ID
  (e.g. `KISS-ANNOUNCE-6-0003`) so the discussion is anchored to specific normative text.
- **Propose a change** — open a pull request, or an issue tagged as an RFC for anything
  substantive (a new clause, a wire-format change, a vocabulary addition). Explain the
  problem before the solution.
- **Report an interoperability failure** — if two implementations read the same clause
  and behaved differently, that is a defect in the *specification*, not just the code.
  File it against the clause; ambiguity that admits two readings is a bug we want.

## Roles and process (summary of umbrella §7)

- Each sub-standard has an **editor of record** who is responsible for its text and for
  integrating accepted RFCs.
- Substantive changes go through a lightweight **RFC**: a written proposal that interested
  parties may co-sign and comment on. The editor integrates or declines with a rationale.
- Anyone may implement, comment, and co-sign. Stewardship is about maintaining a coherent,
  testable, vendor-neutral specification — not about gatekeeping who may use it.

## Maturity and the freeze gate (summary of umbrella §5)

Each sub-standard is versioned independently and moves through maturity stages. Advancing
**Draft → Frozen** requires passing the **freeze gate**:

1. at least **two dissimilar, independently developed implementations** demonstrating the
   normative behavior, and
2. an **adversarial-outsider review** — a reader who did not write the text attempts to
   implement it from the document alone and reports every ambiguity, every under-specified
   value, and every place two conforming implementations could diverge.

A frozen clause does not change incompatibly; growth happens through additive versions and
the extension registry, never by silently redefining frozen text.

## Normative writing conventions (summary of umbrella §3–§4)

If you propose normative text, follow the house style so it stays testable:

- Use RFC 2119 / RFC 8174 keywords (MUST, SHOULD, MAY) with their exact meanings.
- Keep §0–§5 informative and §6+ normative; do not smuggle requirements into the overview.
- Give every normative requirement a clause ID `KISS-<SUB>-<section>-<nnnn>` and a
  corresponding conformance test. **A MUST with no test fails the suite build.**
- Pin values (byte offsets, magic constants, enumerations) exactly. No unquantified
  adjectives ("fast", "small", "reasonable") in normative text.
- Keep project and product names out of normative clauses. They may appear only in
  non-normative examples, provenance, acknowledgments, and the signatory record. Normative
  text uses the generic roles: *provider*, *consumer*, *implementation*, *kernel*,
  *contract*, *target*.

## Contributor licensing terms (summary of umbrella §9)

By contributing to this repository you agree that:

- **Your contribution to the specification text is dedicated to the public domain under
  [CC0 1.0 Universal](LICENSE)**, the same terms as the rest of the specification.
- You grant a **royalty-free license to any of your essential patent claims** necessary to
  implement the contributed specification text, subject to **defensive termination** (the
  grant to a party ends if that party sues asserting that a conforming KISS implementation
  infringes its patents). CC0 waives copyright but not patents, so this grant is made
  separately and is bound at contribution time.

If you cannot make those commitments for a given contribution, say so in the PR and we will
work out how to proceed — do not merge text you cannot license this way.

---

*Thank you for helping make the kernel interface a commons instead of a thicket of private
glue.*

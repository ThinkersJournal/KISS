---
name: RFC — propose a change to a KISS sub-standard
about: A substantive change — a new clause, a wire/ABI change, or a vocabulary addition.
title: "RFC: <one-line summary>"
labels: rfc
---

<!--
A KISS RFC IS this GitHub issue. The issue is the durable record of the change:
the problem, the discussion, who co-signed, and the final accept/decline rationale
all live here and stay visible after the issue is closed. See CONTRIBUTING.md
("The RFC lifecycle") and umbrella §7.2. State the problem before the solution.
-->

## Affected sub-standard(s) and clause(s)
<!-- Name every clause this touches, by ID: e.g. KISS-OPS-6.8-0001, KISS-CONTRACT-6.5-0012.
     If it adds a new section, say where (e.g. "new KISS-Ops §6.20"). -->

## Problem
<!-- What ambiguity, gap, or defect does this address, and why does it matter?
     If two implementations read a clause and behaved differently, that is a
     specification defect — say so, and give the two divergent readings. -->

## Proposal
<!-- The change. For normative text, follow the house style (umbrella §3–§4):
     RFC 2119 keywords, ONE MUST per clause, values pinned as bits/IEEE-754 in wire
     order, a declared determinism/fidelity class for any numeric clause. -->

## Conformance impact
<!-- The new/changed clause IDs and the KISS-Conform test(s) that will back them.
     A normative MUST with no mapped test fails the suite build. -->

## Wire/ABI compatibility
<!-- Additive under the capability model? A new wire/ABI schema version? A pre-freeze
     additive amendment (no version bump)? Note any cross-party-visible version bump. -->

## Interested cosignatories
<!-- The projects/parties this affects (providers, consumers, emitters). They comment
     and co-sign here. Propose-first: float this before it is wired. -->

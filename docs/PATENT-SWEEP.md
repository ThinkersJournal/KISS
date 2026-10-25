# Patent sweep: KISS (specs + `kiss-conformance` reference crate) 0.4.x pre-1.0 drafts @ 1841938

Date: 2026-10-25; Run by: KISS lane, Sonnet 5.5; Method: AI-assisted web search of Google Patents and USPTO
full text through a general web-search tool, then claim reads with `tools/patent-claims.py`;
**NOT legal advice; NOT a freedom-to-operate opinion.** Covers the version line **0.4.x** (the pre-1.0 drafts
as published on the public GitHub repository; nothing is tagged or on crates.io). Process:
`PATENT-SWEEP-PROCESS.md` in the portfolio root.

**Read this first.** This is a **partial record, second pass (2026-10-25, same day).** The first pass read claim 1 only; this pass reads **all independent claims** of every patent listed and adds a CPC/similar-documents walk and two queries. The search tool is a general web search restricted
to patent sites, not a patent database with claim-text or classification search, so a quiet result means
"this method did not surface one", not "none exists". 11 ideas, 13 queries over 10 of them, 14 patents' independent claims read, 1 `needs-review` (US8458680B2, implementers). A 0.4.x draft
suite is a set of documents and a reference test crate, which is a lower-risk object than shipped
product; the sweep must be repeated before any 1.0 freeze or tag.

## Ideas swept

1. A canonical, versioned text key (`structure_key`) naming a kernel's structural class (dtype, operand
   shape class, index width, schedule, target capability) as the join between a provider's advertisement
   and a consumer's request.
2. A cache or match is admissible only if the request key matches AND the artifact's revision hash
   matches (the key says what was asked for, the revision says what was produced).
3. A provider advertises available kernels and capability bits; a consumer requests and discovers them
   through standardized frames (Announce / Contract / Synth).
4. A per-operation declared determinism class (exact-byte, ULP/tolerance, order-invariant), with a
   comparator that treats NaN by value provenance.
5. An accumulator-keyed tolerance band `k(S,A) * N * eps_A * |max partial sum|` bounding reassociation
   error for reductions, scans and contractions.
6. A bidirectional clause-to-test traceability matrix whose coverage count is a ratchet that fails a
   build when it regresses.
7. A pair checker requiring that each key field has a vector pair differing only in that field.
8. Reference vectors minted by several independent high-precision engines and accepted only on agreement.
9. An on-device differential: a generated kernel is run on a GPU over a deterministic corpus and compared
   bit for bit with a CPU oracle.
10. A "foreign reproduction" freeze gate: a standard may freeze only after a reader outside the reference
    lineage reproduces its bytes from the document alone.
11. A neutral quantization sidecar (ten fields: encoding, scale placement/dtype/granularity, block size,
    logical dtype, dequant form, byte layout, alignment) carried alongside, not inside, the key.

## Queries (source, query, date; all 2026-10-25)

All through a general web search restricted to `patents.google.com`, one standard-mode query each. Hits
were then opened in Google Patents.

| idea | query (abridged) |
|---|---|
| 1 | kernel cache key structural signature dtype shape compute kernel selection matching provider consumer |
| 1 | operator signature string encoding data types shapes tensor operation lookup precompiled kernel library dispatch |
| 2 | cache hit requires both request key and artifact revision hash compiled binary admissibility predicate |
| 3 | GPU kernel capability advertisement library announces available kernels framework requests kernel interface standard |
| 4 | deterministic floating point result classes bitwise reproducible reduction order declared guarantee library interface |
| 5 | accumulator rounding error tolerance bound floating point reduction number of terms epsilon test comparison |
| 6 | requirements traceability matrix test coverage build fails untested requirement ratchet |
| 7 | conformance test suite verify each field of identifier affects outcome paired test vectors differing in one field |
| 8 | verifying math library accuracy ULP reference arbitrary precision test vectors floating point function |
| 9 | differential testing generated GPU kernel compare output against CPU reference implementation bit-exact |
| 11 | quantized tensor descriptor scale placement block size zero point metadata interchange format neural network weights |
| 7 | test adequacy each input parameter influences output verified by test pair varying single parameter specification coverage mutation (second pass) |
| 1, 2 | precompiled kernel binary cache lookup key hash of operation signature and version of generator invalidates cached binary on revision change (second pass) |

Second pass also walked Google Patents' CPC codes, "similar documents" and cited-by lists for WO2013090788A1, US8136104B2 and US20190278593A1 (CPC G06F9/50, G06F8/41, G06F9/5044, G06F11/36 families), which led to US8146066B2, US8458680B2 and US8972943B2.

Idea 10 (a process rule for a standards body, not a mechanism) was not searched.

## Hits

**All independent claims** of each patent below were read in full (second pass); dependent claims were not. Legal status was
**not** available from the tool for any patent below (blank); it must be checked on the official record
before relying on any "expired" or "lapsed" assumption.

| # | patent | title | assignee | status | claim(s) read | overlap | our difference | verdict |
|---|---|---|---|---|---|---|---|---|
| 1 | US8799869B2 | System for ensuring comprehensiveness of requirements testing of software applications | Accenture | not checked | all independent (1, 13, 17) | Links tests to requirements and reports untested ones (idea 6) | Claim 1 requires a link generator that infers links by executing tests, comparing topics, and scoring a reliability value from the number of algorithms, plus user invalidation of links. KISS links are declared by clause id in test source and checked by a script; nothing is inferred, scored or user-invalidated, and no build-failing ratchet appears in the claim | clear (independent claim 1 requires inference steps we do not perform) |
| 2 | US8136104B2 | Systems and methods for determining compute kernels for an application in a parallel-processing computer system | Google | not checked | all independent | A reference result on a CPU compared with a GPU result (idea 9, per the search summary; the claim itself is a runtime queue of IR entries) | Claim 1 is a runtime that queues intermediate-representation entries and dynamically selects a processor type and prepares kernels; the comparison is not in claim 1. KISS neither queues IR nor selects processors | clear (claim 1 elements absent); the dependent claims that mention comparison were not read |
| 3 | WO2013090788A1 | Allocating compute kernels to processors in a heterogeneous system | AMD | WO publication; national status not checked | all independent | Kernel metadata compared with a target description (ideas 1, 3, loosely) | Claim 1 compares a kernel *performance* profile with processor profiles and schedules execution. KISS matches a structural key to a request and does not schedule | clear |
| 4 | US20190278593A1 | Accelerating linear algebra kernels for any processor architecture | Nvidia | published application | all independent | Kernel generation interface (idea 3, loosely) | Claim 1 builds a DAG, a polyhedral representation, and an optimized schedule. KISS standardizes description and verification, not scheduling | clear |
| 5 | US11023230B2 | Apparatus for calculating and retaining a bound on error during floating-point operations | individual | not checked | all independent | Error bound carried with floating-point values (idea 5, loosely) | Claim 1 is a hardware bounded-FPU circuit with a bound field in the number format. KISS declares a tolerance in a document and has no such circuit or format | clear |
| 6 | US10936769B2 | Systems and methods for measuring error in terms of unit in last place | MathWorks | not checked | all independent (1, 34, 35, 46, 49) | ULP error values attached to model elements and inferred through a model (ideas 4, 5, 8) | Claim 1 infers and presents a total ULP error across a simulation model's elements. KISS declares ULP ceilings per op and compares results against a reference; it does not propagate ULP error through a model | clear |
| 7 | US8489859B2 | Performing a deterministic reduction operation in a compute node organized into a branched tree topology | IBM | not checked | all independent | A deterministic reduction (idea 4) | Claim 1 fixes the order through a collectives-acceleration unit in a tree of compute nodes with acknowledgement ordering. KISS names a determinism *class*; it does not implement a reduction | clear |
| 8 | US10643297B2 | Dynamic precision management for integer deep learning primitives | Intel | not checked | all independent (1, 9, 15) | Blocked quantized tensors with shared scale/exponent metadata (idea 11) | Claim 1 is a GPU with dynamic-precision fixed-point logic that adjusts precision to prevent overflow. KISS defines a descriptor shape in text, not a processor or a precision manager | clear |
| 9 | US8146066B2 | Systems and methods for caching compute kernels for an application running on a parallel-processing computer system | Google | not checked (priority ~2008; term not computed) | all independent (1, 8, 15) | Looks up previously compiled kernels for requests (ideas 1, 2) | Requires, at runtime, generating IR entries for application operation requests, dynamically choosing a processing-element type, identifying a previously compiled kernel and scheduling it. KISS standardizes the key and the documents; it has no runtime that generates IR entries or schedules | clear for the standard and reference crate; an implementer's runtime is its own question |
| 10 | US8458680B2 | Systems and methods for dynamically choosing a processing element for a compute kernel | Google | not checked | all independent (1, 9, 17) | **Closest hit.** At runtime, receive operation requests, choose a processing-element type, identify a precompiled processor-specific kernel from a library for an intrinsic or primitive operation, and prepare kernels (ideas 1, 3: a consumer requesting a library kernel by op and target) | KISS does not specify a runtime that chooses a processor type; it specifies the vocabulary and frames by which a request names a target and a provider answers. But a *conforming implementation* (a consumer that selects a target and resolves a precompiled kernel from a provider library) could read on these claims | **needs-review** (implementation level, not the documents); see below |
| 11 | US8972943B2 | Systems and methods for generating reference results using parallel-processing computer system | Google | not checked | all independent (1, 8, 15) | Reference processing element and debugging (idea 9) | Every independent claim requires fusing operation requests with a break-point decision and debugging kernels during execution; KISS's on-device differential does neither | clear |
| 12 | US11481311B2 | Automatic evaluation of test code quality | SAP | not checked | all independent (1, 5, 17) | Relates test assertions to the input parameters that influence an output (idea 7, loosely) | Requires analysing target code to find input/output parameters, counting operations processing an input and counting assertions per output against a threshold. KISS's pair checker compares declared vectors per key field, with no code analysis or assertion counts | clear |
| 13 | US7237231B2 | Automatic identification of input values that expose output failures in a software object | Microsoft | not checked | all independent (1, 16, 27, 32, 35) | Generates test cases varying input fields (idea 7, loosely) | Requires annotated test-case templates constraining per-field variation and generating cases from them, with serialization to XML. KISS authors vectors by hand and checks pairing | clear |
| 14 | US20160048376A1 | Portable binary image format (pbif) for pre-compiled kernels | AMD | published application | all independent (1, 7, 13, 17) | Pre-compiled kernels with runtime recompilation (idea 2, loosely) | Requires combining an ISA-specific executable and a code-generator output into one image recompiled at runtime on ISA mismatch. KISS defines no binary image format | clear |

Hits seen in results but **not** opened: US20180197068A1 (neural-network instruction set encoding;
hardware, closest on "encoded tensor operation field"), US20150022538A1 (enqueue kernels from kernels;
closest on capability discovery), US8643656B2, US7773090B1, US9524270B2, US8595701B2, US10559056B2,
US20230342613A1, the five other members of the Intel family above, the USPTO full-text siblings of
US11023230 (11797300, 9817662), and the reduction-engine family (US12393530 and others). They are
adjacent to ideas 3, 4, 5, 9 or 11 and are the first to read in a deeper sweep.

## Needs review (to the PM)

**US8458680B2** (Google; "dynamically choosing a processing element for a compute kernel"). The documents
and reference crate do not perform its steps (they specify vocabulary, frames and tests, not a runtime that
chooses a processor type and resolves a precompiled kernel), so this is not a finding against KISS as a
standard. It is a finding for *implementers* of the consumer role (Fuel, Baracuda, Unpopped runtimes) and
for any KISS reference runtime, if one is ever built. Legal status, filing date and term were not checked;
the family (US8146066B2, US8381202B2, US8443349B2 and others) was only partly read. Routing it to the
implementing lanes and a lawyer is the PM's call.

**Not a clearance:** ideas 1 and 2 (the key and the revision-hash admissibility rule) now have one close
family (the Google runtime family above), read at the independent-claim level and found not to claim a
standardized key or a key-plus-revision admissibility test. Idea 7 (the pair checker) found only
test-quality patents (US11481311B2, US7237231B2) that do not claim single-field pairing. A positive control
that the search runs: it surfaced the adjacent families above. That does not show no relevant patent exists.

## Still not done (before any tag, publish or freeze)

- Idea 10 (foreign-reproduction freeze gate) was not searched.
- No classification (CPC) *search*, only a walk of the CPC, similar-documents and cited-by lists of three
  patents; no assignee search on Nvidia, Intel, Meta, Microsoft, Huawei, Alibaba.
- Dependent claims were not read; the Google runtime family (about ten members) was read for three.
- Legal status, filing dates and expiry were not checked for any patent.
- No non-US coverage beyond what Google Patents surfaced.

## Limitations

- AI-assisted search and read; not legal advice; not a freedom-to-operate opinion.
- General web search restricted to patent sites, plus Google Patents' own page lists; no claim-text
  search, no classification search.
- 13 queries over 10 ideas. Recall is low.
- All independent claims read for 14 patents; dependent claims unread.
- Legal status, expiry and family members were not checked.
- Ideas were written by the lane that owns the project; a reader outside it may list others.
- Patents are not the only risk: licences, copyright and trademark are outside this record. Third-party
  provenance is tracked separately (`conformance/corpus/PROVENANCE.md`).

# ONNX 1.17 against KISS-Ops: the name-matched ops and their attributes (board 414 / 422 audit)

Status: **a measurement, informative, no clause.** Written 2026-11-12 against `origin/main` 73c341b5 (conformance 0.15.11). PM [TASK]
`rulings-417-A-414-C`: "do the 72-match attribute audit now, it needs no ruling." Reproduce: `python -I docs/onnx-audit/onnx_attr_audit.py
<ONNX v1.17.0 docs/Operators.md> conformance/corpus/op_manifest.json` (the ONNX file is fetched text, parsed by regular expression, never run).

## 1. Counts (and what they are)

| Quantity | Value | How |
|---|---|---|
| ONNX default-domain operators, latest version each | 191 | the first operator table of `docs/Operators.md` at tag `v1.17.0` |
| KISS-Ops operators | 126 | `all_ops` of `op_manifest.json` |
| Matched by name, exact (lower-case or snake-case of the ONNX name) | 44 | script |
| Matched by a one-to-one name alias (human list `ALIAS`, e.g. `Equal`->`cmp_eq`, `Where`->`select`) | 28 | script, list in the file |
| **Name-matched** | **72** | 44 + 28 |
| of those, carrying at least one ONNX attribute | 22 (50 attributes) | script |
| Not matched at all | 119 | includes `Conv`, `Gemm`, `Transpose`, `Reshape`, `Slice`, `Split`, `Range`, `If`, `Loop`, `Scan` and the sequence/optional/string families |

The earlier "72 matches" of the 414 note is **not in the repository**, so I could not diff against it. This rebuild lands on the same 72 by
a different route (the alias list is mine); if the PM's set differs, the table below is still correct for the ops it names.

**What this audit does not cover.** The other 50 matched ops carry no ONNX attribute, so a *name* match says nothing yet about their
*semantics* (ONNX `Pow`/`Sign`/`Max`/`Min` NaN rules, integer `Div`/`Mod`, `Round`'s tie rule against `round_even`, broadcasting of `Where`,
variadic `Max`/`Min` against binary `max_prop`/`min_prop`). That is a second audit, op by op against the pinned KISS rule, and is not done here.

## 2. The 22 ops with attributes

Verdict legend: **E** same meaning, KISS carries it; **R** representable, the importer resolves it (static shapes make many of these
constants); **G** no KISS counterpart found in the 6.19 OpAttrs table or the clause text I read; **U** unverified (a clause I did not read
decides it). KISS attributes are from the `spec/ops.md` 6.19 table at 73c341b5.

| ONNX op -> KISS op | ONNX attribute | KISS counterpart | Verdict |
|---|---|---|---|
| `ArgMax`/`ArgMin` -> `argmax`/`argmin` | `keepdims` | `keepdim` | E |
| | `select_last_index` | `tie_rule` (`1` lowest, `2` highest, `3` any; 6.11-0021) | E (ONNX `0` = lowest, `1` = last = `2`) |
| | `axis` | none in the `argmax` row (`keepdim`, `tie_rule` only); the decomposition is a `sort_network` whose `axis` defaults to the trailing axis | **G**, or **U** if 6.11-0014 lets `argmax` name one |
| `AveragePool` -> `avg_pool`; `MaxPool` -> `max_pool` | `kernel_shape` | `window_size` | E |
| | `strides`, `dilations` | `stride`, `dilation` | E |
| | `pads` (begin/end pair per axis) | `padding`: one `vec` | **U**: whether KISS padding is symmetric or begin/end |
| | `auto_pad` (`SAME_UPPER`, ...) | none; padding is explicit | R (static shapes resolve it to explicit pads) |
| | `ceil_mode` | none stated: the window view is a `gather` with OOB taps skipped; no output-extent rounding mode | **G** (a ceil-mode pool has an extra window; not expressible as stated) |
| | `count_include_pad` (`AveragePool`) | `count_include_pad` | E |
| | `storage_order` (`MaxPool` index output order) | none (`max_pool` has no index output) | **G**, only matters for the optional `Indices` output |
| `BitShift` -> `shl`, `shr` | `direction*` (`LEFT`/`RIGHT`) | two ops, not an attribute | R (pick the op) |
| `Cast` -> `cast` | `to*` | `target_dtype` | E (dtype token map needed: ONNX enum -> Classify token) |
| | `saturate` | `saturation` (mandatory, no default; 6.21-0003) | R (ONNX default `1` = saturate; `saturation` is dtype-dependent in KISS, so the legal value must be chosen by the importer) |
| `Concat` -> `concat` | `axis*` | `axis` | E |
| `CumSum` -> `cumsum` (`prefix_scan(sum)`) | `exclusive` | `exclusivity` | E |
| | `reverse` | none | **G** (a reversed scan = scan of the axis-reversed view; KISS has no flip/reverse op) |
| `Gather` -> `gather` | `axis` | `axis` | E (ONNX negative indices wrap; KISS `oob_policy` default `skip`: **U** whether a negative index is "out of bounds" or wraps) |
| `GatherElements` -> `gather` | `axis` | `axis` | **U**: element-wise gather along an axis vs `gather`'s index-operand form |
| `Gelu` -> `gelu` | `approximate` (`none`/`tanh`) | two ops, `gelu` and `gelu_tanh` | R (pick the op) |
| `LayerNormalization` -> `layer_norm` | `axis` (and all axes after it) | `norm_axes` (`u16` mask) | R (a suffix mask) |
| | `epsilon` | `param(0)` (an operand/param, not an OpAttr; 6.13-0008) | R |
| | `stash_type` | none | **G**, only matters for the optional `Mean`/`InvStdDev` outputs |
| `LogSoftmax`, `Softmax` -> `log_softmax`, `softmax` | `axis` (single) | `norm_axes` (`u16` mask, `0x0000` forbidden) | R (a one-bit mask) |
| `ReduceSum`/`Max`/`Min`/`Prod` -> `reduce` | `keepdims` | `keepdim` | E |
| | `noop_with_empty_axes` | none: an empty `reduce_axes` mask `0x0000` is **forbidden** (6.19-0038) | **G** (the ONNX empty-axes `noop` form is the identity op; KISS refuses the empty mask) |
| `ReduceMean`, `ReduceL2`, `ReduceLogSumExp` -> `reduce_mean`, `reduce_norm2`, `logsumexp` | `keepdims`, `noop_with_empty_axes` | as above | E, G |
| `ScatterElements` -> `scatter` | `axis` | `axis` | E |
| | `reduction` (`none`/`add`/`mul`/`max`/`min`) | `combine` (`assign`, `atomic-add`, `atomic-max`, `atomic-min`) | **G** for `mul`; `none`->`assign` E (6.11-0006 pins the tie-break), `add`/`max`/`min` E; note ONNX `none` with duplicate indices is nondeterministic where KISS pins last-writer |

**Tally of the 50 attributes**, counting an attribute once per op that carries it: **E 23**, **R 9**, **G 15**, **U 3**. The 15 G rows are
only **7 distinct gaps**: `noop_with_empty_axes` (7 of the 15, one per reduce-family op), `ceil_mode` (2 pools), `argmax`/`argmin` `axis`
(2, G or U), and one each of `storage_order`, `reverse`, `stash_type`, `reduction = mul`. The 3 U rows are pool `pads` symmetry (2 pools) and
`GatherElements`' form; the `Gather` negative-index question rides on a row counted E.

## 3. What this implies for KISS-Graph (414 C)

* A graph importer is **mostly a renaming and constant-folding layer** over the 72, which supports the 414 "reuse KISS-Ops names" line.
* The G rows are the work list: each is either a KISS gap (add an attribute or op: `reverse`, a ceil-mode, an empty-axes identity) or a
  stated **refusal** the graph standard returns as a typed decline. Neither is decided here; none is a byte move until chosen.
* **`iota` (board 422).** By my reading of the ONNX docs (not measured op by op) `Range`, `EyeLike`, `Trilu` and `OneHot` need an index
  sequence; all four sit in the 119 unmatched. `iota` alone does not cover them (they also need compare/select/cast, which KISS has).
* ONNX versions: the table reads each op at its **latest** opset version in 1.17. A model written at an older opset can differ in attribute
  defaults and in whether an input became an attribute (`Squeeze`'s `axes` is an input since opset 13, which is why it shows no attribute).
  A graph standard must state which opset it targets or say it does not read ONNX at all (an importer concern).

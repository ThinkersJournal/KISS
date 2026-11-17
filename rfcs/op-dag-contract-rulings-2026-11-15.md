# op_dag and Identity-consistency rulings (2026-11-15)

Status: ruled by the PM (not a CireSnave ruling), on FireLine's request for contract vectors before it wires its
`op_dag` reader into `read_contract` (fireline#60). The vectors are `conformance/corpus/contract_vectors.json`
`op_dag_vectors` (KISS#828); the reference reader is `conformance/src/op_dag.rs`. The spec text carries each
ruling as a *Draft change* beside its clause.

| Question | Ruling | Clause |
|---|---|---|
| Is an empty `op_dag` legal under `machine-checkable-IR`? | No: a complete IR has a root for the `op_identity` to equal; a reader declines it. | KISS-CONTRACT-6.4-0007, 6.4-0002 |
| ... under `declared-op-tag`? | Yes, `[]` when the IR names no node for the region; exempt from the cell-class assertion. With nodes it is checked like any DAG. | 6.4-0007, 6.3-0006 |
| What is "any op of the DAG" for a fusion? | Any `Op` node; a `Source` or `Bind` never has a fact. | 6.3-0006 |
| How is the root compared to `op_identity`? | By the bare op name. Attributes and the operand-role tuple stay producer obligations, not checked by these vectors. | 6.4-0002, 6.11-0009 |
| Where do a reader's per-op facts come from? | `conformance/corpus/op_facts.json`, generated from `spec/ops.md`; never a private table. | 6.3-0006 |

None of these changes a published behaviour: nothing was published that read the `op_dag` before these vectors.
Not decided here: whether a reader should also check the identity-bearing attributes of the root against the
`op_identity`; that needs the identity-bearing attribute set stated machine-readably first.

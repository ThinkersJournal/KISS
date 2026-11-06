# sk7 vector changes — old (`sk6`, `origin/main` 3c04110) vs new

The only `structure_key` change at `sk7` is the constant `MAX_OPERANDS`, 8 to 16 (Classify 6.4-0002). The token has no operand-count
field, so a token of at most eight operands is byte-identical to its `sk6` spelling **apart from the prefix**.

| Artifact | Change |
|---|---|
| `structure_key_vectors.json` | every positive and decline token: `sk6|` to `sk7|`, nothing else; the `retired_sk5_token` row still declines (now "not decoded at sk7") |
| `structure_key_derivation_vectors.json` | every expected token: `sk6|` to `sk7|`; `structure_key_schema_version` 6 to 7 |
| `contract_vectors.json` | the Appendix C golden document embeds an `accept_predicate` token: its bytes, `len` and `crc32` move with the prefix |
| `dtype_manifest.json` | `structure_key_schema_version` 7, `token_prefix` `sk7` |

Added (nothing removed):

- positive `operand_count_9_above_the_sk6_limit` — nine operands, the first count `sk6` refused;
- positive `operand_count_16_at_max_operands` — sixteen operands, the boundary;
- decline `operand_count_17_over_max_operands` — `TooManyOperands { got: 17 }`;
- decline `retired_sk6_token` — `UnsupportedSchemaVersion { got: 6 }` (Classify 6.7-0014, 6.7-0015).

A reader that stops at 8 fails the 9-operand positive; one that stops at 15 fails the 16-operand positive; one that never bounds the
count fails the 17-operand decline. Regenerate with `cargo run --bin emit_structure_key_vectors`, `emit_structure_key_derivation_vectors`,
`emit_contract_vectors` (in `conformance/`) and `python tools/kiss_dtypes.py --emit-manifest`.

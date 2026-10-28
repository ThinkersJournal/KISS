// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-CLASSIFY-6.8-0018 (the vector-access byte cap is the target's, declared by its namespace, 16
//! bytes unless declared otherwise) and its use in §6.5-0009(c). The two namespace documents are read at
//! COMPILE time (`include_str!`), never transcribed.

use kiss_conformance::derive::{derive_structure_key, CellOperand, DerivationCell};
use kiss_conformance::structure_key::{
    derive_vec_width, derive_vec_width_capped, MathFidelity, OperandUse::{Input, Output}, VecWidth,
    DEFAULT_VEC_ACCESS_CAP_BYTES,
};
use std::num::NonZeroU32;

const CUDA_DOC: &str = include_str!("../../spec/namespaces/cuda.md");
const VULKAN_DOC: &str = include_str!("../../spec/namespaces/vulkan.md");
const DECLARATION: &str = "vector-access byte cap of 16 bytes";

fn cell(cap: Option<u32>) -> DerivationCell {
    let op = |usage| CellOperand {
        usage,
        dtype: "f32",
        extents: vec![64, 128],
        strides: vec![128, 1],
        alignment: 256,
        dequant: None,
    };
    DerivationCell {
        target: "rocm:gfx942",
        operands: vec![op(Input), op(Output)],
        index_operand: None,
        reduced_axes: vec![],
        accumulator: None,
        math_fidelity: MathFidelity::Stable,
        contraction: None,
        vec_cap_bytes: cap.and_then(NonZeroU32::new),
    }
}

// Backs[enforced]: KISS-CLASSIFY-6.8-0018 — the cap is a per-target byte count: the same cell derives v4 at 16 bytes,
// v8 at 32 and v2 at 8, an undeclared cap derives exactly as 16, and the token set never grows.
#[test]
fn test_classify_vec_cap_per_target() {
    // an undeclared cap is 16 bytes: byte-identical to an explicit 16 (positive control: they are not the
    // same object, one is `None`)
    let undeclared = derive_structure_key(&cell(None)).expect("derives").to_token();
    let sixteen = derive_structure_key(&cell(Some(16))).expect("derives").to_token();
    assert_eq!(undeclared, sixteen);
    assert!(undeclared.contains("co/00/v4/d16/f"), "f32 at 16 bytes is v4: {undeclared}");
    // a different declared cap derives a different width, and only the `vec` field of each operand moves
    let wide = derive_structure_key(&cell(Some(32))).expect("derives").to_token();
    let narrow = derive_structure_key(&cell(Some(8))).expect("derives").to_token();
    assert!(wide.contains("co/00/v8/d16/f"), "{wide}");
    assert!(narrow.contains("co/00/v2/d16/f"), "{narrow}");
    assert_eq!(wide.replace("/v8/", "/v4/"), undeclared);
    assert_eq!(narrow.replace("/v2/", "/v4/"), undeclared);
    // the token set stays {v1,v2,v4,v8}: a huge cap changes nothing past v8, a cap below one element is v1
    assert_eq!(derive_vec_width_capped(1, 256, Some(1), 256, false, 1024), VecWidth::V8);
    assert_eq!(derive_vec_width_capped(1, 256, Some(4), 256, false, 1024), VecWidth::V8);
    assert_eq!(derive_vec_width_capped(1, 256, Some(4), 256, false, 3), VecWidth::V1);
    // f64: v2 at 16 bytes, v4 at 32 (8*4 = 32)
    assert_eq!(derive_vec_width_capped(1, 256, Some(8), 256, false, 16), VecWidth::V2);
    assert_eq!(derive_vec_width_capped(1, 256, Some(8), 256, false, 32), VecWidth::V4);
}

// Backs[enforced]: KISS-CLASSIFY-6.8-0018 — no key derived before the clause moves: `derive_vec_width` (no cap
// argument) equals the 16-byte capped derivation over every dtype width, extent and alignment swept.
#[test]
fn test_classify_vec_cap_default_moves_no_key() {
    assert_eq!(DEFAULT_VEC_ACCESS_CAP_BYTES, 16);
    let mut n = 0;
    for bytes in [None, Some(1u32), Some(2), Some(4), Some(8), Some(16)] {
        for ext in [0i64, 1, 2, 3, 4, 6, 8, 16, 24, 256] {
            for align in [0u32, 2, 4, 8, 16, 24, 32, 48, 256] {
                for (stride, bcast) in [(1i64, false), (1, true), (-1, false), (2, false)] {
                    assert_eq!(
                        derive_vec_width(stride, ext, bytes, align, bcast),
                        derive_vec_width_capped(stride, ext, bytes, align, bcast, 16)
                    );
                    n += 1;
                }
            }
        }
    }
    assert!(n > 1000, "the sweep must be a real population, got {n}");
}

// Backs: KISS-CLASSIFY-6.8-0018 — both namespace documents registered today declare 16 bytes, and the
// detector sees a declaration where one is made and fails on a document that makes none.
#[test]
fn test_classify_vec_cap_namespaces_declare_16() {
    assert!(CUDA_DOC.contains(DECLARATION), "cuda.md must declare the cap");
    assert!(VULKAN_DOC.contains(DECLARATION), "vulkan.md must declare the cap");
    // negative control: the detector is not vacuous
    assert!(!"# a namespace document that declares nothing".contains(DECLARATION));
}

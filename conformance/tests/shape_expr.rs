//! KISS-Conform suite for the KISS-Ops **shape-expression vocabulary** and the
//! shape-side oracle (Ops §6.20), plus the KISS-Contract shape-consistency tie
//! (Contract §6.4-0011). Exercises three KISS-Conform modalities over the
//! exact-byte tier: golden byte-vectors (§6.4), typed-decline vectors (§6.7),
//! and reference-evaluator behaviour. Each test cites its pinning clause.

use kiss_conformance::assert_golden;
use kiss_conformance::shape_expr::*;

// ---- §6.20-0005 canonical serialization (golden byte-vectors) ----------------

// Proven: KISS-OPS-6.20-0005 (subject: impl; ref: PROVEN_BATCH2.md)
#[test]
fn test_shape_expr_serialization_golden() {
    // KISS-OPS-6.20-0005: one-byte tag (0 reserved), fixed-width LE fields,
    // u16-LE length-prefixed children — the §6.19 canonical discipline.
    // SameAs(operand=0): tag 0x01, operand 0x00.
    assert_golden("KISS-OPS-6.20-0005", "same_as_0",
        &ShapeExpr::SameAs { operand: 0 }.encode(), "01 00");
    // Extent(operand=0, axis=last): tag 0x02, operand 0x00, axis 0xFF (the `last`
    // sentinel; concrete axes are 0..MAX_RANK-1, so 0xFF is unambiguously last).
    assert_golden("KISS-OPS-6.20-0005", "extent_0_last",
        &Dim::Extent { operand: 0, axis: LAST }.encode(), "02 00 FF");
    // Const(2): tag 0x03, i64 LE.
    assert_golden("KISS-OPS-6.20-0005", "const_2",
        &Dim::Const(2).encode(), "03 02 00 00 00 00 00 00 00");
    // Param(field=0): tag 0x04, field 0x00.
    assert_golden("KISS-OPS-6.20-0005", "param_0",
        &Dim::Param(0).encode(), "04 00");
    // Div(Extent(0,last), Const(2)) — the rope-half dim: tag 0x08, then each child
    // as u16-LE byte-length + child blob. child1 = 02 00 FF (len 3), child2 =
    // 03 02.. (len 9).
    let half = Dim::Div(
        Box::new(Dim::Extent { operand: 0, axis: LAST }),
        Box::new(Dim::Const(2)),
    );
    assert_golden("KISS-OPS-6.20-0005", "rope_half_div",
        &half.encode(),
        "08 03 00 02 00 FF 09 00 03 02 00 00 00 00 00 00 00");
}

// ---- §6.20-0009 WithDim — experimental extension (umbrella §6.4, issue #80) ---

#[test]
fn test_shape_expr_withdim_extension() {
    // KISS-OPS-6.20-0009: `with_dim(operand, axis, dim)` = the operand's shape with
    // the resolved axis replaced by `dim`. Wire: tag 0x0A, u8 operand, u8 axis
    // (0xFF = last), one u16-LE length-prefixed child DimExpr blob. Experimental
    // extension tag activated via the umbrella §6.4 registry — NOT core.

    // Golden — with_dim(operand=0, axis=1, Const(7)): tag 0x0A, operand 0x00,
    // axis 0x01, child Const(7) = 03 07.. (len 9 -> prefix 09 00).
    let wd = ShapeExpr::WithDim { operand: 0, axis: 1, dim: Box::new(Dim::Const(7)) };
    assert_golden("KISS-OPS-6.20-0009", "with_dim_0_1_const7",
        &wd.encode(), "0A 00 01 09 00 03 07 00 00 00 00 00 00 00");

    // Golden — with_dim(operand=1, axis=last, Param(0)): axis is the 0xFF `last`
    // sentinel, child Param(0) = 04 00 (len 2 -> prefix 02 00).
    let wd_last = ShapeExpr::WithDim { operand: 1, axis: LAST, dim: Box::new(Dim::Param(0)) };
    assert_golden("KISS-OPS-6.20-0009", "with_dim_1_last_param0",
        &wd_last.encode(), "0A 01 FF 02 00 04 00");

    // Round-trip: a well-formed WithDim blob decodes back to its AST.
    assert_eq!(decode_shape(&wd.encode()).unwrap(), wd);
    assert_eq!(decode_shape(&wd_last.encode()).unwrap(), wd_last);

    // Typed decline, never a panic (§6.20-0006): a blob truncated before the
    // axis byte, and a blob whose child-length prefix over-runs the buffer.
    assert_eq!(decode_shape(&[0x0A, 0x00]),
               Err(ShapeExprError::TruncatedBlob { need: 3, got: 2 }));
    assert_eq!(decode_shape(&[0x0A, 0x00, 0x01, 0x09, 0x00]),
               Err(ShapeExprError::TruncatedBlob { need: 9, got: 0 }));

    // Eval — axis-replacement (§6.20-0003 resolution). Operand [2,3,5]:
    let ops = vec![vec![2i64, 3, 5]];
    assert_eq!(eval_shape(&ShapeExpr::WithDim { operand: 0, axis: 1, dim: Box::new(Dim::Const(9)) },
               &ops, &[]).unwrap(), ShapeValue::Concrete(vec![2, 9, 5]));
    // `last` resolves to the trailing axis.
    assert_eq!(eval_shape(&ShapeExpr::WithDim { operand: 0, axis: LAST, dim: Box::new(Dim::Const(9)) },
               &ops, &[]).unwrap(), ShapeValue::Concrete(vec![2, 3, 9]));
    // Gap propagation (§6.20-0004): a symbolic extent in a KEPT axis surfaces the
    // whole shape as a Gap; but REPLACING the symbolic axis clears it.
    let sym = vec![vec![4i64, SYMBOLIC]];
    assert_eq!(eval_shape(&ShapeExpr::WithDim { operand: 0, axis: 0, dim: Box::new(Dim::Const(7)) },
               &sym, &[]).unwrap(), ShapeValue::Gap);
    assert_eq!(eval_shape(&ShapeExpr::WithDim { operand: 0, axis: LAST, dim: Box::new(Dim::Const(7)) },
               &sym, &[]).unwrap(), ShapeValue::Concrete(vec![4, 7]));
    // A Gap replacement expression also surfaces a Gap.
    assert_eq!(eval_shape(&ShapeExpr::WithDim { operand: 0, axis: 0,
               dim: Box::new(Dim::Extent { operand: 0, axis: LAST }) }, &sym, &[]).unwrap(),
               ShapeValue::Gap);
    // An out-of-range axis is a typed decline, never a panic (§6.20-0003).
    assert_eq!(eval_shape(&ShapeExpr::WithDim { operand: 0, axis: 5, dim: Box::new(Dim::Const(1)) },
               &vec![vec![2i64, 3]], &[]),
               Err(ShapeExprError::AxisOutOfRange { axis: 5, rank: 2 }));
}

// ---- §6.20-0010 Dims — experimental extension (umbrella §6.4, issue #80) ------

#[test]
fn test_shape_expr_dims_extension() {
    // KISS-OPS-6.20-0010: `dims([dim, …])` = the whole shape built from N>=0 ordered
    // DimExprs (N=0 = rank-0 scalar). Wire: tag 0x0B, u8 count, then count × u16-LE
    // length-prefixed child DimExpr blobs. Experimental extension — NOT core.

    // Golden — dims([Extent(0,0), Const(2)]): tag 0x0B, count 0x02, child1
    // Extent(0,0) = 02 00 00 (len 3 -> 03 00), child2 Const(2) = 03 02.. (len 9 ->
    // 09 00).
    let dims = ShapeExpr::Dims(vec![
        Dim::Extent { operand: 0, axis: 0 },
        Dim::Const(2),
    ]);
    assert_golden("KISS-OPS-6.20-0010", "dims_extent_const",
        &dims.encode(),
        "0B 02 03 00 02 00 00 09 00 03 02 00 00 00 00 00 00 00");

    // Golden — the empty Dims (N=0): tag 0x0B, count 0x00, no children -> the
    // rank-0 scalar shape.
    let scalar = ShapeExpr::Dims(vec![]);
    assert_golden("KISS-OPS-6.20-0010", "dims_empty_scalar",
        &scalar.encode(), "0B 00");

    // Round-trip.
    assert_eq!(decode_shape(&dims.encode()).unwrap(), dims);
    assert_eq!(decode_shape(&scalar.encode()).unwrap(), scalar);

    // Typed decline, never a panic (§6.20-0006): a count that promises children the
    // blob does not contain; the still-reserved Reduce tag (0x09, no consumer);
    // and trailing bytes after a complete expression.
    assert_eq!(decode_shape(&[0x0B, 0x02]),
               Err(ShapeExprError::TruncatedBlob { need: 2, got: 0 }));
    assert_eq!(decode_shape(&[0x09, 0x00]),
               Err(ShapeExprError::ReservedTag { tag: 0x09 }));
    assert_eq!(decode_shape(&[0x0B, 0x00, 0xAB]),
               Err(ShapeExprError::TrailingBytes { extra: 1 }));

    // Eval — whole-shape construction. A qmatmul/scan-style reweave across two
    // operands: dims([Extent(0,0), Extent(0,2), Extent(0,3), Extent(1,3)]).
    let ops = vec![vec![8i64, 16, 32, 64], vec![1i64, 1, 1, 128]];
    assert_eq!(eval_shape(&ShapeExpr::Dims(vec![
        Dim::Extent { operand: 0, axis: 0 },
        Dim::Extent { operand: 0, axis: 2 },
        Dim::Extent { operand: 0, axis: 3 },
        Dim::Extent { operand: 1, axis: 3 },
    ]), &ops, &[]).unwrap(), ShapeValue::Concrete(vec![8, 32, 64, 128]));
    // The empty Dims evaluates to the rank-0 scalar shape.
    assert_eq!(eval_shape(&ShapeExpr::Dims(vec![]), &ops, &[]).unwrap(),
               ShapeValue::Concrete(vec![]));
    // Gap propagation (§6.20-0004): a symbolic extent in any element surfaces the
    // whole shape as a Gap.
    let sym = vec![vec![4i64, SYMBOLIC]];
    assert_eq!(eval_shape(&ShapeExpr::Dims(vec![
        Dim::Extent { operand: 0, axis: 0 },
        Dim::Extent { operand: 0, axis: LAST },
    ]), &sym, &[]).unwrap(), ShapeValue::Gap);
}

// ---- §6.20-0002 the closed vocabulary, evaluated -----------------------------

#[test]
fn test_shape_expr_vocabulary_eval() {
    // KISS-OPS-6.20-0002: SameAs -> the operand's whole shape; Extent/Const/
    // Param/arithmetic -> a single dimension.
    let ops = vec![vec![2i64, 3, 4]];
    let params = vec![7i64];
    assert_eq!(
        eval_shape(&ShapeExpr::SameAs { operand: 0 }, &ops, &params).unwrap(),
        ShapeValue::Concrete(vec![2, 3, 4])
    );
    assert_eq!(
        eval_dim(&Dim::Extent { operand: 0, axis: 1 }, &ops, &params).unwrap(),
        DimValue::Concrete(3)
    );
    assert_eq!(eval_dim(&Dim::Const(5), &ops, &params).unwrap(), DimValue::Concrete(5));
    assert_eq!(eval_dim(&Dim::Param(0), &ops, &params).unwrap(), DimValue::Concrete(7));
    // (extent(op0, axis0=2) * const 3) + param0(7) = 13
    let e = Dim::Add(
        Box::new(Dim::Mul(
            Box::new(Dim::Extent { operand: 0, axis: 0 }),
            Box::new(Dim::Const(3)),
        )),
        Box::new(Dim::Param(0)),
    );
    assert_eq!(eval_dim(&e, &ops, &params).unwrap(), DimValue::Concrete(13));
}

// ---- §6.20-0003 evaluator contract: axis resolution, floor div, declines -----

#[test]
fn test_shape_expr_axis_and_floordiv() {
    // KISS-OPS-6.20-0003: `last`-sentinel resolution, concrete axis, out-of-range
    // decline, floor division, and divide-by-zero decline. Axes are non-negative
    // (KISS §6.19 convention) with `last` a reserved sentinel — no signed axis.
    let ops = vec![vec![2i64, 3, 5]]; // rank 3

    // `last` resolves to rank-1 (the trailing axis, 5); a concrete axis indexes
    // directly (axis 0 -> 2, axis 2 -> 5).
    assert_eq!(eval_dim(&Dim::Extent { operand: 0, axis: LAST }, &ops, &[]).unwrap(),
               DimValue::Concrete(5));
    assert_eq!(eval_dim(&Dim::Extent { operand: 0, axis: 0 }, &ops, &[]).unwrap(),
               DimValue::Concrete(2));
    assert_eq!(eval_dim(&Dim::Extent { operand: 0, axis: 2 }, &ops, &[]).unwrap(),
               DimValue::Concrete(5));

    // A concrete axis >= rank is a typed decline (not a resolved value).
    assert_eq!(eval_dim(&Dim::Extent { operand: 0, axis: 3 }, &ops, &[]),
               Err(ShapeExprError::AxisOutOfRange { axis: 3, rank: 3 }));

    // ÷ is floor division (rounds toward −∞), including for negatives.
    let fd = |a: i64, b: i64| {
        eval_dim(&Dim::Div(Box::new(Dim::Const(a)), Box::new(Dim::Const(b))), &ops, &[])
    };
    assert_eq!(fd(7, 2).unwrap(), DimValue::Concrete(3));
    assert_eq!(fd(-7, 2).unwrap(), DimValue::Concrete(-4)); // floor(−3.5) = −4, not −3
    assert_eq!(fd(1, 0), Err(ShapeExprError::DivideByZero));
}

// ---- §6.20-0004 symbolic extent -> surfaced gap, never a decline/panic -------

#[test]
fn test_shape_expr_symbolic_gap() {
    // A data-dependent/symbolic extent is the SYMBOLIC sentinel. An expression
    // over it resolves to a surfaced Gap — never a typed decline, never a panic.
    // KISS-OPS-6.20-0004.
    let ops = vec![vec![4i64, SYMBOLIC]]; // last axis is data-dependent
    assert_eq!(eval_dim(&Dim::Extent { operand: 0, axis: LAST }, &ops, &[]).unwrap(),
               DimValue::Gap);
    // Arithmetic that touches a Gap is a Gap (propagates, does not crash).
    let half = Dim::Div(Box::new(Dim::Extent { operand: 0, axis: LAST }), Box::new(Dim::Const(2)));
    assert_eq!(eval_dim(&half, &ops, &[]).unwrap(), DimValue::Gap);
    // A whole-shape SameAs over a partially-symbolic operand surfaces a Gap.
    assert_eq!(eval_shape(&ShapeExpr::SameAs { operand: 0 }, &ops, &[]).unwrap(),
               ShapeValue::Gap);
    // A concrete axis of the same operand still resolves.
    assert_eq!(eval_dim(&Dim::Extent { operand: 0, axis: 0 }, &ops, &[]).unwrap(),
               DimValue::Concrete(4));
}

// ---- §6.20-0006 reader declines a malformed blob, never a panic --------------

#[test]
fn test_shape_expr_decode_declines() {
    // Round-trip: a well-formed blob decodes back to its AST (KISS-OPS-6.20-0005).
    let half = Dim::Div(
        Box::new(Dim::Extent { operand: 0, axis: LAST }),
        Box::new(Dim::Const(2)),
    );
    assert_eq!(decode_dim(&half.encode()).unwrap(), half);

    // KISS-OPS-6.20-0006: reserved 0 tag, a reserved-but-unregistered tag, a
    // truncated blob, and trailing bytes each raise a typed decline.
    assert_eq!(decode_dim(&[0x00]), Err(ShapeExprError::ZeroTag));
    assert_eq!(decode_dim(&[0x09, 0x00]), Err(ShapeExprError::ReservedTag { tag: 0x09 }));
    assert_eq!(decode_dim(&[0x03, 0x02, 0x00]),
               Err(ShapeExprError::TruncatedBlob { need: 9, got: 3 }));
    assert_eq!(decode_dim(&[0x04, 0x00, 0xAB]),
               Err(ShapeExprError::TrailingBytes { extra: 1 }));
}

// ---- §6.20-0007 primitive-floor shape rules ---------------------------------

#[test]
fn test_shape_expr_primitive_floor_rules() {
    // KISS-OPS-6.20-0007. Elementwise: output = SameAs(the broadcast operand).
    let ew = vec![vec![2i64, 3], vec![2i64, 3]];
    assert_eq!(eval_shape(&ShapeExpr::SameAs { operand: 0 }, &ew, &[]).unwrap(),
               ShapeValue::Concrete(vec![2, 3]));

    // Reduce: output = input shape with reduce_axes removed (keepdim=false) or
    // set to 1 (keepdim=true) — derived from op semantics, not a ShapeExpr attr.
    assert_eq!(reduce_shape(&[2, 3, 5], &[2], false), vec![2, 3]);       // drop last
    assert_eq!(reduce_shape(&[2, 3, 5], &[2], true), vec![2, 3, 1]);     // keepdim
    assert_eq!(reduce_shape(&[2, 3, 5], &[0, 2], false), vec![3]);       // multi-axis

    // The irreducible slice/iota offset case rides a DimExpr: rope half of the
    // last axis is Extent(x, last) ÷ 2.
    let half = Dim::Div(Box::new(Dim::Extent { operand: 0, axis: LAST }), Box::new(Dim::Const(2)));
    assert_eq!(eval_dim(&half, &[vec![4, 8]], &[]).unwrap(), DimValue::Concrete(4));
}

// ---- §6.20-0001 shape rule exists per op; companion to §6.4-0006 value oracle -

#[test]
fn test_shape_rule_exists_and_matches() {
    // KISS-OPS-6.20-0001: an op's shape rule is a total function from operand
    // shapes (+ attrs) to the output shape — the shape-side companion to the
    // §6.4-0006 value oracle. For a matmul cell [M,K]·[K,N] -> [M,N] the rule is
    // SameAs on neither operand but WithDim over both; here we check the derived
    // shape for the two representative primitives that need no free parameter.
    // elementwise add [2,3],[2,3] -> [2,3]
    assert_eq!(
        eval_shape(&ShapeExpr::SameAs { operand: 0 }, &[vec![2, 3], vec![2, 3]], &[]).unwrap(),
        ShapeValue::Concrete(vec![2, 3])
    );
    // reduce(sum, axis=last, nokd) [8,4096] -> [8]
    assert_eq!(reduce_shape(&[8, 4096], &[1], false), vec![8]);
}

// ---- KISS-CONTRACT-6.4-0011 Interface output shape ⟷ op shape rule -----------

#[test]
fn test_contract_output_shape_consistency() {
    // KISS-CONTRACT-6.4-0011: the Interface (§6.5) declared output shape MUST
    // equal the op's KISS-Ops shape rule (§6.20) over the operand shapes; a
    // disagreement is a typed decline. This is the shape-side companion to the
    // §6.4-0006 value oracle.

    // Consistent: a non-keepdim reduce over the last axis of [8,4096] yields [8],
    // and the contract declares output shape [8].
    let computed = ShapeValue::Concrete(reduce_shape(&[8, 4096], &[1], false));
    assert!(shape_consistent(&[8], &computed));

    // Inconsistent: the contract declares rank-2 [8,4096] for the same non-keepdim
    // reduce — the exact class of error no KISS clause caught before this tie.
    assert!(!shape_consistent(&[8, 4096], &computed));

    // A Gap (symbolic/data-dependent output) is not a hard inconsistency: it is a
    // surfaced gap, so a consumer cannot assert a mismatch it cannot compute.
    assert!(shape_consistent(&[8], &ShapeValue::Gap));
}

// ---- §6.20-0008 the output-shape ≠ operand-shape class (gather, contraction) --

#[test]
fn test_shape_expr_out_differs_from_operands() {
    // KISS-OPS-6.20-0008: the class the shape oracle most exists to catch — the
    // output shape equals NO operand's shape, so a `same_as(operand)` claim is a
    // real bug (the u32-gather-declaring-same_as(data) class).

    // gather / index_select / embedding: out = data shape with the gathered axis
    // replaced by the index shape. data [8,4096], axis 0, 1-D index [16] -> [16,4096].
    let g = gather_shape(&[8, 4096], &[16], 0);
    assert_eq!(g, vec![16, 4096]);
    // embedding: table [1000,64], axis 0, 2-D index [2,5] -> [2,5,64].
    assert_eq!(gather_shape(&[1000, 64], &[2, 5], 0), vec![2, 5, 64]);
    // The bug: advertising same_as(data) when the output differs from data — the
    // oracle rejects it (same_as(data) = [8,4096] ≠ the gather output [16,4096]).
    assert!(!shape_consistent(&[8, 4096], &ShapeValue::Concrete(g)));

    // matmul (contraction): role-vector-derived shape. lhs [8,4096]·rhs [4096,1024]
    // -> [8,1024]; batched [4,8,16]·[4,16,32] -> [4,8,32].
    let m = matmul_shape(&[8, 4096], &[4096, 1024]);
    assert_eq!(m, vec![8, 1024]);
    assert_eq!(matmul_shape(&[4, 8, 16], &[4, 16, 32]), vec![4, 8, 32]);
    // The output equals neither operand -> a same_as claim on either is caught.
    assert!(!shape_consistent(&[8, 4096], &ShapeValue::Concrete(m.clone())));
    assert!(!shape_consistent(&[4096, 1024], &ShapeValue::Concrete(m)));
}

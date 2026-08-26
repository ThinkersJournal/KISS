//! Per-output determinism class via full-sub-DAG propagation (KISS-OPS-6.0-0007).
//!
//! §6.0-0001 declares ONE class per op; 6.0-0007 refines it for multi-output ops:
//! each output's class is §6.0-0002..0005 applied to the sub-DAG that transitively
//! produces THAT output — the output node plus the transitive closure of the nodes
//! it consumes. Two propagation modes:
//!
//!   * a **value** output takes the most-permissive class (§6.0-0005) over its
//!     producing sub-DAG — exact-byte only if that whole sub-DAG is exact-byte;
//!   * a **selection** output — one reporting WHICH value(s) won a comparison (an
//!     argmax/argmin index, a sort permutation, a top_k index, a comparison mask, a
//!     one-hot, a tie count, a bucket/threshold index) — consumes the compared
//!     values in its sub-DAG by construction (Baracuda's enumeration-free framing:
//!     the rule is not a list of index ops, it is the structural fact that a
//!     selection reports which value won, so the values are inputs), and is
//!     exact-byte iff that sub-DAG is entirely exact-byte, else
//!     order-invariant/nondeterministic, **never ULP/tolerance** — a ≤k-ULP
//!     perturbation of a non-exact compared value can relocate the selection without
//!     bound, so a permutation over ULP-boundable keys is not itself ULP-boundable
//!     (kiss-ref's Ulp refinement, verified against `eval_recipe`).
//!
//! This is the KISS-side minimal realization of kiss-ref's `eval_recipe` per-node
//! determinism join + per-root comparator selection. It is not a full recipe
//! evaluator — it propagates classes over a decomposition DAG, which is what
//! 6.0-0007 and its Conform §6.8 mirror need.

use crate::determinism::{atom_determinism_class, class_permissiveness, is_selection_producer};
use crate::DeterminismClass;

/// A node in an op's reference-decomposition DAG. `local_class` is the node's own
/// determinism class (§6.0-0002..0004); `inputs` are the indices of the nodes whose
/// outputs it consumes (strictly-earlier indices — a topologically-ordered DAG);
/// `is_selection` marks a node that reports WHICH value(s) won a comparison.
#[derive(Debug, Clone)]
pub struct RecipeNode {
    pub local_class: DeterminismClass,
    pub inputs: Vec<usize>,
    pub is_selection: bool,
}

impl RecipeNode {
    /// A **value** node (not a selection) with the given local class and inputs.
    pub fn value(local_class: DeterminismClass, inputs: &[usize]) -> RecipeNode {
        RecipeNode { local_class, inputs: inputs.to_vec(), is_selection: false }
    }

    /// A **selection** node (reports which value(s) won a comparison) over `inputs`.
    /// Its own local class is exact-byte — the index arithmetic is deterministic;
    /// the class of its OUTPUT is decided by [`output_class`] from the values it
    /// consumes, not from this local class.
    pub fn selection(inputs: &[usize]) -> RecipeNode {
        RecipeNode { local_class: DeterminismClass::ExactByte, inputs: inputs.to_vec(), is_selection: true }
    }

    /// Build a node from a KISS-Ops **scalar-atom** op token (KISS-OPS-6.0-0008), deriving
    /// `is_selection` from the token via the **semantic** hook [`is_selection_producer`] — so
    /// the value/selection split comes from op semantics, never a hand-set flag: a comparison
    /// op built here is a selection whose OUTPUT class [`output_class`] escalates over a
    /// non-exact operand, even though the comparison atom's own class is exact-byte.
    ///
    /// `op` MUST be a scalar atom (the set [`atom_determinism_class`] covers — arithmetic,
    /// comparison, rounding, bitwise, transcendental, …). A **fold** op
    /// (`reduce`/`prefix_scan`/`matmul`/`scatter`), whose class needs a monoid (`op_true_class`),
    /// and a **leaf** (`Bind`/`Const`) are NOT scalar atoms and MUST be built with
    /// [`RecipeNode::value`]. Passing one here is misuse — its class cannot be inferred from
    /// the token alone — so it panics rather than silently defaulting to exact-byte.
    pub fn from_op(op: &str, inputs: &[usize]) -> RecipeNode {
        let local_class = match atom_determinism_class(op) {
            Some(c) => c,
            None => panic!(
                "RecipeNode::from_op: `{op}` is not a scalar atom — build a fold node with \
                 RecipeNode::value(op_true_class(..)) and a leaf with RecipeNode::value(..)"
            ),
        };
        RecipeNode { local_class, inputs: inputs.to_vec(), is_selection: is_selection_producer(op) }
    }
}

/// The most-permissive class (§6.0-0005 order) over the transitive producing sub-DAG
/// of `root` — `root` together with the transitive closure of the nodes it consumes.
fn join_over_subdag(nodes: &[RecipeNode], root: usize) -> DeterminismClass {
    let mut worst = DeterminismClass::ExactByte;
    let mut seen = vec![false; nodes.len()];
    let mut stack = vec![root];
    while let Some(i) = stack.pop() {
        assert!(
            i < nodes.len(),
            "RecipeNode index {} out of range (precondition: root and every input are strictly-earlier valid indices)",
            i
        );
        if core::mem::replace(&mut seen[i], true) {
            continue;
        }
        if class_permissiveness(nodes[i].local_class) > class_permissiveness(worst) {
            worst = nodes[i].local_class;
        }
        stack.extend(nodes[i].inputs.iter().copied());
    }
    worst
}

/// The per-output determinism class of the output at `root` (KISS-OPS-6.0-0007). A
/// value output is the [`join_over_subdag`]; a selection output is exact-byte iff
/// that join is exact-byte, else order-invariant/nondeterministic — never ULP.
///
/// Scope: the selection escalation is resolved at the queried `root` only. An intermediate
/// selection that feeds a downstream VALUE output contributes only its LOCAL class to that
/// output's join and is NOT re-escalated here — so a value output whose sub-DAG routes
/// through a comparison mask (e.g. `bit_and(cmp, cmp)` over a non-exact operand)
/// **under-reports** here (its join sees the comparison atoms' local exact-byte, landing
/// ULP from the non-exact operand). This per-**root** model DIVERGES from kiss-ref's
/// per-**node** `eval_recipe`, which escalates each selection at its node so downstream
/// combinators join the already-escalated order-invariant class (empirically confirmed:
/// `bit_and`/`Mul`(cmp_over_ulp, cmp_over_ulp) → OIN there). Per-node is the correct, safe
/// direction — per-root's ULP is an unsafe under-report (a comparator would demand a
/// tolerance a bit-flipping output cannot honor). Making this join per-node is a tracked
/// follow-up; KISS-OPS-6.0-0008 covers the mask-as-OUTPUT case (`root` is the comparison),
/// which is correct here.
pub fn output_class(nodes: &[RecipeNode], root: usize) -> DeterminismClass {
    let join = join_over_subdag(nodes, root);
    if nodes[root].is_selection && join != DeterminismClass::ExactByte {
        // A selection over a non-exact value is not tolerance-boundable: escalate any
        // non-exact producer (ULP or order-invariant) to order-invariant, skipping ULP.
        DeterminismClass::OrderInvariant
    } else {
        join
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DeterminismClass::*;

    /// A recipe DAG with several output roots over shared producers, spanning every
    /// propagation case 6.0-0007 must get right (kiss-ref's stacked witnesses +
    /// Baracuda's enumeration-free guard):
    ///   0 input                 value, exact-byte
    ///   1 reduce(max) over 0    value, exact-byte           [root: max VALUE = EB]
    ///   2 reduce(sum) over 0    value, order-invariant      [root: sum VALUE = OIN]
    ///   3 exp over 0            value, ULP/tolerance        [root: exp VALUE = Ulp]
    ///   4 argmax over 2 (sum)   selection                   [root: index over OIN = OIN]
    ///   5 mask   over 3 (exp)   selection                   [root: mask over Ulp = OIN, NOT Ulp — guard]
    ///   6 sort   over 3 (exp)   selection                   [root: index over Ulp = OIN, NOT Ulp]
    ///   7 argmax over 1 (max)   selection                   [root: index over all-EB = EB]
    fn recipe() -> Vec<RecipeNode> {
        vec![
            RecipeNode::value(ExactByte, &[]),      // 0 input
            RecipeNode::value(ExactByte, &[0]),     // 1 reduce(max)
            RecipeNode::value(OrderInvariant, &[0]), // 2 reduce(sum)
            RecipeNode::value(UlpTolerance, &[0]),  // 3 exp
            RecipeNode::selection(&[2]),            // 4 argmax over the sum
            RecipeNode::selection(&[3]),            // 5 mask over the exp
            RecipeNode::selection(&[3]),            // 6 sort over the exp
            RecipeNode::selection(&[1]),            // 7 argmax over the max
        ]
    }

    // Enforces KISS-OPS-6.0-0007: per-output determinism class by full-sub-DAG
    // propagation, with the value/selection split.
    #[test]
    fn test_ops_per_output_determinism_class() {
        let r = recipe();

        // Per-output DIFFERS on the same op — the point a single whole-op class can't
        // express: three value outputs, three classes.
        assert_eq!(output_class(&r, 1), ExactByte); // max value
        assert_eq!(output_class(&r, 2), OrderInvariant); // sum value
        assert_eq!(output_class(&r, 3), UlpTolerance); // exp value

        // A selection over an order-invariant value is order-invariant.
        assert_eq!(output_class(&r, 4), OrderInvariant); // argmax over sum

        // A MASK (not an index op) over a ULP value ALSO escalates to order-invariant,
        // NOT ULP — the enumeration-free regression guard (Baracuda): a mask is a
        // selection even though it is not an argmax/sort index, so narrowing the rule to
        // index-ops-only would treat this as a plain VALUE and land it at ULP, failing
        // the assert_ne below. It sits over the ULP value (not the OIN sum) on purpose:
        // over an OIN value the join is OIN either way and the guard would be inert.
        assert_eq!(output_class(&r, 5), OrderInvariant); // mask over exp
        assert_ne!(output_class(&r, 5), UlpTolerance);

        // A selection over a ULP value escalates to order-invariant, NOT ULP — the
        // plain join would give ULP; a permutation over Ulp-boundable keys is not
        // Ulp-boundable (kiss-ref). This is the case a naive join gets wrong.
        assert_eq!(output_class(&r, 6), OrderInvariant); // sort over exp
        assert_ne!(output_class(&r, 6), UlpTolerance);

        // A selection over an all-exact-byte sub-DAG stays exact-byte — the rule does
        // not over-escalate when nothing non-exact is present.
        assert_eq!(output_class(&r, 7), ExactByte); // argmax over max
    }

    // Enforces KISS-CONFORM-6.8-0011: the differential comparator is selected PER
    // OUTPUT from that output's class. A reassociated result within the band is
    // accepted for the order-invariant root and rejected for the exact-byte root —
    // a single whole-op comparator cannot do both.
    // Proven: KISS-CONFORM-6.8-0011 (subject: impl; ref: PROVEN_BATCH2.md)
    #[test]
    fn test_conform_per_output_comparator_selection() {
        use crate::structural::{compare_reduced_f32, reassoc_bound_f32};
        let r = recipe();
        let max_class = output_class(&r, 1); // exact-byte
        let sum_class = output_class(&r, 2); // order-invariant

        // A legitimately reassociated value: true 1e8, one order lands 1e8+8.
        let expected = 1e8f32;
        let actual = 1e8f32 + 8.0;
        let tol = 2.0 * reassoc_bound_f32(16, 1e8); // ~179 » 8

        // Per-output selection: the order-invariant (sum) output ACCEPTS it within
        // the band; the exact-byte (max) output REJECTS it byte-exactly.
        assert!(compare_reduced_f32(sum_class, actual, expected, tol, 0.0).is_ok());
        assert!(compare_reduced_f32(max_class, actual, expected, tol, 0.0).is_err());
    }

    // Enforces KISS-OPS-6.0-0008: the value-vs-selection classification of an output is
    // fixed by op SEMANTICS (the comparison op-family, via RecipeNode::from_op /
    // is_selection_producer), NOT by the lane an implementation computes it in. Nothing
    // here hand-tags a selection — from_op derives it from the op token. The witness is
    // kiss-ref's exposing DAG, which both oracles first got wrong by classing the mask
    // through the value lane.
    // Proven: KISS-OPS-6.0-0008 (subject: impl; ref: PROVEN_BATCH2.md)
    #[test]
    fn test_ops_comparison_mask_is_selection() {
        // kiss-ref's exposing recipe: n0 Bind (exact leaf), n1 Exp(n0) (ULP key),
        // n2 Const (exact), n3 cmp_lt(n1, n2) (b1 mask). from_op derives n3's selection
        // tag from the comparison op-family — it is not hand-set.
        let dag = vec![
            RecipeNode::value(ExactByte, &[]),      // n0 Bind
            RecipeNode::from_op("exp", &[0]),       // n1 Exp -> ULP
            RecipeNode::value(ExactByte, &[]),      // n2 Const
            RecipeNode::from_op("cmp_lt", &[1, 2]), // n3 cmp_lt mask -> selection
        ];
        // The mask is a selection over a ULP operand -> OIN, never ULP.
        assert_eq!(output_class(&dag, 3), OrderInvariant);
        assert_ne!(output_class(&dag, 3), UlpTolerance);

        // The deviation made executable: had n3 been classed as a plain VALUE (the
        // value-lane most-permissive join both oracles took), it would land ULP — the
        // exact bug 6.0-0008 forbids. This contrast is what gives "never ULP" teeth.
        let mut value_lane = dag.clone();
        value_lane[3] = RecipeNode::value(ExactByte, &[1, 2]); // same operands, no selection tag
        assert_eq!(output_class(&value_lane, 3), UlpTolerance);

        // Companion (a): a comparison over two exact-byte operands is a no-op escalation —
        // a selection over an all-exact sub-DAG stays exact-byte.
        let exact = vec![
            RecipeNode::value(ExactByte, &[]),      // Bind
            RecipeNode::value(ExactByte, &[]),      // Const
            RecipeNode::from_op("cmp_lt", &[0, 1]), // cmp_lt over exact -> selection, EB
        ];
        assert_eq!(output_class(&exact, 2), ExactByte);

        // Companion (b): the relu = max(a, 0) two-output crystallization over a ULP `a`.
        // Same op-graph, two output classes: the VALUE output max(a,0) stays ULP (differs
        // by <=ULP near a=0), the MASK output (a > 0) escalates to OIN (the sign bit flips
        // across backends near 0). Exactly what the per-output split (6.0-0007) buys.
        let relu = vec![
            RecipeNode::value(ExactByte, &[]),      // 0 Bind
            RecipeNode::from_op("exp", &[0]),       // 1 a = Exp -> ULP
            RecipeNode::value(ExactByte, &[]),      // 2 Const 0
            RecipeNode::value(ExactByte, &[1, 2]),  // 3 VALUE max(a,0): local EB, join -> ULP
            RecipeNode::from_op("cmp_gt", &[1, 2]), // 4 MASK a>0: selection -> OIN
        ];
        assert_eq!(output_class(&relu, 3), UlpTolerance); // value stays ULP
        assert_eq!(output_class(&relu, 4), OrderInvariant); // mask escalates
        assert_ne!(output_class(&relu, 4), UlpTolerance);
    }
}

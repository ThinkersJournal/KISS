//! Gate for KISS-CONFORM-§6.8-0013 — a declared blindness must be EXHIBITED.
//!
//! §6.8-0012 makes a comparison relation declare the dimensions along which two
//! *differing* inputs compare equal. A declaration is prose: it can be written for a
//! blindness the relation does not have, or quietly widened later, and nothing notices.
//! This gate turns each declared dimension into an ARTIFACT — a concrete pair of
//! distinct inputs landing on one token.
//!
//! EVERY CASE IS PAIRED. A collision assertion alone is satisfied by a derivation that
//! returns a constant, so each dimension also carries a DISCRIMINATION control: two
//! inputs that must NOT collide. Without it, "always return the ceiling" exhibits every
//! blindness perfectly and destroys the key.
//!
//! Not exhibited here: the contraction SIZE CLASS. §6.5 of KISS-Classify defines its
//! boundaries (`tiny` <= 8, `small` 9..=128, `mid` 129..=2048, `large` > 2048) but this
//! reference codec never DERIVES it — `SizeClass` is only ever parsed from a token or
//! written as a literal in a vector. A blindness whose derivation is unimplemented
//! cannot be exhibited, and saying so is the honest form; see the issue filed alongside.

use kiss_conformance::structure_key::*;

/// (1) Extent divisibility above the ceiling. Declared: `d16` absorbs every multiple of 16.
#[test]
fn test_conform_declared_blindness_is_exhibited() {
    // -- extent divisibility -------------------------------------------------------
    assert_ne!(16, 64, "vacuity guard: the two extents must actually differ");
    assert_eq!(
        derive_div_bucket(16),
        derive_div_bucket(64),
        "§6.8-0012 declares `d16` absorbs extents above the ceiling; 16 and 64 must collide"
    );
    assert_ne!(
        derive_div_bucket(8),
        derive_div_bucket(12),
        "DISCRIMINATION: below the ceiling the bucket must still separate — a derivation \
         returning a constant would satisfy every collision above and destroy the key"
    );

    // -- element count within a work class -----------------------------------------
    assert_eq!(
        derive_work_class(&[&[4]]),
        derive_work_class(&[&[32]]),
        "§6.8-0012 declares element count collapses within a work class; 4 and 32 are both warp"
    );
    assert_eq!(
        derive_work_class(&[&[2048]]),
        derive_work_class(&[&[99999]]),
        "and both are grid above the upper boundary"
    );
    assert_ne!(
        derive_work_class(&[&[32]]),
        derive_work_class(&[&[33]]),
        "DISCRIMINATION: the class boundary itself must still separate"
    );

    // -- vector width above its widest ---------------------------------------------
    assert_eq!(
        derive_vec_width(1, 8, Some(4), 64, false),
        derive_vec_width(1, 64, Some(4), 64, false),
        "§6.8-0012 declares width saturates; a wider-vectorizable extent lands on the same token"
    );
    assert_ne!(
        derive_vec_width(1, 8, Some(4), 64, false),
        derive_vec_width(1, 3, Some(4), 64, false),
        "DISCRIMINATION: a non-vectorizable extent must not land on the saturated token"
    );

    // -- extent/stride detail within a layout class --------------------------------
    assert_eq!(
        derive_layout_tag(&[4, 4], &[4, 1]),
        derive_layout_tag(&[8, 8], &[8, 1]),
        "§6.8-0012 declares extent/stride detail collapses; both are contiguous"
    );
    assert_ne!(
        derive_layout_tag(&[4, 4], &[4, 1]),
        derive_layout_tag(&[4, 4], &[1, 4]),
        "DISCRIMINATION: a genuinely different layout must not collapse into it"
    );

    // -- maximum touched offset below the index-width boundary ---------------------
    assert_eq!(
        derive_index_width(&[(&[10], &[1])]),
        derive_index_width(&[(&[1000], &[1])]),
        "§6.8-0012 declares offsets below 2^31 collapse; 10 and 1000 are both ix32"
    );
    assert_ne!(
        derive_index_width(&[(&[10], &[1])]),
        derive_index_width(&[(&[3_000_000_000], &[1])]),
        "DISCRIMINATION: the boundary itself must still separate ix32 from ix64"
    );
}

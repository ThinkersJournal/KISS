//! §6.8 transcendental **accuracy tier** — the D2 ceiling-retirement (#39).
//!
//! Retires the fixed per-atom ULP ceiling of the old §6.8-0001. The **declared
//! per-target accuracy tier** is now the SOLE conformance gate (§6.8-0001, KISS-Contract
//! §6.8-0002); the §6.8 table is an *informative advisory floor*, not a normative cap. A
//! truthful Khronos-conformant provider whose atom is looser than a table value MUST NOT
//! be rejected for it — that was the #39 defect (`atan` at 5 ULP is OpenCL-conformant yet
//! the 4-ULP ceiling rejected it; Vulkan `atan` is permitted 4096 ULP).
//!
//! The v1 tier is a **flat, argument-independent** `{max_ulp | max_relative |
//! max_absolute}` (§6.8-0006). The *argument-dependent / range-scoped* form (Vulkan
//! `exp = 3 + 2·|x|` ULP; `sin`/`cos` absolute-error-over-a-range) is a RESERVED post-v1
//! model, represented here as a distinct [`AccuracyModel`] variant the v1 gate rejects.

/// A declared per-target accuracy tier (§6.8-0001, KISS-Contract §6.8-0002): a tagged
/// quantity carrying at least one of `{max_ulp, max_relative, max_absolute}` measured
/// against a named wide-precision reference. `None` means "this dimension is left
/// unconstrained by the tier"; a kernel that meaningfully constrains only one bound leaves
/// the others `None`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AccuracyTier {
    pub max_ulp: Option<f64>,
    pub max_relative: Option<f64>,
    pub max_absolute: Option<f64>,
}

impl AccuracyTier {
    pub const fn ulp(bound: f64) -> Self {
        Self { max_ulp: Some(bound), max_relative: None, max_absolute: None }
    }
    pub const fn relative(bound: f64) -> Self {
        Self { max_ulp: None, max_relative: Some(bound), max_absolute: None }
    }
    pub const fn absolute(bound: f64) -> Self {
        Self { max_ulp: None, max_relative: None, max_absolute: Some(bound) }
    }

    /// §6.8-0001: a tier is well-formed iff it carries at least one bound.
    pub fn is_well_formed(&self) -> bool {
        self.max_ulp.is_some() || self.max_relative.is_some() || self.max_absolute.is_some()
    }

    /// The gate (§6.8-0001): the declared tier is the **sole** gate. A measured error is
    /// admitted iff it is within EVERY bound the tier declares — there is no fixed
    /// suite-wide ceiling to also satisfy, so a tier looser than the advisory floor gates
    /// exactly as declared. An ill-formed (no-bound) tier admits nothing.
    pub fn admits(&self, m: MeasuredError) -> bool {
        self.is_well_formed()
            && self.max_ulp.map_or(true, |b| m.ulp <= b)
            && self.max_relative.map_or(true, |b| m.relative <= b)
            && self.max_absolute.map_or(true, |b| m.absolute <= b)
    }
}

/// A measured error of an implementation against the audited reference, in each of the
/// three units a tier can bound. Only the units the tier declares are consulted.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeasuredError {
    pub ulp: f64,
    pub relative: f64,
    pub absolute: f64,
}

/// The accuracy model of an atom. v1 admits ONLY the flat tier (§6.8-0006); the
/// argument-dependent / range-scoped form is reserved for a named post-v1 extension and
/// is NOT a valid v1 declaration.
#[derive(Clone, Debug, PartialEq)]
pub enum AccuracyModel {
    /// §6.8-0006: the flat, argument-independent v1 tier.
    FlatV1(AccuracyTier),
    /// Reserved post-v1: accuracy as a function of the argument (e.g. `3 + 2·|x|` ULP) or
    /// an absolute error over a bounded range (e.g. `≤ 2⁻¹¹` on `[−π, π]`). Carried as an
    /// opaque descriptor; not admitted by the v1 gate.
    ReservedArgDependent(&'static str),
}

impl AccuracyModel {
    /// §6.8-0006: is this a valid v1 accuracy declaration? Only a well-formed flat tier is.
    pub fn is_v1(&self) -> bool {
        matches!(self, AccuracyModel::FlatV1(t) if t.is_well_formed())
    }
}

/// The §6.8 advisory-floor table (compute dtype ≥ 16-bit float). **INFORMATIVE ONLY** — a
/// reasonableness reference drawn from incumbent practice, NOT a gate. [`AccuracyTier::admits`]
/// MUST NOT consult it; it exists so a reader can see typical values, never to cap a
/// declaration. Retiring this table *as a gate* is the whole of #39 / D2.
pub fn advisory_floor_ulp(atom: &str) -> Option<f64> {
    match atom {
        "sqrt" => Some(2.0),
        "exp" | "log" | "sin" | "cos" | "atan" | "atan2" | "erf" => Some(4.0),
        "lgamma" => Some(8.0),
        _ => None,
    }
}

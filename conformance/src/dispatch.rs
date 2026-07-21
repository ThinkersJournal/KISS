//! §6.6 Dispatch — the geometry-agnostic (optional-Dispatch) model and the thread-index-free
//! structural declaration of `thread_mapping`/`addressing_rule` (D3 / #43).
//!
//! Retires the requirement that every kernel declare a Dispatch section. A **geometry-agnostic**
//! kernel (grid-stride, host-computed `Dim3`) declares NO Dispatch section (§6.6-0007) — launch
//! geometry is the executor's, not the contract's. A kernel that DOES declare Dispatch carries
//! the five fields (§6.6-0001), of which `thread_mapping`/`addressing_rule` are **thread-index-free
//! structural declarations** (§6.6-0004): the per-thread element index is a grid-stride *semantic*,
//! not a declared expression, and the §6.6-0006 grammar carries no per-thread index symbol. A
//! richer thread-mapping form (tile-mapping / bound `gid`) is **reserved post-v1** (§6.6-0008).
//!
//! This module owns the geometry/optionality model; the §6.6-0006 *expression grammar* itself is
//! validated elsewhere (KISS-CONTRACT-6.6-0006). The two are complementary: the grammar is
//! deliberately thread-index-free, which is exactly the property this module checks here.

/// Per-thread index symbols reserved to the post-v1 richer thread-mapping form (§6.6-0008); a v1
/// `thread_mapping`/`addressing_rule` that references one is rejected — the v1 §6.6-0006 grammar is
/// thread-index-free by design.
pub const RESERVED_THREAD_INDEX_SYMBOLS: &[&str] =
    &["gid", "tid", "lane", "gtid", "threadidx", "blockidx", "warp", "subgroup"];

/// Named richer-mapping forms reserved to §6.6-0008 (post-v1); likewise not a valid v1 declaration.
pub const RESERVED_MAPPING_FORMS: &[&str] = &["tile_mapping", "warp_tile", "lane_tile"];

/// The five Dispatch derivation fields (§6.6-0001), each a §6.6-0006 grammar expression.
#[derive(Clone, Debug, PartialEq)]
pub struct DispatchFields {
    pub invocation_domain: String,
    pub workgroup_sizing: String,
    pub count_to_grid: String,
    pub thread_mapping: String,
    pub addressing_rule: String,
}

/// A kernel's Dispatch section (§6.6). `GeometryAgnostic` is the absent sentinel (§6.6-0007) — no
/// launch geometry declared; `Declared` carries all five fields (§6.6-0001).
#[derive(Clone, Debug, PartialEq)]
pub enum DispatchModel {
    /// §6.6-0007: geometry-agnostic kernel — no Dispatch section.
    GeometryAgnostic,
    /// §6.6-0001: a declared launch geometry (all five fields).
    Declared(DispatchFields),
}

/// Extract identifier tokens (`[A-Za-z_][A-Za-z0-9_]*`, lowercased) from an expression, dropping
/// pure-numeric tokens. Word-boundaried so a launch scalar like `idx_extents` is not mistaken for
/// an `idx`/`tid` thread index.
fn ident_tokens(expr: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in expr.chars() {
        if c.is_ascii_alphanumeric() || c == '_' {
            cur.push(c.to_ascii_lowercase());
        } else if !cur.is_empty() {
            out.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out.into_iter().filter(|t| !t.chars().all(|c| c.is_ascii_digit())).collect()
}

/// True iff `expr` references no reserved per-thread index symbol or reserved mapping form — i.e.
/// it is a v1 thread-index-free declaration (§6.6-0004 / §6.6-0008).
pub fn is_thread_index_free(expr: &str) -> bool {
    !ident_tokens(expr).iter().any(|t| {
        RESERVED_THREAD_INDEX_SYMBOLS.contains(&t.as_str())
            || RESERVED_MAPPING_FORMS.contains(&t.as_str())
    })
}

impl DispatchModel {
    /// Validate the Dispatch model (§6.6-0001/-0004/-0007/-0008):
    /// - `GeometryAgnostic` is always accepted (§6.6-0007) — no geometry required.
    /// - `Declared` requires every field non-empty (no **partial** section, §6.6-0007), and
    ///   `thread_mapping`/`addressing_rule` MUST be **thread-index-free** (§6.6-0004 / §6.6-0008).
    pub fn validate(&self) -> Result<(), String> {
        let f = match self {
            DispatchModel::GeometryAgnostic => return Ok(()),
            DispatchModel::Declared(f) => f,
        };
        for (name, val) in [
            ("invocation_domain", &f.invocation_domain),
            ("workgroup_sizing", &f.workgroup_sizing),
            ("count_to_grid", &f.count_to_grid),
            ("thread_mapping", &f.thread_mapping),
            ("addressing_rule", &f.addressing_rule),
        ] {
            if val.trim().is_empty() {
                return Err(format!(
                    "§6.6-0007: partial Dispatch section — `{name}` is empty; declare all five \
                     fields or use the geometry-agnostic absent sentinel"
                ));
            }
        }
        for (name, val) in
            [("thread_mapping", &f.thread_mapping), ("addressing_rule", &f.addressing_rule)]
        {
            if !is_thread_index_free(val) {
                return Err(format!(
                    "§6.6-0004/-0008: `{name}` references a per-thread index / reserved mapping \
                     symbol — the v1 grammar is thread-index-free; the per-thread index is the \
                     grid-stride semantic, and the richer form is reserved post-v1"
                ));
            }
        }
        Ok(())
    }
}

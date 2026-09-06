//! §6.6 Dispatch — the geometry-agnostic (optional-Dispatch) model and the thread-index-free
//! structural declaration of `thread_mapping`/`addressing_rule` (D3 / #43).
//!
//! Retires the requirement that every kernel declare launch GEOMETRY. The Dispatch section is
//! **always present** (§6.11-0004 requires all seven blocks); a **geometry-agnostic** kernel
//! (grid-stride, host-computed `Dim3`) carries the sentinel line `dispatch_model = geometry-agnostic`
//! in place of the five geometry fields (§6.6-0007, as resolved by #456/#480) — launch geometry is
//! then the executor's, not the contract's. A kernel that DOES declare geometry carries
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

/// A kernel's Dispatch section (§6.6). The section is **always present** (§6.11-0004 requires all
/// seven blocks); what varies is its content: `GeometryAgnostic` carries the sentinel line
/// `dispatch_model = geometry-agnostic` (§6.6-0007, as resolved by #456/#480), `Declared` carries
/// all five geometry fields (§6.6-0001).
#[derive(Clone, Debug, PartialEq)]
pub enum DispatchModel {
    /// §6.6-0007: geometry-agnostic kernel — no launch GEOMETRY declared.
    ///
    /// ⚠️ NOT "no Dispatch section", which is what this comment used to say. §6.6-0007 is
    /// explicit, in the sentence written to prevent exactly that reading: "The Dispatch
    /// section is still **present** — §6.11-0004 requires all seven section blocks — so it
    /// is the section's *content* that is the sentinel, never the section that is absent."
    /// A reader following the old comment renders SIX blocks and silently omits Dispatch.
    GeometryAgnostic,
    /// §6.6-0001: a declared launch geometry (all five fields).
    Declared(DispatchFields),
}

/// The Dispatch section id and name (§6.11-0004: block 4, lowercase name).
pub const DISPATCH_SECTION_ID: u8 = 4;
pub const DISPATCH_SECTION_NAME: &str = "dispatch";
/// The carried geometry-agnostic sentinel value (§6.6-0007), keyed under `dispatch_model`.
pub const GEOMETRY_AGNOSTIC_SENTINEL: &str = "geometry-agnostic";
/// The five geometry field keys, in the pinned §6.6-0001 / §6.11-0005 order.
pub const GEOMETRY_FIELD_KEYS: [&str; 5] = [
    "invocation_domain",
    "workgroup_sizing",
    "count_to_grid",
    "thread_mapping",
    "addressing_rule",
];

/// A typed decline from parsing a Dispatch section block (§6.6-0001/§6.6-0007). Never a panic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DispatchDecline {
    /// The block does not begin with `[section:4:dispatch]`.
    BadHeading,
    /// The block carried no field lines (neither the five fields nor the sentinel).
    Empty,
    /// A field key not in the schema (not one of the five, and not `dispatch_model`).
    UnknownField(String),
    /// The five-geometry-field arm is incomplete or out of order (a proper subset, §6.6-0001).
    PartialGeometry,
    /// `dispatch_model` alongside one or more geometry fields — the two arms are exclusive.
    SentinelWithGeometry,
    /// `dispatch_model = <v>` where `<v>` is not `geometry-agnostic`.
    BadSentinelValue(String),
    /// A line that is not a `key = value` field line.
    MalformedLine(String),
    /// The document body carries no `[section:4:dispatch]` block at all (§6.11-0004 requires all
    /// seven section blocks). This is the carried-vs-absent discriminator: the geometry-agnostic
    /// contract is a PRESENT block carrying the sentinel, not an omitted block.
    MissingSection,
}

impl DispatchModel {
    /// Render the Dispatch section block (§6.11-0004, block id 4): the pinned heading line, then
    /// either the five geometry fields in §6.6-0001 order (`Declared`) or the single carried
    /// sentinel line `dispatch_model = geometry-agnostic` (`GeometryAgnostic`). The section is
    /// always present — this is the carried resolution (#456/#480), never an absent block.
    pub fn render_block(&self) -> Vec<u8> {
        use crate::contract::{render_block, Value};
        let fields: Vec<(&str, Value)> = match self {
            DispatchModel::GeometryAgnostic => {
                vec![("dispatch_model", Value::Str(GEOMETRY_AGNOSTIC_SENTINEL.to_string()))]
            }
            DispatchModel::Declared(f) => vec![
                ("invocation_domain", Value::Str(f.invocation_domain.clone())),
                ("workgroup_sizing", Value::Str(f.workgroup_sizing.clone())),
                ("count_to_grid", Value::Str(f.count_to_grid.clone())),
                ("thread_mapping", Value::Str(f.thread_mapping.clone())),
                ("addressing_rule", Value::Str(f.addressing_rule.clone())),
            ],
        };
        render_block(DISPATCH_SECTION_ID, DISPATCH_SECTION_NAME, &fields)
    }
}

/// Parse a Dispatch section block into a `DispatchModel`, enforcing the §6.6-0001 either/or schema:
/// EXACTLY the five geometry fields in order, OR EXACTLY the one `dispatch_model = geometry-agnostic`
/// sentinel line. A proper subset, a wrong order, an unknown field, `dispatch_model` beside a
/// geometry field, or a bad sentinel value each declines — never a panic (§6.6-0007's no-partial).
pub fn parse_dispatch_block(block: &[u8]) -> Result<DispatchModel, DispatchDecline> {
    let fields = parse_dispatch_fields(block)?;
    if fields.iter().any(|(k, _)| k == "dispatch_model") {
        parse_sentinel_arm(&fields)
    } else {
        parse_geometry_arm(&fields)
    }
}

/// Parse a Dispatch block's heading + `key = value` field lines into ordered `(key, value)` pairs.
/// The `key = value` form is pinned by §6.11-0001 (one space, ASCII `=`, one space); a line not in
/// that exact form declines `MalformedLine` rather than being leniently repaired — leniency would
/// accept input the spec pins as malformed.
fn parse_dispatch_fields(block: &[u8]) -> Result<Vec<(String, String)>, DispatchDecline> {
    let text = std::str::from_utf8(block).map_err(|_| DispatchDecline::BadHeading)?;
    let mut lines = text.lines();
    match lines.next() {
        Some(h) if h == format!("[section:{DISPATCH_SECTION_ID}:{DISPATCH_SECTION_NAME}]") => {}
        _ => return Err(DispatchDecline::BadHeading),
    }
    let mut fields: Vec<(String, String)> = Vec::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        match line.split_once(" = ") {
            Some((k, v)) => fields.push((k.to_string(), v.to_string())),
            None => return Err(DispatchDecline::MalformedLine(line.to_string())),
        }
    }
    if fields.is_empty() {
        return Err(DispatchDecline::Empty);
    }
    Ok(fields)
}

/// The `dispatch_model = geometry-agnostic` arm (§6.6-0007): exactly one field, that key, that value.
fn parse_sentinel_arm(fields: &[(String, String)]) -> Result<DispatchModel, DispatchDecline> {
    if fields.iter().any(|(k, _)| GEOMETRY_FIELD_KEYS.contains(&k.as_str())) {
        return Err(DispatchDecline::SentinelWithGeometry);
    }
    if fields.len() != 1 || fields[0].0 != "dispatch_model" {
        return Err(DispatchDecline::UnknownField(
            fields.iter().find(|(k, _)| k != "dispatch_model").map(|(k, _)| k.clone()).unwrap_or_default(),
        ));
    }
    if fields[0].1 != GEOMETRY_AGNOSTIC_SENTINEL {
        return Err(DispatchDecline::BadSentinelValue(fields[0].1.clone()));
    }
    Ok(DispatchModel::GeometryAgnostic)
}

/// The five-geometry-field arm (§6.6-0001): exactly the five keys, in the pinned order.
fn parse_geometry_arm(fields: &[(String, String)]) -> Result<DispatchModel, DispatchDecline> {
    for (k, _) in fields {
        if !GEOMETRY_FIELD_KEYS.contains(&k.as_str()) {
            return Err(DispatchDecline::UnknownField(k.clone()));
        }
    }
    let keys: Vec<&str> = fields.iter().map(|(k, _)| k.as_str()).collect();
    if keys != GEOMETRY_FIELD_KEYS {
        return Err(DispatchDecline::PartialGeometry);
    }
    let g = |i: usize| fields[i].1.clone();
    Ok(DispatchModel::Declared(DispatchFields {
        invocation_domain: g(0),
        workgroup_sizing: g(1),
        count_to_grid: g(2),
        thread_mapping: g(3),
        addressing_rule: g(4),
    }))
}

/// Extract the `[section:4:dispatch]` block from a whole contract document body and parse it. The
/// block runs from its heading to the next `[section:` heading or end of body. A body that carries
/// NO dispatch section declines `MissingSection` — the carried resolution (#456/#480) requires the
/// block to be present (§6.11-0004's seven blocks), so an OMITTED Dispatch block is a decline, not
/// a silently-accepted geometry-agnostic kernel (the old absent reading).
pub fn parse_dispatch_from_document(body: &[u8]) -> Result<DispatchModel, DispatchDecline> {
    let text = std::str::from_utf8(body).map_err(|_| DispatchDecline::BadHeading)?;
    let heading_line = format!("[section:{DISPATCH_SECTION_ID}:{DISPATCH_SECTION_NAME}]\n");
    // LINE-ANCHORED: the heading line is at body start or immediately after an LF, so the search
    // cannot match `[section:4:dispatch]` occurring inside a field value.
    let start = if text.starts_with(&heading_line) {
        0
    } else if let Some(i) = text.find(&format!("\n{heading_line}")) {
        i + 1
    } else {
        return Err(DispatchDecline::MissingSection);
    };
    // the block ends at the next line-anchored section heading, or end of body.
    let after = start + heading_line.len();
    let end = text[after..].find("\n[section:").map(|i| after + i).unwrap_or(text.len());
    parse_dispatch_block(text[start..end].as_bytes())
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
                     fields or carry the `dispatch_model = geometry-agnostic` sentinel line"
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

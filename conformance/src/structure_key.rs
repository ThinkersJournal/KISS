//! Reference codec for the KISS-Classify `structure_key` token (Classify §6.7).
//!
//! The token is the sole normative wire form of a specialization-cell identity
//! (§6.7-0011), a `|`-separated string:
//! ```text
//!   sk<ver> | <op_family> | <dtype> | <target> | <index_width> | <work_class>
//!           | r<rank> | <op0>;<op1>;… | <reduce>
//!           [ | c<m><n><k>/<kdiv>[/b<class>]/<wdt>/<acc>/<out>/<mp> ]  (gem only, sk3)
//! ```
//! where each `<opI>` is `<contig>/<bcasthex>/<vec>/<div>/<flip>`. `to_token`
//! and `from_token` round-trip byte-identically (§6.7-0008); every hex mask is
//! lowercase, zero-padded to two digits (§6.7-0010); a producer emits `rall` /
//! `rlast` for the all-axes / trailing cases, never the equivalent `x<hh>`
//! (§6.7-0005). A malformed token is rejected with a typed decline (§6.7-0009).

pub const SCHEMA_VERSION: u32 = 3;

/// The closed op-family-tag set at this schema version — exactly the 24 codes of
/// Classify §6.5-0006. A token whose op-family field is outside this set is
/// rejected (a reader must not silently encode an "unknown" code).
pub const OP_FAMILIES: [&str; 24] = [
    "gem", "idx", "une", "emb", "bin", "shp", "ter", "srt", "gat", "qnt", "red", "rnd",
    "scn", "los", "nrm", "seg", "sft", "img", "cnv", "fft", "pol", "lin", "att", "moe",
];

/// The closed dtype-token set — the 22 tokens of Classify §6.1. sk3 makes the FP8
/// spellings variant-explicit: `e4m3fn` = OCP finite/no-inf (max 448); `e4m3fnuz` =
/// AMD reserved (distinct bias, no −0); `e5m2` = IEEE inf/NaN (max 57344); `e5m2fnuz`
/// = AMD reserved. The ambiguous bare `e4m3` is gone; an inf-carrying `e4m3`, if ever
/// needed, is added additively with its own explicit spelling, never the bare token.
pub const DTYPES: [&str; 22] = [
    "f16", "bf16", "f32", "f64", "s8", "s16", "u8", "u16", "i32", "i64", "u32", "u64", "bool",
    "e4m3fn", "e4m3fnuz", "e5m2", "e5m2fnuz", "s4", "u4", "b1", "c32", "c64",
];

/// The two **reserved** members of [`DTYPES`] (Classify §6.1-0001): part of the
/// closed vocabulary so the spellings are pinned now, but with **no computation
/// semantics at this schema version** — a `structure_key` using one in any dtype
/// position is answered with the typed [`KeyDecline::ReservedDtype`], distinct
/// from the unknown-token decline. Activation is a future additive schema event.
pub const RESERVED_DTYPES: [&str; 2] = ["e4m3fnuz", "e5m2fnuz"];

// ---- small enum codecs -------------------------------------------------------

macro_rules! code_enum {
    ($name:ident { $($variant:ident = $code:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name { $($variant),+ }
        impl $name {
            pub fn code(self) -> &'static str { match self { $($name::$variant => $code),+ } }
            pub fn parse(s: &str) -> Option<Self> { match s { $($code => Some($name::$variant),)+ _ => None } }
        }
    };
}

code_enum!(Contig { Contiguous = "co", InnerContiguous = "ic", Strided = "st", Broadcast = "br" });
code_enum!(VecWidth { V1 = "v1", V2 = "v2", V4 = "v4", V8 = "v8" });
code_enum!(DivBucket { D16 = "d16", D8 = "d8", D4 = "d4", D2 = "d2", Da = "da" });
code_enum!(WorkClass { Warp = "warp", Block = "block", Grid = "grid" });
code_enum!(SizeClass { Tiny = "t", Small = "s", Medium = "m", Large = "l" });
// sk3: the math-precision code in the gem contraction field. Codes never begin with
// `b` — that prefix is reserved for the conditionally-present batch coordinate — so
// the geometry and precision groups never collide in spelling.
code_enum!(MathPrecision { Stable = "st", ReducedMantissa = "rm" });

/// Derive an operand's vector-access width from its innermost-active-axis facts
/// per KISS-Classify §6.5-0009(c) and the §6.5-0013 forward-unit-stride
/// precondition. This is the reference derivation for the non-reduction,
/// non-broadcast branch: parts (a)/(b) of §6.5-0009 (broadcast `layout_tag`,
/// reduced/scan innermost axis) are decided by the caller and reach this function
/// only as `any_axis_broadcast` or by not being called.
///
/// Arguments are the innermost active axis's signed stride (§6.3-0003, elements)
/// and extent (elements), the dtype's storage width in bytes (`None` for a
/// sub-byte dtype, which derives `v1`), the operand's base-pointer `alignment` in
/// bytes, and whether any axis of the operand broadcasts.
///
/// §6.5-0013: `vL` with `L > 1` requires a **forward-unit** innermost stride
/// (`stride == +1`) and no broadcast axis; a flipped (`stride == -1`) or strided
/// (`|stride| > 1`) innermost axis derives `v1`. §6.5-0009(c): the alignment gate
/// is **exact-modulo** (`alignment mod (L · bytes) == 0`), not a power-of-two
/// floor; an `alignment` of `0` (unspecified) cannot honor a packed load and
/// derives `v1`.
#[must_use]
pub fn derive_vec_width(
    inner_stride: i64,
    inner_extent: i64,
    dtype_storage_bytes: Option<u32>,
    alignment: u32,
    any_axis_broadcast: bool,
) -> VecWidth {
    // Sub-byte dtype: storage under one byte never vectorizes (§6.5-0009(c)).
    let Some(dsz) = dtype_storage_bytes else {
        return VecWidth::V1;
    };
    // §6.5-0013 precondition: forward-unit inner stride, no broadcast, else v1.
    if inner_stride != 1 || any_axis_broadcast {
        return VecWidth::V1;
    }
    // §6.5-0009(c): an unspecified (0) base-pointer alignment cannot honor a
    // packed load.
    if alignment == 0 {
        return VecWidth::V1;
    }
    let ext = inner_extent.max(0) as u64;
    // §6.5-0009(c): a zero-extent innermost axis derives `v1`. The divisibility
    // test below is VACUOUSLY true at E=0 (every L divides 0), which would elect
    // the largest L the byte cap and alignment allow — but a zero-length run has
    // nothing to load. Consistent with §6.5-0012 bucketing E=0 as `da`.
    if ext == 0 {
        return VecWidth::V1;
    }
    let dsz = u64::from(dsz);
    let align = u64::from(alignment);
    for (l, w) in [(8u64, VecWidth::V8), (4, VecWidth::V4), (2, VecWidth::V2)] {
        let vbytes = l * dsz;
        // byte cap, exact-modulo alignment gate, extent divisibility.
        if vbytes <= 16 && align % vbytes == 0 && ext % l == 0 {
            return w;
        }
    }
    VecWidth::V1
}

/// The `<flip>` code: `f` (natural) or `r` (flipped) — §6.7 grammar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperandSubKey {
    pub contig: Contig,
    pub bcast_mask: u8,
    pub vec: VecWidth,
    pub div: DivBucket,
    pub flipped: bool,
}

/// Field 8, the four distinctly-encoded reduce values (§6.7-0005).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reduce {
    None,
    All,
    Trailing,
    Subset(u8),
}

/// The optional dense-contraction field (§6.7-0006), present IFF `op_family == gem`.
/// sk3 grows it from `c<m><n><k>/<kdiv>` to
/// `c<m><n><k>/<kdiv>[/b<class>]/<wdt>/<acc>/<out>/<mp>`: a geometry group
/// (`m`/`n`/`k`, `k_div`, and the conditionally-present `batch` size-class) followed
/// by a precision group (weight / accumulator / output dtypes + the MathPrecision
/// code, in input→compute→output dataflow order). `batch` is present IFF the cell is
/// batched — a non-batched cell omits it entirely, so a non-batched token carries no
/// batch coordinate at all (the general optional-coordinate rule).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Contraction {
    pub m: SizeClass,
    pub n: SizeClass,
    pub k: SizeClass,
    pub k_div: DivBucket,
    /// Conditionally present: `Some(class)` iff batched (serialized `b<class>`).
    pub batch: Option<SizeClass>,
    /// Weight (operand-1) dtype token, from the closed §6.1 set.
    pub wdt: String,
    /// Accumulator / compute dtype token, from the closed §6.1 set.
    pub acc: String,
    /// Output dtype token, from the closed §6.1 set.
    pub out: String,
    /// Math-precision code (`st` bit-stable / `rm` reduced-mantissa).
    pub mp: MathPrecision,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureKey {
    pub op_family: String,
    pub dtype: String,
    pub target: String,
    pub index_width: String,
    pub work_class: WorkClass,
    pub rank: u32,
    pub operands: Vec<OperandSubKey>,
    pub reduce: Reduce,
    pub contraction: Option<Contraction>,
}

/// A typed decline from `from_token` (§6.7-0009): never a panic or OOB read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyDecline {
    WrongFieldCount { got: usize },
    BadVersionPrefix,
    BadRank,
    BadWorkClass,
    BadOperandSubKey,
    BadReduceField,
    BadContractionField,
    UppercaseOrWidthHex,
    /// Op-family code outside the closed §6.5-0006 set.
    UnknownOpFamily,
    /// Dtype token outside the closed §6.1 set.
    UnknownDtype,
    /// Dtype token inside the closed §6.1 vocabulary but **reserved** at this
    /// schema version (`e4m3fnuz` / `e5m2fnuz`): recognized on parse — a reader
    /// distinguishes it from an unknown token — but its use MUST typed-decline
    /// until a future schema version activates it (§6.1-0001).
    ReservedDtype,
    /// Token length is `0` or exceeds `MAX_STRUCTURE_KEY_LEN` (§6.4-0004).
    TokenLengthOutOfBound { len: usize },
    /// More than `MAX_OPERANDS` per-operand sub-keys (§6.4-0002).
    TooManyOperands { got: usize },
    /// `target_capability` is not `<namespace>:<capability-set>` (§6.8-0001).
    BadTargetGrammar,
    /// `target_capability` contains a byte outside the §6.8-0005 charset — a
    /// `structure_key` field separator (`;` or `/`), a whitespace/control byte
    /// (`0x00`–`0x20`, `0x7f`), or a non-ASCII byte (`>= 0x80`) — so it could not
    /// embed in the token as one unambiguous field.
    BadTargetCharset,
    /// The contraction field's presence does not match `op_family` (§6.6-0010):
    /// present IFF `op_family == gem`.
    ContractionPresenceMismatch,
    /// The reduce field is non-`-` for an `op_family` other than `red` (§6.6-0017).
    ReduceNotGatedToRed,
}

/// The pinned closed-set bounds (§6.4). Named so a reader can reject an
/// over-large key without allocating on the unchecked length.
pub const MAX_OPERANDS: usize = 8;
pub const MAX_STRUCTURE_KEY_LEN: usize = 4096;
/// The pinned rank bound (§6.4-0001): `rank` is the inclusive range `0..=MAX_RANK`.
pub const MAX_RANK: u32 = 8;

// ---- serialize (to_token) ---------------------------------------------------

fn hex2(v: u8) -> String {
    format!("{v:02x}") // lowercase, 2 digits (§6.7-0010)
}

impl OperandSubKey {
    fn to_field(self) -> String {
        format!(
            "{}/{}/{}/{}/{}",
            self.contig.code(),
            hex2(self.bcast_mask),
            self.vec.code(),
            self.div.code(),
            if self.flipped { "r" } else { "f" }
        )
    }
}

impl Reduce {
    fn to_field(self) -> String {
        match self {
            Reduce::None => "-".to_string(),
            Reduce::All => "rall".to_string(),
            Reduce::Trailing => "rlast".to_string(),
            Reduce::Subset(mask) => format!("x{}", hex2(mask)),
        }
    }
}

impl StructureKey {
    /// Serialize to the canonical `structure_key` token (§6.7).
    pub fn to_token(&self) -> String {
        let operands = self
            .operands
            .iter()
            .map(|o| o.to_field())
            .collect::<Vec<_>>()
            .join(";");
        let mut token = format!(
            "sk{}|{}|{}|{}|{}|{}|r{}|{}|{}",
            SCHEMA_VERSION,
            self.op_family,
            self.dtype,
            self.target,
            self.index_width,
            self.work_class.code(),
            self.rank,
            operands,
            self.reduce.to_field(),
        );
        if let Some(c) = &self.contraction {
            // geometry group, then the conditionally-present batch coordinate, then
            // the precision group — joined by `/` (§6.7-0006, sk3).
            let mut parts = vec![
                format!("c{}{}{}", c.m.code(), c.n.code(), c.k.code()),
                c.k_div.code().to_string(),
            ];
            if let Some(cls) = c.batch {
                parts.push(format!("b{}", cls.code()));
            }
            parts.push(c.wdt.clone());
            parts.push(c.acc.clone());
            parts.push(c.out.clone());
            parts.push(c.mp.code().to_string());
            token.push('|');
            token.push_str(&parts.join("/"));
        }
        token
    }
}

// ---- parse (from_token) -----------------------------------------------------

fn parse_hex2(s: &str) -> Result<u8, KeyDecline> {
    // exactly two lowercase hex digits (§6.7-0010)
    if s.len() != 2 || !s.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()) {
        return Err(KeyDecline::UppercaseOrWidthHex);
    }
    u8::from_str_radix(s, 16).map_err(|_| KeyDecline::UppercaseOrWidthHex)
}

fn parse_operand(s: &str) -> Result<OperandSubKey, KeyDecline> {
    let p: Vec<&str> = s.split('/').collect();
    if p.len() != 5 {
        return Err(KeyDecline::BadOperandSubKey);
    }
    let contig = Contig::parse(p[0]).ok_or(KeyDecline::BadOperandSubKey)?;
    let bcast_mask = parse_hex2(p[1])?;
    let vec = VecWidth::parse(p[2]).ok_or(KeyDecline::BadOperandSubKey)?;
    let div = DivBucket::parse(p[3]).ok_or(KeyDecline::BadOperandSubKey)?;
    let flipped = match p[4] {
        "f" => false,
        "r" => true,
        _ => return Err(KeyDecline::BadOperandSubKey),
    };
    Ok(OperandSubKey { contig, bcast_mask, vec, div, flipped })
}

fn parse_reduce(s: &str) -> Result<Reduce, KeyDecline> {
    match s {
        "-" => Ok(Reduce::None),
        "rall" => Ok(Reduce::All),
        "rlast" => Ok(Reduce::Trailing),
        _ => {
            let rest = s.strip_prefix('x').ok_or(KeyDecline::BadReduceField)?;
            Ok(Reduce::Subset(parse_hex2(rest).map_err(|_| KeyDecline::BadReduceField)?))
        }
    }
}

fn parse_contraction(s: &str) -> Result<Contraction, KeyDecline> {
    // sk3: c<m><n><k>/<kdiv>[/b<class>]/<wdt>/<acc>/<out>/<mp>
    // 6 `/`-parts when non-batched, 7 when batched (the batch coordinate is the only
    // conditionally-present one; its presence is unambiguous by part count + the `b`
    // prefix, which no dtype token or `<mp>` code shares).
    let body = s.strip_prefix('c').ok_or(KeyDecline::BadContractionField)?;
    let parts: Vec<&str> = body.split('/').collect();
    if parts.len() != 6 && parts.len() != 7 {
        return Err(KeyDecline::BadContractionField);
    }
    // geometry: <m><n><k> (exactly 3 size-class chars) then <kdiv>.
    let sc: Vec<char> = parts[0].chars().collect();
    if sc.len() != 3 {
        return Err(KeyDecline::BadContractionField);
    }
    let m = SizeClass::parse(&sc[0].to_string()).ok_or(KeyDecline::BadContractionField)?;
    let n = SizeClass::parse(&sc[1].to_string()).ok_or(KeyDecline::BadContractionField)?;
    let k = SizeClass::parse(&sc[2].to_string()).ok_or(KeyDecline::BadContractionField)?;
    let k_div = DivBucket::parse(parts[1]).ok_or(KeyDecline::BadContractionField)?;
    // conditionally-present batch coordinate `b<class>` (present iff 7 parts).
    let (batch, rest) = if parts.len() == 7 {
        let cls = parts[2]
            .strip_prefix('b')
            .and_then(SizeClass::parse)
            .ok_or(KeyDecline::BadContractionField)?;
        (Some(cls), 3usize)
    } else {
        (None, 2usize)
    };
    // precision group: <wdt> <acc> <out> from the closed dtype set, then <mp>.
    let wdt = parts[rest];
    let acc = parts[rest + 1];
    let out = parts[rest + 2];
    for dt in [wdt, acc, out] {
        // reserved before unknown: RESERVED_DTYPES ⊂ DTYPES, and the two
        // declines are distinct — recognized-reserved vs unknown token.
        if RESERVED_DTYPES.contains(&dt) {
            return Err(KeyDecline::ReservedDtype);
        }
        if !DTYPES.contains(&dt) {
            return Err(KeyDecline::UnknownDtype);
        }
    }
    let mp = MathPrecision::parse(parts[rest + 3]).ok_or(KeyDecline::BadContractionField)?;
    Ok(Contraction {
        m,
        n,
        k,
        k_div,
        batch,
        wdt: wdt.to_string(),
        acc: acc.to_string(),
        out: out.to_string(),
        mp,
    })
}

/// Parse a token into a `StructureKey`, rejecting a malformed one with a typed
/// decline (§6.7-0009). Round-trips with `to_token` byte-for-byte (§6.7-0008).
pub fn from_token(token: &str) -> Result<StructureKey, KeyDecline> {
    // §6.4-0004: reject an out-of-bound length FIRST, on the O(1) byte length,
    // before the split allocates — "without allocating on the unchecked length".
    if token.is_empty() || token.len() > MAX_STRUCTURE_KEY_LEN {
        return Err(KeyDecline::TokenLengthOutOfBound { len: token.len() });
    }
    let f: Vec<&str> = token.split('|').collect();
    if f.len() != 9 && f.len() != 10 {
        return Err(KeyDecline::WrongFieldCount { got: f.len() });
    }
    // field 0: `sk` + the canonical decimal version. `parse::<u32>()` would accept
    // a non-canonical "sk02" (parses to 2); compare the exact decimal instead so a
    // leading-zero token is refused (§6.7-0002).
    let ver = f[0].strip_prefix("sk").ok_or(KeyDecline::BadVersionPrefix)?;
    if ver != SCHEMA_VERSION.to_string() {
        return Err(KeyDecline::BadVersionPrefix);
    }
    // field 1/2: op-family and dtype must be in their closed sets (§6.5-0006, §6.1)
    if !OP_FAMILIES.contains(&f[1]) {
        return Err(KeyDecline::UnknownOpFamily);
    }
    if RESERVED_DTYPES.contains(&f[2]) {
        return Err(KeyDecline::ReservedDtype);
    }
    if !DTYPES.contains(&f[2]) {
        return Err(KeyDecline::UnknownDtype);
    }
    // field 3: target_capability = <namespace>:<capability-set>, exactly one `:`,
    // both parts non-empty (§6.8-0001).
    {
        let parts: Vec<&str> = f[3].split(':').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(KeyDecline::BadTargetGrammar);
        }
        // §6.8-0005: a target_capability MUST be case-sensitive ASCII and MUST NOT
        // contain a structure_key field separator (`;` or `/`; `|` can never appear
        // here — it is the field delimiter this token was split on), any whitespace
        // or control byte (`0x00`–`0x20`, `0x7f`), or a non-ASCII byte, so it embeds
        // as one unambiguous field. The colon (`0x3a`, required above) and `.`
        // (`0x2e`, e.g. `spirv1.6` — cited here only as a dot-bearing string, not
        // as a capability recommendation) stay legal — neither is in the forbidden
        // set.
        if f[3]
            .bytes()
            .any(|b| b == b';' || b == b'/' || b <= 0x20 || b == 0x7f || b >= 0x80)
        {
            return Err(KeyDecline::BadTargetCharset);
        }
    }
    let work_class = WorkClass::parse(f[5]).ok_or(KeyDecline::BadWorkClass)?;
    let rank = f[6]
        .strip_prefix('r')
        .and_then(|r| r.parse::<u32>().ok())
        .ok_or(KeyDecline::BadRank)?;
    // §6.4-0001: `rank` is the inclusive range `0..=MAX_RANK`; a greater rank is a
    // typed decline (§7.1-0002), never a panic.
    if rank > MAX_RANK {
        return Err(KeyDecline::BadRank);
    }
    let operands = f[7]
        .split(';')
        .map(parse_operand)
        .collect::<Result<Vec<_>, _>>()?;
    // §6.4-0002: no more than MAX_OPERANDS per-operand sub-keys.
    if operands.len() > MAX_OPERANDS {
        return Err(KeyDecline::TooManyOperands { got: operands.len() });
    }
    let reduce = parse_reduce(f[8])?;
    // §6.6-0017: a non-`-` reduce field is valid ONLY for a reduction cell
    // (`op_family == red`); reject any other family carrying one.
    if f[1] != "red" && reduce != Reduce::None {
        return Err(KeyDecline::ReduceNotGatedToRed);
    }
    let contraction = if f.len() == 10 { Some(parse_contraction(f[9])?) } else { None };
    // §6.6-0010: the contraction field is present IFF the cell is a dense-contraction
    // cell (`op_family == gem`).
    if (f[1] == "gem") != contraction.is_some() {
        return Err(KeyDecline::ContractionPresenceMismatch);
    }
    Ok(StructureKey {
        op_family: f[1].to_string(),
        dtype: f[2].to_string(),
        target: f[3].to_string(),
        index_width: f[4].to_string(),
        work_class,
        rank,
        operands,
        reduce,
        contraction,
    })
}

// ---- reference derivations: layout / divisibility / index-width / axis order ----
// These are the from-scratch reference derivations for the cell-level facts that
// feed the per-operand sub-key (§6.5) under the outermost-first axis convention
// (§6.3-0011). They take raw `extents` (elements) and signed `strides`
// (§6.3-0003) and reproduce the spec-pinned classification.

/// §6.3-0011: axes are ordered **outermost-first** — axis `0` is outermost and
/// axis `rank-1` is innermost. The innermost active axis is therefore the highest
/// active axis index. Returns `None` for a rank-0 (scalar) operand, which has no
/// axis. A column-major reader that treats axis `0` as innermost returns `Some(0)`
/// and is caught here.
#[must_use]
pub fn innermost_active_axis(rank: usize) -> Option<usize> {
    rank.checked_sub(1)
}

/// §6.5-0012: the inner-extent divisibility bucket from the innermost active axis
/// (§6.3-0011) extent `E` (in **elements**). `d16` iff `E >= 16 && E % 16 == 0`;
/// else `d8` iff `E >= 8 && E % 8 == 0`; else `d4` iff `E >= 4 && E % 4 == 0`;
/// else `d2` iff `E >= 2 && E % 2 == 0`; else `da` (odd `E`, `E == 1`, `E == 0`).
/// The `E >= N` guard is load-bearing: without it `E == 0` (`0 % 16 == 0`) would
/// mis-bucket a zero-extent axis as `d16`.
#[must_use]
pub fn derive_div_bucket(inner_extent: i64) -> DivBucket {
    let e = inner_extent;
    if e >= 16 && e % 16 == 0 {
        DivBucket::D16
    } else if e >= 8 && e % 8 == 0 {
        DivBucket::D8
    } else if e >= 4 && e % 4 == 0 {
        DivBucket::D4
    } else if e >= 2 && e % 2 == 0 {
        DivBucket::D2
    } else {
        DivBucket::Da
    }
}

/// §6.5-0012 applied over an operand's `extents`: buckets the innermost active
/// axis, i.e. axis `rank-1` = the **last** extent (§6.3-0011). A column-major
/// reader that buckets `extents[0]` disagrees on any non-square extent tuple. A
/// rank-0 operand (empty `extents`) has no inner axis and buckets `da`.
#[must_use]
pub fn derive_div_bucket_of(extents: &[i64]) -> DivBucket {
    match extents.last() {
        Some(&e) => derive_div_bucket(e),
        None => DivBucket::Da,
    }
}

/// §6.5-0011: the maximum touched element offset of a **single** operand — the sum
/// over its active non-broadcast axes of `|stride| * (extent - 1)`, in element
/// units. A broadcast axis (stride `0`) contributes `0`. The `(extent - 1)` term
/// is load-bearing: `(extent)` would over-count by `|stride|` per axis and can push
/// a boundary operand from `ix32` to `ix64`.
#[must_use]
pub fn operand_touched_offset(extents: &[i64], strides: &[i64]) -> i64 {
    extents
        .iter()
        .zip(strides.iter())
        .map(|(&e, &s)| if s == 0 { 0 } else { s.abs() * (e - 1) })
        .sum()
}

/// §6.5-0011 + §6.5-0005: the maximum touched element offset over **all** operands,
/// then the index-width token — `ix32` iff that maximum is `< 2^31`, else `ix64`
/// (the boundary is exclusive: exactly `2^31` is `ix64`).
#[must_use]
pub fn derive_index_width(operands: &[(&[i64], &[i64])]) -> &'static str {
    let max_off = operands
        .iter()
        .map(|&(e, s)| operand_touched_offset(e, s))
        .max()
        .unwrap_or(0);
    if max_off < (1i64 << 31) {
        "ix32"
    } else {
        "ix64"
    }
}

/// §6.5-0010 + §6.6-0013: the work-class **total element count** — the product,
/// over the iteration-frame axes, of each axis's iteration-frame extent, where the
/// iteration-frame extent of an axis is the **maximum extent across operands** at
/// that axis (**frame-max**, §6.6-0013), NOT the output operand's extent. Operands
/// are **right-aligned** to the iteration rank (`= max operand rank`): an operand of
/// rank `r` covers frame axes `[iteration_rank − r, iteration_rank − 1]` and is
/// broadcast (contributes extent `1`) on any lower frame axis. `operands` is the
/// per-operand extent vectors, outermost-first.
///
/// This is the ruled reading (KISS #82 finding 2, 2026-07-23): the output-frame
/// reading — product of the output operand's extents only — is NOT used, because it
/// drops the contracted `K` axis of a `gem` cell and so cannot distinguish `K = 64`
/// from `K = 65536`. For a `gem` cell whose output is much smaller than the frame
/// (e.g. `lhs[8,4096] · rhs[4096,8] → out[8,8]`) the two readings land in different
/// work-class buckets; this function implements frame-max.
#[must_use]
pub fn work_class_element_count(operands: &[&[i64]]) -> i64 {
    let iteration_rank = operands.iter().map(|e| e.len()).max().unwrap_or(0);
    (0..iteration_rank)
        .map(|j| {
            // per frame axis j (outermost-first): max extent across operands that
            // cover it under right-alignment; an operand not covering j broadcasts (1).
            operands
                .iter()
                .map(|e| {
                    let offset = iteration_rank - e.len(); // e's axes start at frame axis `offset`
                    if j < offset { 1 } else { e[j - offset] }
                })
                .max()
                .unwrap_or(1)
        })
        .product()
}

/// §6.5-0010 + §6.5-0007: derive the `work_class` from the frame-max total element
/// count ([`work_class_element_count`]) via the §6.5-0007 boundaries — `warp` iff
/// `count ≤ 32`, `block` iff `count ≤ 1024`, else `grid`.
#[must_use]
pub fn derive_work_class(operands: &[&[i64]]) -> WorkClass {
    match work_class_element_count(operands) {
        c if c <= 32 => WorkClass::Warp,
        c if c <= 1024 => WorkClass::Block,
        _ => WorkClass::Grid,
    }
}

/// §6.5-0002: derive the `layout_tag` from `extents` and `strides` using
/// `|stride|` (a fully reversed view is `contiguous`; the reversal is captured only
/// by the flipped flag, §6.6-0007). Axes are outermost-first (§6.3-0011); the
/// active **non-unit** axes (extent neither `0` nor `1`) are visited innermost-first.
/// **(1)** `broadcast` if any axis of extent `> 1` has stride `0`. **(2)** else
/// `contiguous` if, with a running product `P = 1` visited innermost->outermost,
/// `|stride|` equals `P` immediately before `P *= extent` for every active non-unit
/// axis (an empty product — no non-unit axis — is `contiguous`). **(3)** else
/// `inner-contiguous` if the innermost active non-unit axis has `|stride| == 1`.
/// **(4)** else `strided`.
#[must_use]
pub fn derive_layout_tag(extents: &[i64], strides: &[i64]) -> Contig {
    // (1) broadcast: any axis of extent > 1 with stride 0.
    for (&e, &s) in extents.iter().zip(strides.iter()) {
        if e > 1 && s == 0 {
            return Contig::Broadcast;
        }
    }
    // active non-unit axes (extent neither 0 nor 1), in natural outermost-first order.
    let nonunit: Vec<(i64, i64)> = extents
        .iter()
        .zip(strides.iter())
        .filter(|&(&e, _)| e != 0 && e != 1)
        .map(|(&e, &s)| (e, s))
        .collect();
    // (2) contiguous: |stride| == running product P, visited innermost->outermost.
    // The empty product (no non-unit axis) satisfies this and is contiguous.
    let mut p: i64 = 1;
    let mut contiguous = true;
    for &(e, s) in nonunit.iter().rev() {
        if s.abs() != p {
            contiguous = false;
            break;
        }
        p *= e;
    }
    if contiguous {
        return Contig::Contiguous;
    }
    // (3) inner-contiguous: innermost active non-unit axis (the last) has |stride| == 1.
    if let Some(&(_, s)) = nonunit.last() {
        if s.abs() == 1 {
            return Contig::InnerContiguous;
        }
    }
    // (4) strided.
    Contig::Strided
}

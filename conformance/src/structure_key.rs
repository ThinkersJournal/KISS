//! Reference codec for the KISS-Classify `structure_key` token (Classify §6.7).
//!
//! The token is the sole normative wire form of a specialization-cell identity
//! (§6.7-0011), a `|`-separated string:
//! ```text
//!   sk<ver> | <op_family> | <dtype> | <target> | <index_width> | <work_class>
//!           | r<rank> | <op0>;<op1>;… | <reduce> [ | c<m><n><k>/<kdiv> ]
//! ```
//! where each `<opI>` is `<contig>/<bcasthex>/<vec>/<div>/<flip>`. `to_token`
//! and `from_token` round-trip byte-identically (§6.7-0008); every hex mask is
//! lowercase, zero-padded to two digits (§6.7-0010); a producer emits `rall` /
//! `rlast` for the all-axes / trailing cases, never the equivalent `x<hh>`
//! (§6.7-0005). A malformed token is rejected with a typed decline (§6.7-0009).

pub const SCHEMA_VERSION: u32 = 2;

/// The closed op-family-tag set at this schema version — exactly the 24 codes of
/// Classify §6.5-0006. A token whose op-family field is outside this set is
/// rejected (a reader must not silently encode an "unknown" code).
pub const OP_FAMILIES: [&str; 24] = [
    "gem", "idx", "une", "emb", "bin", "shp", "ter", "srt", "gat", "qnt", "red", "rnd",
    "scn", "los", "nrm", "seg", "sft", "img", "cnv", "fft", "pol", "lin", "att", "moe",
];

/// The closed dtype-token set — exactly the 20 tokens of Classify §6.1.
pub const DTYPES: [&str; 20] = [
    "f16", "bf16", "f32", "f64", "s8", "s16", "u8", "u16", "i32", "i64", "u32", "u64", "bool",
    "e4m3", "e5m2", "s4", "u4", "b1", "c32", "c64",
];

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

/// The optional contraction field `c<m><n><k>/<kdiv>` (§6.7-0006).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Contraction {
    pub m: SizeClass,
    pub n: SizeClass,
    pub k: SizeClass,
    pub k_div: DivBucket,
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
}

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
        if let Some(c) = self.contraction {
            token.push_str(&format!(
                "|c{}{}{}/{}",
                c.m.code(),
                c.n.code(),
                c.k.code(),
                c.k_div.code()
            ));
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
    // c<m><n><k>/<kdiv>
    let body = s.strip_prefix('c').ok_or(KeyDecline::BadContractionField)?;
    let (sizes, kdiv) = body.split_once('/').ok_or(KeyDecline::BadContractionField)?;
    let sc: Vec<char> = sizes.chars().collect();
    if sc.len() != 3 {
        return Err(KeyDecline::BadContractionField);
    }
    let m = SizeClass::parse(&sc[0].to_string()).ok_or(KeyDecline::BadContractionField)?;
    let n = SizeClass::parse(&sc[1].to_string()).ok_or(KeyDecline::BadContractionField)?;
    let k = SizeClass::parse(&sc[2].to_string()).ok_or(KeyDecline::BadContractionField)?;
    let k_div = DivBucket::parse(kdiv).ok_or(KeyDecline::BadContractionField)?;
    Ok(Contraction { m, n, k, k_div })
}

/// Parse a token into a `StructureKey`, rejecting a malformed one with a typed
/// decline (§6.7-0009). Round-trips with `to_token` byte-for-byte (§6.7-0008).
pub fn from_token(token: &str) -> Result<StructureKey, KeyDecline> {
    let f: Vec<&str> = token.split('|').collect();
    if f.len() != 9 && f.len() != 10 {
        return Err(KeyDecline::WrongFieldCount { got: f.len() });
    }
    // field 0: sk<ver>
    let ver = f[0].strip_prefix("sk").ok_or(KeyDecline::BadVersionPrefix)?;
    if ver.parse::<u32>() != Ok(SCHEMA_VERSION) {
        return Err(KeyDecline::BadVersionPrefix);
    }
    // field 1/2: op-family and dtype must be in their closed sets (§6.5-0006, §6.1)
    if !OP_FAMILIES.contains(&f[1]) {
        return Err(KeyDecline::UnknownOpFamily);
    }
    if !DTYPES.contains(&f[2]) {
        return Err(KeyDecline::UnknownDtype);
    }
    let work_class = WorkClass::parse(f[5]).ok_or(KeyDecline::BadWorkClass)?;
    let rank = f[6]
        .strip_prefix('r')
        .and_then(|r| r.parse::<u32>().ok())
        .ok_or(KeyDecline::BadRank)?;
    let operands = f[7]
        .split(';')
        .map(parse_operand)
        .collect::<Result<Vec<_>, _>>()?;
    let reduce = parse_reduce(f[8])?;
    let contraction = if f.len() == 10 { Some(parse_contraction(f[9])?) } else { None };
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

// SPDX-License-Identifier: MIT OR Apache-2.0
//! The pooled window view's output extent (KISS-OPS-6.13-0010) and the window-overlap rule
//! (KISS-OPS-6.13-0011), per spatial axis.
//!
//! The overlap check is the closed form (a range of tap indices per output window); the test
//! scope holds it against a brute-force scan of every tap, so the two derivations differ.

/// One spatial axis of a window view: the operand extent and the four `OpAttrs` values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Axis {
    pub extent: u64,
    pub window_size: u64,
    pub stride: u64,
    pub dilation: u64,
    pub padding: u64,
}

/// Why an axis is not expressible.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decline {
    /// `window_size`, `stride` or `dilation` is 0.
    ZeroParameter,
    /// Output window `window` has no in-bounds tap.
    NoRealTap { window: u64 },
}

impl Axis {
    /// The dilated extent `dilation * (window_size - 1) + 1`.
    pub fn dilated_extent(&self) -> u64 {
        self.dilation * self.window_size.saturating_sub(1) + 1
    }

    /// The output extent: `floor((n + 2p - D) / s) + 1`, or 0 when no window fits.
    pub fn output_extent(&self) -> Result<u64, Decline> {
        if self.window_size == 0 || self.stride == 0 || self.dilation == 0 {
            return Err(Decline::ZeroParameter);
        }
        let span = self.extent + 2 * self.padding;
        let d = self.dilated_extent();
        Ok(if span < d { 0 } else { (span - d) / self.stride + 1 })
    }

    /// Whether output window `o` has an in-bounds tap, by the range of tap indices `j` with
    /// `0 <= o*s - p + j*d < n`.
    fn window_has_real_tap(&self, o: u64) -> bool {
        let (n, k) = (self.extent as i128, self.window_size as i128);
        let (s, d, p) = (self.stride as i128, self.dilation as i128, self.padding as i128);
        let base = o as i128 * s - p;
        let j_lo = if base >= 0 { 0 } else { (-base + d - 1) / d };
        let j_hi = ((n - 1 - base).div_euclid(d)).min(k - 1);
        j_lo <= j_hi
    }

    /// The §6.13-0010 and §6.13-0011 conditions together: `Ok(output extent)` for an
    /// expressible axis, the first violation otherwise.
    pub fn check(&self) -> Result<u64, Decline> {
        let out = self.output_extent()?;
        match (0..out).find(|&o| !self.window_has_real_tap(o)) {
            Some(window) => Err(Decline::NoRealTap { window }),
            None => Ok(out),
        }
    }
}

/// Every axis of a window view must be expressible; the first declining axis is reported with
/// its index. A window over several axes has a real element exactly when each axis has a tap.
pub fn check_axes(axes: &[Axis]) -> Result<Vec<u64>, (usize, Decline)> {
    axes.iter().enumerate().map(|(i, a)| a.check().map_err(|e| (i, e))).collect()
}

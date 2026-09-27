//! Pure coordinate transforms for future display resolutions; no SDL and no simulation changes.
//! Room reference is 640×360; GUI reference is 1280×720. Construct separate transforms.
//! This API is tested independently; the current application has NOT acquired a 4K display mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScalePolicy {
    Fit,
    IntegerWhenPossible,
}
#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub scale: f64,
}
impl Viewport {
    /// Fit a logical rectangle in framebuffer pixels, preserving aspect ratio and centering it.
    /// Integer scaling falls back to fractional downscaling when the window is smaller than logical size.
    /// Reject zero dimensions rather than silently producing invalid hit-test coordinates.
    pub fn fit(
        logical: (u32, u32),
        output: (u32, u32),
        policy: ScalePolicy,
    ) -> Result<Self, &'static str> {
        let (lw, lh) = logical;
        let (ow, oh) = output;
        if lw == 0 || lh == 0 || ow == 0 || oh == 0 {
            return Err("dimensions must be nonzero");
        }
        let mut scale = (ow as f64 / lw as f64).min(oh as f64 / lh as f64);
        if policy == ScalePolicy::IntegerWhenPossible && scale >= 1.0 {
            scale = scale.floor();
        }
        let width = lw as f64 * scale;
        let height = lh as f64 * scale;
        Ok(Self {
            x: (ow as f64 - width) / 2.0,
            y: (oh as f64 - height) / 2.0,
            width,
            height,
            scale,
        })
    }
    /// Transform a logical point to framebuffer pixels. The caller supplies coordinates in this space.
    pub fn to_output(self, p: (f64, f64)) -> (f64, f64) {
        (self.x + p.0 * self.scale, self.y + p.1 * self.scale)
    }
    /// Inverse hit test. Bars and right/bottom edges are outside the half-open viewport.
    pub fn to_logical(self, p: (f64, f64)) -> Option<(f64, f64)> {
        if !p.0.is_finite()
            || !p.1.is_finite()
            || p.0 < self.x
            || p.1 < self.y
            || p.0 >= self.x + self.width
            || p.1 >= self.y + self.height
        {
            return None;
        }
        Some(((p.0 - self.x) / self.scale, (p.1 - self.y) / self.scale))
    }
}

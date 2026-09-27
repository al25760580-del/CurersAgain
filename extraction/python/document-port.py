from pathlib import Path
b=Path('/home/mila/gamemaker-analysis/rust-port')
p=b/'src/bin/title_reference.rs';s=p.read_text();docs={
'fn num(':'/// Read an extracted numeric JSON field; missing/non-numeric values currently default to zero.\n/// Loader utility, not original game logic. Validate new metadata before relying on this fallback.\n',
'fn texture<':'/// Upload extracted RGBA pixels to SDL; optional tint preserves only source alpha.\n/// The tint approximates the locked-character shader, not a recovered GPU program.\n/// No game files are modified; errors name the missing/unreadable asset.\n',
'fn load_sprite<':'/// Load one exported frame and its GameMaker origin (logical sprite pixels).\n/// Metadata discovery is separate from YYC code recovery; this function does not parse data.win.\n',
'fn draw(':'/// Place a sprite using its resource origin and explicit x/y scale in current canvas units.\n/// Scope: unrotated sprites; particles have a separate rotation path. SDL errors propagate.\n',
'    fn load(':'    /// Upload an exported font atlas and cache glyph rectangles, shifts and x offsets.\n    /// Metrics are original font pixels, not host-platform font rasterization.\n',
'    fn text(':'    /// Draw one atlas-text line with left/center/right alignment (0/1/2).\n    /// Preserves texture alpha for intro fades. No word wrap, shaping or Scribble effects.\n',
'fn capture(':'/// Read the current framebuffer on the render thread, encode PNG on a worker.\n/// Diagnostic only: GPU readback can disturb timing; never use capture-frame latency as baseline.\n',
'fn draw_effects(':'/// Draw simulated title lines (depth 300) and triangles (depth 250), before characters.\n/// Coordinates are logical 640×360; RenderDoc event 43 verifies centered 1-pixel line width.\n/// Birth parameters still replay a finite capture; original RNG is not recovered.\n',
'fn main()':'/// Reference application: fixed 60-Hz simulation, separate interpolation and presentation.\n/// Ordinary visual startup only; no audio, unlock overlay or playable submenus.\n/// F6 restarts visual boot; confirm during boot skips without confirming a title selection.\n'}
for key,doc in docs.items():
 assert key in s,key
 s=s.replace(key,doc+key,1)
p.write_text(s)
p=b/'src/title_dynamics.rs';s=p.read_text();s=s.replace('//! Builtin y','//! Character Create 0x1435b80e0; Step 0x1435b8ea0; Draw 0x1435b94f0.\n//! TitleScreen Create 0x1435bd370; Step 0x1435c07d0.\n//! Builtin y');s=s.replace('    pub fn step(&mut self) {','    /// Advance a custom double-precision oscillator; phase is radians, y is logical pixels.\n    /// Character callers must use step_character to additionally round builtin y to float32.\n    pub fn step(&mut self) {');p.write_text(s)
p=b/'src/startup.rs';s=p.read_text().replace('/// Match recorded ordinary startup across every active-room sample, with float tolerance.','/// Return the live ordinary-boot room label for the HUD and external monitor.');p.write_text(s)

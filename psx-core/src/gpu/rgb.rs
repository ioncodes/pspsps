/// Convert RGB888 (24-bit) to RGB555 (15-bit)
#[inline(always)]
pub fn rgb888_to_rgb555(r8: i32, g8: i32, b8: i32) -> u16 {
    let r5 = ((r8 >> 3) & 0x1F) as u16;
    let g5 = ((g8 >> 3) & 0x1F) as u16;
    let b5 = ((b8 >> 3) & 0x1F) as u16;
    (b5 << 10) | (g5 << 5) | r5
}

/// Convert RGB555 (15-bit) to RGB888 (24-bit)
/// Expands 5-bit values to 8-bit by replicating high bits into low bits
#[inline(always)]
pub fn rgb555_to_rgb888(pixel: u16) -> (i32, i32, i32) {
    let r5 = (pixel & 0x1F) as i32;
    let g5 = ((pixel >> 5) & 0x1F) as i32;
    let b5 = ((pixel >> 10) & 0x1F) as i32;

    // Expand 5-bit to 8-bit by replicating high bits into low bits
    let r8 = (r5 << 3) | (r5 >> 2);
    let g8 = (g5 << 3) | (g5 >> 2);
    let b8 = (b5 << 3) | (b5 >> 2);

    (r8, g8, b8)
}

/// Extract RGB888 components from a 32-bit color value
#[inline(always)]
pub fn extract_rgb888(color: u32) -> (i32, i32, i32) {
    let r = (color & 0xFF) as i32;
    let g = ((color >> 8) & 0xFF) as i32;
    let b = ((color >> 16) & 0xFF) as i32;
    (r, g, b)
}

/// Apply semi-transparency blending between foreground and background pixels
///
/// Blend modes:
/// - 0: B/2 + F/2 (50% back + 50% front)
/// - 1: B + F (100% back + 100% front, additive with clamping)
/// - 2: B - F (100% back - 100% front, subtractive with clamping)
/// - 3: B + F/4 (100% back + 25% front)
#[inline(always)]
pub fn blend_semi_transparency(foreground: u16, background: u16, mode: u32) -> u16 {
    // Convert both pixels to RGB888 for blending
    let (fr, fg, fb) = rgb555_to_rgb888(foreground);
    let (br, bg, bb) = rgb555_to_rgb888(background);

    let (r, g, b) = match mode {
        // Mode 0: B/2 + F/2
        0 => (
            (br / 2 + fr / 2),
            (bg / 2 + fg / 2),
            (bb / 2 + fb / 2),
        ),
        // Mode 1: B + F (additive, with clamping)
        1 => (
            (br + fr).min(255),
            (bg + fg).min(255),
            (bb + fb).min(255),
        ),
        // Mode 2: B - F (subtractive, with clamping)
        2 => (
            (br - fr).max(0),
            (bg - fg).max(0),
            (bb - fb).max(0),
        ),
        // Mode 3: B + F/4
        3 => (
            (br + fr / 4).min(255),
            (bg + fg / 4).min(255),
            (bb + fb / 4).min(255),
        ),
        _ => (fr, fg, fb), // Fallback to foreground (shouldn't happen)
    };

    rgb888_to_rgb555(r, g, b)
}
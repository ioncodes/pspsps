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
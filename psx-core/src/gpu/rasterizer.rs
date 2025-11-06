use crate::gpu::cmd::tex::TextureWindowSettingCommand;
use crate::gpu::rgb;
use crate::gpu::{VRAM_HEIGHT, VRAM_WIDTH};

// -4  +0  -3  +1   ;\dither offsets for first two scanlines
// +2  -2  +3  -1   ;/
// -3  +1  -4  +0   ;\dither offsets for next two scanlines
// +3  -1  +2  -2   ;/(same as above, but shifted two pixels horizontally)
const DITHER_TABLE: [[i32; 4]; 4] = [
    [-4, 0, -3, 1], // y=0
    [2, -2, 3, -1], // y=1
    [-3, 1, -4, 0], // y=2
    [3, -1, 2, -2], // y=3
];

pub fn rasterize_polygon(
    vertices: &[(i16, i16)], colors: &[u32], uvs: &[u32], raw_texture: bool,
    texture_window: TextureWindowSettingCommand, drawing_area_x1: u32, drawing_area_y1: u32, drawing_area_x2: u32,
    drawing_area_y2: u32, dither_enabled: bool, vram: &mut [u8],
) {
    // Quads must be split into two triangles
    // Vertices received: V0, V1, V2, V3
    // Triangle 1: [V0, V1, V2] (vertices 1, 2, 3)
    // Triangle 2: [V1, V2, V3] (vertices 2, 3, 4)

    let textured = uvs.len() != 0;

    // We need to extract CLUT and texpage here since inside rasterize_triangle
    // uvs[0] is uvs[1] and uvs[1] is uvs[2] for the second triangle of a quad
    let (clut, texpage) = if textured {
        let clut = ((uvs[0] >> 16) & 0xFFFF) as u16;
        let texpage = ((uvs[1] >> 16) & 0xFFFF) as u16;
        (clut, texpage)
    } else {
        (0, 0)
    };

    if vertices.len() == 4 {
        rasterize_triangle(
            [vertices[0], vertices[1], vertices[2]],
            [colors[0], colors[1], colors[2]],
            if textured { [uvs[0], uvs[1], uvs[2]] } else { [0, 0, 0] },
            textured,
            raw_texture,
            clut,
            texpage,
            texture_window,
            drawing_area_x1,
            drawing_area_y1,
            drawing_area_x2,
            drawing_area_y2,
            dither_enabled,
            vram,
        );
        rasterize_triangle(
            [vertices[1], vertices[2], vertices[3]],
            [colors[1], colors[2], colors[3]],
            if textured { [uvs[1], uvs[2], uvs[3]] } else { [0, 0, 0] },
            textured,
            raw_texture,
            clut,
            texpage,
            texture_window,
            drawing_area_x1,
            drawing_area_y1,
            drawing_area_x2,
            drawing_area_y2,
            dither_enabled,
            vram,
        );
    } else {
        rasterize_triangle(
            [vertices[0], vertices[1], vertices[2]],
            [colors[0], colors[1], colors[2]],
            if textured { [uvs[0], uvs[1], uvs[2]] } else { [0, 0, 0] },
            textured,
            raw_texture,
            clut,
            texpage,
            texture_window,
            drawing_area_x1,
            drawing_area_y1,
            drawing_area_x2,
            drawing_area_y2,
            dither_enabled,
            vram,
        );
    }
}

pub fn rasterize_rectangle(
    x: i16, y: i16, width: u16, height: u16, uv: u32, texpage: u16, texture_window: TextureWindowSettingCommand,
    drawing_area_x1: u32, drawing_area_y1: u32, drawing_area_x2: u32, drawing_area_y2: u32, vram: &mut [u8],
) {
    // Extract UV and CLUT from the uv parameter
    let u_base = (uv & 0xFF) as u8;
    let v_base = ((uv >> 8) & 0xFF) as u8;
    let clut = ((uv >> 16) & 0xFFFF) as u16;

    // Extract texture page parameters
    let texture_x_base = ((texpage & 0b1111) as i32) * 64;
    let texture_y_base = (((texpage >> 4) & 0x1) as i32 * 256) + (((texpage >> 11) & 0x1) as i32 * 512);
    let color_depth = (texpage >> 7) & 0b11;

    // Extract CLUT coordinates
    let clut_x = ((clut & 0x3F) as usize) * 16; // CLUT X in 16-halfword steps
    let clut_y = ((clut >> 6) & 0x1FF) as usize;

    // Rasterize rectangle
    for row in 0..height {
        for col in 0..width {
            let screen_x = x as i32 + col as i32;
            let screen_y = y as i32 + row as i32;

            // Check drawing area bounds
            if screen_x < drawing_area_x1 as i32
                || screen_x >= drawing_area_x2 as i32
                || screen_y < drawing_area_y1 as i32
                || screen_y >= drawing_area_y2 as i32
            {
                continue;
            }

            // Wrap coordinates to VRAM dimensions
            let vram_x = (screen_x & (VRAM_WIDTH as i32 - 1)) as usize;
            let vram_y = (screen_y & (VRAM_HEIGHT as i32 - 1)) as usize;

            // Calculate texture coordinates with wrapping
            let u = (u_base as i32 + col as i32) & 0xFF;
            let v = (v_base as i32 + row as i32) & 0xFF;

            // Apply texture window
            let u = (u & !(texture_window.texture_window_x_mask() as i32 * 8))
                | ((texture_window.texture_window_x_offset() as i32 & texture_window.texture_window_x_mask() as i32)
                    * 8);
            let v = (v & !(texture_window.texture_window_y_mask() as i32 * 8))
                | ((texture_window.texture_window_y_offset() as i32 & texture_window.texture_window_y_mask() as i32)
                    * 8);

            let tex_x_in_page = (u as usize) % 256;
            let tex_y_in_page = (v as usize) % 256;

            // Sample texture
            let pixel = sample_texture(
                tex_x_in_page,
                tex_y_in_page,
                texture_x_base,
                texture_y_base,
                color_depth,
                clut_x,
                clut_y,
                vram,
            );

            // Skip transparent pixels
            if pixel == 0x0000 {
                continue;
            }

            // Write pixel to VRAM
            let vram_idx = (vram_y * VRAM_WIDTH + vram_x) * 2;
            let bytes = pixel.to_le_bytes();
            vram[vram_idx] = bytes[0];
            vram[vram_idx + 1] = bytes[1];
        }
    }
}

fn rasterize_triangle(
    vertices: [(i16, i16); 3], colors: [u32; 3], uvs: [u32; 3], textured: bool, raw_texture: bool, clut: u16,
    texpage: u16, texture_window: TextureWindowSettingCommand, drawing_area_x1: u32, drawing_area_y1: u32,
    drawing_area_x2: u32, drawing_area_y2: u32, dither_enabled: bool, vram: &mut [u8],
) {
    // Vertex coordinates
    let (x0, y0) = (vertices[0].0 as i32, vertices[0].1 as i32);
    let (x1, y1) = (vertices[1].0 as i32, vertices[1].1 as i32);
    let (x2, y2) = (vertices[2].0 as i32, vertices[2].1 as i32);

    // Bounding box constrained to drawing area
    let min_x = x0.min(x1).min(x2).max(drawing_area_x1 as i32);
    let max_x = x0.max(x1).max(x2).min(drawing_area_x2 as i32);
    let min_y = y0.min(y1).min(y2).max(drawing_area_y1 as i32);
    let max_y = y0.max(y1).max(y2).min(drawing_area_y2 as i32);

    let area = edge_function(x0, y0, x1, y1, x2, y2);
    if area == 0 {
        return; // Straight line?? skip
    }

    let clockwise = area < 0;

    // Go through each pixel in bounding box
    // PSX excludes lower-right coordinates
    // "Polygons are displayed up to \<excluding> their lower-right coordinates."
    // HONORARY MENTION: CHICHO I LOVE YOU
    for y in min_y..max_y {
        for x in min_x..max_x {
            // Measure the distance from point (x, y) to the edge opposite to the corresponding vertex
            // w0: distance from edge V1->V2 (opposite to V0)
            // w1: distance from edge V1->V0 (opposite to V1)
            // w2: distance from edge V0->V1 (opposite to V2)

            let w0 = edge_function(x1, y1, x2, y2, x, y);
            let w1 = edge_function(x2, y2, x0, y0, x, y);
            let w2 = edge_function(x0, y0, x1, y1, x, y);

            // Is the point inside the triangle?
            let inside = if clockwise {
                w0 <= 0 && w1 <= 0 && w2 <= 0
            } else {
                w0 >= 0 && w1 >= 0 && w2 >= 0
            };

            if inside {
                // Gouraud shading or textured
                let pixel = if !textured {
                    gouraud_shading(colors, w0, w1, w2, area, dither_enabled, x, y)
                } else {
                    textured_render(
                        uvs,
                        colors,
                        w0,
                        w1,
                        w2,
                        area,
                        clut,
                        texpage,
                        raw_texture,
                        texture_window,
                        dither_enabled,
                        x,
                        y,
                        vram,
                    )
                };

                // Transparent pixel
                if textured && pixel == 0x0000 {
                    continue;
                }

                // Wrap coordinates to VRAM dimensions
                let vram_x = (x & (VRAM_WIDTH as i32 - 1)) as usize;
                let vram_y = (y & (VRAM_HEIGHT as i32 - 1)) as usize;

                // Push to VRAM
                let vram_idx = (vram_y * VRAM_WIDTH + vram_x) * 2;
                let bytes = pixel.to_le_bytes();
                vram[vram_idx] = bytes[0];
                vram[vram_idx + 1] = bytes[1];
            }
        }
    }
}

#[inline]
fn edge_function(ax: i32, ay: i32, bx: i32, by: i32, px: i32, py: i32) -> i32 {
    (bx - ax) * (py - ay) - (by - ay) * (px - ax)
}

/// Look up a color from the CLUT
#[inline]
fn lookup_clut(clut_x: usize, clut_y: usize, index: usize, vram: &[u8]) -> u16 {
    let clut_idx = (clut_y * VRAM_WIDTH + clut_x + index) * 2;
    if clut_idx + 1 < vram.len() {
        u16::from_le_bytes([vram[clut_idx], vram[clut_idx + 1]])
    } else {
        0x0000
    }
}

/// Sample a texture at the given coordinates
/// Returns 0x0000 for transparent pixels or OOB access
fn sample_texture(
    tex_x_in_page: usize, tex_y_in_page: usize, texture_x_base: i32, texture_y_base: i32, color_depth: u16,
    clut_x: usize, clut_y: usize, vram: &[u8],
) -> u16 {
    match color_depth {
        0 => {
            // 4-bit CLUT mode
            let vram_x = texture_x_base as usize + (tex_x_in_page / 4);
            let vram_y = texture_y_base as usize + tex_y_in_page;

            if vram_x >= VRAM_WIDTH || vram_y >= VRAM_HEIGHT {
                return 0x0000;
            }

            let vram_idx = (vram_y * VRAM_WIDTH + vram_x) * 2;
            let halfword = u16::from_le_bytes([vram[vram_idx], vram[vram_idx + 1]]);
            let texel_in_word = tex_x_in_page % 4;
            let texel_index = ((halfword >> (texel_in_word * 4)) & 0xF) as usize;

            lookup_clut(clut_x, clut_y, texel_index, vram)
        }
        1 => {
            // 8-bit CLUT mode
            let vram_x = texture_x_base as usize + (tex_x_in_page / 2);
            let vram_y = texture_y_base as usize + tex_y_in_page;

            if vram_x >= VRAM_WIDTH || vram_y >= VRAM_HEIGHT {
                return 0x0000;
            }

            let vram_idx = (vram_y * VRAM_WIDTH + vram_x) * 2;
            let halfword = u16::from_le_bytes([vram[vram_idx], vram[vram_idx + 1]]);
            let texel_in_word = tex_x_in_page % 2;
            let texel_index = if texel_in_word == 0 {
                (halfword & 0xFF) as usize
            } else {
                ((halfword >> 8) & 0xFF) as usize
            };

            lookup_clut(clut_x, clut_y, texel_index, vram)
        }
        _ => {
            // 15-bit direct color mode
            let texel_x = texture_x_base as usize + tex_x_in_page;
            let texel_y = texture_y_base as usize + tex_y_in_page;

            if texel_x >= VRAM_WIDTH || texel_y >= VRAM_HEIGHT {
                return 0x0000;
            }

            let tex_idx = (texel_y * VRAM_WIDTH + texel_x) * 2;
            if tex_idx + 1 < vram.len() {
                u16::from_le_bytes([vram[tex_idx], vram[tex_idx + 1]])
            } else {
                0x0000
            }
        }
    }
}

fn gouraud_shading(
    colors: [u32; 3], w0: i32, w1: i32, w2: i32, area: i32, dither_enabled: bool, x: i32, y: i32,
) -> u16 {
    let (r0, g0, b0) = rgb::extract_rgb888(colors[0]);
    let (r1, g1, b1) = rgb::extract_rgb888(colors[1]);
    let (r2, g2, b2) = rgb::extract_rgb888(colors[2]);

    // Gouraud shading
    // color = ((w0 x color0) + (w1 x color1) + (w2 x color2)) / area

    let r = ((r0 * w0) + (r1 * w1) + (r2 * w2)) / area;
    let g = ((g0 * w0) + (g1 * w1) + (g2 * w2)) / area;
    let b = ((b0 * w0) + (b1 * w1) + (b2 * w2)) / area;

    let is_gouraud = colors[0] != colors[1] || colors[1] != colors[2];

    if dither_enabled && is_gouraud {
        apply_dither(r, g, b, x, y)
    } else {
        rgb::rgb888_to_rgb555(r, g, b)
    }
}

fn textured_render(
    uvs: [u32; 3], colors: [u32; 3], w0: i32, w1: i32, w2: i32, area: i32, clut: u16, texpage: u16, raw_texture: bool,
    texture_window: TextureWindowSettingCommand, dither_enabled: bool, x: i32, y: i32, vram: &mut [u8],
) -> u16 {
    // Clut Attribute (Color Lookup Table, aka Palette)
    // This attribute is used in all Textured Polygon/Rectangle commands. Of course, it's relevant only for 4bit/8bit textures (don't care for 15bit textures).
    //   0-5    X coordinate X/16  (ie. in 16-halfword steps)
    //   6-14   Y coordinate 0-511 (ie. in 1-line steps)  ;\on v0 GPU (max 1 MB VRAM)
    //   15     Unused (should be 0)                      ;/
    //   6-15   Y coordinate 0-1023 (ie. in 1-line steps) ;on v2 GPU (max 2 MB VRAM)

    // Texpage Attribute (Parameter for Textured-Polygons commands)
    //   0-8    Same as GP0(E1h).Bit0-8 (see there)
    //   9-10   Unused (does NOT change GP0(E1h).Bit9-10)
    //   11     Same as GP0(E1h).Bit11  (see there)
    //   12-13  Unused (does NOT change GP0(E1h).Bit12-13)
    //   14-15  Unused (should be 0)

    // GP0(E1h) - Draw Mode setting (aka "Texpage")
    //   0-3   Texture page X Base   (N*64) (ie. in 64-halfword steps)    ;GPUSTAT.0-3
    //   4     Texture page Y Base 1 (N*256) (ie. 0, 256, 512 or 768)     ;GPUSTAT.4
    //   5-6   Semi-transparency     (0=B/2+F/2, 1=B+F, 2=B-F, 3=B+F/4)   ;GPUSTAT.5-6
    //   7-8   Texture page colors   (0=4bit, 1=8bit, 2=15bit, 3=Reserved);GPUSTAT.7-8
    //   9     Dither 24bit to 15bit (0=Off/strip LSBs, 1=Dither Enabled) ;GPUSTAT.9
    //   10    Drawing to display area (0=Prohibited, 1=Allowed)          ;GPUSTAT.10
    //   11    Texture page Y Base 2 (N*512) (only for 2 MB VRAM)         ;GPUSTAT.15
    //   12    Textured Rectangle X-Flip   (BIOS does set this bit on power-up...?)
    //   13    Textured Rectangle Y-Flip   (BIOS does set it equal to GPUSTAT.13...?)
    //   14-23 Not used (should be 0)
    //   24-31 Command  (E1h)

    let texture_x_base = (texpage & 0b1111) as i32 * 64;
    let texture_y_base = (((texpage >> 4) & 0x1) as i32 * 256) + (((texpage >> 11) & 0x1) as i32 * 512);

    let color_depth = (texpage >> 7) & 0b11;

    let clut_x = (clut & 0x3F) as u16;
    let clut_y = ((clut >> 6) & 0x1FF) as u16;

    let u0 = (uvs[0] & 0xFF) as i32;
    let v0 = ((uvs[0] >> 8) & 0xFF) as i32;
    let u1 = (uvs[1] & 0xFF) as i32;
    let v1 = ((uvs[1] >> 8) & 0xFF) as i32;
    let u2 = (uvs[2] & 0xFF) as i32;
    let v2 = ((uvs[2] >> 8) & 0xFF) as i32;

    let u = ((u0 * w0) + (u1 * w1) + (u2 * w2)) / area;
    let v = ((v0 * w0) + (v1 * w1) + (v2 * w2)) / area;

    let u = (u & !(texture_window.texture_window_x_mask() as i32 * 8))
        | ((texture_window.texture_window_x_offset() as i32 & texture_window.texture_window_x_mask() as i32) * 8);
    let v = (v & !(texture_window.texture_window_y_mask() as i32 * 8))
        | ((texture_window.texture_window_y_offset() as i32 & texture_window.texture_window_y_mask() as i32) * 8);

    let tex_x_in_page = (u as usize) % 256;
    let tex_y_in_page = (v as usize) % 256;

    let texel = sample_texture(
        tex_x_in_page,
        tex_y_in_page,
        texture_x_base,
        texture_y_base,
        color_depth,
        (clut_x as usize) * 16,
        clut_y as usize,
        vram,
    );

    // If raw_texture mode, return texel directly without modulation
    if raw_texture {
        return texel;
    }

    // Blend texture color with vertex color
    // "finalChannel.rgb = (texel.rgb * vertexColour.rgb) / vec3(128.0)"

    // Convert texel from RGB555 to RGB888 for modulation
    let (tex_r8, tex_g8, tex_b8) = rgb::rgb555_to_rgb888(texel);

    // Interpolate vertex colors
    let (r0, g0, b0) = rgb::extract_rgb888(colors[0]);
    let (r1, g1, b1) = rgb::extract_rgb888(colors[1]);
    let (r2, g2, b2) = rgb::extract_rgb888(colors[2]);

    let vert_r = ((r0 * w0) + (r1 * w1) + (r2 * w2)) / area;
    let vert_g = ((g0 * w0) + (g1 * w1) + (g2 * w2)) / area;
    let vert_b = ((b0 * w0) + (b1 * w1) + (b2 * w2)) / area;

    // modulation = (texture * vertex_color) / 128
    let mod_r = (tex_r8 * vert_r) / 128;
    let mod_g = (tex_g8 * vert_g) / 128;
    let mod_b = (tex_b8 * vert_b) / 128;

    let is_modulated = vert_r != 128 || vert_g != 128 || vert_b != 128;

    if dither_enabled && is_modulated {
        apply_dither(mod_r, mod_g, mod_b, x, y)
    } else {
        rgb::rgb888_to_rgb555(mod_r, mod_g, mod_b)
    }
}

/// Apply dithering to an RGB888 color based on screen coordinates
/// Returns RGB555 value with dithering applied
///
/// "For dithering, VRAM is broken to 4x4 pixel blocks, depending on the location
/// in that 4x4 pixel region, the corresponding dither offset is added to the
/// 8bit R/G/B values, the result is saturated to +00h..+FFh,
/// and then divided by 8, resulting in the final 5bit R/G/B values.""
#[inline]
fn apply_dither(r8: i32, g8: i32, b8: i32, x: i32, y: i32) -> u16 {
    // Determine position within 4x4 dither table
    let dither_x = (x & 3) as usize;
    let dither_y = (y & 3) as usize;
    let dither_offset = DITHER_TABLE[dither_y][dither_x];

    // Add dither offset and clamp
    let r_dithered = (r8 + dither_offset).clamp(0, 255);
    let g_dithered = (g8 + dither_offset).clamp(0, 255);
    let b_dithered = (b8 + dither_offset).clamp(0, 255);

    // Convert to RGB555
    let r5 = ((r_dithered >> 3) & 0x1F) as u16;
    let g5 = ((g_dithered >> 3) & 0x1F) as u16;
    let b5 = ((b_dithered >> 3) & 0x1F) as u16;

    (b5 << 10) | (g5 << 5) | r5
}

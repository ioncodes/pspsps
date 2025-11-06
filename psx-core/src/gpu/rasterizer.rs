use crate::gpu::{cmd::tex::{TextureWindowSettingCommand}, VRAM_HEIGHT, VRAM_WIDTH};

pub fn rasterize_polygon(vertices: &[(i16, i16)], colors: &[u32], uvs: &[u32], texture_window: TextureWindowSettingCommand, vram: &mut [u8]) {
    // Quads must be split into two triangles
    // Vertices received: V0, V1, V2, V3
    // Triangle 1: [V0, V1, V2] (vertices 1, 2, 3)
    // Triangle 2: [V1, V2, V3] (vertices 2, 3, 4)

    let textured = uvs.len() != 0;

    if vertices.len() == 4 {
        rasterize_triangle(
            [vertices[0], vertices[1], vertices[2]],
            [colors[0], colors[1], colors[2]],
            if textured {
                [uvs[0], uvs[1], uvs[2]]
            } else {
                [0, 0, 0]
            },
            textured,
            texture_window,
            vram,
        );
        rasterize_triangle(
            [vertices[1], vertices[2], vertices[3]],
            [colors[1], colors[2], colors[3]],
            if textured {
                [uvs[1], uvs[2], uvs[3]]
            } else {
                [0, 0, 0]
            },
            textured,
            texture_window,
            vram,
        );
    } else {
        rasterize_triangle(
            [vertices[0], vertices[1], vertices[2]],
            [colors[0], colors[1], colors[2]],
            if textured {
                [uvs[0], uvs[1], uvs[2]]
            } else {
                [0, 0, 0]
            },
            textured,
            texture_window,
            vram,
        );
    }
}

fn rasterize_triangle(
    vertices: [(i16, i16); 3], colors: [u32; 3], uvs: [u32; 3], textured: bool, texture_window: TextureWindowSettingCommand, vram: &mut [u8],
) {
    // Vertex coordinates
    let (x0, y0) = (vertices[0].0 as i32, vertices[0].1 as i32);
    let (x1, y1) = (vertices[1].0 as i32, vertices[1].1 as i32);
    let (x2, y2) = (vertices[2].0 as i32, vertices[2].1 as i32);

    // Bounding box and clamping to VRAM dimensions
    let min_x = x0.min(x1).min(x2).max(0);
    let max_x = x0.max(x1).max(x2).min(VRAM_WIDTH as i32 - 1);
    let min_y = y0.min(y1).min(y2).max(0);
    let max_y = y0.max(y1).max(y2).min(VRAM_HEIGHT as i32 - 1);

    let area = edge_function(x0, y0, x1, y1, x2, y2);
    if area == 0 {
        return; // Straight line?? skip
    }

    let clockwise = area < 0;

    // Go through each pixel in bounding box
    for y in min_y..=max_y {
        for x in min_x..=max_x {
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
                    gouraud_shading(colors, w0, w1, w2, area)
                } else {
                    textured_render(uvs, w0, w1, w2, area, texture_window, vram)
                };

                // Push to VRAM
                let vram_idx = ((y as usize) * 1024 + (x as usize)) * 2;
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

fn gouraud_shading(colors: [u32; 3], w0: i32, w1: i32, w2: i32, area: i32) -> u16 {
    let r0 = (colors[0] & 0xFF) as i32;
    let g0 = ((colors[0] >> 8) & 0xFF) as i32;
    let b0 = ((colors[0] >> 16) & 0xFF) as i32;

    let r1 = (colors[1] & 0xFF) as i32;
    let g1 = ((colors[1] >> 8) & 0xFF) as i32;
    let b1 = ((colors[1] >> 16) & 0xFF) as i32;

    let r2 = (colors[2] & 0xFF) as i32;
    let g2 = ((colors[2] >> 8) & 0xFF) as i32;
    let b2 = ((colors[2] >> 16) & 0xFF) as i32;

    // Gouraud shading
    // color = ((w0 x color0) + (w1 x color1) + (w2 x color2)) / area

    let r = ((r0 * w0) + (r1 * w1) + (r2 * w2)) / area;
    let g = ((g0 * w0) + (g1 * w1) + (g2 * w2)) / area;
    let b = ((b0 * w0) + (b1 * w1) + (b2 * w2)) / area;

    let r5 = ((r >> 3) & 0x1F) as u16;
    let g5 = ((g >> 3) & 0x1F) as u16;
    let b5 = ((b >> 3) & 0x1F) as u16;

    (b5 << 10) | (g5 << 5) | r5
}

fn textured_render(uvs: [u32; 3], w0: i32, w1: i32, w2: i32, area: i32, texture_window: TextureWindowSettingCommand, vram: &mut [u8]) -> u16 {
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

    let clut = ((uvs[0] >> 16) & 0xFFFF) as u16;
    let texpage = ((uvs[1] >> 16) & 0xFFFF) as u16;

    let texture_x_base = (texpage & 0b1111) as i32 * 64;
    let texture_y_base =
        (((texpage >> 4) & 0x1) as i32 * 256) + (((texpage >> 11) & 0x1) as i32 * 512);

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

    match color_depth {
        0 => {
            // 4-bit CLUT mode
            // Each byte contains 2 texels (4 bits each)
            let tex_x_in_page = (u as usize) % 256;
            let tex_y_in_page = (v as usize) % 256;

            // In 4-bit mode, texture page X base is in 64-halfword steps, but we pack 4 texels per halfword
            let texel_x = texture_x_base as usize * 4 + tex_x_in_page; // 4 texels per 16-bit word
            let texel_y = texture_y_base as usize + tex_y_in_page;

            if texel_x / 4 >= VRAM_WIDTH || texel_y >= VRAM_HEIGHT {
                return 0x0000; // Transparent black
            }

            // Read the byte containing the 4-bit texel
            let byte_x = texel_x / 4; // 4 texels per 16-bit word = 2 per byte
            let vram_idx = (texel_y * VRAM_WIDTH + byte_x) * 2 + (texel_x / 2) % 2;
            let byte = vram[vram_idx];

            // Extract 4-bit index
            let texel_index = if texel_x % 2 == 0 {
                byte & 0x0F
            } else {
                (byte >> 4) & 0x0F
            };

            // Transparent color
            if texel_index == 0 {
                return 0x0000;
            }

            // Lookup in CLUT
            let clut_x = (clut_x as usize) * 16; // CLUT X in 16-halfword steps
            let clut_y = clut_y as usize;
            let clut_idx = (clut_y * VRAM_WIDTH + clut_x + texel_index as usize) * 2;

            if clut_idx + 1 < vram.len() {
                u16::from_le_bytes([vram[clut_idx], vram[clut_idx + 1]])
            } else {
                0x0000
            }
        }
        1 => {
            // 8-bit CLUT mode
            // Each byte is one texel
            let tex_x_in_page = (u as usize) % 256;
            let tex_y_in_page = (v as usize) % 256;

            // In 8-bit mode, texture page X base is in 64-halfword steps, but we pack 2 texels per halfword
            let texel_x = texture_x_base as usize * 2 + tex_x_in_page; // 2 texels per 16-bit word
            let texel_y = texture_y_base as usize + tex_y_in_page;

            if texel_x / 2 >= VRAM_WIDTH || texel_y >= VRAM_HEIGHT {
                return 0x0000;
            }

            // Read the byte containing the 8-bit texel
            let word_x = texel_x / 2;
            let vram_idx = (texel_y * VRAM_WIDTH + word_x) * 2 + (texel_x % 2);
            let texel_index = vram[vram_idx];

            // Transparent color
            if texel_index == 0 {
                return 0x0000;
            }

            // Lookup in CLUT
            let clut_x = (clut_x as usize) * 16; // CLUT X in 16-halfword steps
            let clut_y = clut_y as usize;
            let clut_idx = (clut_y * VRAM_WIDTH + clut_x + texel_index as usize) * 2;

            if clut_idx + 1 < vram.len() {
                u16::from_le_bytes([vram[clut_idx], vram[clut_idx + 1]])
            } else {
                0x0000
            }
        }
        _ => {
            // 15-bit direct color mode (texture_depth == 2 or 3)
            let tex_x_in_page = (u as usize) % 256;
            let tex_y_in_page = (v as usize) % 256;

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

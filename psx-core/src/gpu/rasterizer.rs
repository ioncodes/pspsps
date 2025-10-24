use crate::gpu::{VRAM_HEIGHT, VRAM_WIDTH};

pub fn rasterize_polygon(vertices: &[(i16, i16)], colors: &[u32], uvs: &[u32], vram: &mut [u8]) {
    // Quads must be split into two triangles
    // Vertices received: V0, V1, V2, V3
    // Triangle 1: [V0, V1, V2] (vertices 1, 2, 3)
    // Triangle 2: [V1, V2, V3] (vertices 2, 3, 4)

    if vertices.len() == 4 {
        rasterize_triangle(
            [vertices[0], vertices[1], vertices[2]],
            [colors[0], colors[1], colors[2]],
            uvs,
            vram,
        );
        rasterize_triangle(
            [vertices[1], vertices[2], vertices[3]],
            [colors[1], colors[2], colors[3]],
            uvs,
            vram,
        );
    } else {
        rasterize_triangle(
            [vertices[0], vertices[1], vertices[2]],
            [colors[0], colors[1], colors[2]],
            uvs,
            vram,
        );
    }
}

fn rasterize_triangle(vertices: [(i16, i16); 3], colors: [u32; 3], uvs: &[u32], vram: &mut [u8]) {
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
                let pixel = if uvs.len() == 0 {
                    gouraud_shading(colors, w0, w1, w2, area)
                } else {
                    textured_render([uvs[0], uvs[1], uvs[2]])
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

fn textured_render(uvs: [u32; 3]) -> u16 {
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

    let u0 = (uvs[0] & 0xFF) as u8;
    let v0 = ((uvs[0] >> 8) & 0xFF) as u8;
    let u1 = (uvs[1] & 0xFF) as u8;
    let v1 = ((uvs[1] >> 8) & 0xFF) as u8;
    let u2 = (uvs[2] & 0xFF) as u8;
    let v2 = ((uvs[2] >> 8) & 0xFF) as u8;

    0
}

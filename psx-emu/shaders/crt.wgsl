// ============================================================================
// VERTEX SHADER
// ============================================================================

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = vec4<f32>(model.position, 0.0, 1.0);
    return out;
}

// ============================================================================
// FRAGMENT SHADER - CRT EFFECTS
// ============================================================================

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

// Tunable parameters - adjust these for different CRT looks!
const SCANLINE_INTENSITY: f32 = 0.20;      // How dark the scanlines are (0.0 - 1.0)
const SCANLINE_COUNT: f32 = 480.0;         // Number of scanlines (matches PSX vertical res)
const VIGNETTE_STRENGTH: f32 = 0.35;       // Edge darkening (0.0 - 1.0)
const PHOSPHOR_BLOOM: f32 = 0.08;          // Phosphor glow amount (0.0 - 0.2)
const BRIGHTNESS: f32 = 1.05;              // Overall brightness multiplier
const CONTRAST: f32 = 1.1;                 // Contrast adjustment (1.0 = neutral)

// Scanline effect - simulates horizontal lines on CRT
fn apply_scanlines(uv: vec2<f32>, color: vec3<f32>) -> vec3<f32> {
    let scanline = sin(uv.y * SCANLINE_COUNT * 3.14159265) * 0.5 + 0.5;
    let scanline_effect = 1.0 - SCANLINE_INTENSITY * (1.0 - scanline);
    return color * scanline_effect;
}

// Vignette effect - darkens edges
fn apply_vignette(uv: vec2<f32>, color: vec3<f32>) -> vec3<f32> {
    let centered = uv * 2.0 - 1.0;
    let dist = length(centered);
    let vignette = smoothstep(1.4, 0.5, dist);
    let vignette_effect = mix(1.0 - VIGNETTE_STRENGTH, 1.0, vignette);
    return color * vignette_effect;
}

// Phosphor bloom - adds slight glow
fn apply_bloom(color: vec3<f32>) -> vec3<f32> {
    return color * (1.0 + PHOSPHOR_BLOOM);
}

// Contrast and brightness adjustment
fn adjust_contrast_brightness(color: vec3<f32>) -> vec3<f32> {
    let contrasted = (color - 0.5) * CONTRAST + 0.5;
    return contrasted * BRIGHTNESS;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.tex_coords;

    // Sample texture
    var color = textureSample(t_diffuse, s_diffuse, uv).rgb;

    // Apply CRT effects in order
    color = apply_scanlines(uv, color);
    color = apply_bloom(color);
    color = apply_vignette(uv, color);
    // color = adjust_contrast_brightness(color);

    // Clamp to valid range
    color = clamp(color, vec3<f32>(0.0), vec3<f32>(1.0));

    return vec4<f32>(color, 1.0);
}

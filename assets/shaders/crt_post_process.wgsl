#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

struct CrtSettings {
    scanline_intensity: f32,
    scanline_count: f32,
    vignette_intensity: f32,
    vignette_radius: f32,
    curvature_amount: f32,
    _padding: f32,
    _padding2: f32,
    _padding3: f32,
}

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
@group(0) @binding(2) var<uniform> settings: CrtSettings;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    var uv = in.uv;

    // Barrel distortion (CRT curvature)
    let offset = uv - vec2<f32>(0.5, 0.5);
    uv = uv + offset * dot(offset, offset) * settings.curvature_amount;

    // Sample the screen texture
    var color = textureSample(screen_texture, texture_sampler, uv);

    // Phosphor bleed (horizontal neighbor sampling)
    let dims = vec2<f32>(textureDimensions(screen_texture));
    let pixel_x = 1.0 / dims.x;
    let left = textureSample(screen_texture, texture_sampler, uv + vec2<f32>(-pixel_x, 0.0));
    let right = textureSample(screen_texture, texture_sampler, uv + vec2<f32>(pixel_x, 0.0));
    color = color + (left + right) * 0.05;

    // Scanlines
    let scanline = 1.0 - settings.scanline_intensity * (0.5 + 0.5 * sin(uv.y * settings.scanline_count * 3.14159265 * 2.0));
    color = vec4<f32>(color.rgb * scanline, color.a);

    // Vignette
    let dist = length((uv - vec2<f32>(0.5, 0.5)) * 2.0);
    let vignette = smoothstep(0.0, settings.vignette_radius, 1.0 - dist);
    let vignette_factor = pow(vignette, settings.vignette_intensity);
    color = vec4<f32>(color.rgb * vignette_factor, color.a);

    return color;
}

struct Uniforms {
    resolution: vec2<f32>,
    time: f32,
    _pad: f32,
};

@group(0) @binding(0)
var<uniform> u: Uniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) screen_pos: vec2<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.position, 0.0, 1.0);
    out.color = in.color;
    out.screen_pos = (in.position * 0.5 + 0.5) * u.resolution;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = in.color;

    let scanline = sin(in.screen_pos.y * 2.0 + u.time * 2.0) * 0.03;
    color = vec4<f32>(color.rgb + scanline, color.a);

    let flicker = 0.97 + sin(u.time * 17.3) * 0.015 + sin(u.time * 23.7) * 0.015;
    color = vec4<f32>(color.rgb * flicker, color.a);

    let glow = color.rgb * 0.15;
    color = vec4<f32>(color.rgb + glow, color.a);

    return color;
}

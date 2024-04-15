struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@group(0)
@binding(0)
var<uniform> view: mat4x4<f32>;

@group(0)
@binding(1)
var<uniform> screen_size: vec2<f32>;

struct InstanceInput {
    @location(5) transform_0: vec4<f32>,
    @location(6) transform_1: vec4<f32>,
    @location(7) transform_2: vec4<f32>,
    @location(8) transform_3: vec4<f32>,
    @location(9) color: vec4<f32>,
};

@vertex
fn vs_main(
    vin: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let transform = mat4x4<f32>(
        instance.transform_0,
        instance.transform_1,
        instance.transform_2,
        instance.transform_3,
    );
    var result: VertexOutput;
    result.position = view * transform * vec4(vin.position, 1.0);
    result.tex_coords = vin.tex_coords;
    result.color = instance.color;
    return result;
}

@fragment
fn fs_main(vout: VertexOutput) -> @location(0) vec4<f32> {
    // given screen size, calculate pixel coordinate of screen position
    let uv = vout.tex_coords;
    // let px_pos = (vout.position.xy * 0.5 + vec2(0.5)) * screen_size;
    let uv_px = vec2(1.0 / screen_size.x, 1.0 / screen_size.y);
    let left = step(uv, vec2(0.05));
    let bott = step(1.0 - uv, vec2(0.05));
    let border = 1.0 - min(max(left.x + left.y + bott.x + bott.y, 0.0), 1.0);
    return vec4(vout.color.rgb * border, 1.0);
    // return vec4(vec2(px / screen_size), 0.0, 1.0);
}

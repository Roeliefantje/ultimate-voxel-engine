
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>
}

// Vertex shader
struct VertexOutput {
    @builtin(position) out_position: vec4<f32>,
    @location(0) out_tex_coords: vec2<f32>,
};

@vertex
fn vs_main(vertex: VertexInput,) -> VertexOutput {
    var output: VertexOutput;
    output.out_position = vec4<f32>(vertex.position, 0.0, 1.0);
    output.out_tex_coords = vertex.tex_coords;
    return output;
}

// Fragment shader bindings struct
@group(0) @binding(0) 
var texture: texture_2d<f32>;
@group(0) @binding(1)
var t_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, t_sampler, in.out_tex_coords);
}
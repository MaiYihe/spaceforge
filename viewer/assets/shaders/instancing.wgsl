#import bevy_pbr::mesh_functions
#import bevy_pbr::forward_io::Vertex

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vertex(
    vertex: Vertex,
    @location(8) instance_pos_scale: vec4<f32>,
    @location(9) instance_color: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    let model = mesh_functions::get_model_matrix(vertex.instance_index);
    let local_position = vec4<f32>(vertex.position * instance_pos_scale.w + instance_pos_scale.xyz, 1.0);
    out.clip_position = mesh_functions::mesh_position_local_to_clip(model, local_position);
    out.color = instance_color;
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}

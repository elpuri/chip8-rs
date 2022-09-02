struct Output {
    @builtin(position) position: vec4<f32>,
    @location(0) displayUv: vec2<f32>,
    @location(1) windowUv: vec2<f32>
}

@vertex
fn main(@builtin(vertex_index) VertexIndex : u32) -> Output {
    var pos = array(
        vec2(-1.0, -1.0),
        vec2(1.0, -1.0),
        vec2(-1.0, 1.0),
        vec2(1.0, 1.0)
    );

    var uv = array(
        vec2(0.0, 1.0),
        vec2(1.0, 1.0),
        vec2(0.0, 0.0),
        vec2(1.0, 0.0)
    );

    var output: Output;
    let scale = vec2(1.2, 1.1);
    output.displayUv = uv[VertexIndex] * scale - vec2(-0.0, 0.05);
    output.windowUv = uv[VertexIndex];
    output.position = vec4<f32>(pos[VertexIndex], 0.0, 1.0);
    return output;
}

@group(0) @binding(0) var texSampler: sampler;
@group(0) @binding(1) var pixels: texture_2d<f32>;
@group(0) @binding(2) var overlaySampler: sampler;
@group(0) @binding(3) var overlayTexture: texture_2d<f32>;

@fragment
fn main(
    @location(0) displayUv: vec2<f32>,
    @location(1) windowUv: vec2<f32>
) -> @location(0) vec4<f32> {

    let uv = displayUv - 0.5;
    let uvL = length(uv);
    let distL = uvL * (1.0 + 0.15 * uvL * uvL);
    let distUv = (uv / uvL * distL) + vec2(0.5);

    var p: f32 = textureSample(pixels, texSampler, distUv).r;
    if (any(distUv < vec2(0.0)) || any(distUv > vec2(1.0))) {
        p = 0.0;
    }

    var c: vec4<f32> = select(
        vec4(0.0, 0.004, 0.002, 1.0),
        vec4(0.1, 0.5, 0.1, 1.0),
        p > 0.0
    );

    let raster = clamp(sin(distUv.y * 600.0) * 0.5 + 0.7, 0.0, 1.0);
    c = vec4(c.rgb * raster * raster, 1.0);

    let overlay = textureSample(overlayTexture, overlaySampler, windowUv);
    let overlay = vec4(overlay.rgb * overlay.a, overlay.a);
    c = c * (1.0 - overlay.a) + overlay;

    return c;
}
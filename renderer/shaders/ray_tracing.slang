import include.ray;

[vk::binding(0, 0)]
[format("rgba32f")]
WTexture2D output_texture;

struct Camera
{
    float4 position;
    float4 forward;
    float4 up;
    float4 right;
    int3 hovered_tile;
    uint is_hovering;
    float fov;
}

[vk::binding(0, 1)]
ConstantBuffer<Camera> camera;

[shader("compute")]
[numthreads(16, 16, 1)]
void trace_rays(uint3 global_index: SV_DispatchThreadID)
{
    var width : uint;
    var height : uint;
    output_texture.GetDimensions(width, height);

    if (global_index.x >= width || global_index.y >= height)
        return;

    var uv = ((float2(global_index.xy) + 0.5) / float2(width, height)) * 2.0 - 1.0;
    uv *= tan(camera.fov * 0.5);
    uv.x *= float(width) / float(height);

    var ray : Ray;
    ray.origin = camera.position;
    ray.direction = normalize(camera.forward + camera.up * uv.y + camera.right * uv.x);

    var color = skybox(ray.direction);

    var hovered_tile: Optional<int3> = camera.hovered_tile;
    if (camera.is_hovering == 0)
        hovered_tile = none;

    let ground = hit_ground(ray, hovered_tile);
    if (ground.hasValue)
        color = ground.value;

    output_texture.Store(global_index.xy, float4(color, 1.0));
}

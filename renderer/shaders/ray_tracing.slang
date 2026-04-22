import include.ray;

[vk::binding(0, 0)]
[format("rgba8")]
WTexture2D output_texture;

struct Camera
{
    float4 position;
    float4 forward;
    float4 up;
    float4 right;
    float4 ana;
    int3 hovered_tile;
    uint is_hovering;
    float fov;
    float screen_door_percentage;
}

[vk::binding(0, 1)]
ConstantBuffer<Camera> camera;

struct ObjectsInfo
{
    uint hypersphere_count;
}

[vk::binding(0, 2)]
ConstantBuffer<ObjectsInfo> objects_info;

[vk::binding(1, 2)]
StructuredBuffer<Hypersphere> hyperspheres;

[shader("compute")]
[numthreads(16, 16, 1)]
void trace_slice_rays(uint3 global_index: SV_DispatchThreadID)
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

    let color = ray_color(ray);
    output_texture.Store(global_index.xy, float4(color, 1.0));
}

[shader("compute")]
[numthreads(16, 16, 1)]
void trace_screen_door_rays(uint3 global_index: SV_DispatchThreadID)
{
    var width : uint;
    var height : uint;
    output_texture.GetDimensions(width, height);

    if (global_index.x >= width || global_index.y >= height)
        return;

    let pixel_coord = global_index.xy % uint2(4);

    var color = float3(0.0);
    if (all(pixel_coord != 3))
    {
        let index = pixel_coord.x + (2 - pixel_coord.y) * 3;

        var uv = float3(((float2(global_index.xy / 4) + 0.5) / float2(width / 4, height / 4)), (float(index) + 0.5) / 9.0) * 2.0 - 1.0;
        uv *= tan(camera.fov * 0.5);
        uv.x *= float(width) / float(height);
        uv.z *= camera.screen_door_percentage;

        var ray : Ray;
        ray.origin = camera.position;
        ray.direction = normalize(camera.forward + camera.up * uv.y + camera.right * uv.x + camera.ana * uv.z);

        color = ray_color(ray);
    }
    output_texture.Store(global_index.xy, float4(color, 1.0));
}

float3 ray_color(Ray ray)
{
    var color = skybox(ray.direction);

    let hit = hit_scene(ray);
    if (hit.hasValue)
    {
        let hit = hit.value;

        var light_ray : Ray;
        light_ray.origin = hit.position + hit.normal * 0.001;
        light_ray.direction = SunDirection;
        let light_hit = hit_scene(light_ray);

        color = hit.color * max(0.3, float(!light_hit.hasValue) * dot(hit.normal, light_ray.direction));
    }

    return color;
}

Optional<Hit> hit_scene(Ray ray)
{
    var hovered_tile : Optional<int3> = camera.hovered_tile;
    if (camera.is_hovering == 0)
        hovered_tile = none;

    var closest_hit = hit_ground(ray, hovered_tile);

    for (var i = 0u; i < objects_info.hypersphere_count; i++)
    {
        let hit = hit_hypersphere(ray, hyperspheres[i]);
        if (hit.hasValue)
            if (!closest_hit.hasValue || closest_hit.value.distance > hit.value.distance)
                closest_hit = hit;
    }

    {
        let hit = hit_tetrahedron(
            ray,
            Tetrahedron(
                float4(0.0, 0.01, 0.0, 0.0),
                float4(1.0, 0.01, 0.0, 0.0),
                float4(0.0, 0.01, 1.0, 0.0),
                float4(0.0, 0.01, 0.0, 1.0),
                float3(1.0, 0.0, 0.0),
                0.0));
        if (hit.hasValue)
            if (!closest_hit.hasValue || closest_hit.value.distance > hit.value.distance)
                closest_hit = hit;
    }

    return closest_hit;
}

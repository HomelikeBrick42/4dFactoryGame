import include.ui_objects_info;

[vk::binding(0, 0)]
ConstantBuffer<ObjectsInfo> objects_info;

struct Circle
{
    float2 position;
    float radius;
    float _padding;
    float4 color;
}

[vk::binding(2, 0)]
StructuredBuffer<Circle> circles;

struct VertexOutput
{
    float4 clip_position : SV_Position;
    uint instance_index;
    float2 uv;
}

[shader("vertex")]
VertexOutput vertex(uint vertex_index: SV_VulkanVertexID, uint instance_index: SV_VulkanInstanceID)
{
    var out : VertexOutput;
    out.instance_index = instance_index;
    out.uv = float2(vertex_index & 1, (vertex_index >> 1) & 1);

    let circle = circles[instance_index];
    out.clip_position = float4(circle.position + circle.radius * (out.uv * 2.0 - 1.0), 0.0, 1.0);
    out.clip_position.x /= objects_info.aspect;

    return out;
}

struct FragmentOutput
{
    float4 color : SV_Target;
}

[shader("fragment")]
FragmentOutput fragment(VertexOutput in)
{
    var out : FragmentOutput;

    let coord = in.uv * 2.0 - 1.0;
    if (dot(coord, coord) > 1.0)
        discard;

    let circle = circles[in.instance_index];
    out.color = circle.color;

    return out;
}

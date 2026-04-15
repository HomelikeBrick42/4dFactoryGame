import include.ui_objects_info;

[vk::binding(0, 0)]
ConstantBuffer<ObjectsInfo> objects_info;

struct Line
{
    float2 a;
    float2 b;
    float3 _padding;
    float width;
    float4 color;
}

[vk::binding(3, 0)]
StructuredBuffer<Line> lines;

struct VertexOutput
{
    float4 clip_position : SV_Position;
    uint instance_index;
    float2 uv;
}

[shader("vertex")]
VertexOutput vertex(uint vertex_index: SV_VertexID, uint instance_index: SV_InstanceID)
{
    var out : VertexOutput;
    out.instance_index = instance_index;
    out.uv = float2(vertex_index & 1, (vertex_index >> 1) & 1);

    let line = lines[instance_index];
    let dir = line.b - line.a;
    let dir_perp = float2(-dir.y, dir.x);
    out.clip_position = float4(line.a + out.uv.x * dir + line.width * (out.uv.y - 0.5) * normalize(dir_perp), 0.0, 1.0);
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

    let line = lines[in.instance_index];
    out.color = line.color;

    return out;
}

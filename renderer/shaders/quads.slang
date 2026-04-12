struct Quad
{
    float2 position;
    float2 size;
    float2 uv_offset;
    float2 uv_size;
    float4 color;
}

[vk::binding(0, 0)]
StructuredBuffer<Quad> quads;

[vk::binding(0, 1)]
Texture2D texture;

[vk::binding(1, 1)]
SamplerState sampler;

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

    let quad = quads[instance_index];
    out.clip_position = float4(quad.position + quad.size * (out.uv - 0.5), 0.0, 1.0);

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

    let quad = quads[in.instance_index];
    out.color = texture.Sample(sampler, quad.uv_offset + in.uv * quad.uv_size) * quad.color;

    return out;
}

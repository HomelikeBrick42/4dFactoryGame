struct Ray
{
    float4 origin;
    float4 direction;
}

static const float4 SunDirection = normalize(float4(0.3, 1.0, 0.2, 0.1));

float3 skybox(float4 direction)
{
    var color : float3;

    if (direction.y > 0.0)
        color = lerp(float3(0.4, 0.4, 0.6), float3(0.3, 0.3, 1.0), direction.y);
    else
        color = lerp(float3(0.4, 0.4, 0.4), float3(0.2, 0.2, 0.2), -direction.y);

    if (dot(direction, SunDirection) > 0.98)
        color = float3(1.0, 1.0, 1.0);

    return color;
}

Optional<float3> hit_ground(Ray ray)
{
    let distance = ray.origin.y / -ray.direction.y;
    if (distance < 0.0)
        return none;

    let point = ray.origin + ray.direction * distance;

    var checker_index = int(floor(point.x)) + int(floor(point.z)) + int(floor(point.w));
    if (checker_index % 2 == 0)
        return float3(0.5);
    return float3(1.0);
}

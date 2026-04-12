struct Ray
{
    float4 origin;
    float4 direction;
}

struct Hit
{
    float distance;
    float4 position;
    float4 normal;
    float3 color;
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

Optional<Hit> hit_ground(Ray ray, Optional<int3> hovered_tile)
{
    var hit : Hit;

    hit.distance = ray.origin.y / -ray.direction.y;
    if (hit.distance <= 0.0)
        return none;

    hit.position = ray.origin + ray.direction * hit.distance;
    hit.normal = float4(0.0, sign(ray.origin.y), 0.0, 0.0);

    let tile = int3(floor(hit.position.xzw));

    if (any(abs(tile) > int3(16)))
        return none;

    if (hovered_tile.hasValue && all(hovered_tile.value == tile))
    {
        let distance_from_center = abs(hit.position.xzw - (float3(tile) + 0.5));
        if (max3(distance_from_center.x, distance_from_center.y, distance_from_center.z) > 0.4)
        {
            hit.color = float3(0.2, 0.2, 0.5);
            return hit;
        }
    }

    if ((tile.x + tile.y + tile.z) % 2 == 0)
        hit.color = float3(0.5);
    else
        hit.color = float3(1.0);
    return hit;
}

struct Hypersphere
{
    float4 position;
    float3 color;
    float radius;
}

Optional<Hit> hit_hypersphere(Ray ray, Hypersphere hypersphere)
{
    let oc = hypersphere.position - ray.origin;
    // TODO: can this be replaced with 1?
    let a = dot(ray.direction, ray.direction);
    let h = dot(ray.direction, oc);
    let c = dot(oc, oc) - hypersphere.radius * hypersphere.radius;
    let discriminant = h * h - a * c;

    if (discriminant < 0.0)
        return none;

    let d1 = (h - sqrt(discriminant)) / a;
    let d2 = (h + sqrt(discriminant)) / a;

    var hit : Hit;
    hit.distance = d1;

    // if (hit.distance < 0.0)
    //     hit.distance = d2;
    if (hit.distance < 0.0)
        return none;

    hit.position = ray.origin + ray.direction * hit.distance;
    hit.normal = (hit.position - hypersphere.position) / hypersphere.radius;
    hit.color = hypersphere.color;
    return hit;
}

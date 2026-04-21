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
        if (max(max(distance_from_center.x, distance_from_center.y), distance_from_center.z) > 0.4)
        {
            hit.color = float3(0.2, 0.2, 0.5);
            return hit;
        }
    }

    if ((tile.x + tile.y + tile.z) % 2 == 0)
        hit.color = float3(0.3, 0.5, 0.4);
    else
        hit.color = float3(0.3, 1.0, 0.4);
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

float4 hyperplane_from_points_abcd(float4 b, float4 c, float4 d)
{
    let _0 = b.w;
    let _1 = b.z;
    let _2 = b.y;
    let _3 = b.x;
    let _4 = c.w;
    let _5 = c.z;
    let _6 = c.y;
    let _7 = c.x;
    let _8 = d.w;
    let _9 = d.z;
    let _10 = d.y;
    let _11 = d.x;
    let e1 = (_0 * _6 * _9) + (_1 * _10 * _4) + (_2 * _5 * _8) + (-1.0 * _0 * _10 * _5) + (-1.0 * _1 * _6 * _8) + (-1.0 * _2 * _4 * _9);
    let e2 = (_0 * _7 * _9) + (_1 * _11 * _4) + (_3 * _5 * _8) + (-1.0 * _0 * _11 * _5) + (-1.0 * _1 * _7 * _8) + (-1.0 * _3 * _4 * _9);
    let e3 = (_0 * _10 * _7) + (_11 * _2 * _4) + (_3 * _6 * _8) + (-1.0 * _0 * _11 * _6) + (-1.0 * _10 * _3 * _4) + (-1.0 * _2 * _7 * _8);
    let e4 = (_1 * _10 * _7) + (_11 * _2 * _5) + (_3 * _6 * _9) + (-1.0 * _1 * _11 * _6) + (-1.0 * _10 * _3 * _5) + (-1.0 * _2 * _7 * _9);
    return float4(e1, e2, e3, e4);
}

float4 tetrahedron_side_abc_from_points_abcd(float4 b, float4 c, float4 d)
{
    let _0 = b.w;
    let _1 = b.z;
    let _2 = b.y;
    let _3 = b.x;
    let _4 = c.w;
    let _5 = c.z;
    let _6 = c.y;
    let _7 = c.x;
    let _8 = d.w;
    let _9 = d.z;
    let _10 = d.y;
    let _11 = d.x;
    let e1 = (_0 * _0 * _11 * _5 * _5) + (_0 * _0 * _11 * _6 * _6) + (_0 * _1 * _4 * _7 * _9) + (_0 * _1 * _5 * _7 * _8) + (_0 * _10 * _2 * _4 * _7) + (_0 * _10 * _3 * _4 * _6) + (_0 * _2 * _6 * _7 * _8) + (_0 * _3 * _4 * _5 * _9) + (_1 * _1 * _11 * _4 * _4) + (_1 * _1 * _11 * _6 * _6) + (_1 * _10 * _2 * _5 * _7) + (_1 * _10 * _3 * _5 * _6) + (_1 * _2 * _6 * _7 * _9) + (_1 * _3 * _4 * _5 * _8) + (_11 * _2 * _2 * _4 * _4) + (_11 * _2 * _2 * _5 * _5) + (_2 * _3 * _4 * _6 * _8) + (_2 * _3 * _5 * _6 * _9) + (-2.0 * _0 * _1 * _11 * _4 * _5) + (-2.0 * _0 * _11 * _2 * _4 * _6) + (-2.0 * _1 * _11 * _2 * _5 * _6) + (-1.0 * _0 * _0 * _10 * _6 * _7) + (-1.0 * _0 * _0 * _5 * _7 * _9) + (-1.0 * _0 * _3 * _5 * _5 * _8) + (-1.0 * _0 * _3 * _6 * _6 * _8) + (-1.0 * _1 * _1 * _10 * _6 * _7) + (-1.0 * _1 * _1 * _4 * _7 * _8) + (-1.0 * _1 * _3 * _4 * _4 * _9) + (-1.0 * _1 * _3 * _6 * _6 * _9) + (-1.0 * _10 * _2 * _3 * _4 * _4) + (-1.0 * _10 * _2 * _3 * _5 * _5) + (-1.0 * _2 * _2 * _4 * _7 * _8) + (-1.0 * _2 * _2 * _5 * _7 * _9);
    let e2 = (_0 * _0 * _11 * _6 * _7) + (_0 * _0 * _5 * _6 * _9) + (_0 * _2 * _5 * _5 * _8) + (_0 * _2 * _7 * _7 * _8) + (_1 * _1 * _11 * _6 * _7) + (_1 * _1 * _4 * _6 * _8) + (_1 * _2 * _4 * _4 * _9) + (_1 * _2 * _7 * _7 * _9) + (_11 * _2 * _3 * _4 * _4) + (_11 * _2 * _3 * _5 * _5) + (_3 * _3 * _4 * _6 * _8) + (_3 * _3 * _5 * _6 * _9) + (-1.0 * _0 * _0 * _10 * _5 * _5) + (-1.0 * _0 * _0 * _10 * _7 * _7) + (-1.0 * _0 * _1 * _4 * _6 * _9) + (-1.0 * _0 * _1 * _5 * _6 * _8) + (-1.0 * _0 * _11 * _2 * _4 * _7) + (-1.0 * _0 * _11 * _3 * _4 * _6) + (-1.0 * _0 * _2 * _4 * _5 * _9) + (-1.0 * _0 * _3 * _6 * _7 * _8) + (-1.0 * _1 * _1 * _10 * _4 * _4) + (-1.0 * _1 * _1 * _10 * _7 * _7) + (-1.0 * _1 * _11 * _2 * _5 * _7) + (-1.0 * _1 * _11 * _3 * _5 * _6) + (-1.0 * _1 * _2 * _4 * _5 * _8) + (-1.0 * _1 * _3 * _6 * _7 * _9) + (-1.0 * _10 * _3 * _3 * _4 * _4) + (-1.0 * _10 * _3 * _3 * _5 * _5) + (-1.0 * _2 * _3 * _4 * _7 * _8) + (-1.0 * _2 * _3 * _5 * _7 * _9) + (2.0 * _0 * _1 * _10 * _4 * _5) + (2.0 * _0 * _10 * _3 * _4 * _7) + (2.0 * _1 * _10 * _3 * _5 * _7);
    let e3 = (_0 * _0 * _6 * _6 * _9) + (_0 * _0 * _7 * _7 * _9) + (_0 * _1 * _10 * _4 * _6) + (_0 * _1 * _11 * _4 * _7) + (_0 * _10 * _2 * _4 * _5) + (_0 * _11 * _3 * _4 * _5) + (_0 * _2 * _5 * _6 * _8) + (_0 * _3 * _5 * _7 * _8) + (_1 * _10 * _3 * _6 * _7) + (_1 * _11 * _2 * _6 * _7) + (_1 * _2 * _4 * _6 * _8) + (_1 * _3 * _4 * _7 * _8) + (_10 * _2 * _3 * _5 * _7) + (_11 * _2 * _3 * _5 * _6) + (_2 * _2 * _4 * _4 * _9) + (_2 * _2 * _7 * _7 * _9) + (_3 * _3 * _4 * _4 * _9) + (_3 * _3 * _6 * _6 * _9) + (-2.0 * _0 * _2 * _4 * _6 * _9) + (-2.0 * _0 * _3 * _4 * _7 * _9) + (-2.0 * _2 * _3 * _6 * _7 * _9) + (-1.0 * _0 * _0 * _10 * _5 * _6) + (-1.0 * _0 * _0 * _11 * _5 * _7) + (-1.0 * _0 * _1 * _6 * _6 * _8) + (-1.0 * _0 * _1 * _7 * _7 * _8) + (-1.0 * _1 * _10 * _2 * _4 * _4) + (-1.0 * _1 * _10 * _2 * _7 * _7) + (-1.0 * _1 * _11 * _3 * _4 * _4) + (-1.0 * _1 * _11 * _3 * _6 * _6) + (-1.0 * _10 * _3 * _3 * _5 * _6) + (-1.0 * _11 * _2 * _2 * _5 * _7) + (-1.0 * _2 * _2 * _4 * _5 * _8) + (-1.0 * _3 * _3 * _4 * _5 * _8);
    let e4 = (_0 * _1 * _6 * _6 * _9) + (_0 * _1 * _7 * _7 * _9) + (_0 * _10 * _2 * _5 * _5) + (_0 * _10 * _2 * _7 * _7) + (_0 * _11 * _3 * _5 * _5) + (_0 * _11 * _3 * _6 * _6) + (_1 * _1 * _10 * _4 * _6) + (_1 * _1 * _11 * _4 * _7) + (_10 * _3 * _3 * _4 * _6) + (_11 * _2 * _2 * _4 * _7) + (_2 * _2 * _4 * _5 * _9) + (_3 * _3 * _4 * _5 * _9) + (-1.0 * _0 * _1 * _10 * _5 * _6) + (-1.0 * _0 * _1 * _11 * _5 * _7) + (-1.0 * _0 * _10 * _3 * _6 * _7) + (-1.0 * _0 * _11 * _2 * _6 * _7) + (-1.0 * _0 * _2 * _5 * _6 * _9) + (-1.0 * _0 * _3 * _5 * _7 * _9) + (-1.0 * _1 * _1 * _6 * _6 * _8) + (-1.0 * _1 * _1 * _7 * _7 * _8) + (-1.0 * _1 * _10 * _2 * _4 * _5) + (-1.0 * _1 * _11 * _3 * _4 * _5) + (-1.0 * _1 * _2 * _4 * _6 * _9) + (-1.0 * _1 * _3 * _4 * _7 * _9) + (-1.0 * _10 * _2 * _3 * _4 * _7) + (-1.0 * _11 * _2 * _3 * _4 * _6) + (-1.0 * _2 * _2 * _5 * _5 * _8) + (-1.0 * _2 * _2 * _7 * _7 * _8) + (-1.0 * _3 * _3 * _5 * _5 * _8) + (-1.0 * _3 * _3 * _6 * _6 * _8) + (2.0 * _1 * _2 * _5 * _6 * _8) + (2.0 * _1 * _3 * _5 * _7 * _8) + (2.0 * _2 * _3 * _6 * _7 * _8);
    return float4(e1, e2, e3, e4);
}

struct Tetrahedron
{
    float4 a;
    float4 b;
    float4 c;
    float4 d;
    float3 color;
    float _padding;
}

Optional<Hit> hit_tetrahedron(Ray ray, Tetrahedron tetrahedron)
{
    float4 hyperplane = hyperplane_from_points_abcd(
        tetrahedron.b - tetrahedron.a,
        tetrahedron.c - tetrahedron.a,
        tetrahedron.d - tetrahedron.a);

    var hit : Hit;
    hit.distance = dot(hyperplane, ray.origin - tetrahedron.a) / -dot(hyperplane, ray.direction);
    if (hit.distance <= 0.0)
        return none;

    hit.position = ray.origin + ray.direction * hit.distance;

    let abc = tetrahedron_side_abc_from_points_abcd(
        tetrahedron.b - tetrahedron.a,
        tetrahedron.c - tetrahedron.a,
        tetrahedron.d - tetrahedron.a);
    if (sign(dot(abc, hit.position - tetrahedron.a)) != sign(dot(abc, tetrahedron.d - tetrahedron.a)))
        return none;

    let abd = tetrahedron_side_abc_from_points_abcd(
        tetrahedron.b - tetrahedron.a,
        tetrahedron.d - tetrahedron.a,
        tetrahedron.c - tetrahedron.a);
    if (sign(dot(abd, hit.position - tetrahedron.a)) != sign(dot(abd, tetrahedron.c - tetrahedron.a)))
        return none;

    let acd = tetrahedron_side_abc_from_points_abcd(
        tetrahedron.c - tetrahedron.a,
        tetrahedron.d - tetrahedron.a,
        tetrahedron.b - tetrahedron.a);
    if (sign(dot(acd, hit.position - tetrahedron.a)) != sign(dot(acd, tetrahedron.b - tetrahedron.a)))
        return none;

    let bcd = tetrahedron_side_abc_from_points_abcd(
        tetrahedron.c - tetrahedron.b,
        tetrahedron.d - tetrahedron.b,
        tetrahedron.a - tetrahedron.b);
    if (sign(dot(bcd, hit.position - tetrahedron.b)) != sign(dot(bcd, tetrahedron.a - tetrahedron.b)))
        return none;

    hit.normal = normalize(hyperplane);
    hit.normal *= sign(dot(hit.normal, ray.origin - tetrahedron.a));
    hit.color = tetrahedron.color;
    return hit;
}

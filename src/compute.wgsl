#import bevy_amoeba::vertices::SoftBody2dVertex;
#import bevy_amoeba::nodes::SoftBody2dNodeData;
#import bevy_amoeba::instances::SoftBody2dInstanceData;

struct SoftBodyComputeUniform {
    smooth_steps: u32,
};
struct ComputeInput {
    @builtin(global_invocation_id) id: vec3<u32>,
};

@group(0) @binding(0) var<uniform> num_vertices_per_instance: u32;
@group(0) @binding(1) var<uniform> uniforms: SoftBodyComputeUniform;
@group(0) @binding(2) var<storage, read_write> vertices: array<SoftBody2dVertex>;
@group(0) @binding(3) var<storage, read> nodes: array<SoftBody2dNodeData>;
@group(0) @binding(4) var<storage, read> instances: array<SoftBody2dInstanceData>;

const TAU: f32 = 6.28318531;
const PI: f32 =  3.14159274;

const ORIGIN = vec2<f32>(0.0, 0.0);
const RADIUS: f32 = 1.0;

// Compute initial position from the given index.
fn get_position(i: u32, n: u32) -> vec2<f32> {
    let angle = (f32(i - 1) / f32(n - 1)) * TAU;
    return vec2<f32>(cos(angle) * RADIUS, sin(angle) * RADIUS);
}

// Compute L2 distance squared.
fn l2_squared(p: vec2<f32>) -> f32 {
    return p.x * p.x + p.y * p.y;
}

// Get the closest circle-line intersection point to p1.
// Quadratic coefficients: A t^2 + B t + C = 0
fn get_closest_circle_line_intersection(
    c: vec2<f32>,
    r: f32,
    p1: vec2<f32>,
    p2: vec2<f32>,
) -> vec2<f32> {
    let d = p2 - p1;
    let A = dot(d, d);
    let f = p1 - c;
    let B = 2.0 * dot(f, d);
    let C = dot(f, f) - (r * r);

    let discriminant = (B * B) - (4.0 * A * C);
    if (discriminant < 0.0) {
        return p2;
    }

    let epsilon = 0.00001; 
    let t1 = (-B - sqrt(discriminant)) / (2.0 * A);
    let t2 = (-B + sqrt(discriminant)) / (2.0 * A);

    if (abs(discriminant) < epsilon) {
        return p1 + t1 * d;
    }

    let q1 = p1 + t1 * d;
    let q2 = p1 + t2 * d;
    if (l2_squared(q1 - p1) < l2_squared(q2 - p1)) {
        return q1;
    } else {
        return q2;
    }
}

// Find the closest intersection from point p to the origin and the nodes.
fn get_closest_intersection(p: vec2<f32>, ii: u32) -> vec2<f32> {
    var min_d2 = 1000000.0;
    var min_q = p;
    let o = instances[ii].node_offset;
    let n = instances[ii].node_length;
    for (var i: u32 = o; i < o + n; i += 1) {
        let c = nodes[i].position.xy;
        let r = nodes[i].radius;
        let q = get_closest_circle_line_intersection(c, r, p, ORIGIN);
        let d2 = l2_squared(p - q);
        if (d2 < min_d2) {
            min_d2 = d2;
            min_q = q;
        }
    }
    return min_q;
}

// Python-like index wrapping to get the ring of neighboring vertices.
fn wrap_index(i: i32, n: i32) -> i32 {
    if (i < 0) {
        return n + i;
    }
    return i % n;
}

// Mean position of the 5 neighbors of particle i.
fn mean_position5(vi: u32, n: u32, o: i32) -> vec2<f32> {
    let j: i32 = i32(vi) - o;
    let m: i32 = i32(n) - 1;
    return (
          vertices[wrap_index(j - 2, m) + o].position
        + vertices[wrap_index(j - 1, m) + o].position
        + vertices[wrap_index(j + 0, m) + o].position
        + vertices[wrap_index(j + 1, m) + o].position
        + vertices[wrap_index(j + 2, m) + o].position
    ) / 5.0;
}

// Update positions of each particle based on the nodes of the softbody.
@compute @workgroup_size(#{WORKGROUP_SIZE_X})
fn update(in: ComputeInput) {
    let n: u32 = num_vertices_per_instance;
    let i: u32 = in.id.x;
    let ii: u32 = in.id.y;
    let vi: u32 = ii * n + i;
    let o: i32 = i32(ii * n + 1); // n is a center, so we should shift to n+1

    if (i == 0) {
        vertices[vi].position = vec2<f32>(0.0, 0.0);
        return;
    }
    
    let position0 = get_position(vi, n);
    vertices[vi].position = get_closest_intersection(position0, ii);

    for (var s: u32 = 0; s < uniforms.smooth_steps; s += 1) {   
        storageBarrier();
        vertices[vi].position = mean_position5(vi, n, o);
    }
}

#define_import_path bevy_amoeba::rand

var<private> seed : u32 = 0u;

// Rand: PCG
// https://www.reedbeta.com/blog/hash-functions-for-gpu-rendering/
fn pcg_hash(input: u32) -> u32 {
    var state: u32 = input * 747796405u + 2891336453u;
    var word: u32 = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

/// Random floating-point number in [0:1] from PCG hash
fn to_float01(u: u32) -> f32 {
    // Note: could generate only 24 bits of randomness
    return bitcast<f32>((u & 0x007fffffu) | 0x3f800000u) - 1.;
}

// Random floating-point number in [0:1]
fn frand() -> f32 {
    seed = pcg_hash(seed);
    return to_float01(pcg_hash(seed));
}

// Random floating-point number in [0:1]^2
fn frand2() -> vec2<f32> {
    seed = pcg_hash(seed);
    var x = to_float01(seed);
    seed = pcg_hash(seed);
    var y = to_float01(seed);
    return vec2<f32>(x, y);
}

// Random floating-point number in [0:1]^3
fn frand3() -> vec3<f32> {
    seed = pcg_hash(seed);
    var x = to_float01(seed);
    seed = pcg_hash(seed);
    var y = to_float01(seed);
    seed = pcg_hash(seed);
    var z = to_float01(seed);
    return vec3<f32>(x, y, z);
}

// Random floating-point number in [0:1]^4
fn frand4() -> vec4<f32> {
    // Each rand() produces 32 bits, and we need 24 bits per component,
    // so can get away with only 3 calls.
    var r0 = pcg_hash(seed);
    var r1 = pcg_hash(r0);
    var r2 = pcg_hash(r1);
    seed = r2;
    var x = to_float01(r0);
    var r01 = (r0 & 0xff000000u) >> 8u | (r1 & 0x0000ffffu);
    var y = to_float01(r01);
    var r12 = (r1 & 0xffff0000u) >> 8u | (r2 & 0x000000ffu);
    var z = to_float01(r12);
    var r22 = r2 >> 8u;
    var w = to_float01(r22);
    return vec4<f32>(x, y, z, w);
}

fn rand_uniform_f(a: f32, b: f32) -> f32 {
    return a + frand() * (b - a);
}

fn rand_uniform_vec2(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return a + frand2() * (b - a);
}

fn rand_uniform_vec3(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
    return a + frand3() * (b - a);
}

fn rand_uniform_vec4(a: vec4<f32>, b: vec4<f32>) -> vec4<f32> {
    return a + frand4() * (b - a);
}

// Normal distribution computed using Box-Muller transform
fn rand_normal_f(mean: f32, std_dev: f32) -> f32 {
    var u = frand();
    var v = frand();
    var r = sqrt(-2.0 * log(u));
    return mean + std_dev * r * cos(TAU * v);
}

fn rand_normal_vec2(mean: vec2f, std_dev: vec2f) -> vec2f {
    var u = frand();
    var v = frand2();
    var r = sqrt(-2.0 * log(u));
    return mean + std_dev * r * cos(TAU * v);
}

fn rand_normal_vec3(mean: vec3f, std_dev: vec3f) -> vec3f {
    var u = frand();
    var v = frand3();
    var r = sqrt(-2.0 * log(u));
    return mean + std_dev * r * cos(TAU * v);
}

fn rand_normal_vec4(mean: vec4f, std_dev: vec4f) -> vec4f {
    var u = frand();
    var v = frand4();
    var r = sqrt(-2.0 * log(u));
    return mean + std_dev * r * cos(TAU * v);
}

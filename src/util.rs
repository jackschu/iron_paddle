pub const DEPTH: f32 = 800.;
const PERSPECTIVE: f32 = 250.0;

pub fn scale_project(x: f32, depth: f32, is_deep: bool) -> f32 {
    let depth = if is_deep { DEPTH - depth } else { depth };
    return x * PERSPECTIVE / (depth + PERSPECTIVE);
}

pub fn point_project(x: f32, y: f32, z: f32) -> (f32, f32) {
    let xp = scale_project(x, z, false);
    let yp = scale_project(y, z, false);

    return (xp, yp);
}

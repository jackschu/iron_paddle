const PERSPECTIVE: f32 = 250.0;

pub fn scale_project(x: f32, depth: f32) -> f32 {
    return x * PERSPECTIVE / (depth + PERSPECTIVE);
}

pub fn point_project(x: f32, y: f32, z: f32) -> (f32, f32) {
    let xp = scale_project(x, z);
    let yp = scale_project(y, z);

    return (xp, yp);
}

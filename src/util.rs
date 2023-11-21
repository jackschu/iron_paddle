use bevy::prelude::*;

const PERSPECTIVE: f32 = 250.0;

pub fn point_project(x: f32, y: f32, z: f32) -> Transform {
    let xp = (x * PERSPECTIVE) / (z + PERSPECTIVE);
    let yp = (y * PERSPECTIVE) / (z + PERSPECTIVE);

    return Transform::from_xyz(xp, yp, 0.0);
}

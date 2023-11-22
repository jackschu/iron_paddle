use crate::{components::Ball, util::scale_project, IsDeepPlayer};
use bevy::prelude::*;
use bevy_ggrs::Rollback;

pub fn ball_movement(
    mut query: Query<(&mut Transform, &mut Ball), With<Rollback>>,
    time: Res<Time>,
    is_deep: Res<IsDeepPlayer>,
) {
    let (mut transform, mut ball) = query.single_mut();
    ball.pos.x = ball.pos.x + ball.speed.x * time.delta_seconds();
    ball.pos.y = ball.pos.y + ball.speed.y * time.delta_seconds();
    ball.pos.z = ball.pos.z + ball.speed.z * time.delta_seconds();
    transform.translation.x = scale_project(ball.pos.x, ball.pos.z);
    transform.translation.y = scale_project(ball.pos.y, ball.pos.z);

    let sided_scale = if is_deep.0 {
        800. - ball.pos.z
    } else {
        ball.pos.z
    };
    let scale = scale_project(1., sided_scale);

    transform.scale = (scale, scale, scale).into();
}

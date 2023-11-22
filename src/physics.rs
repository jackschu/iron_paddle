use crate::{
    components::Ball,
    util::{scale_project, DEPTH},
    IsDeepPlayer,
};
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_ggrs::Rollback;

pub fn ball_movement(
    mut query: Query<(&mut Transform, &mut Ball), With<Rollback>>,
    time: Res<Time>,
    is_deep: Res<IsDeepPlayer>,
    q_window: Query<&Window, With<PrimaryWindow>>,
) {
    let window = q_window.single();
    let max_x = window.width() / 2.;
    let max_y = window.height() / 2.;

    let (mut transform, mut ball) = query.single_mut();
    if ball.pos.z >= DEPTH || ball.pos.z <= 0.0 {
        ball.speed.z *= -1.
    }
    if ball.pos.x.abs() - max_x > 0. {
        ball.speed.x *= -1.
    }
    if ball.pos.y.abs() - max_y > 0. {
        ball.speed.y *= -1.
    }
    let ds = time.delta_seconds();

    ball.pos.x += ball.speed.x * ds;
    ball.pos.y += ball.speed.y * ds;
    ball.pos.z += ball.speed.z * ds;
    transform.translation.x = scale_project(ball.pos.x, ball.pos.z, is_deep.0);
    transform.translation.y = scale_project(ball.pos.y, ball.pos.z, is_deep.0);
    info!("ball pos {} {} {}", ball.pos.y, ball.speed.y, ds);

    let scale = scale_project(1., ball.pos.z, is_deep.0);

    transform.scale = (scale, scale, scale).into();
}

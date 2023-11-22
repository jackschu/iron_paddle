use bevy::{prelude::*, window::PrimaryWindow};
use bevy_ggrs::Rollback;
use bevy_prototype_lyon::prelude::*;

use crate::{
    components::{Ball, DepthIndicator},
    util::{point_project, scale_project, DEPTH},
    IsDeepPlayer,
};

pub fn setup_grid_system(mut commands: Commands, q_window: Query<&Window, With<PrimaryWindow>>) {
    let window = q_window.single();

    let path = draw_box(window, DEPTH / 2.);
    commands.spawn((
        ShapeBundle {
            path,
            spatial: SpatialBundle {
                transform: Transform::from_xyz(0., 0., -10.),
                ..default()
            },
            ..default()
        },
        Stroke::new(Color::BLUE, scale_project(4., DEPTH / 2., false)),
        DepthIndicator,
    ));

    for i in 0..9 {
        let path = draw_box(window, i as f32 * 100.);
        commands.spawn((
            ShapeBundle {
                path,
                spatial: SpatialBundle {
                    transform: Transform::from_xyz(0., 0., -10.),
                    ..default()
                },
                ..default()
            },
            Stroke::new(
                Color::GREEN,
                scale_project(4., i as f32 * (DEPTH / 8.0), false),
            ),
        ));
    }
    let dx: [f32; 4] = [-1., 1., 1., -1.];
    let dy: [f32; 4] = [1., 1., -1., -1.];
    for i in 0..4 {
        let x = window.width() / 2. * dx[i];
        let y = window.height() / 2. * dy[i];
        let (xp, yp) = point_project(x, y, DEPTH);
        let mut path_builder = PathBuilder::new();
        path_builder.move_to(Vec2::new(x, y));
        path_builder.line_to(Vec2::new(xp, yp));
        path_builder.close();
        let path = path_builder.build();
        commands.spawn((
            ShapeBundle {
                path,
                spatial: SpatialBundle {
                    transform: Transform::from_xyz(0., 0., -10.),
                    ..default()
                },
                ..default()
            },
            Stroke::new(Color::GREEN, 3.),
        ));
    }
}

pub fn update_depth_indicator(
    query: Query<&Ball, With<Rollback>>,
    is_deep: Res<IsDeepPlayer>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut indicator: Query<&mut Path, With<DepthIndicator>>,
) {
    let window = q_window.single();
    let ball = query.single();

    let path = draw_box(
        window,
        if is_deep.0 {
            DEPTH - ball.pos.z
        } else {
            ball.pos.z
        },
    );
    let mut depth_path = indicator.single_mut();
    *depth_path = path;
}

fn draw_box(window: &Window, depth: f32) -> Path {
    let mut path_builder = PathBuilder::new();

    let dx: [f32; 4] = [-1., 1., 1., -1.];
    let dy: [f32; 4] = [1., 1., -1., -1.];

    for i in 0..4 {
        let x = window.width() / 2. * dx[i];
        let y = window.height() / 2. * dy[i];
        let (xp, yp) = point_project(x, y, depth);
        if i == 0 {
            path_builder.move_to(Vec2::new(xp, yp));
        } else {
            path_builder.line_to(Vec2::new(xp, yp));
        }
    }

    path_builder.close();
    path_builder.build()
}

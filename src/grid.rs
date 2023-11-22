use bevy::{prelude::*, window::PrimaryWindow};
use bevy_prototype_lyon::prelude::*;

use crate::util::{point_project, scale_project};

pub fn setup_grid_system(mut commands: Commands, q_window: Query<&Window, With<PrimaryWindow>>) {
    let window = q_window.single();

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
            Stroke::new(Color::GREEN, scale_project(10.0, i as f32 * 100.)),
        ));
    }
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

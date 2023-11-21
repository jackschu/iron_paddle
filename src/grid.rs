use bevy::{prelude::*, window::PrimaryWindow};

use crate::util::point_project;

pub fn setup_grid_system(mut commands: Commands, q_window: Query<&Window, With<PrimaryWindow>>) {
    let window = q_window.single();

    for i in 1..10 {
        commands.spawn(SpriteBundle {
            transform: point_project(
                window.width() / -2.0,
                window.height() / 2.0,
                (i as f32) * 50.0,
            ),
            sprite: Sprite {
                color: Color::GREEN,
                custom_size: Some(Vec2::new(5.0, 5.0)),
                ..default()
            },
            ..default()
        });
    }
}

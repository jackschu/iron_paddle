use bevy::{prelude::*, utils::HashSet};

use bevy::core::Zeroable;
use bevy::utils::HashMap;
use bevy::window::PrimaryWindow;
use bevy_ggrs::{GgrsConfig, LocalInputs, LocalPlayers, PlayerInputs, Rollback};
use bevy_matchbox::prelude::PeerId;
use bytemuck::Pod;

use crate::util::scale_project;
use crate::{components::Paddle, MainCamera};

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Pod, Zeroable, Debug)]
pub struct BoxInput {
    pub inp: i64,
    pub inp2: i64,
}

pub type GGRSConfig = GgrsConfig<BoxInput, PeerId>;

pub fn my_cursor_system(
    mut commands: Commands,
    // query to get the window (so we can read the current cursor position)
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    touches: Res<Touches>,
    local_players: Res<LocalPlayers>,
) {
    let mut local_inputs = HashMap::new();
    for handle in &local_players.0 {
        local_inputs.insert(
            *handle,
            BoxInput {
                inp: 0 as i64,
                inp2: 0 as i64,
            },
        );
    }

    // get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK
    if let Ok((camera, camera_transform)) = q_camera.get_single() {
        // There is only one primary window, so we can similarly get it from the query:
        let window = q_window.single();

        let mut initial_pos = window.cursor_position();
        for finger in touches.iter() {
            initial_pos = Some(finger.position());
        }

        // check if the cursor is inside the window and get its position
        // then, ask bevy to convert into world coordinates, and truncate to discard Z
        if let Some(world_position) = initial_pos
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate())
        {
            for handle in &local_players.0 {
                local_inputs.insert(
                    *handle,
                    BoxInput {
                        inp: (world_position.x * 1000.0).floor() as i64,
                        inp2: (world_position.y * 1000.0).floor() as i64,
                    },
                );
            }
        }
    }

    commands.insert_resource(LocalInputs::<GGRSConfig>(local_inputs));
}

/// The sprite is animated by changing its translation depending on the time that has passed since
/// the last frame.
pub fn paddle_movement(
    mut query: Query<(&mut Sprite, &mut Transform, &mut Paddle), With<Rollback>>,
    inputs: Res<PlayerInputs<GGRSConfig>>,
    local_players: Res<LocalPlayers>,
) {
    let mut set = HashSet::default();
    for handle in &local_players.0 {
        set.insert(handle);
    }
    for (mut sprite, mut transform, p) in query.iter_mut() {
        let is_self = set.contains(&p.handle);
        let depth = if is_self { 0. } else { 800. };
        let input = inputs[p.handle].0;
        transform.translation.x = scale_project((input.inp as f32) / 1000.0, depth);
        transform.translation.y = scale_project((input.inp2 as f32) / 1000.0, depth);

        // TODO: move this stuff to some setup phase?
        transform.translation.z = if is_self { 1. } else { -1. };
        sprite.custom_size = Some(Vec2::new(
            scale_project(200.0, depth),
            scale_project(150.0, depth),
        ));
    }
}

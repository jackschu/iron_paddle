use bevy::prelude::*;
#[derive(Default, Component)]
pub struct Paddle {
    pub handle: usize,
}

/// Used to help identify our main camera
#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct Ball;

use bevy::prelude::*;
#[derive(Default, Component)]
pub struct Paddle {
    pub handle: usize,
}

/// Used to help identify our main camera
#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct DepthIndicator;

#[derive(Component, Clone, Copy)]
pub struct Ball {
    pub pos: Vec3,
    pub speed: Vec3,
}

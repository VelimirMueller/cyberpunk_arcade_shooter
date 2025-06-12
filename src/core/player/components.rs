use bevy::prelude::*;
#[derive(Component)]
pub struct Player {
    pub current: u32,
    pub max: u32,
}
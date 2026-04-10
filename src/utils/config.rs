use bevy::prelude::*;

/// Runtime quality tier for Bevy systems that take Res<>.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource)]
pub enum QualityTier {
    Desktop,
    Mobile,
}

impl Default for QualityTier {
    fn default() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            QualityTier::Mobile
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            QualityTier::Desktop
        }
    }
}

// --- Compile-time platform constants ---
// Used by non-system functions (attacks, spawning helpers) to avoid
// threading Res<QualityTier> through every function signature.

#[cfg(target_arch = "wasm32")]
pub const ENTITY_SCALE: f32 = 0.85;
#[cfg(not(target_arch = "wasm32"))]
pub const ENTITY_SCALE: f32 = 1.0;

#[cfg(target_arch = "wasm32")]
pub const STAR_COUNT: usize = 25;
#[cfg(not(target_arch = "wasm32"))]
pub const STAR_COUNT: usize = 40;

#[cfg(target_arch = "wasm32")]
pub const DEATH_PARTICLE_MIN: u32 = 6;
#[cfg(target_arch = "wasm32")]
pub const DEATH_PARTICLE_MAX: u32 = 10;
#[cfg(not(target_arch = "wasm32"))]
pub const DEATH_PARTICLE_MIN: u32 = 12;
#[cfg(not(target_arch = "wasm32"))]
pub const DEATH_PARTICLE_MAX: u32 = 20;

#[cfg(target_arch = "wasm32")]
pub const AFTERIMAGE_INTERVAL: f32 = 0.10;
#[cfg(not(target_arch = "wasm32"))]
pub const AFTERIMAGE_INTERVAL: f32 = 0.05;

#[cfg(target_arch = "wasm32")]
pub const AMBIENT_PARTICLE_INTERVAL: f32 = 0.8;
#[cfg(not(target_arch = "wasm32"))]
pub const AMBIENT_PARTICLE_INTERVAL: f32 = 0.4;

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource)]
pub enum QualityTier {
    Low,
    High,
}

#[allow(clippy::derivable_impls)] // not derivable: default differs per target (Low on wasm32)
impl Default for QualityTier {
    fn default() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            QualityTier::Low
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            QualityTier::High
        }
    }
}

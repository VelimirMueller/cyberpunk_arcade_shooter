use crate::core::player::components::Player;
use crate::systems::audio::{SoundEffect, SoundEvent};
use bevy::prelude::*;

pub const PHASE_SHIFT_DURATION: f32 = 2.0;

#[derive(Component)]
pub struct PhaseShiftActive(pub Timer);

/// Apply Phase Shift: add or refresh `PhaseShiftActive` on player; play sound.
/// If the component already exists, its timer is reset to full.
pub fn apply_phase_shift(
    commands: &mut Commands,
    player_entity: Entity,
    existing: Option<&mut PhaseShiftActive>,
    sound_events: &mut EventWriter<SoundEvent>,
) {
    let fresh = Timer::from_seconds(PHASE_SHIFT_DURATION, TimerMode::Once);
    if let Some(active) = existing {
        active.0 = fresh;
    } else {
        commands.entity(player_entity).insert(PhaseShiftActive(fresh));
    }
    sound_events.write(SoundEvent(SoundEffect::PhaseShiftPickup));
}

/// Tick timer and remove component when done. Also drives sprite alpha flicker.
pub fn phase_shift_tick_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut PhaseShiftActive, &mut Sprite), With<Player>>,
) {
    let t = time.elapsed_secs();
    for (entity, mut active, mut sprite) in query.iter_mut() {
        active.0.tick(time.delta());
        if active.0.finished() {
            commands.entity(entity).remove::<PhaseShiftActive>();
            // Restore base color (full alpha); approximated by preserving current color channels at alpha 1.0
            let [r, g, b, _a] = sprite.color.to_srgba().to_f32_array();
            sprite.color = Color::srgba(r, g, b, 1.0);
        } else {
            // Flicker alpha at 8 Hz between 0.35 and 0.75
            let flicker = 0.35 + 0.40 * (0.5 + 0.5 * (t * 8.0 * std::f32::consts::TAU).sin());
            let [r, g, b, _a] = sprite.color.to_srgba().to_f32_array();
            sprite.color = Color::srgba(r, g, b, flicker);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_shift_duration_matches_constant() {
        let timer = Timer::from_seconds(PHASE_SHIFT_DURATION, TimerMode::Once);
        assert!((timer.duration().as_secs_f32() - PHASE_SHIFT_DURATION).abs() < f32::EPSILON);
    }

    #[test]
    fn timer_reset_is_full_duration() {
        // Simulates the "refresh on re-pickup" rule in isolation
        let fresh = Timer::from_seconds(PHASE_SHIFT_DURATION, TimerMode::Once);
        assert_eq!(fresh.fraction(), 0.0);
        assert!(!fresh.finished());
    }
}

use crate::app::GameEntity;
use crate::systems::audio::{SoundEffect, SoundEvent};
use crate::utils::config::{ENTITY_SCALE, QualityTier};
use bevy::prelude::*;
use rand::Rng;

pub const BLINK_MIN_BOSS_DIST: f32 = 200.0;
pub const BLINK_CANDIDATES: usize = 20;
pub const BLINK_BOUNDS: (f32, f32, f32, f32) = (-550.0, 550.0, -300.0, 300.0);

#[derive(Component)]
pub struct BlinkParticle {
    pub lifetime: Timer,
}

/// Choose a teleport destination minimizing a threat-weighted score.
/// Pure function — testable without a Bevy world.
pub fn pick_safe_spot(
    boss_pos: Vec2,
    threats: &[Vec2],
    bounds: (f32, f32, f32, f32),
    candidates: &[Vec2],
) -> Vec2 {
    // Filter candidates that meet the boss-distance minimum
    let mut filtered: Vec<Vec2> = candidates
        .iter()
        .copied()
        .filter(|c| (*c - boss_pos).length() >= BLINK_MIN_BOSS_DIST)
        .collect();

    // Fallback: if none pass the hard gate, use farthest corner from boss
    if filtered.is_empty() {
        let (x_min, x_max, y_min, y_max) = bounds;
        let corners = [
            Vec2::new(x_min, y_min),
            Vec2::new(x_min, y_max),
            Vec2::new(x_max, y_min),
            Vec2::new(x_max, y_max),
        ];
        return corners
            .into_iter()
            .max_by(|a, b| {
                let da = (*a - boss_pos).length();
                let db = (*b - boss_pos).length();
                da.partial_cmp(&db).unwrap()
            })
            .expect("4 corners always present");
    }

    // Score each candidate (lower = safer)
    filtered.sort_by(|a, b| {
        let score_a = score_candidate(*a, boss_pos, threats);
        let score_b = score_candidate(*b, boss_pos, threats);
        score_a.partial_cmp(&score_b).unwrap()
    });
    filtered[0]
}

fn score_candidate(candidate: Vec2, boss_pos: Vec2, threats: &[Vec2]) -> f32 {
    let mut score = 0.0;
    for threat in threats {
        let d = (candidate - *threat).length();
        score += 1.0 / (d + 10.0);
    }
    score += 1.0 / ((candidate - boss_pos).length() + 10.0);
    score
}

/// Sample N random candidates in bounds.
pub fn sample_candidates(bounds: (f32, f32, f32, f32), n: usize) -> Vec<Vec2> {
    let mut rng = rand::thread_rng();
    let (x_min, x_max, y_min, y_max) = bounds;
    (0..n)
        .map(|_| {
            Vec2::new(
                rng.gen_range(x_min..x_max),
                rng.gen_range(y_min..y_max),
            )
        })
        .collect()
}

/// Spawn a lightning-burst of N particles at `pos` (for blink origin + destination).
pub fn spawn_blink_burst(
    commands: &mut Commands,
    pos: Vec3,
    particle_count: usize,
) {
    let mut rng = rand::thread_rng();
    for _ in 0..particle_count {
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let length = rng.gen_range(6.0..14.0) * ENTITY_SCALE;
        let offset = Vec2::new(angle.cos(), angle.sin()) * length;
        commands.spawn((
            Sprite {
                color: Color::srgba(6.0, 0.5, 8.0, 0.9),
                custom_size: Some(Vec2::new(2.0, length)),
                ..default()
            },
            Transform::from_translation(pos.with_z(0.55) + offset.extend(0.0))
                .with_rotation(Quat::from_rotation_z(angle)),
            BlinkParticle {
                lifetime: Timer::from_seconds(0.2, TimerMode::Once),
            },
            GameEntity,
        ));
    }
}

/// Particle count depends on quality tier.
pub fn blink_particle_count(quality: &QualityTier) -> usize {
    match quality {
        QualityTier::Desktop => 16,
        QualityTier::Mobile => 6,
    }
}

pub fn blink_particle_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut BlinkParticle, &mut Sprite)>,
) {
    for (entity, mut particle, mut sprite) in query.iter_mut() {
        particle.lifetime.tick(time.delta());
        if particle.lifetime.finished() {
            commands.entity(entity).despawn();
            continue;
        }
        let progress = particle.lifetime.fraction();
        let alpha = (1.0 - progress) * 0.9;
        let [r, g, b, _a] = sprite.color.to_srgba().to_f32_array();
        sprite.color = Color::srgba(r, g, b, alpha);
    }
}

/// Sound effect for blink pickup.
pub fn play_blink_sound(sound_events: &mut EventWriter<SoundEvent>) {
    sound_events.write(SoundEvent(SoundEffect::GlitchBlink));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_candidate_far_from_threats() {
        let boss_pos = Vec2::new(0.0, 0.0);
        let threats = vec![Vec2::new(300.0, 0.0), Vec2::new(-300.0, 0.0)];
        let candidates = vec![
            Vec2::new(300.0, 5.0),  // very close to a threat
            Vec2::new(-280.0, 10.0), // very close to other threat
            Vec2::new(0.0, 400.0),  // far from both (passes boss-dist gate)
        ];
        let bounds = (-550.0, 550.0, -300.0, 300.0);
        let result = pick_safe_spot(boss_pos, &threats, bounds, &candidates);
        assert_eq!(result, Vec2::new(0.0, 400.0));
    }

    #[test]
    fn filters_candidates_too_close_to_boss() {
        let boss_pos = Vec2::new(0.0, 0.0);
        let threats = vec![];
        // Two candidates: one within 200px of boss (filtered), one outside (kept)
        let candidates = vec![Vec2::new(50.0, 50.0), Vec2::new(250.0, 0.0)];
        let bounds = (-550.0, 550.0, -300.0, 300.0);
        let result = pick_safe_spot(boss_pos, &threats, bounds, &candidates);
        assert_eq!(result, Vec2::new(250.0, 0.0));
    }

    #[test]
    fn fallback_to_corner_when_all_too_close() {
        let boss_pos = Vec2::new(0.0, 0.0);
        let threats = vec![];
        // All candidates too close to boss
        let candidates = vec![Vec2::new(50.0, 50.0), Vec2::new(-50.0, 50.0)];
        let bounds = (-550.0, 550.0, -300.0, 300.0);
        let result = pick_safe_spot(boss_pos, &threats, bounds, &candidates);
        // Farthest corner from (0,0) is one of the four — expect length ~= sqrt(550^2 + 300^2)
        let dist = (result - boss_pos).length();
        assert!(dist > BLINK_MIN_BOSS_DIST, "result dist = {}", dist);
    }

    #[test]
    fn particle_count_varies_by_quality() {
        assert_eq!(blink_particle_count(&QualityTier::Desktop), 16);
        assert_eq!(blink_particle_count(&QualityTier::Mobile), 6);
    }
}

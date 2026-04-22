use crate::app::{GameEntity, ScreenShake};
use crate::core::boss::components::Boss;
use crate::core::boss::components::{BossProjectile, ChargeTelegraph, DashTrail, HazardZone};
use crate::core::player::components::Player;
use crate::systems::audio::SoundEvent;
use crate::systems::collision::collide;
use crate::systems::combat::EnemyParticle;
use crate::utils::config::ENTITY_SCALE;
use bevy::prelude::*;

pub mod catalog;
pub mod effects;
pub use catalog::{PowerUpKind, PowerUpTier, meta};
#[allow(unused_imports)]
pub use effects::laser::{
    LASER_ACTIVE_DURATION, LASER_CHARGE_DURATION, LASER_FADE_DURATION, LASER_TOTAL_DURATION,
    LaserActive, LaserBeamCore, LaserBeamShell, LaserChargeOrb, LaserChargeParticle, LaserImpact,
    LaserMuzzle, LaserPhase, LaserStreamParticle, laser_charge_orb_system,
    laser_charge_particle_system, laser_impact_system, laser_phase_from_elapsed,
    laser_stream_particle_system, laser_system,
};
pub use effects::shockwave::{PowerUpShockwave, powerup_shockwave_system};

#[derive(Component)]
pub struct PowerUp {
    pub kind: PowerUpKind,
    pub lifetime: Timer,
}

#[derive(Component)]
pub struct PowerUpGlow;

#[derive(Resource)]
pub struct PowerUpTimer {
    pub timer: Timer,
}

pub fn setup_powerup_timer(mut commands: Commands) {
    let duration = 15.0 + rand::random::<f32>() * 5.0;
    commands.insert_resource(PowerUpTimer {
        timer: Timer::from_seconds(duration, TimerMode::Once),
    });
}

pub fn powerup_spawn_system(
    mut commands: Commands,
    time: Res<Time>,
    mut powerup_timer: ResMut<PowerUpTimer>,
    existing_powerups: Query<&PowerUp>,
) {
    powerup_timer.timer.tick(time.delta());
    if !powerup_timer.timer.finished() {
        return;
    }

    // Only 1 power-up on screen at a time
    if !existing_powerups.is_empty() {
        let duration = 15.0 + rand::random::<f32>() * 5.0;
        powerup_timer.timer = Timer::from_seconds(duration, TimerMode::Once);
        return;
    }

    // Random position within play bounds
    let x = (rand::random::<f32>() - 0.5) * 1000.0;
    let y = (rand::random::<f32>() - 0.5) * 400.0;

    let kind = catalog::roll_random_kind();
    let meta = meta(kind);
    let color = meta.color;
    let base_size = meta.tier.base_size_px();

    let pickup_entity = commands
        .spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::new(base_size * ENTITY_SCALE, base_size * ENTITY_SCALE)),
                ..default()
            },
            Transform::from_xyz(x, y, 0.5)
                .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
            PowerUp {
                kind,
                lifetime: Timer::from_seconds(10.0, TimerMode::Once),
            },
            GameEntity,
        ))
        .id();

    if let Some((scale, _alpha, glow_color)) = meta.tier.glow() {
        commands.entity(pickup_entity).with_children(|children| {
            children.spawn((
                Sprite {
                    color: glow_color,
                    custom_size: Some(Vec2::new(
                        base_size * ENTITY_SCALE * scale,
                        base_size * ENTITY_SCALE * scale,
                    )),
                    ..default()
                },
                // z offset behind the main diamond
                Transform::from_xyz(0.0, 0.0, -0.05),
                PowerUpGlow,
            ));
        });
    }

    // Reset timer for next spawn
    let duration = 15.0 + rand::random::<f32>() * 5.0;
    powerup_timer.timer = Timer::from_seconds(duration, TimerMode::Once);
}

pub fn powerup_lifetime_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut PowerUp, &mut Sprite)>,
) {
    let t = time.elapsed_secs();
    for (entity, mut powerup, mut sprite) in query.iter_mut() {
        powerup.lifetime.tick(time.delta());

        let hz = meta(powerup.kind).tier.pulse_hz();
        let pulse = 0.6 + 0.4 * (t * hz).sin();
        let base = meta(powerup.kind).color;
        // Preserve original HDR RGB; pulse modulates alpha only
        let [r, g, b, _a] = base.to_srgba().to_f32_array();
        sprite.color = Color::srgba(r, g, b, pulse);

        if powerup.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn powerup_pickup_system(
    mut commands: Commands,
    mut player_query: Query<(Entity, &Transform, &Sprite, &mut Player)>,
    powerup_query: Query<(Entity, &Transform, &Sprite, &PowerUp)>,
    mut boss_query: Query<&mut Boss>,
    enemy_particle_query: Query<Entity, With<EnemyParticle>>,
    boss_projectile_query: Query<Entity, With<BossProjectile>>,
    dash_trail_query: Query<Entity, With<DashTrail>>,
    hazard_zone_query: Query<Entity, With<HazardZone>>,
    telegraph_query: Query<Entity, With<ChargeTelegraph>>,
    mut screen_shake: ResMut<ScreenShake>,
    mut sound_events: EventWriter<SoundEvent>,
) {
    let Ok((player_entity, player_transform, player_sprite, mut player)) =
        player_query.single_mut()
    else {
        return;
    };
    let player_pos = player_transform.translation;
    let player_size = player_sprite.custom_size.unwrap_or(Vec2::ONE);

    for (powerup_entity, powerup_transform, powerup_sprite, powerup) in &powerup_query {
        let powerup_pos = powerup_transform.translation;
        let powerup_size = powerup_sprite.custom_size.unwrap_or(Vec2::new(16.0, 16.0));

        if !collide(player_pos, player_size, powerup_pos, powerup_size) {
            continue;
        }

        // Consume power-up
        commands.entity(powerup_entity).despawn();

        match powerup.kind {
            PowerUpKind::Shockwave => {
                crate::systems::powerups::effects::shockwave::apply_shockwave(
                    &mut commands,
                    player_pos,
                    &mut boss_query,
                    &enemy_particle_query,
                    &boss_projectile_query,
                    &dash_trail_query,
                    &hazard_zone_query,
                    &telegraph_query,
                    &mut screen_shake,
                    &mut sound_events,
                );
            }
            PowerUpKind::Laser => {
                crate::systems::powerups::effects::laser::apply_laser_pickup(
                    &mut commands,
                    player_entity,
                    player_pos,
                    &mut screen_shake,
                    &mut sound_events,
                );
            }
            PowerUpKind::RepairKit => {
                crate::systems::powerups::effects::instant::apply_repair_kit(
                    &mut player,
                    &mut sound_events,
                );
            }
            PowerUpKind::EnergyCell => {
                crate::systems::powerups::effects::instant::apply_energy_cell(
                    &mut player,
                    &mut sound_events,
                );
            }
            PowerUpKind::PhaseShift | PowerUpKind::GlitchBlink => {
                // Implemented in Tasks 9 and 10
            }
        }
    }
}

use crate::app::GameEntity;
use crate::app::{GameData, ScreenShake};
use crate::core::boss::attacks;
use crate::core::boss::components::*;
use crate::core::player::components::Player;
use crate::systems::audio::{SoundEffect, SoundEvent};
use crate::systems::collision::DeathEvent;
use bevy::prelude::*;

pub fn score_multiplier(round: u32) -> f32 {
    match round {
        1 => 1.0,
        2 => 1.5,
        3 => 2.0,
        4 => 2.5,
        5 => 3.0,
        _ => 3.0,
    }
}

#[derive(Component)]
pub struct PhaseShiftText {
    pub timer: Timer,
}

#[derive(Component)]
pub struct PhaseFlashEffect {
    pub timer: Timer,
}

pub fn boss_type_for_round(round: u32) -> BossType {
    match round {
        1 => BossType::GridPhantom,
        2 => BossType::NeonSentinel,
        3 => BossType::ChromeBerserker,
        4 => BossType::VoidWeaver,
        5 => BossType::ApexProtocol,
        _ => BossType::ApexProtocol,
    }
}

fn boss_config(boss_type: BossType) -> (u32, TransitionStyle, Color, f32) {
    match boss_type {
        BossType::GridPhantom => (
            150,
            TransitionStyle::Stagger,
            Color::srgb(0.0, 8.0, 8.0),
            1.0,
        ),
        BossType::NeonSentinel => (
            200,
            TransitionStyle::Stagger,
            Color::srgb(8.0, 0.0, 8.0),
            1.2,
        ),
        BossType::ChromeBerserker => (
            250,
            TransitionStyle::RageBurst,
            Color::srgb(8.0, 4.0, 0.0),
            1.4,
        ),
        BossType::VoidWeaver => (
            300,
            TransitionStyle::Stagger,
            Color::srgb(4.0, 0.0, 8.0),
            1.1,
        ),
        BossType::ApexProtocol => (
            400,
            TransitionStyle::RageBurst,
            Color::srgb(8.0, 8.0, 8.0),
            1.6,
        ),
    }
}

pub fn spawn_boss(commands: &mut Commands, round: u32) {
    let boss_type = boss_type_for_round(round);
    let (max_hp, transition_style, color, size_mult) = boss_config(boss_type);
    let base_size = 50.0;
    let size = base_size * size_mult;

    let primary_timer = match boss_type {
        BossType::GridPhantom => Timer::from_seconds(3.0, TimerMode::Repeating),
        BossType::NeonSentinel => Timer::from_seconds(4.0, TimerMode::Repeating),
        BossType::ChromeBerserker => Timer::from_seconds(2.8, TimerMode::Repeating),
        BossType::VoidWeaver => Timer::from_seconds(5.0, TimerMode::Repeating),
        BossType::ApexProtocol => Timer::from_seconds(3.0, TimerMode::Repeating),
    };

    // Scale attack speed by round: 0.9^(round-1) makes bosses attack faster each round
    let speed_scale = 0.9_f32.powi((round as i32) - 1);
    let scaled_duration = primary_timer.duration().as_secs_f32() * speed_scale;
    let primary_timer = Timer::from_seconds(scaled_duration, TimerMode::Repeating);

    commands.spawn((
        Sprite {
            color,
            custom_size: Some(Vec2::new(size, size)),
            ..default()
        },
        Transform::from_xyz(0.0, 150.0, 0.0),
        Boss {
            boss_type,
            phase: BossPhase::Phase1,
            current_hp: max_hp,
            max_hp,
            phase_thresholds: (0.60, 0.30, 0.10),
            transition_style,
            primary_timer,
            secondary_timer: None,
            attack_state: AttackState::Idle,
            base_color: color,
            last_hit_time: None,
            last_laser_hit_time: None,
            combo_count: 0,
            max_combo: 1,
            cycle_index: 0,
            is_invulnerable: false,
        },
        GameEntity,
    ));
}

pub fn boss_phase_system(
    mut commands: Commands,
    mut boss_query: Query<(Entity, &mut Boss, &Transform), Without<PhaseTransitionSequence>>,
) {
    for (entity, mut boss, _transform) in boss_query.iter_mut() {
        if boss.current_hp == 0 {
            continue;
        }
        let new_phase = boss.phase_for_hp_pct();
        if new_phase != boss.phase {
            let shake_intensity = match new_phase {
                BossPhase::Phase2 => 1.0,
                BossPhase::Phase3 => 1.5,
                BossPhase::Phase4 => 2.0,
                BossPhase::Phase1 => 0.0,
            };
            boss.is_invulnerable = true;
            boss.attack_state = AttackState::Idle;
            commands.entity(entity).insert(PhaseTransitionSequence {
                timer: Timer::from_seconds(0.3, TimerMode::Once),
                step: TransitionStep::DimScreen,
                target_phase: new_phase,
                shake_intensity,
            });
        }
    }
}

pub fn phase_transition_system(
    time: Res<Time>,
    mut commands: Commands,
    mut boss_query: Query<(Entity, &mut Boss, &mut PhaseTransitionSequence, &Transform)>,
    mut screen_shake: ResMut<ScreenShake>,
    mut sound_events: EventWriter<SoundEvent>,
    dim_query: Query<Entity, With<ScreenDimOverlay>>,
) {
    for (entity, mut boss, mut seq, boss_transform) in boss_query.iter_mut() {
        seq.timer.tick(time.delta());
        if !seq.timer.finished() {
            continue;
        }

        match seq.step {
            TransitionStep::DimScreen => {
                // Spawn a dim overlay
                commands.spawn((
                    Sprite {
                        color: Color::srgba(0.0, 0.0, 0.0, 0.5),
                        custom_size: Some(Vec2::new(1400.0, 800.0)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 0.0, 5.0),
                    ScreenDimOverlay,
                    GameEntity,
                ));
                seq.step = TransitionStep::MorphPulse;
                seq.timer = Timer::from_seconds(0.4, TimerMode::Once);
            }
            TransitionStep::MorphPulse => {
                // Despawn dim overlay
                for dim_entity in dim_query.iter() {
                    commands.entity(dim_entity).despawn();
                }
                seq.step = TransitionStep::PhaseText;
                seq.timer = Timer::from_seconds(0.3, TimerMode::Once);
            }
            TransitionStep::PhaseText => {
                let phase_name = match seq.target_phase {
                    BossPhase::Phase1 => "PHASE 1",
                    BossPhase::Phase2 => "PHASE 2: AWAKENING",
                    BossPhase::Phase3 => "PHASE 3: RAGE",
                    BossPhase::Phase4 => "PHASE 4: DESPERATION",
                };
                commands.spawn((
                    Text::new(phase_name),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::srgba(1.0, 0.5, 0.0, 1.0)),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(50.0),
                        top: Val::Percent(38.0),
                        ..default()
                    },
                    PhaseNameText {
                        timer: Timer::from_seconds(1.5, TimerMode::Once),
                    },
                    GameEntity,
                ));
                seq.step = TransitionStep::ShockwaveRing;
                seq.timer = Timer::from_seconds(0.2, TimerMode::Once);
            }
            TransitionStep::ShockwaveRing => {
                // Spawn a shockwave ring (reuse PhaseFlashEffect)
                commands.spawn((
                    Sprite {
                        color: Color::srgba(8.0, 8.0, 8.0, 0.9),
                        custom_size: Some(Vec2::new(20.0, 20.0)),
                        ..default()
                    },
                    Transform::from_translation(boss_transform.translation),
                    PhaseFlashEffect {
                        timer: Timer::from_seconds(0.4, TimerMode::Once),
                    },
                    GameEntity,
                ));
                sound_events.write(SoundEvent(SoundEffect::RageBurst));
                seq.step = TransitionStep::ScreenShake;
                seq.timer = Timer::from_seconds(0.3, TimerMode::Once);
            }
            TransitionStep::ScreenShake => {
                let intensity = seq.shake_intensity;
                screen_shake.intensity = intensity * 1.5;
                screen_shake.duration = 0.5;
                screen_shake.timer = 0.5;
                sound_events.write(SoundEvent(SoundEffect::PhaseShift));
                seq.step = TransitionStep::Done;
                seq.timer = Timer::from_seconds(0.5, TimerMode::Once);
            }
            TransitionStep::Done => {
                let target_phase = seq.target_phase;
                boss.phase = target_phase;
                boss.is_invulnerable = false;

                // Phase4 desperation: speed up attacks and increase combo
                if target_phase == BossPhase::Phase4 {
                    let current_duration = boss.primary_timer.duration().as_secs_f32();
                    let new_duration = current_duration * 0.6;
                    boss.primary_timer = Timer::from_seconds(new_duration, TimerMode::Repeating);
                    if boss.boss_type == BossType::ChromeBerserker {
                        boss.max_combo = 4;
                    }
                } else if boss.boss_type == BossType::ChromeBerserker {
                    boss.max_combo = match target_phase {
                        BossPhase::Phase1 => 1,
                        BossPhase::Phase2 => 3,
                        BossPhase::Phase3 => 3,
                        BossPhase::Phase4 => 4,
                    };
                }

                commands.entity(entity).remove::<PhaseTransitionSequence>();
            }
        }
    }
}

pub fn phase_name_text_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut PhaseNameText, &mut TextColor)>,
) {
    for (entity, mut text, mut color) in query.iter_mut() {
        text.timer.tick(time.delta());
        let alpha = 1.0 - text.timer.fraction();
        color.0 = Color::srgba(1.0, 0.5, 0.0, alpha);
        if text.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn boss_idle_movement(time: Res<Time>, mut boss_query: Query<(&Boss, &mut Transform)>) {
    for (boss, mut transform) in boss_query.iter_mut() {
        // Sentinel and Weaver handle their own positioning
        if boss.boss_type == BossType::NeonSentinel || boss.boss_type == BossType::VoidWeaver {
            // Sentinel stays stationary (position set in attack fn)
            // Weaver teleports (position set in attack fn)
            // Still allow charging/dashing movement if somehow in that state
            if let AttackState::Dashing { target, speed }
            | AttackState::Charging { target, speed } = &boss.attack_state
            {
                let direction = (*target - transform.translation.truncate()).normalize_or_zero();
                transform.translation += (direction * *speed * time.delta_secs()).extend(0.0);
            }
            continue;
        }

        match &boss.attack_state {
            AttackState::Idle => {
                let t = time.elapsed_secs();
                transform.translation.y = 150.0 + (t * 1.5).sin() * 30.0;
            }
            AttackState::Dashing { target, speed } | AttackState::Charging { target, speed } => {
                let direction = (*target - transform.translation.truncate()).normalize_or_zero();
                transform.translation += (direction * *speed * time.delta_secs()).extend(0.0);
            }
            _ => {}
        }
    }
}

pub fn boss_attack_system(
    time: Res<Time>,
    mut commands: Commands,
    mut boss_query: Query<(&mut Boss, &mut Transform)>,
    player_query: Query<&Transform, (With<Player>, Without<Boss>)>,
    mut screen_shake: ResMut<ScreenShake>,
    hazard_query: Query<&HazardZone>,
    mut sound_events: EventWriter<SoundEvent>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let hazard_count = hazard_query.iter().count();
    for (mut boss, mut boss_transform) in boss_query.iter_mut() {
        let delta = time.delta_secs();
        match boss.boss_type {
            BossType::GridPhantom => {
                attacks::phantom_attack(
                    &mut boss,
                    &boss_transform,
                    player_transform,
                    &mut commands,
                    delta,
                    &mut sound_events,
                );
            }
            BossType::NeonSentinel => {
                attacks::sentinel_attack(
                    &mut boss,
                    &mut boss_transform,
                    player_transform,
                    &mut commands,
                    delta,
                    &mut sound_events,
                );
            }
            BossType::ChromeBerserker => {
                attacks::berserker_attack(
                    &mut boss,
                    &boss_transform,
                    player_transform,
                    &mut commands,
                    delta,
                    &mut screen_shake,
                    &mut sound_events,
                );
            }
            BossType::VoidWeaver => {
                attacks::weaver_attack(
                    &mut boss,
                    &mut boss_transform,
                    player_transform,
                    &mut commands,
                    delta,
                    hazard_count,
                    &mut sound_events,
                );
            }
            BossType::ApexProtocol => {
                attacks::apex_attack(
                    &mut boss,
                    &mut boss_transform,
                    player_transform,
                    &mut commands,
                    delta,
                    &mut screen_shake,
                    hazard_count,
                    &mut sound_events,
                );
            }
        }
    }
}

pub fn hazard_lifetime_system(
    time: Res<Time>,
    mut commands: Commands,
    mut trail_query: Query<(Entity, &mut DashTrail, &mut Sprite)>,
    mut telegraph_query: Query<(Entity, &mut ChargeTelegraph)>,
) {
    for (entity, mut trail, mut sprite) in trail_query.iter_mut() {
        trail.lifetime.tick(time.delta());
        let alpha = 1.0 - trail.lifetime.fraction();
        sprite.color = Color::srgba(0.0, 8.0, 8.0, 0.8 * alpha);
        if trail.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }

    for (entity, mut telegraph) in telegraph_query.iter_mut() {
        telegraph.lifetime.tick(time.delta());
        if telegraph.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn boss_projectile_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut BossProjectile, &mut Transform)>,
    player_query: Query<&Transform, (With<Player>, Without<BossProjectile>)>,
) {
    let player_pos = player_query.single().map(|t| t.translation.truncate()).ok();

    for (entity, mut projectile, mut transform) in query.iter_mut() {
        // Slow homing: steer toward player
        if let Some(player_pos) = player_pos {
            let current_pos = transform.translation.truncate();
            let desired_dir = (player_pos - current_pos).normalize_or_zero();
            let current_dir = projectile.velocity.normalize_or_zero();
            let new_dir = (current_dir + desired_dir * 0.02).normalize_or_zero();
            let speed = projectile.velocity.length();
            projectile.velocity = new_dir * speed;
        }

        transform.translation += (projectile.velocity * time.delta_secs()).extend(0.0);

        // Despawn if off screen
        let pos = transform.translation;
        if pos.x.abs() > 700.0 || pos.y.abs() > 400.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn hazard_zone_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut HazardZone, &mut Transform, &mut Sprite)>,
    mut sound_events: EventWriter<SoundEvent>,
) {
    for (entity, mut hazard, mut transform, mut sprite) in query.iter_mut() {
        hazard.lifetime.tick(time.delta());

        // Drift toward player if velocity set
        if let Some(drift) = hazard.drift_velocity {
            transform.translation += (drift * time.delta_secs()).extend(0.0);
        }

        // Handle explosion timer
        if hazard.explodes {
            if let Some(ref mut exp_timer) = hazard.explosion_timer {
                exp_timer.tick(time.delta());
                if exp_timer.just_finished() {
                    sound_events.write(SoundEvent(SoundEffect::HazardExplode));
                    // Explode: expand to 120x120 magenta, set radius to 60, short lifetime
                    sprite.custom_size = Some(Vec2::new(120.0, 120.0));
                    sprite.color = Color::srgba(8.0, 0.0, 8.0, 0.8);
                    hazard.radius = 60.0;
                    hazard.lifetime = Timer::from_seconds(0.3, TimerMode::Once);
                    hazard.explodes = false; // Don't re-explode
                    hazard.explosion_timer = None;
                }
            }
        }

        // Fade alpha over lifetime
        let alpha = 1.0 - hazard.lifetime.fraction();
        let base_alpha = if hazard.radius >= 60.0 { 0.8 } else { 0.3 };
        let current = sprite.color.to_srgba();
        sprite.color = Color::srgba(
            current.red,
            current.green,
            current.blue,
            base_alpha * (1.0 - alpha),
        );

        // Despawn when lifetime done
        if hazard.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn phase_shift_text_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut PhaseShiftText, &mut TextColor)>,
) {
    for (entity, mut text, mut color) in query.iter_mut() {
        text.timer.tick(time.delta());
        let alpha = 1.0 - text.timer.fraction();
        color.0 = Color::srgba(1.0, 0.2, 0.2, alpha);
        if text.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn phase_flash_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut PhaseFlashEffect, &mut Transform, &mut Sprite)>,
) {
    for (entity, mut flash, mut transform, mut sprite) in query.iter_mut() {
        flash.timer.tick(time.delta());
        let progress = flash.timer.fraction();
        // Expand rapidly
        let scale = 1.0 + progress * 15.0;
        transform.scale = Vec3::splat(scale);
        // Fade out
        let alpha = (1.0 - progress) * 0.9;
        sprite.color = Color::srgba(8.0, 8.0, 8.0, alpha);
        if flash.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn boss_visual_system(
    time: Res<Time>,
    mut boss_query: Query<(
        &Boss,
        &mut Sprite,
        &mut Transform,
        Option<&PhaseTransitionSequence>,
    )>,
) {
    let t = time.elapsed_secs();

    for (boss, mut sprite, mut transform, transition) in boss_query.iter_mut() {
        // MorphPulse: animate scale during transition
        if let Some(seq) = transition {
            if seq.step == TransitionStep::MorphPulse {
                let progress = seq.timer.fraction();
                // 1.0 → 1.2 → 0.8 → 1.0 over the timer
                let scale = if progress < 0.33 {
                    1.0 + (progress / 0.33) * 0.2
                } else if progress < 0.66 {
                    1.2 - ((progress - 0.33) / 0.33) * 0.4
                } else {
                    0.8 + ((progress - 0.66) / 0.34) * 0.2
                };
                transform.scale = Vec3::splat(scale);
            } else {
                transform.scale = Vec3::ONE;
            }
        } else {
            transform.scale = Vec3::ONE;
        }

        let (pulse_alpha, color_mult) = match boss.phase {
            BossPhase::Phase1 => (1.0_f32, 1.0_f32),
            BossPhase::Phase2 => {
                let pulse = 0.7 + 0.3 * (t * std::f32::consts::TAU).sin();
                (pulse, 1.3)
            }
            BossPhase::Phase3 => {
                let pulse = 0.6 + 0.4 * (t * std::f32::consts::TAU / 0.3).sin();
                (pulse, 1.6)
            }
            BossPhase::Phase4 => {
                let flash = (t * 4.0 * std::f32::consts::TAU).sin();
                if flash > 0.0 { (1.0, 2.0) } else { (0.8, 1.0) }
            }
        };

        // Always derive from base_color — never read back from sprite (HDR values degrade)
        let base = boss.base_color.to_srgba();
        sprite.color = Color::srgba(
            base.red * color_mult,
            base.green * color_mult,
            base.blue * color_mult,
            pulse_alpha,
        );
    }
}

pub fn boss_death_check_system(
    mut commands: Commands,
    mut boss_query: Query<(Entity, &mut Boss, &Transform, &Sprite), Without<BossDeathSequence>>,
    mut game_data: ResMut<GameData>,
    mut sound_events: EventWriter<SoundEvent>,
) {
    for (entity, mut boss, transform, sprite) in boss_query.iter_mut() {
        if boss.current_hp == 0 && !boss.is_invulnerable {
            let mult = score_multiplier(game_data.round);
            game_data.score += (100.0 * mult) as u32;
            game_data.enemies_killed += 1;
            sound_events.write(SoundEvent(SoundEffect::Explosion));

            boss.is_invulnerable = true;
            boss.attack_state = AttackState::Idle;

            commands.entity(entity).insert(BossDeathSequence {
                step: DeathStep::Freeze,
                timer: Timer::from_seconds(0.3, TimerMode::Once),
                boss_position: transform.translation,
                boss_color: sprite.color,
                kill_score: (100.0 * mult) as u32,
            });
        }
    }
}

pub fn boss_death_system(
    time: Res<Time>,
    mut commands: Commands,
    mut boss_query: Query<(Entity, &mut BossDeathSequence, &Transform, &Sprite)>,
    mut screen_shake: ResMut<ScreenShake>,
    mut sound_events: EventWriter<SoundEvent>,
    mut death_events: EventWriter<DeathEvent>,
) {
    for (entity, mut death_seq, boss_transform, _boss_sprite) in boss_query.iter_mut() {
        death_seq.timer.tick(time.delta());
        if !death_seq.timer.finished() {
            continue;
        }

        match death_seq.step {
            DeathStep::Freeze => {
                // Brief freeze — screen shake begins
                screen_shake.intensity = 1.5;
                screen_shake.duration = 0.3;
                screen_shake.timer = 0.3;
                death_seq.step = DeathStep::Explosion1;
                death_seq.timer = Timer::from_seconds(0.3, TimerMode::Once);
            }
            DeathStep::Explosion1 => {
                let offset = Vec3::new(
                    (rand::random::<f32>() - 0.5) * 40.0,
                    (rand::random::<f32>() - 0.5) * 40.0,
                    0.1,
                );
                let pos = death_seq.boss_position + offset;
                commands.spawn((
                    Sprite {
                        color: Color::srgba(8.0, 6.0, 2.0, 0.9),
                        custom_size: Some(Vec2::new(10.0, 10.0)),
                        ..default()
                    },
                    Transform::from_translation(pos),
                    DeathExplosion {
                        timer: Timer::from_seconds(0.3, TimerMode::Once),
                    },
                    GameEntity,
                ));
                sound_events.write(SoundEvent(SoundEffect::Explosion));
                screen_shake.intensity = 2.0;
                screen_shake.duration = 0.3;
                screen_shake.timer = 0.3;
                death_seq.step = DeathStep::Explosion2;
                death_seq.timer = Timer::from_seconds(0.3, TimerMode::Once);
            }
            DeathStep::Explosion2 => {
                let offset = Vec3::new(
                    (rand::random::<f32>() - 0.5) * 40.0,
                    (rand::random::<f32>() - 0.5) * 40.0,
                    0.1,
                );
                let pos = death_seq.boss_position + offset;
                commands.spawn((
                    Sprite {
                        color: Color::srgba(8.0, 6.0, 2.0, 0.9),
                        custom_size: Some(Vec2::new(10.0, 10.0)),
                        ..default()
                    },
                    Transform::from_translation(pos),
                    DeathExplosion {
                        timer: Timer::from_seconds(0.3, TimerMode::Once),
                    },
                    GameEntity,
                ));
                sound_events.write(SoundEvent(SoundEffect::Explosion));
                screen_shake.intensity = 2.5;
                screen_shake.duration = 0.3;
                screen_shake.timer = 0.3;
                death_seq.step = DeathStep::Explosion3;
                death_seq.timer = Timer::from_seconds(0.3, TimerMode::Once);
            }
            DeathStep::Explosion3 => {
                let offset = Vec3::new(
                    (rand::random::<f32>() - 0.5) * 40.0,
                    (rand::random::<f32>() - 0.5) * 40.0,
                    0.1,
                );
                let pos = death_seq.boss_position + offset;
                commands.spawn((
                    Sprite {
                        color: Color::srgba(8.0, 6.0, 2.0, 0.9),
                        custom_size: Some(Vec2::new(10.0, 10.0)),
                        ..default()
                    },
                    Transform::from_translation(pos),
                    DeathExplosion {
                        timer: Timer::from_seconds(0.3, TimerMode::Once),
                    },
                    GameEntity,
                ));
                sound_events.write(SoundEvent(SoundEffect::Explosion));
                screen_shake.intensity = 3.0;
                screen_shake.duration = 0.15;
                screen_shake.timer = 0.15;
                death_seq.step = DeathStep::WhiteFlash;
                death_seq.timer = Timer::from_seconds(0.15, TimerMode::Once);
            }
            DeathStep::WhiteFlash => {
                // Spawn a white flash overlay using PhaseFlashEffect
                commands.spawn((
                    Sprite {
                        color: Color::srgba(8.0, 8.0, 8.0, 0.9),
                        custom_size: Some(Vec2::new(20.0, 20.0)),
                        ..default()
                    },
                    Transform::from_translation(death_seq.boss_position),
                    PhaseFlashEffect {
                        timer: Timer::from_seconds(0.4, TimerMode::Once),
                    },
                    GameEntity,
                ));
                death_seq.step = DeathStep::Shatter;
                death_seq.timer = Timer::from_seconds(0.1, TimerMode::Once);
            }
            DeathStep::Shatter => {
                // Fire the DeathEvent — handle_death_events will despawn entity + spawn shatter particles
                let boss_position = death_seq.boss_position;
                let boss_color = death_seq.boss_color;
                death_events.write(DeathEvent {
                    position: boss_position,
                    color: boss_color,
                    entity,
                });
                // Advance to Text (the boss entity will be despawned by handle_death_events,
                // so this component won't tick further — that's fine)
                death_seq.step = DeathStep::Text;
                death_seq.timer = Timer::from_seconds(1.5, TimerMode::Once);
            }
            DeathStep::Text => {
                commands.spawn((
                    Text::new("ELIMINATED"),
                    TextFont {
                        font_size: 32.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.0, 8.0, 8.0)),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(35.0),
                        top: Val::Percent(30.0),
                        ..default()
                    },
                    EliminatedText {
                        timer: Timer::from_seconds(1.5, TimerMode::Once),
                    },
                    GameEntity,
                ));
                death_seq.step = DeathStep::Pause;
                death_seq.timer = Timer::from_seconds(1.5, TimerMode::Once);
            }
            DeathStep::Pause => {
                // Done — nothing more to do. Entity is already despawned by handle_death_events.
            }
        }

        // Ensure boss_transform is used to suppress unused variable warning
        let _ = boss_transform;
    }
}

pub fn death_explosion_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut DeathExplosion, &mut Transform, &mut Sprite)>,
) {
    for (entity, mut explosion, mut transform, mut sprite) in query.iter_mut() {
        explosion.timer.tick(time.delta());
        let progress = explosion.timer.fraction();
        // Expand scale
        let scale = 1.0 + progress * 10.0;
        transform.scale = Vec3::splat(scale);
        // Fade alpha
        let alpha = (1.0 - progress) * 0.9;
        sprite.color = Color::srgba(8.0, 6.0, 2.0, alpha);
        if explosion.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn eliminated_text_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut EliminatedText, &mut TextColor)>,
) {
    for (entity, mut text, mut color) in query.iter_mut() {
        text.timer.tick(time.delta());
        let alpha = 1.0 - text.timer.fraction();
        color.0 = Color::srgb(0.0, 8.0, 8.0).with_alpha(alpha);
        if text.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn desperation_ambient_shake(boss_query: Query<&Boss>, mut screen_shake: ResMut<ScreenShake>) {
    for boss in boss_query.iter() {
        if boss.phase == BossPhase::Phase4 && boss.current_hp > 0 && screen_shake.intensity < 0.3 {
            screen_shake.intensity = 0.3;
            screen_shake.duration = 0.2;
            screen_shake.timer = 0.2;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_multiplier_per_round() {
        assert_eq!(score_multiplier(1), 1.0);
        assert_eq!(score_multiplier(2), 1.5);
        assert_eq!(score_multiplier(3), 2.0);
        assert_eq!(score_multiplier(4), 2.5);
        assert_eq!(score_multiplier(5), 3.0);
        assert_eq!(score_multiplier(6), 3.0);
    }

    #[test]
    fn test_boss_type_for_round() {
        assert_eq!(boss_type_for_round(1), BossType::GridPhantom);
        assert_eq!(boss_type_for_round(2), BossType::NeonSentinel);
        assert_eq!(boss_type_for_round(3), BossType::ChromeBerserker);
        assert_eq!(boss_type_for_round(4), BossType::VoidWeaver);
        assert_eq!(boss_type_for_round(5), BossType::ApexProtocol);
        assert_eq!(boss_type_for_round(6), BossType::ApexProtocol);
    }
}

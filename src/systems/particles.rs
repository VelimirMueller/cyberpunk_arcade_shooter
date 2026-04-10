use crate::app::GameEntity;
use crate::systems::collision::DeathEvent;
use crate::utils::config::{AFTERIMAGE_INTERVAL, AMBIENT_PARTICLE_INTERVAL, DEATH_PARTICLE_MIN, DEATH_PARTICLE_MAX};
use bevy::prelude::*;
use rand::Rng;

// ---- Shatter Particles ----

#[derive(Component)]
pub struct ShatterParticle {
    pub velocity: Vec2,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub gravity: f32,
}

// ---- Shockwave Ring ----

#[derive(Component)]
pub struct ShockwaveRing {
    pub timer: f32,
    pub duration: f32,
    pub max_radius: f32,
}

#[derive(Resource)]
pub struct ShockwaveAssets {
    pub mesh: Handle<Mesh>,
}

pub fn setup_shockwave_assets(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mesh = meshes.add(Circle::new(1.0));
    commands.insert_resource(ShockwaveAssets { mesh });
}

// ---- Death Effect Handler ----

pub fn handle_death_events(
    mut commands: Commands,
    mut death_events: EventReader<DeathEvent>,
    shockwave_assets: Res<ShockwaveAssets>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut rng = rand::thread_rng();

    for event in death_events.read() {
        // Despawn the enemy entity
        commands.entity(event.entity).despawn();

        let particle_count = rng.gen_range(DEATH_PARTICLE_MIN..=DEATH_PARTICLE_MAX);
        for _ in 0..particle_count {
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let speed = rng.gen_range(200.0..400.0);
            let size = rng.gen_range(4.0..8.0);

            commands.spawn((
                Sprite {
                    color: event.color,
                    custom_size: Some(Vec2::splat(size)),
                    ..default()
                },
                Transform::from_translation(event.position),
                ShatterParticle {
                    velocity: Vec2::new(angle.cos() * speed, angle.sin() * speed),
                    lifetime: 0.5,
                    max_lifetime: 0.5,
                    gravity: 150.0,
                },
                GameEntity,
            ));
        }

        // Spawn shockwave ring
        let material = materials.add(ColorMaterial::from(event.color));
        commands.spawn((
            Mesh2d(shockwave_assets.mesh.clone()),
            MeshMaterial2d(material),
            Transform::from_translation(event.position).with_scale(Vec3::ZERO),
            ShockwaveRing {
                timer: 0.0,
                duration: 0.3,
                max_radius: 150.0,
            },
            GameEntity,
        ));
    }
}

// ---- Animation Systems ----

pub fn animate_shatter(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut ShatterParticle, &mut Transform, &mut Sprite)>,
) {
    let dt = time.delta().as_secs_f32();

    for (entity, mut particle, mut transform, mut sprite) in &mut query {
        particle.lifetime -= dt;

        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // Move with velocity + gravity
        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;
        particle.velocity.y -= particle.gravity * dt;

        // Rotate
        transform.rotate_z(5.0 * dt);

        // Fade alpha
        let alpha = (particle.lifetime / particle.max_lifetime).clamp(0.0, 1.0);
        sprite.color = sprite.color.with_alpha(alpha);
    }
}

pub fn animate_shockwave(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut ShockwaveRing, &mut Transform)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    material_query: Query<&MeshMaterial2d<ColorMaterial>>,
) {
    let dt = time.delta().as_secs_f32();

    for (entity, mut ring, mut transform) in &mut query {
        ring.timer += dt;

        if ring.timer >= ring.duration {
            commands.entity(entity).despawn();
            continue;
        }

        let progress = ring.timer / ring.duration;
        let radius = ring.max_radius * progress;
        transform.scale = Vec3::splat(radius);

        // Fade alpha on the material
        if let Ok(mat_handle) = material_query.get(entity) {
            if let Some(material) = materials.get_mut(&mat_handle.0) {
                material.color = material.color.with_alpha(1.0 - progress);
            }
        }
    }
}

// ---- Player Afterimage Trail ----

#[derive(Component)]
pub struct Afterimage {
    pub lifetime: f32,
    pub max_lifetime: f32,
}

#[derive(Resource)]
pub struct AfterimageTimer {
    pub timer: Timer,
}

impl Default for AfterimageTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(AFTERIMAGE_INTERVAL, TimerMode::Repeating),
        }
    }
}

pub fn spawn_afterimages(
    time: Res<Time>,
    mut timer: ResMut<AfterimageTimer>,
    mut commands: Commands,
    player_query: Query<(&Transform, &Sprite), With<crate::core::player::components::Player>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    // Always tick the timer so first afterimage spawns immediately on movement
    timer.timer.tick(time.delta());

    // Only spawn afterimages while the player is moving
    let is_moving = keyboard_input.pressed(KeyCode::KeyW)
        || keyboard_input.pressed(KeyCode::KeyA)
        || keyboard_input.pressed(KeyCode::KeyS)
        || keyboard_input.pressed(KeyCode::KeyD);

    if !is_moving || !timer.timer.just_finished() {
        return;
    }

    for (transform, sprite) in &player_query {
        let ghost_color = sprite.color.with_alpha(0.5);

        commands.spawn((
            Sprite {
                color: ghost_color,
                custom_size: sprite.custom_size,
                ..default()
            },
            *transform,
            Afterimage {
                lifetime: 0.15,
                max_lifetime: 0.15,
            },
            GameEntity,
        ));
    }
}

pub fn animate_afterimages(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Afterimage, &mut Sprite)>,
) {
    let dt = time.delta().as_secs_f32();

    for (entity, mut image, mut sprite) in &mut query {
        image.lifetime -= dt;

        if image.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        let alpha = (image.lifetime / image.max_lifetime).clamp(0.0, 1.0) * 0.5;
        sprite.color = sprite.color.with_alpha(alpha);
    }
}

// ---- Player Ambient Particles ----

#[derive(Component)]
pub struct AmbientParticle {
    pub velocity: Vec2,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

#[derive(Resource)]
pub struct AmbientParticleTimer {
    pub timer: Timer,
}

impl Default for AmbientParticleTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(AMBIENT_PARTICLE_INTERVAL, TimerMode::Repeating),
        }
    }
}

pub fn spawn_ambient_particles(
    time: Res<Time>,
    mut timer: ResMut<AmbientParticleTimer>,
    mut commands: Commands,
    player_query: Query<&Transform, With<crate::core::player::components::Player>>,
) {
    timer.timer.tick(time.delta());

    if !timer.timer.just_finished() {
        return;
    }

    let mut rng = rand::thread_rng();

    for transform in &player_query {
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let speed = rng.gen_range(20.0..40.0);
        let offset_x = rng.gen_range(-25.0..25.0);
        let offset_y = rng.gen_range(-25.0..25.0);

        commands.spawn((
            Sprite {
                color: Color::srgba(0.5, 1.5, 0.3, 0.15),
                custom_size: Some(Vec2::splat(rng.gen_range(1.0..2.0))),
                ..default()
            },
            Transform::from_xyz(
                transform.translation.x + offset_x,
                transform.translation.y + offset_y,
                transform.translation.z - 0.1,
            ),
            AmbientParticle {
                velocity: Vec2::new(angle.cos() * speed, angle.sin() * speed),
                lifetime: 0.8,
                max_lifetime: 0.8,
            },
            GameEntity,
        ));
    }
}

pub fn animate_ambient_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut AmbientParticle, &mut Transform, &mut Sprite)>,
) {
    let dt = time.delta().as_secs_f32();

    for (entity, mut particle, mut transform, mut sprite) in &mut query {
        particle.lifetime -= dt;

        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;

        let alpha = (particle.lifetime / particle.max_lifetime).clamp(0.0, 1.0) * 0.15;
        sprite.color = sprite.color.with_alpha(alpha);
    }
}

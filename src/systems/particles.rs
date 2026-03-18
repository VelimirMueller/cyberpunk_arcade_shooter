use bevy::prelude::*;
use rand::Rng;
use crate::systems::collision::DeathEvent;
use crate::app::GameEntity;

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

pub fn setup_shockwave_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
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

        // Spawn shatter particles (12-20)
        let particle_count = rng.gen_range(12..=20);
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

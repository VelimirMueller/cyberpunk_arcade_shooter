use crate::app::{GameData, trigger_damage_flash, trigger_screen_shake};
use crate::core::boss::components::{Boss, BossProjectile, DashTrail, HazardZone, ProjectileOwner};
use crate::core::boss::systems::score_multiplier;
use crate::core::player::components::{Player, PlayerParticle};
use crate::data::game_state::GameState;
use crate::systems::audio::{SoundEffect, SoundEvent};
use crate::systems::combat::EnemyParticle;
use bevy::prelude::*;

#[derive(Event)]
pub struct DeathEvent {
    pub position: Vec3,
    pub color: Color,
    pub entity: Entity,
}

#[allow(clippy::too_many_arguments)]
pub fn detect_collisions(
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut Player, &Transform, &Sprite)>,
    mut boss_query: Query<(Entity, &mut Boss, &Transform, &Sprite), With<Boss>>,
    particle_query: Query<&Transform, With<EnemyParticle>>,
    player_particle_query: Query<(Entity, &Transform), With<PlayerParticle>>,
    boss_projectile_query: Query<(&Transform, &BossProjectile)>,
    dash_trail_query: Query<(&Transform, &Sprite), With<DashTrail>>,
    hazard_zone_query: Query<(&Transform, &HazardZone)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut game_data: ResMut<GameData>,
    mut screen_shake: ResMut<crate::app::ScreenShake>,
    mut sound_events: EventWriter<SoundEvent>,
) {
    for (player_entity, mut player, player_transform, player_sprite) in &mut player_query {
        let player_size = player_sprite.custom_size.unwrap_or(Vec2::ONE);
        let player_pos = player_transform.translation;

        // Player vs Boss body collision
        for (_entity, _boss, enemy_transform, enemy_sprite) in &boss_query {
            if _boss.is_invulnerable {
                continue;
            }
            let enemy_size = enemy_sprite.custom_size.unwrap_or(Vec2::ONE);
            let enemy_pos = enemy_transform.translation;

            if collide(player_pos, player_size, enemy_pos, enemy_size) {
                if player.current > 0
                    && player
                        .last_collision_time
                        .is_none_or(|t| t.elapsed().as_secs_f32() > 0.025)
                {
                    player.current -= 1;
                    player.last_collision_time = Some(crate::utils::time_compat::Instant::now());
                    trigger_screen_shake(&mut screen_shake);
                    trigger_damage_flash(player_entity, commands.reborrow());
                    sound_events.write(SoundEvent(SoundEffect::PlayerHit));
                }

                if player.current == 0 {
                    next_state.set(GameState::GameOver);
                }
            }
        }

        // EnemyParticle vs Player
        for particle_transform in &particle_query {
            let particle_size = Vec2::new(2.0, 2.0);
            let particle_pos = particle_transform.translation;

            if collide(player_pos, player_size, particle_pos, particle_size) {
                if player.current > 0
                    && player
                        .last_collision_time
                        .is_none_or(|t| t.elapsed().as_secs_f32() > 0.075)
                {
                    player.current -= 1;
                    player.last_collision_time = Some(crate::utils::time_compat::Instant::now());
                    trigger_screen_shake(&mut screen_shake);
                    trigger_damage_flash(player_entity, commands.reborrow());
                }

                if player.current == 0 {
                    next_state.set(GameState::GameOver);
                }
            }
        }

        // BossProjectile vs Player — only Boss-owned projectiles damage the player
        for (projectile_transform, projectile) in boss_projectile_query.iter() {
            if projectile.owner != ProjectileOwner::Boss {
                continue;
            }
            let projectile_size = Vec2::new(6.0, 6.0);
            let projectile_pos = projectile_transform.translation;

            if collide(player_pos, player_size, projectile_pos, projectile_size) {
                if player.current > 0
                    && player
                        .last_collision_time
                        .is_none_or(|t| t.elapsed().as_secs_f32() > 0.075)
                {
                    player.current -= 1;
                    player.last_collision_time = Some(crate::utils::time_compat::Instant::now());
                    trigger_screen_shake(&mut screen_shake);
                    trigger_damage_flash(player_entity, commands.reborrow());
                    sound_events.write(SoundEvent(SoundEffect::PlayerHit));
                }

                if player.current == 0 {
                    next_state.set(GameState::GameOver);
                }
            }
        }

        // DashTrail vs Player
        for (trail_transform, trail_sprite) in &dash_trail_query {
            let trail_size = trail_sprite.custom_size.unwrap_or(Vec2::new(20.0, 20.0));
            let trail_pos = trail_transform.translation;

            if collide(player_pos, player_size, trail_pos, trail_size) {
                if player.current > 0
                    && player
                        .last_collision_time
                        .is_none_or(|t| t.elapsed().as_secs_f32() > 0.075)
                {
                    player.current -= 1;
                    player.last_collision_time = Some(crate::utils::time_compat::Instant::now());
                    trigger_screen_shake(&mut screen_shake);
                    trigger_damage_flash(player_entity, commands.reborrow());
                    sound_events.write(SoundEvent(SoundEffect::PlayerHit));
                }

                if player.current == 0 {
                    next_state.set(GameState::GameOver);
                }
            }
        }

        // HazardZone vs Player
        for (hazard_transform, hazard_zone) in &hazard_zone_query {
            let hazard_size = Vec2::new(hazard_zone.radius * 2.0, hazard_zone.radius * 2.0);
            let hazard_pos = hazard_transform.translation;

            if collide(player_pos, player_size, hazard_pos, hazard_size) {
                if player.current > 0
                    && player
                        .last_collision_time
                        .is_none_or(|t| t.elapsed().as_secs_f32() > 0.075)
                {
                    player.current -= 1;
                    player.last_collision_time = Some(crate::utils::time_compat::Instant::now());
                    trigger_screen_shake(&mut screen_shake);
                    trigger_damage_flash(player_entity, commands.reborrow());
                    sound_events.write(SoundEvent(SoundEffect::PlayerHit));
                }

                if player.current == 0 {
                    next_state.set(GameState::GameOver);
                }
            }
        }

        // Player-owned BossProjectile vs Boss (for reflected/hacked projectiles)
        for (_boss_entity, mut boss, boss_transform, boss_sprite) in &mut boss_query {
            let boss_size = boss_sprite.custom_size.unwrap_or(Vec2::ONE);
            let boss_pos = boss_transform.translation;
            for (projectile_transform, projectile) in boss_projectile_query.iter() {
                if projectile.owner != ProjectileOwner::Player {
                    continue;
                }
                let projectile_size = Vec2::new(6.0, 6.0);
                if collide(
                    projectile_transform.translation,
                    projectile_size,
                    boss_pos,
                    boss_size,
                ) {
                    if boss.is_invulnerable {
                        continue;
                    }
                    if boss
                        .last_hit_time
                        .is_some_and(|t| t.elapsed().as_secs_f32() < 0.075)
                    {
                        continue;
                    }
                    if boss.current_hp > 0 {
                        let dmg = projectile.damage.max(1);
                        boss.current_hp = boss.current_hp.saturating_sub(dmg);
                        boss.last_hit_time = Some(crate::utils::time_compat::Instant::now());
                        let mult = score_multiplier(game_data.round);
                        game_data.score += (10.0 * mult) as u32;
                        sound_events.write(SoundEvent(SoundEffect::EnemyHit));
                    }
                }
            }
        }

        // PlayerParticle vs Boss — despawn particle on hit, cooldown per boss
        for (_boss_entity, mut boss, boss_transform, boss_sprite) in &mut boss_query {
            let boss_size = boss_sprite.custom_size.unwrap_or(Vec2::ONE);

            for (particle_entity, player_particle_transform) in &player_particle_query {
                let particle_size = Vec2::new(3.0, 3.0);
                let particle_pos = player_particle_transform.translation;
                let boss_pos = boss_transform.translation;

                if collide(particle_pos, particle_size, boss_pos, boss_size) {
                    if boss.is_invulnerable {
                        continue;
                    }
                    // Cooldown: 75ms between hits
                    if boss
                        .last_hit_time
                        .is_some_and(|t| t.elapsed().as_secs_f32() < 0.075)
                    {
                        continue;
                    }
                    if boss.current_hp > 0 {
                        boss.current_hp -= 1;
                        boss.last_hit_time = Some(crate::utils::time_compat::Instant::now());
                        let mult = score_multiplier(game_data.round);
                        game_data.score += (10.0 * mult) as u32;
                        sound_events.write(SoundEvent(SoundEffect::EnemyHit));

                        // Despawn the particle on hit
                        commands.entity(particle_entity).despawn();

                        // Boss death handled by boss_death_check_system
                    }
                }
            }
        }
    }
}

pub(crate) fn collide(pos_a: Vec3, size_a: Vec2, pos_b: Vec3, size_b: Vec2) -> bool {
    let a_min = pos_a.truncate() - size_a / 2.0;
    let a_max = pos_a.truncate() + size_a / 2.0;
    let b_min = pos_b.truncate() - size_b / 2.0;
    let b_max = pos_b.truncate() + size_b / 2.0;

    a_min.x < b_max.x && a_max.x > b_min.x && a_min.y < b_max.y && a_max.y > b_min.y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collide_overlapping() {
        let pos_a = Vec3::new(0.0, 0.0, 0.0);
        let size_a = Vec2::new(10.0, 10.0);
        let pos_b = Vec3::new(5.0, 5.0, 0.0);
        let size_b = Vec2::new(10.0, 10.0);
        assert!(collide(pos_a, size_a, pos_b, size_b));
    }

    #[test]
    fn test_collide_separated() {
        let pos_a = Vec3::new(0.0, 0.0, 0.0);
        let size_a = Vec2::new(10.0, 10.0);
        let pos_b = Vec3::new(100.0, 100.0, 0.0);
        let size_b = Vec2::new(10.0, 10.0);
        assert!(!collide(pos_a, size_a, pos_b, size_b));
    }

    #[test]
    fn test_collide_touching_edges() {
        let pos_a = Vec3::new(0.0, 0.0, 0.0);
        let size_a = Vec2::new(10.0, 10.0);
        let pos_b = Vec3::new(10.0, 0.0, 0.0);
        let size_b = Vec2::new(10.0, 10.0);
        assert!(!collide(pos_a, size_a, pos_b, size_b));
    }

    #[test]
    fn test_collide_one_contains_other() {
        let pos_a = Vec3::new(0.0, 0.0, 0.0);
        let size_a = Vec2::new(100.0, 100.0);
        let pos_b = Vec3::new(0.0, 0.0, 0.0);
        let size_b = Vec2::new(10.0, 10.0);
        assert!(collide(pos_a, size_a, pos_b, size_b));
    }

    #[test]
    fn projectile_owner_enum_equality() {
        use crate::core::boss::components::ProjectileOwner;
        assert_eq!(ProjectileOwner::Boss, ProjectileOwner::Boss);
        assert_ne!(ProjectileOwner::Boss, ProjectileOwner::Player);
    }
}

use bevy::prelude::*;
use crate::core::player::components::{Player, PlayerParticle};
use crate::core::enemies::components::Enemy;
use crate::systems::combat::EnemyParticle;
use crate::data::game_state::GameState;
use crate::app::{GameData, trigger_screen_shake, trigger_damage_flash};
use crate::systems::audio::{play_sound, SoundEffect};

pub fn detect_collisions(
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut Player, &Transform, &Sprite)>,
    mut enemy_query: Query<(&mut Enemy, &Transform, &Sprite), With<Enemy>>,
    particle_query: Query<&Transform, With<EnemyParticle>>,
    player_particle_query: Query<&Transform, With<PlayerParticle>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut game_data: ResMut<GameData>,
    mut screen_shake: ResMut<crate::app::ScreenShake>,
    audio: Res<crate::systems::audio::AudioManager>,
) {
    for (player_entity, mut player, player_transform, player_sprite) in &mut player_query {
        let player_size = player_sprite.custom_size.unwrap_or(Vec2::ONE);
        let player_pos = player_transform.translation;

        for (_enemy, enemy_transform, enemy_sprite) in &enemy_query {
            let enemy_size = enemy_sprite.custom_size.unwrap_or(Vec2::ONE);
            let enemy_pos = enemy_transform.translation;

            if collide(player_pos, player_size, enemy_pos, enemy_size) {
                // Handle collision with enemies
                if player.current > 0 {
                    if player.last_collision_time.map_or(true, |t| t.elapsed().as_secs_f32() > 0.025) {
                        player.current -= 1;
                        player.last_collision_time = Some(std::time::Instant::now());
                        trigger_screen_shake(&mut screen_shake);
                        trigger_damage_flash(player_entity, commands.reborrow());
                        play_sound(&audio, SoundEffect::PlayerHit);
                        info!("Collision! with Enemy HP {}", player.current);
                    }
                }

                if player.current == 0 {
                    next_state.set(GameState::GameOver);
                }
            }
        }

        for particle_transform in &particle_query {
            let particle_size = Vec2::new(2.0, 2.0); // Assuming a fixed size for particles
            let particle_pos = particle_transform.translation;

            if collide(player_pos, player_size, particle_pos, particle_size) {
                // Handle collision with enemy particles
                info!("Collision with enemy particle! {}", player.current);
                if player.current > 0 {
                    if player.last_collision_time.map_or(true, |t| t.elapsed().as_secs_f32() > 0.075) {
                        player.current -= 1;
                        player.last_collision_time = Some(std::time::Instant::now());
                        trigger_screen_shake(&mut screen_shake);
                        trigger_damage_flash(player_entity, commands.reborrow());
                        info!("Collision! with Enemy HP {}", player.current);
                    }
                }

                if player.current == 0 {
                    next_state.set(GameState::GameOver);
                }
            }
        }
        for (mut enemy, enemy_transform, enemy_sprite) in &mut enemy_query {
            let enemy_size = enemy_sprite.custom_size.unwrap_or(Vec2::ONE);

            for player_particle_transform in &player_particle_query {
                let particle_size = Vec2::new(2.0, 2.0); // Assuming a fixed size for particles
                let particle_pos = player_particle_transform.translation;
                let enemy_pos = enemy_transform.translation;

                if collide(particle_pos, particle_size, enemy_pos, enemy_size) {
                    if enemy.current > 0 {
                        if enemy.last_collision_time.map_or(true, |t| t.elapsed().as_secs_f32() > 0.075) {
                            enemy.current -= 1;
                            enemy.last_collision_time = Some(std::time::Instant::now());
                            game_data.score += 10; // Add score for hitting enemy
                            play_sound(&audio, SoundEffect::EnemyHit);
                            info!("You hit the Enemy HP: {}", enemy.current);
                        }
                    } else {
                        info!("Enemy defeated!");
                        game_data.score += 100; // Bonus for defeating enemy
                        play_sound(&audio, SoundEffect::Explosion);
                    }
                }
            }
        }
    }

}

fn collide(pos_a: Vec3, size_a: Vec2, pos_b: Vec3, size_b: Vec2) -> bool {
    let a_min = pos_a.truncate() - size_a / 2.0;
    let a_max = pos_a.truncate() + size_a / 2.0;
    let b_min = pos_b.truncate() - size_b / 2.0;
    let b_max = pos_b.truncate() + size_b / 2.0;

    a_min.x < b_max.x && a_max.x > b_min.x &&
        a_min.y < b_max.y && a_max.y > b_min.y
}
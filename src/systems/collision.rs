use bevy::prelude::*;
use crate::core::player::components::Player;
use crate::core::enemies::components::Enemy;

pub fn detect_collisions(
    mut player_query: Query<(&mut Player, &Transform, &Sprite)>,
    enemy_query: Query<(&Transform, &Sprite), With<Enemy>>,
) {
    for (mut player, player_transform, player_sprite) in &mut player_query {
        let player_size = player_sprite.custom_size.unwrap_or(Vec2::ONE);
        let player_pos = player_transform.translation;

        for (enemy_transform, enemy_sprite) in &enemy_query {
            let enemy_size = enemy_sprite.custom_size.unwrap_or(Vec2::ONE);
            let enemy_pos = enemy_transform.translation;

            if collide(player_pos, player_size, enemy_pos, enemy_size) {
                // ✅ Now this works because player is mutable
                if player.max > 0 {
                    player.max -= 1;
                    info!("Collision! HP {}", player.max);
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
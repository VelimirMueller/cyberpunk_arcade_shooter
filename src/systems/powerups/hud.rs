use crate::core::player::components::Player;
use crate::systems::powerups::effects::phase_shift::PhaseShiftActive;
use bevy::prelude::*;

/// Marker on the UI node where buff dots are children.
#[derive(Component)]
pub struct BuffHudRoot;

/// Marker on each dot so we can despawn them on refresh.
#[derive(Component)]
pub struct BuffHudDot;

/// Sync the buff row: each frame, clear existing dots and respawn per active buff.
/// Kept simple because there are ≤ 6 possible dots and sync happens at 60fps — cheap.
pub fn sync_buff_hud_system(
    mut commands: Commands,
    root_query: Query<Entity, With<BuffHudRoot>>,
    existing_dots: Query<Entity, With<BuffHudDot>>,
    player_query: Query<&PhaseShiftActive, With<Player>>,
) {
    let Ok(root) = root_query.single() else { return };

    // Despawn previous-frame dots
    for entity in existing_dots.iter() {
        commands.entity(entity).despawn();
    }

    // Enumerate currently active buffs and spawn a dot per buff
    for phase_shift in player_query.iter() {
        let remaining = (1.0 - phase_shift.0.fraction()).clamp(0.0, 1.0);
        let meta = crate::systems::powerups::catalog::meta(
            crate::systems::powerups::catalog::PowerUpKind::PhaseShift,
        );
        spawn_buff_dot(&mut commands, root, meta.color, remaining);
    }
}

fn spawn_buff_dot(commands: &mut Commands, root: Entity, color: Color, remaining: f32) {
    commands.entity(root).with_children(|parent| {
        parent
            .spawn((
                Node {
                    width: Val::Px(14.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BuffHudDot,
            ))
            .with_children(|col| {
                // The dot
                col.spawn((
                    Node {
                        width: Val::Px(12.0),
                        height: Val::Px(12.0),
                        ..default()
                    },
                    BackgroundColor(color),
                ));
                // Duration bar
                col.spawn((
                    Node {
                        width: Val::Px(14.0),
                        height: Val::Px(2.0),
                        margin: UiRect::top(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.04, 0.04, 0.04)),
                ))
                .with_children(|bar| {
                    bar.spawn((
                        Node {
                            width: Val::Percent(remaining * 100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(color),
                    ));
                });
            });
    });
}

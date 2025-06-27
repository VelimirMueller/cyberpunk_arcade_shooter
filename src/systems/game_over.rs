use bevy::prelude::*;
use crate::core::enemies::systems::create_enemies;
use crate::core::player::systems::spawn_player;
use crate::app::GameEntity;
use crate::data::game_state::GameState;

#[derive(Component)]
pub struct AnimatedGameOverText;

pub(crate) fn game_over_system(mut commands: Commands) {
    commands.spawn((
        // Accepts a `String` or any type that converts into a `String`, such as `&str`
        Text::new("GAME OVER"),
        TextFont {
            // This font is loaded and will be used instead of the default font.
            font_size: 97.0,
            ..default()
        },
        TextShadow::default(),
        // Set the justification of the Text
        TextLayout::new_with_justify(JustifyText::Center),
        // Set the style of the Node itself.
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(35.0),
            top: Val::Percent(45.0),
            ..default()
        },
        AnimatedGameOverText,
        GameEntity
    ));
}

pub(crate) fn game_won_system(mut commands: Commands) {
    commands.spawn((
        // Accepts a `String` or any type that converts into a `String`, such as `&str`
        Text::new("GAME WON"),
        TextFont {
            // This font is loaded and will be used instead of the default font.
            font_size: 97.0,
            ..default()
        },
        TextShadow::default(),
        // Set the justification of the Text
        TextLayout::new_with_justify(JustifyText::Center),
        // Set the style of the Node itself.
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(35.0),
            top: Val::Percent(45.0),
            ..default()
        },
        AnimatedGameOverText,
        GameEntity
    ));
}

pub fn restart_listener(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    query: Query<Entity, With<GameEntity>>,
    mut commands: Commands
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for entity in &query {
            commands.entity(entity).despawn();
        }

        create_enemies(commands.reborrow());
        spawn_player(commands);

        next_state.set(GameState::Playing);
    }
}

pub fn despawn_game_over_text(
    mut commands: Commands,
    query: Query<Entity, With<AnimatedGameOverText>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

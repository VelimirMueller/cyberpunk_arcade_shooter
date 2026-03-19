use bevy::prelude::*;
use crate::app::GameData;

// ── Marker Components ────────────────────────────────────────────

#[derive(Component)]
pub struct MenuEntity;

#[derive(Component)]
pub struct PauseEntity;

#[derive(Component)]
pub struct GameOverEntity;

#[derive(Component)]
pub struct GameWonEntity;

// ── Colors ───────────────────────────────────────────────────────

const CYAN: Color = Color::srgb(0.0, 1.0, 1.0);
const RED: Color = Color::srgb(1.0, 0.0, 0.24);
const MAGENTA: Color = Color::srgb(1.0, 0.0, 1.0);
const GRAY: Color = Color::srgb(0.33, 0.33, 0.33);
const OVERLAY_BG: Color = Color::srgba(0.0, 0.0, 0.0, 0.85);
const CONTAINER_BORDER: Color = Color::srgb(0.15, 0.15, 0.15);
const PAUSE_OVERLAY: Color = Color::srgba(0.0, 0.0, 0.0, 0.7);

// ── Helpers ──────────────────────────────────────────────────────

/// Spawn glow text: a blurred shadow layer behind the main text.
fn spawn_glow_text(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    font_size: f32,
    color: Color,
    glow_alpha: f32,
) {
    let glow_color = match color {
        Color::Srgba(c) => Color::srgba(c.red, c.green, c.blue, glow_alpha),
        _ => Color::srgba(0.0, 1.0, 1.0, glow_alpha),
    };

    // Glow layer (behind)
    parent.spawn((
        Text::new(text),
        TextFont { font_size: font_size + 2.0, ..default() },
        TextColor(glow_color),
        Node { position_type: PositionType::Absolute, ..default() },
    ));
    // Main text (on top)
    parent.spawn((
        Text::new(text),
        TextFont { font_size, ..default() },
        TextColor(color),
    ));
}

fn centered_column() -> Node {
    Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        row_gap: Val::Px(16.0),
        ..default()
    }
}

fn container_node() -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        padding: UiRect::all(Val::Px(24.0)),
        row_gap: Val::Px(12.0),
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    }
}

// ═════════════════════════════════════════════════════════════════
// 1. Title Screen  (GameState::Menu)
// ═════════════════════════════════════════════════════════════════

pub fn spawn_title_menu(mut commands: Commands, game_data: Res<GameData>) {
    commands
        .spawn((
            centered_column(),
            BackgroundColor(OVERLAY_BG),
            MenuEntity,
        ))
        .with_children(|parent| {
            // Title with glow
            spawn_glow_text(parent, "CYBERPUNK BLOOM CUBE", 40.0, CYAN, 0.3);

            // Container
            parent
                .spawn((
                    container_node(),
                    BorderColor(CONTAINER_BORDER),
                ))
                .with_children(|container| {
                    // High score
                    container.spawn((
                        Text::new(format!("High Score: {}", game_data.high_score)),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(GRAY),
                    ));

                    // Start instruction
                    container.spawn((
                        Text::new("PRESS ENTER TO START"),
                        TextFont { font_size: 16.0, ..default() },
                        TextColor(MAGENTA),
                    ));
                });
        });
}

pub fn despawn_title_menu(mut commands: Commands, query: Query<Entity, With<MenuEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

// ═════════════════════════════════════════════════════════════════
// 2. Pause Screen  (GameState::Paused)
// ═════════════════════════════════════════════════════════════════

pub fn spawn_pause_menu(
    mut commands: Commands,
    audio: NonSendMut<crate::systems::audio::SynthAudio>,
) {
    let sound_status = if audio.sound_enabled { "ON" } else { "OFF" };

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(PAUSE_OVERLAY),
            PauseEntity,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("PAUSED"),
                TextFont { font_size: 36.0, ..default() },
                TextColor(CYAN),
            ));

            // Container with options
            parent
                .spawn((
                    container_node(),
                    BorderColor(CONTAINER_BORDER),
                ))
                .with_children(|container| {
                    container.spawn((
                        Text::new("Press ESC to Resume"),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(GRAY),
                    ));
                    container.spawn((
                        Text::new("Press Q to Return to Menu"),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(GRAY),
                    ));
                    container.spawn((
                        Text::new(format!("Press M to Toggle Sound ({})", sound_status)),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(GRAY),
                    ));
                });
        });
}

pub fn despawn_pause_menu(mut commands: Commands, query: Query<Entity, With<PauseEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

// ═════════════════════════════════════════════════════════════════
// 3. Game Over Screen  (GameState::GameOver)
// ═════════════════════════════════════════════════════════════════

pub fn spawn_game_over_screen(mut commands: Commands, game_data: Res<GameData>) {
    commands
        .spawn((
            centered_column(),
            BackgroundColor(OVERLAY_BG),
            GameOverEntity,
        ))
        .with_children(|parent| {
            // Title with glow
            spawn_glow_text(parent, "GAME OVER", 60.0, RED, 0.3);

            // Score info
            parent.spawn((
                Text::new(format!("Final Score: {}", game_data.score)),
                TextFont { font_size: 20.0, ..default() },
                TextColor(CYAN),
            ));

            parent.spawn((
                Text::new(format!("Reached Round: {}", game_data.round)),
                TextFont { font_size: 16.0, ..default() },
                TextColor(GRAY),
            ));

            // Restart instruction
            parent.spawn((
                Text::new("PRESS SPACE TO RESTART"),
                TextFont { font_size: 14.0, ..default() },
                TextColor(MAGENTA),
            ));
        });
}

pub fn despawn_game_over_screen(
    mut commands: Commands,
    query: Query<Entity, With<GameOverEntity>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

// ═════════════════════════════════════════════════════════════════
// 4. Victory Screen  (GameState::Won)
// ═════════════════════════════════════════════════════════════════

pub fn spawn_game_won_screen(mut commands: Commands, game_data: Res<GameData>) {
    commands
        .spawn((
            centered_column(),
            BackgroundColor(OVERLAY_BG),
            GameWonEntity,
        ))
        .with_children(|parent| {
            // Title with glow
            spawn_glow_text(parent, "VICTORY", 60.0, CYAN, 0.3);

            // Score info
            parent.spawn((
                Text::new(format!("Final Score: {}", game_data.score)),
                TextFont { font_size: 20.0, ..default() },
                TextColor(CYAN),
            ));

            // Return instruction
            parent.spawn((
                Text::new("PRESS SPACE TO RETURN TO MENU"),
                TextFont { font_size: 14.0, ..default() },
                TextColor(MAGENTA),
            ));
        });
}

pub fn despawn_game_won_screen(
    mut commands: Commands,
    query: Query<Entity, With<GameWonEntity>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

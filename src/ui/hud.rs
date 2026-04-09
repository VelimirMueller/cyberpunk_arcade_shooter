use crate::app::{GameData, GameEntity};
use crate::core::boss::components::{Boss, BossPhase, BossType};
use crate::core::boss::systems::boss_type_for_round;
use crate::core::player::components::Player;
use bevy::prelude::*;

// ── Component Markers ──────────────────────────────────────────────

#[derive(Component)]
pub struct HudRoot;

#[derive(Component)]
pub struct BossNameText;

#[derive(Component)]
pub struct BossHpFill;

#[derive(Component)]
pub struct BossPhasePip(pub u8);

#[derive(Component)]
pub struct PlayerHpFill;

#[derive(Component)]
pub struct PlayerHpText;

#[derive(Component)]
pub struct PlayerEnergyFill;

#[derive(Component)]
pub struct PlayerEnergyText;

#[derive(Component)]
pub struct ScoreValueText;

#[derive(Component)]
pub struct RoundPip(pub u32);

#[derive(Component)]
pub struct RoundLabelText;

// ── Colors ─────────────────────────────────────────────────────────

const COLOR_PLAYER_HP: Color = Color::srgb(0.0, 1.0, 0.53);
const COLOR_PLAYER_ENERGY: Color = Color::srgb(0.67, 0.27, 1.0);
const COLOR_BOSS_HP: Color = Color::srgb(1.0, 0.0, 0.24);
const COLOR_SCORE_CYAN: Color = Color::srgb(0.0, 1.0, 0.8);
const COLOR_LABEL: Color = Color::srgb(0.33, 0.33, 0.33);
const COLOR_BAR_BG: Color = Color::srgb(0.04, 0.04, 0.04);
const COLOR_DIM_PIP: Color = Color::srgb(0.12, 0.12, 0.12);

// ── Boss Name Helper ───────────────────────────────────────────────

fn boss_name(boss_type: BossType) -> &'static str {
    match boss_type {
        BossType::GridPhantom => "GRID PHANTOM",
        BossType::NeonSentinel => "NEON SENTINEL",
        BossType::ChromeBerserker => "CHROME BERSERKER",
        BossType::VoidWeaver => "VOID WEAVER",
        BossType::ApexProtocol => "APEX PROTOCOL",
    }
}

// ── spawn_hud ──────────────────────────────────────────────────────

pub fn spawn_hud(mut commands: Commands, existing: Query<&HudRoot>, game_data: Res<GameData>) {
    // Guard: don't spawn if already exists
    if !existing.is_empty() {
        return;
    }

    let boss_type = boss_type_for_round(game_data.round);

    commands
        .spawn((
            HudRoot,
            GameEntity,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            GlobalZIndex(5),
            // Fully transparent so it doesn't block visuals
            BackgroundColor(Color::NONE),
        ))
        .with_children(|root| {
            // ─── TOP CENTER: Boss section ───────────────────────────
            root.spawn(Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                left: Val::Percent(25.0),
                width: Val::Percent(50.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|top| {
                // Boss name with glow
                top.spawn(Node { ..default() })
                    .with_children(|name_container| {
                        // Glow layer (behind)
                        name_container.spawn((
                            Text::new(boss_name(boss_type)),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::srgba(1.0, 0.0, 0.24, 0.3)),
                            TextLayout::new_with_justify(JustifyText::Center),
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(-1.0),
                                top: Val::Px(-1.0),
                                ..default()
                            },
                        ));
                        // Main boss name text
                        name_container.spawn((
                            Text::new(boss_name(boss_type)),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(COLOR_BOSS_HP),
                            TextLayout::new_with_justify(JustifyText::Center),
                            BossNameText,
                        ));
                    });

                // Boss HP bar container
                top.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(10.0),
                        margin: UiRect::top(Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(COLOR_BAR_BG),
                ))
                .with_children(|bar| {
                    bar.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(COLOR_BOSS_HP),
                        BossHpFill,
                    ));
                });

                // Phase pips row
                top.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    margin: UiRect::top(Val::Px(4.0)),
                    column_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(|pip_row| {
                    for i in 0..3u8 {
                        pip_row.spawn((
                            Node {
                                width: Val::Px(8.0),
                                height: Val::Px(8.0),
                                ..default()
                            },
                            BackgroundColor(COLOR_BOSS_HP),
                            BossPhasePip(i),
                        ));
                    }
                });
            });

            // ─── BOTTOM LEFT: Player stats ──────────────────────────
            root.spawn(Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(16.0),
                left: Val::Px(16.0),
                flex_direction: FlexDirection::Column,
                ..default()
            })
            .with_children(|bl| {
                // "OPERATOR" label
                bl.spawn((
                    Text::new("OPERATOR"),
                    TextFont {
                        font_size: 9.0,
                        ..default()
                    },
                    TextColor(COLOR_SCORE_CYAN),
                    TextLayout::new_with_justify(JustifyText::Left),
                ));

                // HP bar row
                bl.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(4.0)),
                    column_gap: Val::Px(6.0),
                    ..default()
                })
                .with_children(|hp_row| {
                    // HP bar container
                    hp_row
                        .spawn((
                            Node {
                                width: Val::Px(140.0),
                                height: Val::Px(8.0),
                                ..default()
                            },
                            BackgroundColor(COLOR_BAR_BG),
                        ))
                        .with_children(|bar| {
                            bar.spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                BackgroundColor(COLOR_PLAYER_HP),
                                PlayerHpFill,
                            ));
                        });

                    // HP numeric value
                    hp_row.spawn((
                        Text::new("100"),
                        TextFont {
                            font_size: 10.0,
                            ..default()
                        },
                        TextColor(COLOR_PLAYER_HP),
                        PlayerHpText,
                    ));
                });

                // Energy bar row
                bl.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(3.0)),
                    column_gap: Val::Px(6.0),
                    ..default()
                })
                .with_children(|en_row| {
                    // Energy bar container
                    en_row
                        .spawn((
                            Node {
                                width: Val::Px(140.0),
                                height: Val::Px(5.0),
                                ..default()
                            },
                            BackgroundColor(COLOR_BAR_BG),
                        ))
                        .with_children(|bar| {
                            bar.spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                BackgroundColor(COLOR_PLAYER_ENERGY),
                                PlayerEnergyFill,
                            ));
                        });

                    // Energy numeric value
                    en_row.spawn((
                        Text::new("100"),
                        TextFont {
                            font_size: 10.0,
                            ..default()
                        },
                        TextColor(COLOR_PLAYER_ENERGY),
                        PlayerEnergyText,
                    ));
                });
            });

            // ─── BOTTOM CENTER: Round pips ──────────────────────────
            root.spawn(Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(16.0),
                left: Val::Percent(50.0),
                // Shift left by half to center
                margin: UiRect::left(Val::Px(-60.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|bc| {
                // Pip row
                bc.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(6.0),
                    ..default()
                })
                .with_children(|pip_row| {
                    for i in 1..=5u32 {
                        let color = if i >= game_data.round {
                            COLOR_SCORE_CYAN
                        } else {
                            COLOR_DIM_PIP
                        };
                        pip_row.spawn((
                            Node {
                                width: Val::Px(8.0),
                                height: Val::Px(8.0),
                                ..default()
                            },
                            BackgroundColor(color),
                            RoundPip(i),
                        ));
                    }
                });

                // Round label with glow
                bc.spawn(Node {
                    margin: UiRect::top(Val::Px(4.0)),
                    ..default()
                })
                .with_children(|label_container| {
                    // Glow
                    label_container.spawn((
                        Text::new(format!("ROUND {} / 5", game_data.round)),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgba(0.0, 1.0, 0.8, 0.3)),
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(-1.0),
                            top: Val::Px(-1.0),
                            ..default()
                        },
                    ));
                    // Main
                    label_container.spawn((
                        Text::new(format!("ROUND {} / 5", game_data.round)),
                        TextFont {
                            font_size: 10.0,
                            ..default()
                        },
                        TextColor(COLOR_SCORE_CYAN),
                        RoundLabelText,
                    ));
                });
            });

            // ─── BOTTOM RIGHT: Score ────────────────────────────────
            root.spawn(Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(16.0),
                right: Val::Px(16.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
                ..default()
            })
            .with_children(|br| {
                // "SCORE" label
                br.spawn((
                    Text::new("SCORE"),
                    TextFont {
                        font_size: 9.0,
                        ..default()
                    },
                    TextColor(COLOR_LABEL),
                ));

                // Score value with glow
                br.spawn(Node {
                    margin: UiRect::top(Val::Px(2.0)),
                    ..default()
                })
                .with_children(|score_container| {
                    // Glow
                    score_container.spawn((
                        Text::new(format!("{}", game_data.score)),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgba(0.0, 1.0, 0.8, 0.3)),
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(-1.0),
                            top: Val::Px(-1.0),
                            ..default()
                        },
                    ));
                    // Main
                    score_container.spawn((
                        Text::new(format!("{}", game_data.score)),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(COLOR_SCORE_CYAN),
                        ScoreValueText,
                    ));
                });
            });
        });
}

// ── Update Systems ─────────────────────────────────────────────────

pub fn update_boss_hud(
    boss_query: Query<&Boss>,
    mut hp_fill_query: Query<&mut Node, With<BossHpFill>>,
    mut name_query: Query<&mut Text, With<BossNameText>>,
    mut pip_query: Query<(&BossPhasePip, &mut BackgroundColor)>,
) {
    let Some(boss) = boss_query.iter().next() else {
        return;
    };

    // Update HP fill width
    let hp_pct = (boss.current_hp as f32 / boss.max_hp.max(1) as f32) * 100.0;
    for mut node in &mut hp_fill_query {
        node.width = Val::Percent(hp_pct);
    }

    // Update boss name
    let name = boss_name(boss.boss_type);
    for mut text in &mut name_query {
        text.0 = name.to_string();
    }

    // Update phase pips
    // Phase1 -> threshold at pip 0, Phase2 -> pip 1, Phase3 -> pip 2, Phase4 -> pip 3
    let cleared_phases = match boss.phase {
        BossPhase::Phase1 => 0,
        BossPhase::Phase2 => 1,
        BossPhase::Phase3 => 2,
        BossPhase::Phase4 => 3,
    };

    for (pip, mut bg) in &mut pip_query {
        if pip.0 < cleared_phases {
            // Cleared phase = dim
            bg.0 = COLOR_DIM_PIP;
        } else {
            // Remaining phase = red
            bg.0 = COLOR_BOSS_HP;
        }
    }
}

pub fn update_player_hud(
    player_query: Query<&Player>,
    mut hp_fill_query: Query<&mut Node, (With<PlayerHpFill>, Without<PlayerEnergyFill>)>,
    mut hp_text_query: Query<&mut Text, (With<PlayerHpText>, Without<PlayerEnergyText>)>,
    mut energy_fill_query: Query<&mut Node, (With<PlayerEnergyFill>, Without<PlayerHpFill>)>,
    mut energy_text_query: Query<&mut Text, (With<PlayerEnergyText>, Without<PlayerHpText>)>,
) {
    let Some(player) = player_query.iter().next() else {
        return;
    };

    let hp_pct = (player.current as f32 / player.max.max(1) as f32) * 100.0;
    for mut node in &mut hp_fill_query {
        node.width = Val::Percent(hp_pct);
    }
    for mut text in &mut hp_text_query {
        text.0 = format!("{}", player.current);
    }

    let energy_pct = player.energy as f32; // energy is 0-100
    for mut node in &mut energy_fill_query {
        node.width = Val::Percent(energy_pct);
    }
    for mut text in &mut energy_text_query {
        text.0 = format!("{}", player.energy);
    }
}

pub fn update_score_hud(
    game_data: Res<GameData>,
    mut score_query: Query<&mut Text, With<ScoreValueText>>,
    mut round_label_query: Query<&mut Text, (With<RoundLabelText>, Without<ScoreValueText>)>,
    mut pip_query: Query<(&RoundPip, &mut BackgroundColor)>,
) {
    for mut text in &mut score_query {
        text.0 = format!("{}", game_data.score);
    }

    for mut text in &mut round_label_query {
        text.0 = format!("ROUND {} / 5", game_data.round);
    }

    for (pip, mut bg) in &mut pip_query {
        if pip.0 < game_data.round {
            bg.0 = COLOR_DIM_PIP; // completed
        } else {
            bg.0 = COLOR_SCORE_CYAN; // remaining (including current)
        }
    }
}

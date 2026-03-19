use bevy::prelude::*;
use crate::app::GameData;
use crate::core::boss::components::BossType;
use crate::core::boss::systems::boss_type_for_round;
use crate::systems::round::RoundTimer;

#[derive(Component)]
pub struct AnnouncementEntity;

#[derive(Component)]
pub struct AnnouncementText {
    pub visible_after: f32,
}

fn boss_name(boss_type: BossType) -> &'static str {
    match boss_type {
        BossType::GridPhantom => "GRID PHANTOM",
        BossType::NeonSentinel => "NEON SENTINEL",
        BossType::ChromeBerserker => "CHROME BERSERKER",
        BossType::VoidWeaver => "VOID WEAVER",
        BossType::ApexProtocol => "APEX PROTOCOL",
    }
}

fn boss_flavor(boss_type: BossType) -> &'static str {
    match boss_type {
        BossType::GridPhantom => "A glitch in the matrix... it phases through walls.",
        BossType::NeonSentinel => "Illuminated guardian of the neon wastelands.",
        BossType::ChromeBerserker => "All chrome, no mercy. Brace for impact.",
        BossType::VoidWeaver => "Reality bends where it treads.",
        BossType::ApexProtocol => "The final firewall. No second chances.",
    }
}

pub fn spawn_announcement_ui(
    mut commands: Commands,
    game_data: Res<GameData>,
) {
    let boss_type = boss_type_for_round(game_data.round);

    // Full-screen dark overlay
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        GlobalZIndex(10),
        AnnouncementEntity,
    )).with_children(|parent| {
        // "// INCOMING THREAT //" - magenta, 14px, visible at 0.0s
        parent.spawn((
            Text::new("// INCOMING THREAT //"),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::srgba(8.0, 0.0, 8.0, 0.0)),
            TextLayout::new_with_justify(JustifyText::Center),
            AnnouncementText { visible_after: 0.0 },
            AnnouncementEntity,
        ));

        // "ROUND N" - cyan, 48px, visible at 0.4s
        parent.spawn((
            Text::new(format!("ROUND {}", game_data.round)),
            TextFont { font_size: 48.0, ..default() },
            TextColor(Color::srgba(0.0, 8.0, 8.0, 0.0)),
            TextLayout::new_with_justify(JustifyText::Center),
            AnnouncementText { visible_after: 0.4 },
            AnnouncementEntity,
        ));

        // Boss name - magenta, 24px, visible at 0.8s
        parent.spawn((
            Text::new(boss_name(boss_type)),
            TextFont { font_size: 24.0, ..default() },
            TextColor(Color::srgba(8.0, 0.0, 8.0, 0.0)),
            TextLayout::new_with_justify(JustifyText::Center),
            AnnouncementText { visible_after: 0.8 },
            AnnouncementEntity,
        ));

        // Flavor text - gray, 11px, visible at 1.2s
        parent.spawn((
            Text::new(boss_flavor(boss_type)),
            TextFont { font_size: 11.0, ..default() },
            TextColor(Color::srgba(0.5, 0.5, 0.5, 0.0)),
            TextLayout::new_with_justify(JustifyText::Center),
            AnnouncementText { visible_after: 1.2 },
            AnnouncementEntity,
        ));
    });
}

pub fn update_announcement_ui(
    round_timer: Option<Res<RoundTimer>>,
    mut query: Query<(&AnnouncementText, &mut TextColor)>,
) {
    let Some(round_timer) = round_timer else {
        return;
    };

    let elapsed = round_timer.elapsed;

    for (ann_text, mut text_color) in query.iter_mut() {
        let alpha = if elapsed < ann_text.visible_after {
            // Not visible yet
            0.0
        } else if elapsed < ann_text.visible_after + 0.15 {
            // Fade in over 0.15s
            (elapsed - ann_text.visible_after) / 0.15
        } else if elapsed < 2.2 {
            // Fully visible
            1.0
        } else {
            // Fade out from 2.2 to 2.5
            let fade_progress = (elapsed - 2.2) / 0.3;
            (1.0 - fade_progress).max(0.0)
        };

        // Preserve the RGB, just update alpha
        let c = text_color.0.to_srgba();
        text_color.0 = Color::srgba(c.red, c.green, c.blue, alpha);
    }
}

pub fn despawn_announcement_ui(
    mut commands: Commands,
    query: Query<Entity, With<AnnouncementEntity>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

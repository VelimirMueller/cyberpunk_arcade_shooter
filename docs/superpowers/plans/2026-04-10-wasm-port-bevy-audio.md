# WASM Port + bevy_audio Migration

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace kira with bevy_audio and configure the project for WASM compilation so the game can be embedded as an iframe in a Next.js app.

**Architecture:** Remove kira/hound dependencies, pre-generate all 22 sound effects at startup as Bevy `AudioSource` assets stored in a `SoundLibrary` resource, play via one-shot `AudioPlayer` entity spawning. Add a `webgpu` cargo feature for WASM builds. Create a trunk-based HTML entry point. The `SoundEvent`/`SoundEffect` event API used by all game systems stays identical — only the audio backend changes.

**Tech Stack:** Bevy 0.16.1 (bevy_audio), trunk (WASM build), WebGPU (for HDR/Bloom/CRT shader)

---

## File Map

| Action | File | Responsibility |
|--------|------|----------------|
| Modify | `Cargo.toml` | Remove kira+hound deps, add webgpu feature |
| Modify | `src/systems/audio.rs` | Full rewrite: SoundLibrary resource, bevy_audio playback |
| Modify | `src/app.rs` | Wire new audio setup, update pause_menu_system, add WASM window config |
| Modify | `src/ui/menus.rs` | Replace NonSendMut with Res in spawn_pause_menu |
| Create | `index.html` | Trunk WASM entry point |

---

### Task 1: Update Cargo.toml

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Remove kira and hound, add webgpu feature**

Replace the entire `Cargo.toml` contents with:

```toml
[package]
name = "cyberpunk_rpg"
version = "0.1.0"
edition = "2024"

[features]
default = []
webgpu = ["bevy/webgpu"]

[dependencies]
bevy = "0.16.1"
rand = "0.8"
```

Key changes:
- `kira` removed (was native-only audio manager)
- `hound` removed (was used to encode WAV for kira — bevy_audio generates samples differently)
- `webgpu` feature added (forwards to `bevy/webgpu` for HDR rendering on WASM)

- [ ] **Step 2: Verify it compiles (will fail — audio.rs still references kira)**

Run: `cargo check 2>&1 | head -5`
Expected: errors about `kira` and `hound` imports in `src/systems/audio.rs` — this confirms the dependency removal worked. We fix this in Task 2.

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "chore: remove kira and hound deps, add webgpu feature"
```

---

### Task 2: Rewrite src/systems/audio.rs

**Files:**
- Modify: `src/systems/audio.rs`

This is the core change. The file goes from 401 lines (kira backend) to ~250 lines (bevy_audio backend). The `SoundEffect` enum and `SoundEvent` type stay identical. All downstream systems (`EventWriter<SoundEvent>`) are unaffected.

- [ ] **Step 1: Replace the entire file contents**

Write `src/systems/audio.rs` with this content:

```rust
use bevy::prelude::*;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Sound effect enum (unchanged — all callsites use this)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum SoundEffect {
    PlayerShoot,
    PlayerHit,
    EnemyShoot,
    EnemyHit,
    Explosion,
    GameOver,
    GameWon,
    MenuSelect,
    BossSpawn,
    PhaseShift,
    RageBurst,
    DashTelegraph,
    BeamSweep,
    ChargeWindUp,
    HazardSpawn,
    HazardExplode,
    RoundClear,
    ShockwavePowerUp,
    LaserHum,
    LaserCharge,
    LaserFire,
    LaserFadeOut,
}

const ALL_EFFECTS: &[SoundEffect] = &[
    SoundEffect::PlayerShoot,
    SoundEffect::PlayerHit,
    SoundEffect::EnemyShoot,
    SoundEffect::EnemyHit,
    SoundEffect::Explosion,
    SoundEffect::GameOver,
    SoundEffect::GameWon,
    SoundEffect::MenuSelect,
    SoundEffect::BossSpawn,
    SoundEffect::PhaseShift,
    SoundEffect::RageBurst,
    SoundEffect::DashTelegraph,
    SoundEffect::BeamSweep,
    SoundEffect::ChargeWindUp,
    SoundEffect::HazardSpawn,
    SoundEffect::HazardExplode,
    SoundEffect::RoundClear,
    SoundEffect::ShockwavePowerUp,
    SoundEffect::LaserHum,
    SoundEffect::LaserCharge,
    SoundEffect::LaserFire,
    SoundEffect::LaserFadeOut,
];

// ---------------------------------------------------------------------------
// Event (unchanged)
// ---------------------------------------------------------------------------

#[derive(Event)]
pub struct SoundEvent(pub SoundEffect);

// ---------------------------------------------------------------------------
// SoundLibrary resource (replaces kira's SynthAudio)
// ---------------------------------------------------------------------------

#[derive(Resource)]
pub struct SoundLibrary {
    sounds: std::collections::HashMap<SoundEffect, Handle<AudioSource>>,
    pub sound_enabled: bool,
}

// ---------------------------------------------------------------------------
// Setup: pre-generate all sounds as AudioSource assets
// ---------------------------------------------------------------------------

pub fn setup_audio(
    mut commands: Commands,
    mut audio_assets: ResMut<Assets<AudioSource>>,
) {
    let mut sounds = std::collections::HashMap::new();

    for &effect in ALL_EFFECTS {
        let samples = generate_sound(effect, 0.7);
        let wav_bytes = samples_to_wav_bytes(&samples, 44100);
        let source = AudioSource {
            bytes: Arc::from(wav_bytes),
        };
        let handle = audio_assets.add(source);
        sounds.insert(effect, handle);
    }

    commands.insert_resource(SoundLibrary {
        sounds,
        sound_enabled: true,
    });
}

// ---------------------------------------------------------------------------
// Play system: spawn one-shot AudioPlayer entities from SoundEvents
// ---------------------------------------------------------------------------

pub fn play_sounds(
    mut commands: Commands,
    library: Res<SoundLibrary>,
    mut events: EventReader<SoundEvent>,
) {
    if !library.sound_enabled {
        events.clear();
        return;
    }

    for event in events.read() {
        if let Some(handle) = library.sounds.get(&event.0) {
            commands.spawn((
                AudioPlayer::new(handle.clone()),
                PlaybackSettings::DESPAWN,
            ));
        }
    }
}

// ---------------------------------------------------------------------------
// Toggle (called from pause menu)
// ---------------------------------------------------------------------------

pub fn toggle_sound(library: &mut SoundLibrary) {
    library.sound_enabled = !library.sound_enabled;
    info!(
        "Sound toggled: {}",
        if library.sound_enabled { "ON" } else { "OFF" }
    );
}

// ---------------------------------------------------------------------------
// Procedural sound generation (unchanged math, baked into WAV bytes)
// ---------------------------------------------------------------------------

fn generate_sound(effect: SoundEffect, volume: f32) -> Vec<f32> {
    let sample_rate = 44100.0;
    match effect {
        SoundEffect::PlayerShoot => {
            let duration = 0.08;
            let num_samples = (sample_rate * duration) as usize;
            (0..num_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate;
                    let freq = 800.0 - (400.0 * t / duration);
                    let envelope = 1.0 - (t / duration);
                    (t * freq * std::f32::consts::TAU).sin() * envelope * volume * 0.3
                })
                .collect()
        }
        SoundEffect::EnemyShoot => {
            let duration = 0.12;
            let num_samples = (sample_rate * duration) as usize;
            (0..num_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate;
                    let freq = 400.0 - (200.0 * t / duration);
                    let envelope = 1.0 - (t / duration);
                    (t * freq * std::f32::consts::TAU).sin() * envelope * volume * 0.25
                })
                .collect()
        }
        SoundEffect::PlayerHit => {
            let duration = 0.15;
            let num_samples = (sample_rate * duration) as usize;
            (0..num_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate;
                    let envelope = 1.0 - (t / duration);
                    let noise = (rand::random::<f32>() * 2.0 - 1.0) * 0.3;
                    let thump = (t * 150.0 * std::f32::consts::TAU).sin() * 0.7;
                    (noise + thump) * envelope * volume * 0.4
                })
                .collect()
        }
        SoundEffect::EnemyHit => {
            let duration = 0.1;
            let num_samples = (sample_rate * duration) as usize;
            (0..num_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate;
                    let envelope = 1.0 - (t / duration);
                    let click = (rand::random::<f32>() * 2.0 - 1.0) * 0.2;
                    let tone = (t * 500.0 * std::f32::consts::TAU).sin() * 0.5;
                    (click + tone) * envelope * volume * 0.3
                })
                .collect()
        }
        SoundEffect::Explosion => {
            let duration = 0.4;
            let num_samples = (sample_rate * duration) as usize;
            (0..num_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate;
                    let envelope = (1.0 - (t / duration)).powf(0.5);
                    let noise = (rand::random::<f32>() * 2.0 - 1.0) * 0.4;
                    let freq = 200.0 - (150.0 * t / duration);
                    let sweep = (t * freq * std::f32::consts::TAU).sin() * 0.8;
                    let mixed = (noise + sweep) * envelope;
                    (mixed * 1.5).tanh() * volume * 0.5
                })
                .collect()
        }
        SoundEffect::GameOver => {
            let duration = 0.8;
            let num_samples = (sample_rate * duration) as usize;
            (0..num_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate;
                    let freq = 600.0 - (500.0 * t / duration);
                    let envelope = 1.0 - (t / duration);
                    (t * freq * std::f32::consts::TAU).sin() * envelope * volume * 0.4
                })
                .collect()
        }
        SoundEffect::GameWon => {
            let duration = 0.5;
            let num_samples = (sample_rate * duration) as usize;
            let notes = [261.63, 329.63, 392.0];
            (0..num_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate;
                    let note_idx = ((t / duration) * 3.0) as usize;
                    let note_idx = note_idx.min(2);
                    let freq = notes[note_idx];
                    let local_t = t - (note_idx as f32 * duration / 3.0);
                    let envelope = (1.0 - (local_t / (duration / 3.0)).min(1.0)) * 0.8;
                    (t * freq * std::f32::consts::TAU).sin() * envelope * volume * 0.3
                })
                .collect()
        }
        SoundEffect::MenuSelect => {
            let duration = 0.05;
            let num_samples = (sample_rate * duration) as usize;
            (0..num_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate;
                    let envelope = 1.0 - (t / duration);
                    (t * 1000.0 * std::f32::consts::TAU).sin() * envelope * volume * 0.2
                })
                .collect()
        }
        SoundEffect::BossSpawn => {
            let duration = 0.5;
            let num_samples = (sample_rate * duration) as usize;
            (0..num_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate;
                    let freq = 80.0 + (120.0 * t / duration);
                    let envelope = (1.0 - (t / duration)).powf(0.3);
                    (t * freq * std::f32::consts::TAU).sin() * envelope * volume * 0.5
                })
                .collect()
        }
        SoundEffect::PhaseShift => {
            let duration = 0.2;
            let num_samples = (sample_rate * duration) as usize;
            (0..num_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate;
                    let freq = 300.0 - (200.0 * t / duration);
                    let envelope = 1.0 - (t / duration);
                    let noise = (rand::random::<f32>() * 2.0 - 1.0) * 0.5;
                    let sweep = (t * freq * std::f32::consts::TAU).sin() * 0.5;
                    (noise + sweep) * envelope * volume * 0.4
                })
                .collect()
        }
        SoundEffect::RageBurst => {
            let duration = 0.3;
            let num_samples = (sample_rate * duration) as usize;
            (0..num_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate;
                    let envelope = (1.0 - (t / duration)).powf(0.5);
                    let noise = (rand::random::<f32>() * 2.0 - 1.0) * 0.3;
                    let thump = (t * 50.0 * std::f32::consts::TAU).sin() * 0.8;
                    ((noise + thump) * 1.5).tanh() * envelope * volume * 0.5
                })
                .collect()
        }
        SoundEffect::DashTelegraph => {
            let duration = 0.8;
            let num_samples = (sample_rate * duration) as usize;
            (0..num_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate;
                    let freq = 200.0 + (600.0 * t / duration);
                    let envelope = (t / duration).min(1.0) * (1.0 - (t / duration)).max(0.0);
                    (t * freq * std::f32::consts::TAU).sin() * envelope * volume * 0.3
                })
                .collect()
        }
        SoundEffect::BeamSweep => {
            let duration = 0.5;
            let num_samples = (sample_rate * duration) as usize;
            (0..num_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate;
                    let envelope = 1.0 - (t / duration).powf(2.0);
                    (t * 400.0 * std::f32::consts::TAU).sin() * envelope * volume * 0.3
                })
                .collect()
        }
        SoundEffect::ChargeWindUp => {
            let duration = 0.8;
            let num_samples = (sample_rate * duration) as usize;
            (0..num_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate;
                    let progress = t / duration;
                    let freq = 100.0 + (300.0 * progress * progress);
                    let envelope = progress.min(1.0);
                    (t * freq * std::f32::consts::TAU).sin() * envelope * volume * 0.35
                })
                .collect()
        }
        SoundEffect::HazardSpawn => {
            let duration = 0.1;
            let num_samples = (sample_rate * duration) as usize;
            (0..num_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate;
                    let freq = 600.0 - (400.0 * t / duration);
                    let envelope = 1.0 - (t / duration);
                    (t * freq * std::f32::consts::TAU).sin() * envelope * volume * 0.25
                })
                .collect()
        }
        SoundEffect::HazardExplode => {
            let duration = 0.15;
            let num_samples = (sample_rate * duration) as usize;
            (0..num_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate;
                    let envelope = 1.0 - (t / duration);
                    let noise = (rand::random::<f32>() * 2.0 - 1.0) * 0.5;
                    let tone = (t * 200.0 * std::f32::consts::TAU).sin() * 0.5;
                    (noise + tone) * envelope * volume * 0.4
                })
                .collect()
        }
        SoundEffect::RoundClear => {
            let duration = 0.8;
            let num_samples = (sample_rate * duration) as usize;
            (0..num_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate;
                    let envelope = (1.0 - (t / duration)).powf(0.5);
                    let tone1 = (t * 400.0 * std::f32::consts::TAU).sin();
                    let tone2 = (t * 500.0 * std::f32::consts::TAU).sin();
                    let tone3 = (t * 600.0 * std::f32::consts::TAU).sin();
                    (tone1 + tone2 + tone3) / 3.0 * envelope * volume * 0.4
                })
                .collect()
        }
        SoundEffect::ShockwavePowerUp => {
            let duration = 0.4;
            let num_samples = (sample_rate * duration) as usize;
            (0..num_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate;
                    let freq = 40.0 + (40.0 * t / duration);
                    let envelope = (1.0 - (t / duration)).powf(0.5);
                    let noise = (rand::random::<f32>() * 2.0 - 1.0) * 0.4;
                    let thump = (t * freq * std::f32::consts::TAU).sin() * 0.8;
                    ((noise + thump) * 1.5).tanh() * envelope * volume * 0.5
                })
                .collect()
        }
        SoundEffect::LaserHum => {
            let duration = 0.5;
            let num_samples = (sample_rate * duration) as usize;
            (0..num_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate;
                    let envelope = 1.0 - (t / duration).powf(2.0);
                    (t * 300.0 * std::f32::consts::TAU).sin() * envelope * volume * 0.3
                })
                .collect()
        }
        SoundEffect::LaserCharge => {
            let duration = 0.8;
            let num_samples = (sample_rate * duration) as usize;
            (0..num_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate;
                    let progress = t / duration;
                    let freq = 100.0 + (700.0 * progress * progress);
                    let envelope = progress.min(1.0);
                    (t * freq * std::f32::consts::TAU).sin() * envelope * volume * 0.4
                })
                .collect()
        }
        SoundEffect::LaserFire => {
            let duration = 0.15;
            let num_samples = (sample_rate * duration) as usize;
            (0..num_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate;
                    let freq = 800.0 - (600.0 * t / duration);
                    let envelope = (1.0 - (t / duration)).powf(0.5);
                    let noise = (rand::random::<f32>() * 2.0 - 1.0) * 0.2;
                    let tone = (t * freq * std::f32::consts::TAU).sin() * 0.8;
                    (noise + tone) * envelope * volume * 0.6
                })
                .collect()
        }
        SoundEffect::LaserFadeOut => {
            let duration = 0.5;
            let num_samples = (sample_rate * duration) as usize;
            (0..num_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate;
                    let freq = 400.0 - (320.0 * t / duration);
                    let envelope = 1.0 - (t / duration);
                    (t * freq * std::f32::consts::TAU).sin() * envelope * volume * 0.3
                })
                .collect()
        }
    }
}

// ---------------------------------------------------------------------------
// WAV encoding: f32 samples -> WAV byte buffer for Bevy's AudioSource
// ---------------------------------------------------------------------------

fn samples_to_wav_bytes(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    // WAV header for 32-bit float mono PCM
    let data_size = (samples.len() * 4) as u32;
    let file_size = 36 + data_size;

    let mut buf = Vec::with_capacity(file_size as usize + 8);

    // RIFF header
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&file_size.to_le_bytes());
    buf.extend_from_slice(b"WAVE");

    // fmt chunk
    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16u32.to_le_bytes()); // chunk size
    buf.extend_from_slice(&3u16.to_le_bytes()); // format: IEEE float
    buf.extend_from_slice(&1u16.to_le_bytes()); // channels: mono
    buf.extend_from_slice(&sample_rate.to_le_bytes());
    buf.extend_from_slice(&(sample_rate * 4).to_le_bytes()); // byte rate
    buf.extend_from_slice(&4u16.to_le_bytes()); // block align
    buf.extend_from_slice(&32u16.to_le_bytes()); // bits per sample

    // data chunk
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&data_size.to_le_bytes());
    for &sample in samples {
        buf.extend_from_slice(&sample.to_le_bytes());
    }

    buf
}
```

Note: the `hound` dependency is gone. The WAV encoding is now a simple inline function (~20 lines) that writes the header + float samples directly. This avoids pulling in a crate just for a trivial format.

- [ ] **Step 2: Verify the module compiles in isolation**

Run: `cargo check 2>&1 | head -20`
Expected: errors in `app.rs` and `menus.rs` about `SynthAudio` being gone — that's correct, we fix those next.

- [ ] **Step 3: Commit**

```bash
git add src/systems/audio.rs
git commit -m "feat: replace kira audio with bevy_audio SoundLibrary"
```

---

### Task 3: Update src/app.rs

**Files:**
- Modify: `src/app.rs`

Three changes: (1) swap the startup system, (2) rewrite `pause_menu_system` to use `ResMut<SoundLibrary>`, (3) configure the window for WASM canvas embedding.

- [ ] **Step 1: Update the Startup audio system registration**

In `src/app.rs`, find line 122:

```rust
        .add_systems(Startup, crate::systems::audio::setup_synth_audio)
```

Replace with:

```rust
        .add_systems(Startup, crate::systems::audio::setup_audio)
```

- [ ] **Step 2: Update pause_menu_system signature and body**

Replace the entire `pause_menu_system` function (lines 318-405) with:

```rust
pub fn pause_menu_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    mut library: ResMut<crate::systems::audio::SoundLibrary>,
    pause_query: Query<Entity, With<PauseEntity>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::RoundActive);
    }

    if keyboard_input.just_pressed(KeyCode::KeyQ) {
        next_state.set(GameState::Menu);
    }

    if keyboard_input.just_pressed(KeyCode::KeyM) {
        crate::systems::audio::toggle_sound(&mut library);
        // Respawn pause menu with updated sound status
        for entity in &pause_query {
            commands.entity(entity).despawn();
        }
        let sound_status = if library.sound_enabled { "ON" } else { "OFF" };
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
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
                PauseEntity,
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("PAUSED"),
                    TextFont {
                        font_size: 36.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.0, 1.0, 1.0)),
                ));
                parent
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            padding: UiRect::all(Val::Px(24.0)),
                            row_gap: Val::Px(12.0),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BorderColor(Color::srgb(0.15, 0.15, 0.15)),
                    ))
                    .with_children(|container| {
                        let gray = Color::srgb(0.33, 0.33, 0.33);
                        container.spawn((
                            Text::new("Press ESC to Resume"),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(gray),
                        ));
                        container.spawn((
                            Text::new("Press Q to Return to Menu"),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(gray),
                        ));
                        container.spawn((
                            Text::new(format!("Press M to Toggle Sound ({})", sound_status)),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(gray),
                        ));
                    });
            });
    }
}
```

- [ ] **Step 3: Configure DefaultPlugins with WASM-friendly window settings**

Replace the `App::new()` line (line 109-110):

```rust
    App::new()
        .add_plugins((DefaultPlugins, CrtPostProcessPlugin))
```

With:

```rust
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Cyberpunk: The Incredible Bloom Cube".to_string(),
                    canvas: Some("#game-canvas".to_string()),
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: true,
                    ..default()
                }),
                ..default()
            }),
            CrtPostProcessPlugin,
        ))
```

Add this import at the top of the file if not already present:

```rust
use bevy::window::WindowPlugin;
```

This does three things for WASM:
- `canvas`: targets a specific `<canvas id="game-canvas">` element
- `fit_canvas_to_parent`: auto-resizes to fill the iframe
- `prevent_default_event_handling`: stops Space from scrolling, etc.

On native these settings are harmless (canvas is ignored, the others just set the window title).

- [ ] **Step 4: Verify compilation**

Run: `cargo check 2>&1 | head -20`
Expected: errors only in `src/ui/menus.rs` about `SynthAudio` (fixed in Task 4).

- [ ] **Step 5: Commit**

```bash
git add src/app.rs
git commit -m "feat: wire bevy_audio setup, update pause menu, add WASM window config"
```

---

### Task 4: Update src/ui/menus.rs

**Files:**
- Modify: `src/ui/menus.rs:138-141`

- [ ] **Step 1: Replace NonSendMut with Res in spawn_pause_menu**

Find lines 138-142:

```rust
pub fn spawn_pause_menu(
    mut commands: Commands,
    audio: NonSendMut<crate::systems::audio::SynthAudio>,
) {
    let sound_status = if audio.sound_enabled { "ON" } else { "OFF" };
```

Replace with:

```rust
pub fn spawn_pause_menu(
    mut commands: Commands,
    library: Res<crate::systems::audio::SoundLibrary>,
) {
    let sound_status = if library.sound_enabled { "ON" } else { "OFF" };
```

- [ ] **Step 2: Verify full project compiles**

Run: `cargo check`
Expected: clean compilation, no errors.

- [ ] **Step 3: Run existing tests**

Run: `cargo test`
Expected: all tests pass. The tests only register `SoundEvent` as an event — they never instantiate the audio backend, so they are unaffected by this change.

- [ ] **Step 4: Commit**

```bash
git add src/ui/menus.rs
git commit -m "fix: update spawn_pause_menu to use SoundLibrary resource"
```

---

### Task 5: Create index.html for trunk

**Files:**
- Create: `index.html`

- [ ] **Step 1: Create the trunk entry point**

Write `index.html` at the project root:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Cyberpunk: The Incredible Bloom Cube</title>
    <style>
        * { margin: 0; padding: 0; }
        html, body { width: 100%; height: 100%; overflow: hidden; background: #000; }
        canvas { display: block; width: 100%; height: 100%; }
    </style>
</head>
<body>
    <canvas id="game-canvas"></canvas>
    <link data-trunk rel="rust" data-wasm-opt="z" data-cargo-features="webgpu" />
</body>
</html>
```

The `data-cargo-features="webgpu"` tells trunk to compile with the `webgpu` feature, enabling HDR/Bloom/CRT shader on WebGPU-capable browsers.

The `data-wasm-opt="z"` optimizes the WASM binary for size.

The `<canvas id="game-canvas">` matches the `canvas: Some("#game-canvas".to_string())` in the Bevy Window config from Task 3.

- [ ] **Step 2: Commit**

```bash
git add index.html
git commit -m "feat: add trunk index.html for WASM builds"
```

---

### Task 6: Verify native build and tests

**Files:** none (verification only)

- [ ] **Step 1: Run full test suite**

Run: `cargo test`
Expected: all tests pass (same as before the migration).

- [ ] **Step 2: Run native build**

Run: `cargo run --release`
Expected: game launches, audio plays, all systems work identically to before. Keyboard controls (WASD, Enter, Escape, M, Q, Space) all function. Sound toggle in pause menu works.

- [ ] **Step 3: Verify no clippy warnings**

Run: `cargo clippy -- -D warnings`
Expected: clean.

---

### Task 7: Build and verify WASM

**Files:** none (verification only)

- [ ] **Step 1: Install trunk and WASM target if not present**

Run:
```bash
cargo install trunk
rustup target add wasm32-unknown-unknown
```

- [ ] **Step 2: Build WASM**

Run: `trunk build --release`
Expected: successful build. Output in `dist/` directory containing `index.html`, `.wasm` file, and `.js` glue code.

- [ ] **Step 3: Serve locally and test in browser**

Run: `trunk serve --release`
Expected: opens at `http://127.0.0.1:8080`. Game renders with HDR bloom and CRT shader. WASD movement, Enter to start, Escape to pause, M to toggle sound all work via keyboard.

Note: requires a WebGPU-capable browser (Chrome 113+, Firefox 141+, Safari 18.2+, Edge 113+).

- [ ] **Step 4: Verify dist/ output structure for Next.js**

Run: `ls -la dist/`
Expected: `index.html`, `cyberpunk_rpg-<hash>_bg.wasm`, `cyberpunk_rpg-<hash>.js`

These files go into your Next.js project's `public/game/` directory. Embed with:
```jsx
<iframe src="/game/index.html" width="1200" height="600" />
```

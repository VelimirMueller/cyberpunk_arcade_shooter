# Enterprise Juice Pass — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add 4 layers of game juice (background, particles, CRT shader, synth audio) to transform the cyberpunk arcade shooter from prototype to premium feel.

**Architecture:** Layer Cake — each layer is a self-contained set of new files + minimal edits to existing systems. Each layer compiles and runs independently. Event-driven communication between systems (DeathEvent, SoundEvent).

**Tech Stack:** Bevy 0.16.1, Rust (edition 2024), WGSL shaders, kira audio crate

**Spec:** `docs/superpowers/specs/2026-03-18-enterprise-juice-design.md`

---

## File Map

### New Files
| File | Purpose |
|------|---------|
| `src/systems/background.rs` | Background grid (gizmos) + star particle entities |
| `src/systems/particles.rs` | Death effects (shatter + shockwave), player trails (afterimage + ambient) |
| `src/systems/post_processing.rs` | CRT post-processing render node + pipeline |
| `assets/shaders/crt_post_process.wgsl` | WGSL fragment shader for scanlines, vignette, barrel distortion |

### Modified Files
| File | Changes |
|------|---------|
| `Cargo.toml` | Add `kira`, `hound` dependencies |
| `src/main.rs` | No changes |
| `src/systems/mod.rs` | Add `pub mod background;`, `pub mod particles;`, `pub mod post_processing;` |
| `src/core/enemies/components.rs` | Add `is_dead: bool` field to `Enemy` |
| `src/core/enemies/systems.rs` | Initialize `is_dead: false` in `spawn_enemy` |
| `src/app.rs:16-31` | Add `enemies_killed` + `total_enemies` to `GameData`; update win condition in `update_enemy_health_ui` |
| `src/systems/collision.rs` | Add `is_dead` guard, send `DeathEvent`; later (Task 8) migrate to `SoundEvent` |
| `src/systems/combat.rs` | Send `SoundEvent` instead of calling `play_sound` directly |
| `src/systems/audio.rs` | Full rewrite: `SynthAudio` (NonSend), `SoundEvent`, procedural synthesis via kira |
| `src/app.rs` | Register new systems, events, resources, CRT component on camera |

---

### Task 1: Background Stars

**Files:**
- Create: `src/systems/background.rs`
- Modify: `src/systems/mod.rs`
- Modify: `src/app.rs`

- [ ] **Step 1: Create `src/systems/background.rs` with star components and spawn system**

```rust
use bevy::prelude::*;
use rand::Rng;
use crate::env::{LEFT_BOUND, RIGHT_BOUND, GROUND_Y, CEILING_Y};

#[derive(Component)]
pub struct BackgroundStar {
    pub velocity: Vec2,
}

pub fn spawn_background_stars(mut commands: Commands) {
    let mut rng = rand::thread_rng();
    let colors = [
        Color::srgba(1.0, 1.0, 1.0, 0.3),
        Color::srgba(0.5, 1.5, 0.5, 0.2),
        Color::srgba(1.5, 0.5, 1.0, 0.15),
    ];

    for _ in 0..40 {
        let x = rng.gen_range(LEFT_BOUND..RIGHT_BOUND);
        let y = rng.gen_range(GROUND_Y..CEILING_Y);
        let size = rng.gen_range(1.0..3.0);
        let speed = rng.gen_range(5.0..15.0);
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let color = colors[rng.gen_range(0..colors.len())];

        commands.spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_xyz(x, y, -90.0),
            BackgroundStar {
                velocity: Vec2::new(angle.cos() * speed, angle.sin() * speed),
            },
        ));
    }
}

pub fn animate_stars(
    time: Res<Time>,
    mut query: Query<(&BackgroundStar, &mut Transform)>,
) {
    let dt = time.delta().as_secs_f32();
    for (star, mut transform) in &mut query {
        transform.translation.x += star.velocity.x * dt;
        transform.translation.y += star.velocity.y * dt;

        // Wrap at arena bounds
        if transform.translation.x > RIGHT_BOUND {
            transform.translation.x = LEFT_BOUND;
        } else if transform.translation.x < LEFT_BOUND {
            transform.translation.x = RIGHT_BOUND;
        }
        if transform.translation.y > CEILING_Y {
            transform.translation.y = GROUND_Y;
        } else if transform.translation.y < GROUND_Y {
            transform.translation.y = CEILING_Y;
        }
    }
}
```

- [ ] **Step 2: Add module declaration in `src/systems/mod.rs`**

Add this line:
```rust
pub mod background;
```

- [ ] **Step 3: Register systems in `src/app.rs`**

Add import at top:
```rust
use crate::systems::background::{spawn_background_stars, animate_stars};
```

Add `spawn_background_stars` to the `Startup` systems tuple:
```rust
.add_systems(Startup, (setup, setup_menu, crate::systems::audio::setup_audio, spawn_background_stars))
```

Add `animate_stars` to the `Playing` state systems. Also add it to `Menu`, `Paused`, `GameOver`, and `Won` states so stars always drift:
```rust
.add_systems(Update, animate_stars) // runs in ALL states
```

- [ ] **Step 4: Build and verify**

Run: `cargo build`
Expected: Compiles with no errors.

Run: `cargo run`
Expected: Stars visible drifting slowly on black background in menu screen. Stars persist across all game states.

- [ ] **Step 5: Commit**

```bash
git add src/systems/background.rs src/systems/mod.rs src/app.rs
git commit -m "feat: add drifting background star particles"
```

---

### Task 2: Background Grid (Gizmos)

**Files:**
- Modify: `src/systems/background.rs`
- Modify: `src/app.rs`

- [ ] **Step 1: Add grid drawing system to `src/systems/background.rs`**

Append to the file:
```rust
pub fn draw_background_grid(mut gizmos: Gizmos) {
    let grid_color = Color::srgba(0.0, 1.0, 0.25, 0.04);
    let cell_size = 40.0;

    // Vertical lines
    let mut x = LEFT_BOUND;
    while x <= RIGHT_BOUND {
        gizmos.line_2d(
            Vec2::new(x, GROUND_Y),
            Vec2::new(x, CEILING_Y),
            grid_color,
        );
        x += cell_size;
    }

    // Horizontal lines
    let mut y = GROUND_Y;
    while y <= CEILING_Y {
        gizmos.line_2d(
            Vec2::new(LEFT_BOUND, y),
            Vec2::new(RIGHT_BOUND, y),
            grid_color,
        );
        y += cell_size;
    }
}
```

- [ ] **Step 2: Register `draw_background_grid` in `src/app.rs`**

Add to the import:
```rust
use crate::systems::background::{spawn_background_stars, animate_stars, draw_background_grid};
```

Add as an always-running system (alongside `animate_stars`):
```rust
.add_systems(Update, (animate_stars, draw_background_grid))
```

- [ ] **Step 3: Build and verify**

Run: `cargo run`
Expected: Faint green grid visible behind stars on all screens. Grid stays static, stars drift over it.

- [ ] **Step 4: Commit**

```bash
git add src/systems/background.rs src/app.rs
git commit -m "feat: add background grid via gizmos"
```

---

### Task 3: Enemy `is_dead` Flag + DeathEvent

**Files:**
- Modify: `src/core/enemies/components.rs`
- Modify: `src/core/enemies/systems.rs`
- Modify: `src/systems/collision.rs`
- Modify: `src/app.rs`

- [ ] **Step 1: Add `is_dead` field to `Enemy` in `src/core/enemies/components.rs`**

```rust
#[derive(Component)]
pub struct Enemy {
    pub current: u32,
    pub max: u32,
    pub fire_timer: Option<Timer>,
    pub last_collision_time: Option<std::time::Instant>,
    pub is_dead: bool,
}
```

- [ ] **Step 2: Initialize `is_dead: false` in `src/core/enemies/systems.rs`**

In the `spawn_enemy` function, add `is_dead: false` to the `Enemy` struct initialization:
```rust
Enemy { current: 100, max: 100, fire_timer: if count == 0.0 {
    Some(Timer::from_seconds(0.25, TimerMode::Repeating))
} else {
    None
}, last_collision_time: None, is_dead: false },
```

- [ ] **Step 3: Define `DeathEvent` and update collision system in `src/systems/collision.rs`**

Add at the top of the file:
```rust
#[derive(Event)]
pub struct DeathEvent {
    pub position: Vec3,
    pub color: Color,
    pub entity: Entity,
}
```

In `detect_collisions`, update the player-particle-vs-enemy section (lines 69-93). Change the `enemy_query` to include `Entity`:
```rust
// Change the inner enemy loop query iteration to also get Entity
for (enemy_entity, mut enemy, enemy_transform, enemy_sprite) in &mut enemy_query {
```

Update the query signature to include `Entity`:
```rust
mut enemy_query: Query<(Entity, &mut Enemy, &Transform, &Sprite), With<Enemy>>,
```

Add `mut death_events: EventWriter<DeathEvent>,` to the system parameters.

Update the player-vs-enemy collision loop (line 24) to also destructure `Entity`:
```rust
for (_, _enemy, enemy_transform, enemy_sprite) in &enemy_query {
```

In the player-particle-vs-enemy loop, **replace the entire `if collide(...)` block** (including the existing `else` branch at lines 86-90 that fires every frame for dead enemies — that `else` block must be deleted entirely). New code:
```rust
if collide(particle_pos, particle_size, enemy_pos, enemy_size) {
    if enemy.is_dead {
        continue;
    }
    if enemy.current > 0 {
        if enemy.last_collision_time.map_or(true, |t| t.elapsed().as_secs_f32() > 0.075) {
            enemy.current -= 1;
            enemy.last_collision_time = Some(std::time::Instant::now());
            game_data.score += 10;
            play_sound(&audio, SoundEffect::EnemyHit);
            info!("You hit the Enemy HP: {}", enemy.current);

            if enemy.current == 0 {
                enemy.is_dead = true;
                game_data.score += 100;
                play_sound(&audio, SoundEffect::Explosion);
                death_events.write(DeathEvent {
                    position: enemy_transform.translation,
                    color: enemy_sprite.color,
                    entity: enemy_entity,
                });
            }
        }
    }
}
```

Also add `is_dead` skip to the player-vs-enemy body collision (line 24 area):
```rust
for (_entity, _enemy, enemy_transform, enemy_sprite) in &enemy_query {
    // Skip dead enemies
    if _enemy.is_dead { continue; }
    // ... existing collision code
}
```

- [ ] **Step 4: Fix win condition in `src/app.rs`**

Since Task 4 will despawn dead enemies, the current `update_enemy_health_ui` (which sums HP of remaining enemies to detect win) will break — it'll see total_hp == 0 after the first kill because the dead enemy is gone.

Add `enemies_killed` and `total_enemies` fields to `GameData`:
```rust
#[derive(Resource)]
pub struct GameData {
    pub score: u32,
    pub wave: u32,
    pub high_score: u32,
    pub total_play_time: f32,
    pub enemies_killed: u32,
    pub total_enemies: u32,
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            score: 0,
            wave: 1,
            high_score: 0,
            total_play_time: 0.0,
            enemies_killed: 0,
            total_enemies: 3, // 3 bosses
        }
    }
}
```

In the collision system's death branch (Step 3), add after `enemy.is_dead = true;`:
```rust
game_data.enemies_killed += 1;
```

Update `update_enemy_health_ui` in `app.rs` to check kills instead of HP sum:
```rust
pub fn update_enemy_health_ui(
    enemy_query: Query<&Enemy>,
    mut span_query: Query<&mut TextSpan, With<EnemyHpText>>,
    mut next_state: ResMut<NextState<GameState>>,
    game_data: Res<GameData>,
) {
    let total_hp: u32 = enemy_query.iter().map(|enemy| enemy.current).sum();
    for mut span in &mut span_query {
        **span = format!("{} %", total_hp);
    }

    if game_data.enemies_killed >= game_data.total_enemies {
        next_state.set(GameState::Won);
    }
}
```

Also reset `enemies_killed` in `restart_listener` (in `game_over.rs`) alongside the other `game_data` resets:
```rust
game_data.enemies_killed = 0;
```

- [ ] **Step 5: Register `DeathEvent` in `src/app.rs`**

Add import:
```rust
use crate::systems::collision::DeathEvent;
```

Add before `.add_systems(Startup, ...)`:
```rust
.add_event::<DeathEvent>()
```

- [ ] **Step 6: Build and verify**

Run: `cargo build`
Expected: Compiles. Enemies that reach 0 HP now send a `DeathEvent` and are marked `is_dead`. Win condition now based on kill count, not HP sum. No visual change yet (death effects come in Task 4).

**NOTE:** Task 3 still uses the old `play_sound(&audio, ...)` pattern. This will be migrated to `SoundEvent` in Task 8.

- [ ] **Step 7: Commit**

```bash
git add src/core/enemies/components.rs src/core/enemies/systems.rs src/systems/collision.rs src/app.rs src/systems/game_over.rs
git commit -m "feat: add is_dead flag, DeathEvent, and kill-count win condition"
```

---

### Task 4: Death Effects (Shatter Particles + Shockwave Ring)

**Files:**
- Create: `src/systems/particles.rs`
- Modify: `src/systems/mod.rs`
- Modify: `src/app.rs`

- [ ] **Step 1: Create `src/systems/particles.rs` with shatter + shockwave**

```rust
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
```

- [ ] **Step 2: Add module declaration in `src/systems/mod.rs`**

```rust
pub mod particles;
```

- [ ] **Step 3: Register systems in `src/app.rs`**

Add imports:
```rust
use crate::systems::particles::{
    setup_shockwave_assets, handle_death_events,
    animate_shatter, animate_shockwave,
};
```

Add to Startup:
```rust
.add_systems(Startup, (setup, setup_menu, crate::systems::audio::setup_audio, spawn_background_stars, setup_shockwave_assets))
```

Add to the Playing state systems, with `handle_death_events` ordered after `detect_collisions`:
```rust
.add_systems(Update, handle_death_events.after(detect_collisions).run_if(in_state(GameState::Playing)))
.add_systems(Update, (animate_shatter, animate_shockwave).run_if(in_state(GameState::Playing)))
```

- [ ] **Step 4: Build and verify**

Run: `cargo run`
Expected: When enemies die (HP reaches 0 from player projectiles), they explode into colored cube fragments that scatter with gravity, and a ring of light expands outward. Both fade and disappear after ~0.3-0.5s.

- [ ] **Step 5: Commit**

```bash
git add src/systems/particles.rs src/systems/mod.rs src/app.rs
git commit -m "feat: add enemy death shatter particles and shockwave ring"
```

---

### Task 5: Player Afterimage Trail

**Files:**
- Modify: `src/systems/particles.rs`
- Modify: `src/app.rs`

- [ ] **Step 1: Add afterimage components and systems to `src/systems/particles.rs`**

Append to the file:
```rust
// ---- Player Afterimage Trail ----

#[derive(Component)]
pub struct Afterimage {
    pub lifetime: f32,
    pub max_lifetime: f32,
}

#[derive(Resource)]
pub struct AfterimageTimer {
    pub timer: Timer,
}

impl Default for AfterimageTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.05, TimerMode::Repeating),
        }
    }
}

pub fn spawn_afterimages(
    time: Res<Time>,
    mut timer: ResMut<AfterimageTimer>,
    mut commands: Commands,
    player_query: Query<(&Transform, &Sprite), With<crate::core::player::components::Player>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    // Always tick the timer so first afterimage spawns immediately on movement
    timer.timer.tick(time.delta());

    // Only spawn afterimages while the player is moving
    let is_moving = keyboard_input.pressed(KeyCode::KeyW)
        || keyboard_input.pressed(KeyCode::KeyA)
        || keyboard_input.pressed(KeyCode::KeyS)
        || keyboard_input.pressed(KeyCode::KeyD);

    if !is_moving || !timer.timer.just_finished() {
        return;
    }

    for (transform, sprite) in &player_query {
        let ghost_color = sprite.color.with_alpha(0.5);

        commands.spawn((
            Sprite {
                color: ghost_color,
                custom_size: sprite.custom_size,
                ..default()
            },
            *transform,
            Afterimage {
                lifetime: 0.15,
                max_lifetime: 0.15,
            },
            GameEntity,
        ));
    }
}

pub fn animate_afterimages(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Afterimage, &mut Sprite)>,
) {
    let dt = time.delta().as_secs_f32();

    for (entity, mut image, mut sprite) in &mut query {
        image.lifetime -= dt;

        if image.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        let alpha = (image.lifetime / image.max_lifetime).clamp(0.0, 1.0) * 0.5;
        sprite.color = sprite.color.with_alpha(alpha);
    }
}
```

- [ ] **Step 2: Register in `src/app.rs`**

Add to imports:
```rust
use crate::systems::particles::{
    setup_shockwave_assets, handle_death_events,
    animate_shatter, animate_shockwave,
    AfterimageTimer, spawn_afterimages, animate_afterimages,
};
```

Add resource init:
```rust
.init_resource::<AfterimageTimer>()
```

Add to Playing state systems:
```rust
.add_systems(Update, (spawn_afterimages, animate_afterimages).run_if(in_state(GameState::Playing)))
```

- [ ] **Step 3: Build and verify**

Run: `cargo run`
Expected: When the player cube moves, ghostly green copies trail behind it, fading out over ~0.15s. ~5-6 ghosts visible at once during movement.

- [ ] **Step 4: Commit**

```bash
git add src/systems/particles.rs src/app.rs
git commit -m "feat: add player afterimage trail effect"
```

---

### Task 6: Player Ambient Particles

**Files:**
- Modify: `src/systems/particles.rs`
- Modify: `src/app.rs`

- [ ] **Step 1: Add ambient particle components and systems to `src/systems/particles.rs`**

Append:
```rust
// ---- Player Ambient Particles ----

#[derive(Component)]
pub struct AmbientParticle {
    pub velocity: Vec2,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

#[derive(Resource)]
pub struct AmbientParticleTimer {
    pub timer: Timer,
}

impl Default for AmbientParticleTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.4, TimerMode::Repeating), // ~2.5 per second
        }
    }
}

pub fn spawn_ambient_particles(
    time: Res<Time>,
    mut timer: ResMut<AmbientParticleTimer>,
    mut commands: Commands,
    player_query: Query<&Transform, With<crate::core::player::components::Player>>,
) {
    timer.timer.tick(time.delta());

    if !timer.timer.just_finished() {
        return;
    }

    let mut rng = rand::thread_rng();

    for transform in &player_query {
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let speed = rng.gen_range(20.0..40.0);
        let offset_x = rng.gen_range(-25.0..25.0);
        let offset_y = rng.gen_range(-25.0..25.0);

        commands.spawn((
            Sprite {
                color: Color::srgba(0.5, 1.5, 0.3, 0.15),
                custom_size: Some(Vec2::splat(rng.gen_range(1.0..2.0))),
                ..default()
            },
            Transform::from_xyz(
                transform.translation.x + offset_x,
                transform.translation.y + offset_y,
                transform.translation.z - 0.1,
            ),
            AmbientParticle {
                velocity: Vec2::new(angle.cos() * speed, angle.sin() * speed),
                lifetime: 0.8,
                max_lifetime: 0.8,
            },
            GameEntity,
        ));
    }
}

pub fn animate_ambient_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut AmbientParticle, &mut Transform, &mut Sprite)>,
) {
    let dt = time.delta().as_secs_f32();

    for (entity, mut particle, mut transform, mut sprite) in &mut query {
        particle.lifetime -= dt;

        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;

        let alpha = (particle.lifetime / particle.max_lifetime).clamp(0.0, 1.0) * 0.15;
        sprite.color = sprite.color.with_alpha(alpha);
    }
}
```

- [ ] **Step 2: Register in `src/app.rs`**

Add to imports and resource init:
```rust
use crate::systems::particles::{
    // ... existing imports ...
    AmbientParticleTimer, spawn_ambient_particles, animate_ambient_particles,
};
```

```rust
.init_resource::<AmbientParticleTimer>()
```

Add to Playing state:
```rust
.add_systems(Update, (spawn_ambient_particles, animate_ambient_particles).run_if(in_state(GameState::Playing)))
```

- [ ] **Step 3: Build and verify**

Run: `cargo run`
Expected: Tiny faint green particles continuously float off the player cube in random directions, even when idle. Gives the cube a "radiating energy" look.

- [ ] **Step 4: Commit**

```bash
git add src/systems/particles.rs src/app.rs
git commit -m "feat: add player ambient energy particles"
```

---

### Task 7: CRT Post-Processing Shader

**Files:**
- Create: `src/shaders/crt_post_process.wgsl`
- Create: `src/systems/post_processing.rs`
- Modify: `src/systems/mod.rs`
- Modify: `src/app.rs`

This is the most complex task. Bevy 0.16 post-processing requires a custom render node. The Rust code below is a **scaffold with the component and plugin structure** — the render node implementation (`ViewNode::run()`) must be completed by cross-referencing the Bevy 0.16 `examples/shader/post_processing.rs` example, as the render graph API is version-sensitive and cannot be reliably provided ahead of time.

**Before starting this task, the implementer MUST:**
1. Clone/browse the Bevy 0.16.1 repo and read `examples/shader/post_processing.rs`
2. Run `cargo doc --open -p bevy_core_pipeline -p bevy_render` for exact type signatures
3. Use the example as the template — adapt it to use our `CrtSettings` uniform and `crt_post_process.wgsl` shader

- [ ] **Step 1: Create the WGSL shader at `assets/shaders/crt_post_process.wgsl`**

```bash
mkdir -p assets/shaders
```

```wgsl
#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

struct CrtSettings {
    scanline_intensity: f32,
    scanline_count: f32,
    vignette_intensity: f32,
    vignette_radius: f32,
    curvature_amount: f32,
    _padding: f32,
    _padding2: f32,
    _padding3: f32,
}

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
@group(0) @binding(2) var<uniform> settings: CrtSettings;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    var uv = in.uv;

    // Barrel distortion (CRT curvature)
    let offset = uv - vec2<f32>(0.5, 0.5);
    uv = uv + offset * dot(offset, offset) * settings.curvature_amount;

    // Sample the screen texture
    var color = textureSample(screen_texture, texture_sampler, uv);

    // Phosphor bleed (horizontal neighbor sampling)
    let dims = vec2<f32>(textureDimensions(screen_texture));
    let pixel_x = 1.0 / dims.x;
    let left = textureSample(screen_texture, texture_sampler, uv + vec2<f32>(-pixel_x, 0.0));
    let right = textureSample(screen_texture, texture_sampler, uv + vec2<f32>(pixel_x, 0.0));
    color = color + (left + right) * 0.05;

    // Scanlines
    let scanline = 1.0 - settings.scanline_intensity * (0.5 + 0.5 * sin(uv.y * settings.scanline_count * 3.14159265 * 2.0));
    color = vec4<f32>(color.rgb * scanline, color.a);

    // Vignette
    let dist = length((uv - vec2<f32>(0.5, 0.5)) * 2.0);
    let vignette = smoothstep(0.0, settings.vignette_radius, 1.0 - dist);
    let vignette_factor = pow(vignette, settings.vignette_intensity);
    color = vec4<f32>(color.rgb * vignette_factor, color.a);

    return color;
}
```

- [ ] **Step 2: Create `src/systems/post_processing.rs`**

This file sets up the Bevy render node. The exact implementation depends on Bevy 0.16's post-processing API. Follow the pattern from Bevy's `examples/shader/post_processing.rs`:

```rust
use bevy::{
    core_pipeline::{
        core_2d::graph::{Core2d, Node2d},
        fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    },
    ecs::query::QueryItem,
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_graph::{
            NodeRunError, RenderGraphApp, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            BindGroupLayout, BindGroupLayoutEntries, CachedRenderPipelineId,
            ColorTargetState, ColorWrites, FragmentState, MultisampleState,
            Operations, PipelineCache, PrimitiveState, RenderPassColorAttachment,
            RenderPassDescriptor, RenderPipelineDescriptor, Sampler,
            SamplerBindingType, SamplerDescriptor, ShaderStages, TextureFormat,
            TextureSampleType,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::BevyDefault,
        view::ViewTarget,
        RenderApp,
    },
};
use bevy::render::render_resource::{
    BindGroup, BindGroupEntries, BufferInitDescriptor, BufferUsages,
    ShaderType,
};

// Component on camera
#[derive(Component, Clone, ExtractComponent)]
pub struct CrtSettings {
    pub scanline_intensity: f32,
    pub scanline_count: f32,
    pub vignette_intensity: f32,
    pub vignette_radius: f32,
    pub curvature_amount: f32,
}

impl Default for CrtSettings {
    fn default() -> Self {
        Self {
            scanline_intensity: 0.15,
            scanline_count: 200.0,
            vignette_intensity: 0.4,
            vignette_radius: 0.7,
            curvature_amount: 0.02,
        }
    }
}

// GPU uniform struct (must match WGSL layout, 16-byte aligned)
#[derive(Clone, Copy, ShaderType)]
struct CrtSettingsUniform {
    scanline_intensity: f32,
    scanline_count: f32,
    vignette_intensity: f32,
    vignette_radius: f32,
    curvature_amount: f32,
    _padding: f32,
    _padding2: f32,
    _padding3: f32,
}

pub struct CrtPostProcessPlugin;

impl Plugin for CrtPostProcessPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<CrtSettings>::default());

        // The render-side setup must be done in the RenderApp.
        // NOTE: The exact Bevy 0.16 API for this may differ.
        // Consult `examples/shader/post_processing.rs` in the Bevy 0.16 repo.
        // The pattern is:
        // 1. Add the ViewNodeRunner to the render graph
        // 2. Create the pipeline in FromWorld
        // 3. Implement ViewNode::run() to bind the screen texture + uniforms and draw fullscreen
        //
        // This is a skeleton — the implementer MUST cross-reference with the
        // Bevy 0.16 post_processing example for the exact API.
    }
}
```

**IMPORTANT NOTE TO IMPLEMENTER:** The exact Bevy 0.16 render graph API for post-processing is complex and version-sensitive. Before implementing this file:
1. Run `cargo doc --open -p bevy_core_pipeline` to check available types
2. Look at the Bevy repo's `examples/shader/post_processing.rs` for the canonical pattern
3. The shader file must be loaded via `asset_server.load("shaders/crt_post_process.wgsl")` — copy the WGSL file to `assets/shaders/` (not `src/shaders/`)

- [ ] **Step 3: Add module declaration in `src/systems/mod.rs`**

```rust
pub mod post_processing;
```

- [ ] **Step 4: Register plugin and component in `src/app.rs`**

Add import:
```rust
use crate::systems::post_processing::{CrtPostProcessPlugin, CrtSettings};
```

Add plugin:
```rust
.add_plugins((DefaultPlugins, CrtPostProcessPlugin))
```

Add `CrtSettings::default()` to the camera spawn in `setup()`:
```rust
commands.spawn((
    Camera2d,
    Transform::default(),
    GlobalTransform::default(),
    Camera {
        hdr: true,
        clear_color: ClearColorConfig::Custom(Color::BLACK),
        ..default()
    },
    Tonemapping::TonyMcMapface,
    Bloom::default(),
    DebandDither::Enabled,
    CrtSettings::default(), // NEW
));
```

- [ ] **Step 5: Build and verify**

Run: `cargo run`
Expected: Subtle scanlines visible across the screen, edges slightly darkened by vignette, and a very slight barrel distortion. The existing bloom effect should still work.

- [ ] **Step 6: Commit**

```bash
git add assets/shaders/crt_post_process.wgsl src/systems/post_processing.rs src/systems/mod.rs src/app.rs
git commit -m "feat: add CRT post-processing shader (scanlines, vignette, barrel distortion)"
```

---

### Task 8: Synthesized Audio — SoundEvent System

**Files:**
- Modify: `src/systems/audio.rs`
- Modify: `src/systems/collision.rs`
- Modify: `src/systems/combat.rs`
- Modify: `src/app.rs`
- Modify: `Cargo.toml`

- [ ] **Step 1: Add `kira` and `hound` to `Cargo.toml`**

```toml
[dependencies]
bevy = "0.16.1"
rand = "0.8"
kira = "0.9"
hound = "3.5"
```

- [ ] **Step 2: Rewrite `src/systems/audio.rs`**

Replace the entire file with the new event-based synth audio system:

```rust
use bevy::prelude::*;
use kira::manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend};
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use std::io::Cursor;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SoundEffect {
    PlayerShoot,
    PlayerHit,
    EnemyShoot,
    EnemyHit,
    Explosion,
    GameOver,
    GameWon,
    MenuSelect,
}

#[derive(Event)]
pub struct SoundEvent(pub SoundEffect);

pub struct SynthAudio {
    pub manager: AudioManager<DefaultBackend>,
    pub sound_enabled: bool,
    pub volume: f32,
}

pub fn setup_synth_audio(world: &mut World) {
    let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
        .expect("Failed to initialize audio manager");
    world.insert_non_send_resource(SynthAudio {
        manager,
        sound_enabled: true,
        volume: 0.7,
    });
}

pub fn play_sounds(
    mut audio: NonSendMut<SynthAudio>,
    mut events: EventReader<SoundEvent>,
) {
    if !audio.sound_enabled {
        events.clear();
        return;
    }

    for event in events.read() {
        let samples = generate_sound(event.0, audio.volume);
        let data = samples_to_sound_data(samples, 44100);
        if let Ok(data) = data {
            let _ = audio.manager.play(data);
        }
    }
}

fn generate_sound(effect: SoundEffect, volume: f32) -> Vec<f32> {
    let sample_rate = 44100.0;
    match effect {
        SoundEffect::PlayerShoot => {
            // Sine burst 800Hz -> 400Hz, 0.08s
            let duration = 0.08;
            let num_samples = (sample_rate * duration) as usize;
            (0..num_samples).map(|i| {
                let t = i as f32 / sample_rate;
                let freq = 800.0 - (400.0 * t / duration);
                let envelope = 1.0 - (t / duration);
                (t * freq * std::f32::consts::TAU).sin() * envelope * volume * 0.3
            }).collect()
        }
        SoundEffect::EnemyShoot => {
            let duration = 0.12;
            let num_samples = (sample_rate * duration) as usize;
            (0..num_samples).map(|i| {
                let t = i as f32 / sample_rate;
                let freq = 400.0 - (200.0 * t / duration);
                let envelope = 1.0 - (t / duration);
                (t * freq * std::f32::consts::TAU).sin() * envelope * volume * 0.25
            }).collect()
        }
        SoundEffect::PlayerHit => {
            // White noise burst + low sine thump
            let duration = 0.15;
            let num_samples = (sample_rate * duration) as usize;
            (0..num_samples).map(|i| {
                let t = i as f32 / sample_rate;
                let envelope = 1.0 - (t / duration);
                let noise = (rand::random::<f32>() * 2.0 - 1.0) * 0.3;
                let thump = (t * 150.0 * std::f32::consts::TAU).sin() * 0.7;
                (noise + thump) * envelope * volume * 0.4
            }).collect()
        }
        SoundEffect::EnemyHit => {
            let duration = 0.1;
            let num_samples = (sample_rate * duration) as usize;
            (0..num_samples).map(|i| {
                let t = i as f32 / sample_rate;
                let envelope = 1.0 - (t / duration);
                let click = (rand::random::<f32>() * 2.0 - 1.0) * 0.2;
                let tone = (t * 500.0 * std::f32::consts::TAU).sin() * 0.5;
                (click + tone) * envelope * volume * 0.3
            }).collect()
        }
        SoundEffect::Explosion => {
            // Layered: noise + low sweep 200->50Hz + distortion
            let duration = 0.4;
            let num_samples = (sample_rate * duration) as usize;
            (0..num_samples).map(|i| {
                let t = i as f32 / sample_rate;
                let envelope = (1.0 - (t / duration)).powf(0.5);
                let noise = (rand::random::<f32>() * 2.0 - 1.0) * 0.4;
                let freq = 200.0 - (150.0 * t / duration);
                let sweep = (t * freq * std::f32::consts::TAU).sin() * 0.8;
                let mixed = (noise + sweep) * envelope;
                // Soft clip distortion
                (mixed * 1.5).tanh() * volume * 0.5
            }).collect()
        }
        SoundEffect::GameOver => {
            let duration = 0.8;
            let num_samples = (sample_rate * duration) as usize;
            (0..num_samples).map(|i| {
                let t = i as f32 / sample_rate;
                let freq = 600.0 - (500.0 * t / duration);
                let envelope = 1.0 - (t / duration);
                (t * freq * std::f32::consts::TAU).sin() * envelope * volume * 0.4
            }).collect()
        }
        SoundEffect::GameWon => {
            // Ascending arpeggio C-E-G
            let duration = 0.5;
            let num_samples = (sample_rate * duration) as usize;
            let notes = [261.63, 329.63, 392.0]; // C4, E4, G4
            (0..num_samples).map(|i| {
                let t = i as f32 / sample_rate;
                let note_idx = ((t / duration) * 3.0) as usize;
                let note_idx = note_idx.min(2);
                let freq = notes[note_idx];
                let local_t = t - (note_idx as f32 * duration / 3.0);
                let envelope = (1.0 - (local_t / (duration / 3.0)).min(1.0)) * 0.8;
                (t * freq * std::f32::consts::TAU).sin() * envelope * volume * 0.3
            }).collect()
        }
        SoundEffect::MenuSelect => {
            let duration = 0.05;
            let num_samples = (sample_rate * duration) as usize;
            (0..num_samples).map(|i| {
                let t = i as f32 / sample_rate;
                let envelope = 1.0 - (t / duration);
                (t * 1000.0 * std::f32::consts::TAU).sin() * envelope * volume * 0.2
            }).collect()
        }
    }
}

fn samples_to_sound_data(samples: Vec<f32>, sample_rate: u32) -> Result<StaticSoundData, Box<dyn std::error::Error>> {
    // Build a WAV in memory from raw samples
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let mut buffer = Cursor::new(Vec::new());
    {
        let mut writer = hound::WavWriter::new(&mut buffer, spec)?;
        for sample in &samples {
            writer.write_sample(*sample)?;
        }
        writer.finalize()?;
    }

    let data = StaticSoundData::from_cursor(
        Cursor::new(buffer.into_inner()),
        StaticSoundSettings::default(),
    )?;

    Ok(data)
}

pub fn toggle_sound(audio: &mut SynthAudio) {
    audio.sound_enabled = !audio.sound_enabled;
    info!("Sound toggled: {}", if audio.sound_enabled { "ON" } else { "OFF" });
}
```

**NOTE:** The `hound` crate (added in Step 1) builds WAV data in memory for kira playback. If `kira`'s `StaticSoundData::from_cursor` API differs in the installed version, check kira 0.9 docs — the import paths may be `kira::sound::static_sound::StaticSoundData` or similar. Alternative: use kira's `Sound` trait to implement a custom procedural sound source directly, bypassing WAV encoding.

- [ ] **Step 3: Update `src/systems/collision.rs` to use SoundEvent**

Replace `play_sound(&audio, SoundEffect::...)` calls with event writes.

Remove the `audio: Res<crate::systems::audio::AudioManager>` parameter.
Add `mut sound_events: EventWriter<crate::systems::audio::SoundEvent>` parameter.

Replace all `play_sound(&audio, SoundEffect::X)` with:
```rust
sound_events.write(SoundEvent(SoundEffect::X));
```

- [ ] **Step 4: Update `src/systems/combat.rs` to use SoundEvent**

Same pattern: remove `audio: Res<AudioManager>` params from `boss_shoot_system` and `player_shoot_system`. Add `EventWriter<SoundEvent>`. Replace `play_sound` calls.

- [ ] **Step 5: Update `src/app.rs`**

Remove old AudioManager resource init:
```rust
// DELETE: .init_resource::<crate::systems::audio::AudioManager>()
// DELETE: the setup_audio from Startup systems
```

Add event registration:
```rust
.add_event::<crate::systems::audio::SoundEvent>()
```

Add synth audio setup as a startup system. In Bevy 0.16, functions taking `&mut World` are automatically detected as exclusive systems — no `.exclusive_system()` call needed. **IMPORTANT:** Exclusive systems cannot be placed in a tuple with regular systems — register it in a separate call:
```rust
// Remove setup_audio from the existing Startup tuple
.add_systems(Startup, (setup, setup_menu, spawn_background_stars, setup_shockwave_assets))
// Add synth audio in its own call (exclusive system)
.add_systems(Startup, crate::systems::audio::setup_synth_audio)
```

Add the `play_sounds` system (runs in all states so sounds play during menus/game-over too):
```rust
.add_systems(Update, crate::systems::audio::play_sounds)
```

Update `pause_menu_system` in `src/app.rs` to use the new audio type. Change the parameter:
```rust
// OLD: mut audio: ResMut<crate::systems::audio::AudioManager>,
// NEW:
mut audio: NonSendMut<crate::systems::audio::SynthAudio>,
```
And the toggle call:
```rust
// OLD: toggle_sound(&mut audio);
// NEW:
crate::systems::audio::toggle_sound(&mut audio);
```
`NonSendMut` is in `bevy::prelude` so no extra import needed.

- [ ] **Step 6: Build and verify**

Run: `cargo run`
Expected: Procedural sounds play when shooting, hitting enemies, enemy death explosions, game over, and game won. Sounds are synthesized — no audio files needed.

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml src/systems/audio.rs src/systems/collision.rs src/systems/combat.rs src/app.rs
git commit -m "feat: add procedural synthesized audio via kira"
```

---

## Summary

| Task | Layer | Description |
|------|-------|-------------|
| 1 | Background | Drifting star particles |
| 2 | Background | Green grid via gizmos |
| 3 | Particles | Enemy `is_dead` flag + DeathEvent |
| 4 | Particles | Shatter particles + shockwave ring |
| 5 | Particles | Player afterimage trail |
| 6 | Particles | Player ambient energy particles |
| 7 | Shader | CRT post-processing (scanlines, vignette, distortion) |
| 8 | Audio | Procedural synthesized sound effects |

Each task produces a compilable, runnable game with visible/audible improvements.

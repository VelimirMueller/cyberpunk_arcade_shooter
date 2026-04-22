# WASM Mobile Optimization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the game compile cleanly to WASM with mobile-optimized performance — less stuttering, more room to navigate, iconic bloom preserved, smaller binary.

**Architecture:** Platform detection via compile-time `#[cfg(target_arch = "wasm32")]` constants for entity sizes and particle budgets (avoids threading a Resource through every function). `QualityTier` Bevy Resource for systems that naturally take `Res<>` (camera, CRT shader). Two CRT shader variants via Bevy's `shader_defs` preprocessor.

**Tech Stack:** Bevy 0.16.1, WGSL shaders, Trunk 0.21, wasm-opt

---

## File Map

| File | Action | Responsibility |
|------|--------|---------------|
| `Cargo.toml` | Modify | Feature gating, wasm-release profile |
| `Trunk.toml` | Create | wasm-opt config |
| `src/utils/config.rs` | Rewrite | QualityTier resource + compile-time platform constants |
| `src/app.rs` | Modify | Camera projection, bloom tuning, QualityTier init, entity scale on player spawn |
| `assets/shaders/crt_post_process.wgsl` | Modify | `#ifdef MOBILE` guards around barrel distortion + phosphor bleed |
| `src/systems/post_processing.rs` | Modify | Mobile shader def in pipeline key, mobile CRT settings |
| `src/systems/particles.rs` | Modify | Death particle count, afterimage/ambient intervals from config constants |
| `src/systems/background.rs` | Modify | Star count from config constant |
| `src/core/boss/systems.rs` | Modify | Boss spawn size scaled by ENTITY_SCALE |
| `src/core/boss/attacks.rs` | Modify | Projectile/trail/hazard sizes scaled |
| `src/core/player/systems.rs` | Modify | Player respawn size scaled |
| `src/systems/combat.rs` | Modify | Player projectile size scaled |
| `src/systems/powerups.rs` | Modify | Powerup orb, laser widths scaled |

---

### Task 1: Build Config — Cargo.toml Feature Gating + WASM Profile

**Files:**
- Modify: `Cargo.toml`
- Create: `Trunk.toml`

- [ ] **Step 1: Replace Bevy dependency with feature-gated version**

In `Cargo.toml`, replace the `bevy` dependency line:

```toml
[dependencies]
bevy = { version = "0.16.1", default-features = false, features = [
    "bevy_asset",
    "bevy_audio",
    "bevy_color",
    "bevy_core_pipeline",
    "bevy_gizmos",
    "bevy_log",
    "bevy_render",
    "bevy_sprite",
    "bevy_state",
    "bevy_text",
    "bevy_ui",
    "bevy_window",
    "bevy_winit",
    "default_font",
    "hdr",
    "tonemapping_luts",
    "std",
    "webgl2",
] }
rand = "0.8"
rodio = { version = "0.20", default-features = false, features = ["wav"] }

[target.'cfg(target_arch = "wasm32")'.dependencies]
getrandom = { version = "0.2", features = ["js"] }
web-time = "1.1"

[target.'cfg(not(target_arch = "wasm32"))'.dependencies.bevy]
version = "0.16.1"
features = ["multi_threaded", "x11"]
```

- [ ] **Step 2: Add wasm-release profile**

Append to `Cargo.toml` after the existing `[profile.release]`:

```toml
[profile.wasm-release]
inherits = "release"
opt-level = "z"
lto = "fat"
codegen-units = 1
strip = true
panic = "abort"
```

- [ ] **Step 3: Create Trunk.toml**

```toml
[build]
filehash = true

[tools]
wasm_opt = "version_116"
```

- [ ] **Step 4: Verify native build compiles**

Run: `cargo check 2>&1`
Expected: `Finished` with no errors. There may be warnings about unused features — that's fine, we'll address in later steps.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock Trunk.toml
git commit -m "feat: gate bevy features + add wasm-release profile and Trunk config"
```

---

### Task 2: Platform Config Constants + QualityTier Resource

**Files:**
- Rewrite: `src/utils/config.rs`

- [ ] **Step 1: Write config.rs with QualityTier and platform constants**

```rust
use bevy::prelude::*;

/// Runtime quality tier for Bevy systems that take Res<>.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource)]
pub enum QualityTier {
    Desktop,
    Mobile,
}

impl Default for QualityTier {
    fn default() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            QualityTier::Mobile
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            QualityTier::Desktop
        }
    }
}

// --- Compile-time platform constants ---
// Used by non-system functions (attacks, spawning helpers) to avoid
// threading Res<QualityTier> through every function signature.

#[cfg(target_arch = "wasm32")]
pub const ENTITY_SCALE: f32 = 0.85;
#[cfg(not(target_arch = "wasm32"))]
pub const ENTITY_SCALE: f32 = 1.0;

#[cfg(target_arch = "wasm32")]
pub const STAR_COUNT: usize = 25;
#[cfg(not(target_arch = "wasm32"))]
pub const STAR_COUNT: usize = 40;

#[cfg(target_arch = "wasm32")]
pub const DEATH_PARTICLE_MIN: u32 = 6;
#[cfg(target_arch = "wasm32")]
pub const DEATH_PARTICLE_MAX: u32 = 10;
#[cfg(not(target_arch = "wasm32"))]
pub const DEATH_PARTICLE_MIN: u32 = 12;
#[cfg(not(target_arch = "wasm32"))]
pub const DEATH_PARTICLE_MAX: u32 = 20;

#[cfg(target_arch = "wasm32")]
pub const AFTERIMAGE_INTERVAL: f32 = 0.10;
#[cfg(not(target_arch = "wasm32"))]
pub const AFTERIMAGE_INTERVAL: f32 = 0.05;

#[cfg(target_arch = "wasm32")]
pub const AMBIENT_PARTICLE_INTERVAL: f32 = 0.8;
#[cfg(not(target_arch = "wasm32"))]
pub const AMBIENT_PARTICLE_INTERVAL: f32 = 0.4;
```

- [ ] **Step 2: Re-register QualityTier resource in app.rs**

In `src/app.rs`, add back the import and resource init that was removed earlier:

Add import: `use crate::utils::config::QualityTier;`

Add `.init_resource::<QualityTier>()` back after `.init_state::<GameState>()`.

- [ ] **Step 3: Verify compilation**

Run: `cargo check 2>&1`
Expected: `Finished` with no errors.

- [ ] **Step 4: Commit**

```bash
git add src/utils/config.rs src/app.rs
git commit -m "feat: add QualityTier resource and platform config constants"
```

---

### Task 3: Camera Zoom + Bloom Tuning

**Files:**
- Modify: `src/app.rs` (setup function)

- [ ] **Step 1: Update camera setup with mobile projection and bloom tuning**

Replace the `setup` function in `src/app.rs` with:

```rust
#[allow(dead_code)]
fn setup(
    mut commands: Commands,
    _next_state: ResMut<NextState<GameState>>,
    quality: Res<QualityTier>,
) {
    let bloom = match *quality {
        QualityTier::Desktop => Bloom::default(),
        QualityTier::Mobile => Bloom {
            intensity: 0.2,
            low_frequency_boost: 0.5,
            ..default()
        },
    };

    let crt = match *quality {
        QualityTier::Desktop => CrtSettings::default(),
        QualityTier::Mobile => CrtSettings {
            scanline_intensity: 0.10,
            scanline_count: 150.0,
            vignette_intensity: 0.3,
            vignette_radius: 0.75,
            curvature_amount: 0.0,
        },
    };

    let mut camera = commands.spawn((
        Camera2d,
        Transform::default(),
        GlobalTransform::default(),
        Camera {
            hdr: true,
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        Tonemapping::TonyMcMapface,
        bloom,
        DebandDither::Enabled,
        crt,
    ));

    if *quality == QualityTier::Mobile {
        camera.insert(OrthographicProjection {
            scaling_mode: bevy::render::camera::ScalingMode::AutoMin {
                min_width: 1500.0,
                min_height: 620.0,
            },
            ..OrthographicProjection::default_2d()
        });
    }
}
```

This adds the `QualityTier` param back, tunes bloom (lower intensity on mobile), sets lighter CRT defaults for mobile, and zooms out the camera 25% on mobile via `ScalingMode::AutoMin`.

- [ ] **Step 2: Verify compilation**

Run: `cargo check 2>&1`
Expected: `Finished` with no errors.

- [ ] **Step 3: Commit**

```bash
git add src/app.rs
git commit -m "feat: camera zoom-out and bloom/CRT tuning for mobile"
```

---

### Task 4: CRT Shader — Mobile Variant

**Files:**
- Modify: `assets/shaders/crt_post_process.wgsl`
- Modify: `src/systems/post_processing.rs`

- [ ] **Step 1: Add #ifdef MOBILE guards to shader**

Replace the contents of `assets/shaders/crt_post_process.wgsl`:

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

#ifndef MOBILE
    // Barrel distortion (CRT curvature) — desktop only
    let offset = uv - vec2<f32>(0.5, 0.5);
    uv = uv + offset * dot(offset, offset) * settings.curvature_amount;
#endif

    // Sample the screen texture
    var color = textureSample(screen_texture, texture_sampler, uv);

#ifndef MOBILE
    // Phosphor bleed (horizontal neighbor sampling) — desktop only
    let dims = vec2<f32>(textureDimensions(screen_texture));
    let pixel_x = 1.0 / dims.x;
    let left = textureSample(screen_texture, texture_sampler, uv + vec2<f32>(-pixel_x, 0.0));
    let right = textureSample(screen_texture, texture_sampler, uv + vec2<f32>(pixel_x, 0.0));
    color = color + (left + right) * 0.05;
#endif

    // Scanlines — both platforms
    let scanline = 1.0 - settings.scanline_intensity * (0.5 + 0.5 * sin(uv.y * settings.scanline_count * 3.14159265 * 2.0));
    color = vec4<f32>(color.rgb * scanline, color.a);

    // Vignette — both platforms
    let dist = length((uv - vec2<f32>(0.5, 0.5)) * 2.0);
    let vignette = smoothstep(0.0, settings.vignette_radius, 1.0 - dist);
    let vignette_factor = pow(vignette, settings.vignette_intensity);
    color = vec4<f32>(color.rgb * vignette_factor, color.a);

    return color;
}
```

- [ ] **Step 2: Add `is_mobile` to CrtPipelineKey and inject shader def**

In `src/systems/post_processing.rs`, update the `CrtPipelineKey` struct:

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct CrtPipelineKey {
    texture_format: TextureFormat,
    is_mobile: bool,
}
```

Then update `specialize()` to add the shader def when mobile:

```rust
impl SpecializedRenderPipeline for CrtPipeline {
    type Key = CrtPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let mut shader_defs = vec![];
        if key.is_mobile {
            shader_defs.push("MOBILE".into());
        }

        RenderPipelineDescriptor {
            label: Some("crt_post_process_pipeline".into()),
            layout: vec![self.bind_group_layout.clone()],
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader: CRT_SHADER_HANDLE,
                shader_defs,
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: key.texture_format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: default(),
            depth_stencil: None,
            multisample: default(),
            push_constant_ranges: vec![],
            zero_initialize_workgroup_memory: false,
        }
    }
}
```

- [ ] **Step 3: Update prepare_crt_pipelines to set is_mobile**

Update `prepare_crt_pipelines` to detect mobile from the CrtSettings curvature amount (curvature_amount == 0.0 means mobile — it's set in the setup function):

```rust
fn prepare_crt_pipelines(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedRenderPipelines<CrtPipeline>>,
    crt_pipeline: Res<CrtPipeline>,
    views: Query<(Entity, &ExtractedView, &CrtSettings)>,
) {
    for (entity, view, settings) in views.iter() {
        let pipeline_id = pipelines.specialize(
            &pipeline_cache,
            &crt_pipeline,
            CrtPipelineKey {
                texture_format: if view.hdr {
                    ViewTarget::TEXTURE_FORMAT_HDR
                } else {
                    TextureFormat::bevy_default()
                },
                is_mobile: settings.curvature_amount == 0.0,
            },
        );
        commands.entity(entity).insert(CrtPipelineId(pipeline_id));
    }
}
```

- [ ] **Step 4: Verify compilation**

Run: `cargo check 2>&1`
Expected: `Finished` with no errors.

- [ ] **Step 5: Commit**

```bash
git add assets/shaders/crt_post_process.wgsl src/systems/post_processing.rs
git commit -m "feat: mobile CRT shader variant — drop barrel distortion and phosphor bleed"
```

---

### Task 5: Entity Size Scaling

**Files:**
- Modify: `src/app.rs` (player spawn in menu_input_system)
- Modify: `src/core/boss/systems.rs` (spawn_boss)
- Modify: `src/core/boss/attacks.rs` (projectile/trail/hazard sizes)
- Modify: `src/core/player/systems.rs` (spawn_player)
- Modify: `src/systems/combat.rs` (player projectile size)
- Modify: `src/systems/powerups.rs` (powerup orb, laser widths)

- [ ] **Step 1: Scale player spawn in app.rs**

In `src/app.rs`, add import `use crate::utils::config::ENTITY_SCALE;` and update the player spawn in `menu_input_system`:

Change:
```rust
custom_size: Some(Vec2::new(50.0, 50.0)),
```
To:
```rust
custom_size: Some(Vec2::new(50.0 * ENTITY_SCALE, 50.0 * ENTITY_SCALE)),
```

- [ ] **Step 2: Scale player spawn in player/systems.rs**

In `src/core/player/systems.rs`, add import `use crate::utils::config::ENTITY_SCALE;` and update `spawn_player`:

Change:
```rust
custom_size: Some(Vec2::new(50.0, 50.0)),
```
To:
```rust
custom_size: Some(Vec2::new(50.0 * ENTITY_SCALE, 50.0 * ENTITY_SCALE)),
```

- [ ] **Step 3: Scale boss spawn in boss/systems.rs**

In `src/core/boss/systems.rs`, add import `use crate::utils::config::ENTITY_SCALE;` and update `spawn_boss`:

Change:
```rust
let base_size = 50.0;
let size = base_size * size_mult;
```
To:
```rust
let base_size = 50.0 * ENTITY_SCALE;
let size = base_size * size_mult;
```

- [ ] **Step 4: Scale projectiles in boss/attacks.rs**

In `src/core/boss/attacks.rs`, add import `use crate::utils::config::ENTITY_SCALE;`.

Scale these sizes (use find-and-replace carefully — only gameplay-affecting entities):

| Original | Scaled | Context |
|----------|--------|---------|
| `Vec2::new(20.0, 20.0)` in DashTrail spawns | `Vec2::new(20.0 * ENTITY_SCALE, 20.0 * ENTITY_SCALE)` | Dash trail hitboxes |
| `Vec2::new(8.0, 8.0)` in BossProjectile spawn | `Vec2::new(8.0 * ENTITY_SCALE, 8.0 * ENTITY_SCALE)` | Homing projectiles |
| `Vec2::new(4.0, 4.0)` in sentinel beam segments | `Vec2::new(4.0 * ENTITY_SCALE, 4.0 * ENTITY_SCALE)` | Beam segments |
| `Vec2::new(80.0, 80.0)` in berserker shockwave | `Vec2::new(80.0 * ENTITY_SCALE, 80.0 * ENTITY_SCALE)` | Berserker landing |
| `Vec2::new(60.0, 60.0)` in hazard zone | `Vec2::new(60.0 * ENTITY_SCALE, 60.0 * ENTITY_SCALE)` | Weaver hazard zones |

Do NOT scale: telegraph lines (`Vec2::new(2.0, 900.0)`) — these are visual indicators only.

- [ ] **Step 5: Scale player projectile in combat.rs**

In `src/systems/combat.rs`, add import `use crate::utils::config::ENTITY_SCALE;` and update the player projectile spawn:

Change:
```rust
custom_size: Some(Vec2::splat(3.0)),
```
To:
```rust
custom_size: Some(Vec2::splat(3.0 * ENTITY_SCALE)),
```

- [ ] **Step 6: Scale powerup orb and laser widths in powerups.rs**

In `src/systems/powerups.rs`, add import `use crate::utils::config::ENTITY_SCALE;`.

Scale these:
- Powerup orb spawn: `Vec2::new(16.0, 16.0)` → `Vec2::new(16.0 * ENTITY_SCALE, 16.0 * ENTITY_SCALE)`
- Laser beam core: `Vec2::new(6.0, 600.0)` → `Vec2::new(6.0 * ENTITY_SCALE, 600.0)`  (scale width only, length stays full-screen)
- Laser beam shell: `Vec2::new(32.0, 600.0)` → `Vec2::new(32.0 * ENTITY_SCALE, 600.0)`
- Laser muzzle flash: `Vec2::new(40.0, 20.0)` → `Vec2::new(40.0 * ENTITY_SCALE, 20.0 * ENTITY_SCALE)`
- Laser width pulse update (the `sprite.custom_size = Some(Vec2::new(width, 600.0))` line): the `width` variable is computed dynamically — multiply it by `ENTITY_SCALE` where it's assigned.

- [ ] **Step 7: Verify compilation and tests**

Run: `cargo check 2>&1 && cargo test 2>&1`
Expected: All pass. The collision tests use hardcoded positions, not entity sizes, so they should be unaffected.

- [ ] **Step 8: Commit**

```bash
git add src/app.rs src/core/boss/systems.rs src/core/boss/attacks.rs src/core/player/systems.rs src/systems/combat.rs src/systems/powerups.rs
git commit -m "feat: scale entity sizes by 0.85x on mobile for more room to navigate"
```

---

### Task 6: Particle Budget

**Files:**
- Modify: `src/systems/particles.rs`
- Modify: `src/systems/background.rs`

- [ ] **Step 1: Update death particle count in particles.rs**

In `src/systems/particles.rs`, add import:
```rust
use crate::utils::config::{DEATH_PARTICLE_MIN, DEATH_PARTICLE_MAX};
```

Change:
```rust
let particle_count = rng.gen_range(12..=20);
```
To:
```rust
let particle_count = rng.gen_range(DEATH_PARTICLE_MIN..=DEATH_PARTICLE_MAX);
```

- [ ] **Step 2: Update afterimage timer interval**

In `src/systems/particles.rs`, add `AFTERIMAGE_INTERVAL` to the import, then change `AfterimageTimer::default()`:

```rust
impl Default for AfterimageTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(AFTERIMAGE_INTERVAL, TimerMode::Repeating),
        }
    }
}
```

- [ ] **Step 3: Update ambient particle timer interval**

Add `AMBIENT_PARTICLE_INTERVAL` to the import, then change `AmbientParticleTimer::default()`:

```rust
impl Default for AmbientParticleTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(AMBIENT_PARTICLE_INTERVAL, TimerMode::Repeating),
        }
    }
}
```

- [ ] **Step 4: Update star count in background.rs**

In `src/systems/background.rs`, add import:
```rust
use crate::utils::config::STAR_COUNT;
```

Change:
```rust
for _ in 0..40 {
```
To:
```rust
for _ in 0..STAR_COUNT {
```

- [ ] **Step 5: Verify compilation**

Run: `cargo check 2>&1`
Expected: `Finished` with no errors.

- [ ] **Step 6: Commit**

```bash
git add src/systems/particles.rs src/systems/background.rs
git commit -m "feat: reduce particle budgets on mobile — fewer stars, slower afterimages/ambient"
```

---

### Task 7: Verification — Clippy, Tests, Format

**Files:** None (verification only)

- [ ] **Step 1: Run cargo fmt**

Run: `cargo fmt`

- [ ] **Step 2: Run clippy**

Run: `cargo clippy -- -D warnings 2>&1`
Expected: No errors. Fix any warnings.

- [ ] **Step 3: Run tests**

Run: `cargo test 2>&1`
Expected: All 38 tests pass.

- [ ] **Step 4: Verify WASM compilation**

Run: `cargo check --target wasm32-unknown-unknown 2>&1`

If the wasm32 target isn't installed: `rustup target add wasm32-unknown-unknown`

Expected: `Finished` with no errors. This confirms the feature-gated Bevy compiles for WASM.

- [ ] **Step 5: Commit any fmt/clippy fixes**

```bash
git add -A
git commit -m "chore: fmt + clippy fixes"
```

(Skip if no changes.)

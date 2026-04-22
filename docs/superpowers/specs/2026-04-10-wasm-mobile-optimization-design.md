# WASM Mobile Optimization Design

## Goal

Make the game compile cleanly to WASM and run smoothly on mobile devices. Fix stuttering, give players more room to navigate, keep the iconic bloom aesthetic, and shrink the binary.

## 1. QualityTier Resource (Mobile Detection)

Reintroduce `QualityTier` as a platform-aware resource, but this time it gates **performance knobs** rather than disabling entire systems. On WASM it defaults to `Mobile`; on native it defaults to `Desktop`.

```rust
// src/utils/config.rs
#[derive(Resource, Clone, Copy, PartialEq, Eq)]
pub enum QualityTier { Desktop, Mobile }

impl Default for QualityTier {
    fn default() -> Self {
        #[cfg(target_arch = "wasm32")]
        { QualityTier::Mobile }
        #[cfg(not(target_arch = "wasm32"))]
        { QualityTier::Desktop }
    }
}
```

Every section below reads this resource to decide its tuning.

## 2. Camera Zoom (More Room to Navigate)

Currently the camera uses Bevy's default orthographic projection (no scaling). On mobile, spawn the camera with a larger `OrthographicProjection::scaling_mode` to show more of the arena.

**In `setup()`** (src/app.rs):
- Desktop: default projection (no change)
- Mobile: `ScalingMode::AutoMin { min_width: 1500.0, min_height: 620.0 }` — roughly 25% wider/taller view than default

This alone gives ~25% more visible dodge space without touching entity sizes.

## 3. Entity Size Scaling (~15% Shrink on Mobile)

Apply a global scale factor to entity sizes on mobile. Introduce a constant:

```rust
// src/utils/config.rs
impl QualityTier {
    pub fn entity_scale(&self) -> f32 {
        match self {
            QualityTier::Desktop => 1.0,
            QualityTier::Mobile => 0.85,
        }
    }
}
```

**Files to change** (multiply `custom_size` by `entity_scale()`):
- `src/app.rs` — player spawn (50 -> 42.5)
- `src/core/boss/systems.rs` — boss spawn size, explosion size, projectile sizes
- `src/core/boss/attacks.rs` — attack projectile sizes, telegraph widths
- `src/core/player/systems.rs` — player respawn size
- `src/systems/combat.rs` — player projectile size
- `src/systems/powerups.rs` — powerup orb sizes, laser widths

**Not scaled**: barriers (they're walls, keep them full-width), background stars, grid lines, shockwave rings (purely visual, cheap).

Combined with the camera zoom, this gives ~44% more navigable space on mobile.

## 4. CRT Shader — Simplified for Mobile

The CRT shader currently runs 4 effects per pixel: barrel distortion, phosphor bleed, scanlines, vignette. On mobile, drop the two expensive ones.

**Approach**: Use two shader variants controlled by a preprocessor define.

### Shader changes (assets/shaders/crt_post_process.wgsl):
Add `#ifdef MOBILE` guards:
- **Keep on all platforms**: scanlines, vignette (these define the CRT look)
- **Desktop only**: barrel distortion (texture re-sample with offset math), phosphor bleed (2 extra texture samples per pixel)

### CRT settings tuning for mobile:
```
scanline_intensity: 0.10  (down from 0.15 — lighter on small screens)
scanline_count:     150.0 (down from 200.0 — fewer lines, less moiré)
vignette_intensity: 0.3   (down from 0.4 — subtler)
vignette_radius:    0.75  (up from 0.7 — less aggressive darkening)
curvature_amount:   0.0   (disabled — not used in mobile shader path)
```

### Implementation:
Rather than runtime branching in the shader, use Bevy's `shader_defs` to compile two pipeline variants. In `CrtPipeline::specialize()`, when the `CrtPipelineKey` indicates mobile, add `"MOBILE"` to `shader_defs`. The CrtSettings component stores the quality tier so the prepare system can set the key.

## 5. Bloom Tuning

Bloom stays enabled on all platforms (it's the "iconic" look). But reduce its intensity on mobile to ease the HDR pipeline:

```rust
// Mobile bloom settings
Bloom {
    intensity: 0.2,           // default is ~0.3
    low_frequency_boost: 0.5, // default is 0.7
    ..default()
}
```

Desktop keeps `Bloom::default()`.

## 6. Particle Budget

| System | Desktop | Mobile |
|--------|---------|--------|
| Death shatter particles | 12-20 | 6-10 |
| Afterimage spawn interval | 0.05s | 0.10s |
| Ambient particle spawn interval | 0.4s | 0.8s |
| Background stars | 40 | 25 |

Afterimages and ambient particles stay enabled (they contribute to the neon feel) — just spawn half as often.

## 7. Bevy Feature Gating (Cargo.toml)

Currently Bevy pulls in all default features including 3D PBR, GLTF, animation, gamepad, scene serialization, etc. Disable defaults and opt-in to only what's used:

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
```

**Removed** (not used by this 2D game):
- `bevy_animation`, `bevy_scene`, `bevy_gltf`, `bevy_pbr` — 3D/scene features
- `bevy_gilrs` — gamepad support
- `bevy_picking`, `bevy_*_picking_backend` — picking/raycasting
- `bevy_input_focus` — input focus management
- `custom_cursor` — custom OS cursors
- `png`, `ktx2`, `zstd` — image/texture codecs (no image assets)
- `vorbis` — ogg codec (using rodio + wav directly)
- `smaa_luts` — anti-aliasing LUT tables
- `sysinfo_plugin` — system diagnostics
- `android-game-activity`, `android_shared_stdcxx` — Android native
- `multi_threaded` — WASM is single-threaded

This should significantly reduce compile time and binary size.

## 8. WASM Release Profile + Trunk Config

### Cargo.toml — WASM-optimized release profile:

Add a custom `wasm-release` profile that trunk will use via `--cargo-profile`:

```toml
[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
strip = true

[profile.wasm-release]
inherits = "release"
opt-level = "z"        # optimize for size over speed
lto = "fat"            # aggressive cross-crate inlining then DCE
codegen-units = 1
strip = true
panic = "abort"        # no unwinding on WASM — saves binary size
```

### Trunk.toml — wasm-opt + profile config:

New file. Tells trunk to use the custom profile and run wasm-opt:

```toml
[build]
filehash = true

[tools]
wasm_opt = "version_116"
```

Build command becomes: `trunk build --release --cargo-profile wasm-release`

Trunk 0.21 automatically runs wasm-opt on release builds when the tool version is specified. The `--enable-bulk-memory` and `--enable-mutable-globals` flags are already set via `.cargo/config.toml` rustflags.

## 9. Files Changed (Summary)

| File | Change |
|------|--------|
| `Cargo.toml` | Feature gating, wasm-release profile, panic=abort |
| `Trunk.toml` | New file — wasm-opt config |
| `src/utils/config.rs` | Rewrite QualityTier with Desktop/Mobile + entity_scale() |
| `src/app.rs` | Camera setup reads QualityTier for projection, bloom, entity scale |
| `src/systems/post_processing.rs` | Mobile shader def key, CRT settings per quality |
| `assets/shaders/crt_post_process.wgsl` | `#ifdef MOBILE` guards around distortion/bleed |
| `src/systems/particles.rs` | Particle counts + timers read QualityTier |
| `src/systems/background.rs` | Star count reads QualityTier |
| `src/core/boss/systems.rs` | Entity sizes scaled by entity_scale() |
| `src/core/boss/attacks.rs` | Projectile sizes scaled by entity_scale() |
| `src/core/player/systems.rs` | Player respawn size scaled |
| `src/systems/combat.rs` | Player projectile size scaled |
| `src/systems/powerups.rs` | Powerup/laser sizes scaled |

## 10. What's NOT in Scope

- Touch controls / virtual joystick (separate feature, not requested)
- Sound optimization beyond the existing PCM fix
- New visual effects or gameplay changes
- Native mobile builds (iOS/Android) — this is WASM-in-mobile-browser only

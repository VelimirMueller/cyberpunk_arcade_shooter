<picture>
  <source media="(prefers-color-scheme: light)" srcset="assets/banner/hero-v2-light.svg">
  <img alt="LAB ARCADE SHOOTER. Neon in, zero GC out. Rust, Bevy, ECS, WASM. Working." src="assets/banner/hero-v2-dark.svg" width="100%">
</picture>

<p align="center">

[![status: playable](https://img.shields.io/badge/status-playable-10b981?style=flat-square&labelColor=0a0a0b)](#-05-status) [![VM. lab](https://img.shields.io/badge/VM.-lab-8b5cf6?style=flat-square&labelColor=0a0a0b)](https://github.com/VelimirMueller) [![version: 0.1.0](https://img.shields.io/badge/version-0.1.0-a1a1aa?style=flat-square&labelColor=0a0a0b)](#-05-status) [![stack: rust · bevy · wasm](https://img.shields.io/badge/rust_bevy_wasm-a1a1aa?style=flat-square&labelColor=0a0a0b)](https://www.rust-lang.org/)

</p>

> Neon in, zero GC out.

```text
██       ████   █████           ████   █████    █████   ████   █████   ██████
██      ██  ██  ██  ██         ██  ██  ██  ██  ██      ██  ██  ██  ██  ██
██      ██████  █████   █████  ██████  █████   ██      ██████  ██  ██  █████
██      ██  ██  ██  ██         ██  ██  ██ ██   ██      ██  ██  ██  ██  ██
██████  ██  ██  █████          ██  ██  ██  ██   █████  ██  ██  █████   ██████
 █████  ██  ██   ████    ████   ██████  ██████  █████
██      ██  ██  ██  ██  ██  ██    ██    ██      ██  ██
 ████   ██████  ██  ██  ██  ██    ██    █████   █████
    ██  ██  ██  ██  ██  ██  ██    ██    ██      ██ ██
█████   ██  ██   ████    ████     ██    ██████  ██  ██
```

**lab-arcade-shooter** is a neon geometry shooter in [Rust](https://www.rust-lang.org/) (2024 edition) and [Bevy](https://bevyengine.org/) 0.16.
The shapes pulse. You shoot back.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="assets/readme/stats-v2-dark.svg">
  <img alt="0 TEXTURE FILES. 0 AUDIO FILES. 1 WGSL CRT SHADER. 6253 LINES OF RUST" src="assets/readme/stats-v2-light.svg" width="100%">
</picture>

<br>

## // 01 WHAT IT DOES

<img alt="01 WHAT IT DOES. GEOMETRY TRIES TO KILL YOU." src="assets/readme/divider-what-v2.svg" width="100%">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="assets/readme/features-v2-dark.svg">
  <img alt="NEON ARENA: A bloom-soaked geometry shooter. Multi-stage boss fights, bullet hells, arcade pace. CRT LOOK: One hand-written WGSL shader: scanlines, vignette, barrel distortion. A quality tier per platform. RUNS ANYWHERE: Native on Vulkan, Metal or DirectX 12. In the browser via WASM. Keyboard only" src="assets/readme/features-v2-light.svg" width="100%">
</picture>

- Fast arcade shooter: multi-stage boss fights and bullet hells in a neon arena.
- CRT look from one hand-written WGSL shader: scanlines, vignette, barrel distortion.
- Runs native on Windows, macOS and Linux, and in the browser via WebAssembly.
- Sprites are shapes. Audio is synthesized. No asset files ship with the game.
- Open source under the MIT License. [CI](https://github.com/VelimirMueller/lab-arcade-shooter/actions/workflows/ci.yml) checks format, lints and tests on every push and pull request to `main`.

<br>

## // 02 QUICK START

<img alt="02 QUICK START. COPY. PASTE. SHOOT." src="assets/readme/divider-start-v2.svg" width="100%">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="assets/readme/start-v2-dark.svg">
  <img alt="Terminal: $ git clone https://github.com/VelimirMueller/lab-arcade-shooter.git | $ cd lab-arcade-shooter | $ cargo run --release | # browser build: trunk build --release" src="assets/readme/start-v2-light.svg" width="100%">
</picture>

You need [Rust](https://www.rust-lang.org/tools/install) (latest stable, 2024 edition) and a GPU with Vulkan, Metal or DirectX 12 support.

```bash
git clone https://github.com/VelimirMueller/lab-arcade-shooter.git
cd lab-arcade-shooter
cargo run --release
```

### In the browser (WASM)

The web build uses [Trunk](https://trunkrs.dev/):

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
trunk build --release   # output lands in dist/
trunk serve --release   # local dev server
```

- The browser picks a lighter quality tier: less bloom, flat CRT.
- Keyboard required. There are no touch controls yet.
- See [WASM_BUILD.md](WASM_BUILD.md) for the browser-specific fixes and pitfalls.

<br>

## // 03 HOW IT WORKS

<img alt="03 HOW IT WORKS. SMALL SYSTEMS. ONE LOOP." src="assets/readme/divider-how-v2.svg" width="100%">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="assets/readme/flow-v2-dark.svg">
  <img alt="INPUT -> BEVY ECS -> EVENTS -> RENDER. Sprites are shapes. Audio is synthesized. Nothing to load." src="assets/readme/flow-v2-light.svg" width="100%">
</picture>

```text
 keyboard (WASD · Space · Esc)
    │
    ▼
 ┌──────────────────┐   GameState: Menu ▸ RoundAnnounce ▸ RoundActive
 │  Bevy App (ECS)  │──▸ player · boss phases · collision · powerups
 │  small systems,  │──▸ particles · screen shake · HUD
 │  gated on state  │
 └────────┬─────────┘   events: SoundEvent · DeathEvent
          ▼
 ┌────────────────┐     ┌──────────────────────────┐
 │ audio (synth)  │     │ render: shapes + bloom   │
 └────────────────┘     │ ▸ CRT WGSL post-process  │
                        │   quality tier per plat. │
                        └────────────┬─────────────┘
                                     ▼
                       native window · browser canvas
```

- Small systems, gated on `GameState`. One system does one thing.
- Systems talk through events (`SoundEvent`, `DeathEvent`). Features stay independent.
- The renderer draws shapes with bloom. The CRT shader runs last, at a quality tier per platform.
- The same code builds native and WASM. Only the quality tier changes.

<br>

## // 04 USAGE

<img alt="04 USAGE. BUTTONS AND LESSONS." src="assets/readme/divider-usage-v2.svg" width="100%">

### Controls

- Start game: `Enter`
- Move: `W` / `A` / `S` / `D`
- Shoot: `Space`
- Pause / resume: `Esc`
- Toggle sound (paused): `M`
- Quit to menu (paused): `Q`

### Platform notes

- **Windows** — works out of the box.
- **macOS** — requires Metal support and a recent OS version.
- **Linux** — install `libasound2-dev`, `libudev-dev`, and Vulkan drivers.

### Project structure

```text
lab-arcade-shooter/
├── src/               # game source code
├── assets/shaders/    # WGSL CRT post-process shader
├── tests/             # headless tests (boss, laser, round)
├── docs/superpowers/  # design specs and plans per feature
├── .github/           # CI workflows
├── Cargo.toml         # Rust dependencies and metadata
└── index.html         # web entrypoint for WASM builds
```

- Sprites are shapes. Audio is synthesized. The shader is the only asset.

### What I learned

- **ECS over one big loop** — small systems gated on game states and connected by events keep features independent.
- **Custom post-processing** — a hand-written WGSL shader (scanlines, vignette, barrel distortion) with quality tiers per platform.
- **WASM is not free** — `Instant::now()` panics in the browser, WebGPU is not ready everywhere, and audio needs the right decoder. Details in [WASM_BUILD.md](WASM_BUILD.md).
- **Headless game tests** — a `MinimalPlugins` app can run real boss and laser systems in `cargo test`, without a window.

### Contributing

Issues and pull requests are welcome. Fork, create a feature branch, and make sure CI passes:

```bash
cargo fmt --check && cargo clippy -- -D warnings && cargo test
```

<br>

## // 05 STATUS

<img alt="05 STATUS. HONEST NUMBERS ONLY." src="assets/readme/divider-status-v2.svg" width="100%">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="assets/readme/status-v2-dark.svg">
  <img alt="native build: playable, v0.1.0. browser build: wasm, webgl2, lighter tier. headless tests: boss, laser, round transition. touch controls: keyboard only. webgpu backend: cargo feature, off by default" src="assets/readme/status-v2-light.svg" width="100%">
</picture>

```text
[ STATE  ]  playable, v0.1.0
[ CHECKS ]  fmt + clippy + test on every push
[ SOUND  ]  synthesized; M toggles it when paused
```

The exact CI command, with the dead-code allowance the tree currently needs:

```bash
cargo fmt --check && cargo clippy -- -D warnings -A dead_code && cargo test
```

- [CI workflow](https://github.com/VelimirMueller/lab-arcade-shooter/actions/workflows/ci.yml): fmt, clippy, tests on Ubuntu.
- No touch controls yet. The web build needs a keyboard.

<br>

```text
-- EOF --------------------------------------------- INSERT COIN TO CLOSE --
```

---

<sub>VM. studio / lab · open source · look per <code>vm-brand</code> playbook · [MIT](LICENSE) © 2026 Velimir Mueller</sub>

<p>
<picture>
  <source media="(prefers-color-scheme: light)" srcset="assets/banner/hero-v2-light.svg">
  <img alt="LAB ARCADE SHOOTER. Neon in, zero GC out. Rust, Bevy, ECS, WASM. Working." src="assets/banner/hero-v2-dark.svg" width="100%">
</picture>
</p>

<p>
<picture>
  <source media="(prefers-color-scheme: light)" srcset="assets/readme/lead-v2-light.svg">
  <img alt="lab-arcade-shooter is a neon geometry shooter in Rust (https://www.rust-lang.org/) (2024 edition) and Bevy (https://bevyengine.org/) 0.16. The shapes pulse. You shoot back." src="assets/readme/lead-v2-dark.svg" width="100%">
</picture>
</p>

<p>
<picture>
  <source media="(prefers-color-scheme: light)" srcset="assets/readme/stats-v2-light.svg">
  <img alt="0 TEXTURE FILES. 0 AUDIO FILES. 1 WGSL CRT SHADER. 6253 LINES OF RUST" src="assets/readme/stats-v2-dark.svg" width="100%">
</picture>
</p>

<br>

<p>
<img alt="01 WHAT IT DOES. GEOMETRY TRIES TO KILL YOU." src="assets/readme/divider-what-v2.svg" width="100%">
</p>

<p>
<picture>
  <source media="(prefers-color-scheme: light)" srcset="assets/readme/features-v2-light.svg">
  <img alt="NEON ARENA: A bloom-soaked geometry shooter. Multi-stage boss fights, bullet hells, arcade pace. CRT LOOK: One hand-written WGSL shader: scanlines, vignette, barrel distortion. A quality tier per platform. RUNS ANYWHERE: Native on Vulkan, Metal or DirectX 12. In the browser via WASM. Keyboard only" src="assets/readme/features-v2-dark.svg" width="100%">
</picture>
</p>

<p>
<picture>
  <source media="(prefers-color-scheme: light)" srcset="assets/readme/what-v2-light.svg">
  <img alt="Fast arcade shooter: Multi-stage boss fights and bullet hells in a neon arena. CRT look: One hand-written WGSL shader: scanlines, vignette, barrel distortion. Native and browser: Runs native on Windows, macOS and Linux, and in the browser via WebAssembly. No asset files: Sprites are shapes. Audio is synthesized. No asset files ship with the game. Open source, MIT: Open source under the MIT License. CI on every push: CI (https://github.com/VelimirMueller/lab-arcade-shooter/actions/workflows/ci.yml) checks format, lints and tests on every push and pull request to main" src="assets/readme/what-v2-dark.svg" width="100%">
</picture>
</p>

<br>

<p>
<img alt="02 QUICK START. COPY. PASTE. SHOOT." src="assets/readme/divider-start-v2.svg" width="100%">
</p>

<p>
<picture>
  <source media="(prefers-color-scheme: light)" srcset="assets/readme/needs-v2-light.svg">
  <img alt="Rust: Latest stable, 2024 edition. Install it from rust-lang.org (https://www.rust-lang.org/tools/install). A GPU: With Vulkan, Metal or DirectX 12 support" src="assets/readme/needs-v2-dark.svg" width="100%">
</picture>
</p>

<p>
<picture>
  <source media="(prefers-color-scheme: light)" srcset="assets/readme/start-v2-light.svg">
  <img alt="Terminal: $ git clone https://github.com/VelimirMueller/lab-arcade-shooter.git | $ cd lab-arcade-shooter | $ cargo run --release | # browser build: trunk build --release" src="assets/readme/start-v2-dark.svg" width="100%">
</picture>
</p>

<p>
<picture>
  <source media="(prefers-color-scheme: light)" srcset="assets/readme/wasm-v2-light.svg">
  <img alt="Terminal: $ rustup target add wasm32-unknown-unknown | $ cargo install trunk | $ trunk build --release   # output lands in dist/ | $ trunk serve --release   # local dev server" src="assets/readme/wasm-v2-dark.svg" width="100%">
</picture>
</p>

<details>
<summary><picture><source media="(prefers-color-scheme: light)" srcset="assets/readme/links-7-v2-light.svg"><img alt="Plain text" src="assets/readme/links-7-v2-dark.svg" width="32%"></picture></summary>

```bash
git clone https://github.com/VelimirMueller/lab-arcade-shooter.git
cd lab-arcade-shooter
cargo run --release
```

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
trunk build --release   # output lands in dist/
trunk serve --release   # local dev server
```

</details>

<p>
<picture>
  <source media="(prefers-color-scheme: light)" srcset="assets/readme/web-v2-light.svg">
  <img alt="Trunk: The web build uses Trunk (https://trunkrs.dev/). Lighter quality tier: The browser picks a lighter quality tier: less bloom, flat CRT. Keyboard required: There are no touch controls yet. Fixes and pitfalls: WASM_BUILD.md has the browser-specific fixes and pitfalls" src="assets/readme/web-v2-dark.svg" width="100%">
</picture>
</p>

<p align="center">
<a href="https://www.rust-lang.org/tools/install"><picture><source media="(prefers-color-scheme: light)" srcset="assets/readme/links-1-v2-light.svg"><img alt="Install Rust (https://www.rust-lang.org/tools/install)" src="assets/readme/links-1-v2-dark.svg" width="32%"></picture></a>
<a href="https://trunkrs.dev/"><picture><source media="(prefers-color-scheme: light)" srcset="assets/readme/links-2-v2-light.svg"><img alt="Trunk (https://trunkrs.dev/)" src="assets/readme/links-2-v2-dark.svg" width="32%"></picture></a>
<a href="WASM_BUILD.md"><picture><source media="(prefers-color-scheme: light)" srcset="assets/readme/links-3-v2-light.svg"><img alt="WASM build notes (WASM_BUILD.md)" src="assets/readme/links-3-v2-dark.svg" width="32%"></picture></a>
</p>

<br>

<p>
<img alt="03 HOW IT WORKS. SMALL SYSTEMS. ONE LOOP." src="assets/readme/divider-how-v2.svg" width="100%">
</p>

<p>
<picture>
  <source media="(prefers-color-scheme: light)" srcset="assets/readme/flow-v2-light.svg">
  <img alt="INPUT -&gt; BEVY ECS -&gt; EVENTS -&gt; RENDER. Sprites are shapes. Audio is synthesized. Nothing to load." src="assets/readme/flow-v2-dark.svg" width="100%">
</picture>
</p>

<p>
<picture>
  <source media="(prefers-color-scheme: light)" srcset="assets/readme/systems-v2-light.svg">
  <img alt="GameState: Menu &gt; RoundAnnounce &gt; RoundActive. Every system is gated on it. Gameplay systems: Player, boss phases, collision, powerups. Feedback systems: Particles, screen shake, HUD. Audio: Synthesized. Driven by SoundEvent. Render: Shapes + bloom, then the CRT WGSL post-process at a quality tier per platform. Output: A native window or a browser canvas" src="assets/readme/systems-v2-dark.svg" width="100%">
</picture>
</p>

<p>
<picture>
  <source media="(prefers-color-scheme: light)" srcset="assets/readme/how-v2-light.svg">
  <img alt="Small systems: Small systems, gated on GameState. One system does one thing. Events: Systems talk through events (SoundEvent, DeathEvent). Features stay independent. Render order: The renderer draws shapes with bloom. The CRT shader runs last, at a quality tier per platform. One codebase: The same code builds native and WASM. Only the quality tier changes" src="assets/readme/how-v2-dark.svg" width="100%">
</picture>
</p>

<br>

<p>
<img alt="04 USAGE. BUTTONS AND LESSONS." src="assets/readme/divider-usage-v2.svg" width="100%">
</p>

<p>
<picture>
  <source media="(prefers-color-scheme: light)" srcset="assets/readme/controls-v2-light.svg">
  <img alt="Enter: Start game. W / A / S / D: Move. Space: Shoot. Esc: Pause / resume. M: Toggle sound (paused). Q: Quit to menu (paused)" src="assets/readme/controls-v2-dark.svg" width="100%">
</picture>
</p>

<p>
<picture>
  <source media="(prefers-color-scheme: light)" srcset="assets/readme/platforms-v2-light.svg">
  <img alt="Windows: Works out of the box. macOS: Requires Metal support and a recent OS version. Linux: Install libasound2-dev, libudev-dev, and Vulkan drivers" src="assets/readme/platforms-v2-dark.svg" width="100%">
</picture>
</p>

<p>
<picture>
  <source media="(prefers-color-scheme: light)" srcset="assets/readme/tree-v2-light.svg">
  <img alt="src/: Game source code. assets/shaders/: WGSL CRT post-process shader. tests/: Headless tests (boss, laser, round). docs/superpowers/: Design specs and plans per feature. .github/: CI workflows. Cargo.toml: Rust dependencies and metadata. index.html: Web entrypoint for WASM builds" src="assets/readme/tree-v2-dark.svg" width="100%">
</picture>
</p>

<p>
<picture>
  <source media="(prefers-color-scheme: light)" srcset="assets/readme/learned-v2-light.svg">
  <img alt="ECS over one big loop: Small systems gated on game states and connected by events keep features independent. Custom post-processing: A hand-written WGSL shader (scanlines, vignette, barrel distortion) with quality tiers per platform. WASM is not free: Instant::now() panics in the browser, WebGPU is not ready everywhere, and audio needs the right decoder. Details in WASM_BUILD.md. Headless game tests: A MinimalPlugins app can run real boss and laser systems in cargo test, without a window" src="assets/readme/learned-v2-dark.svg" width="100%">
</picture>
</p>

<p>
<picture>
  <source media="(prefers-color-scheme: light)" srcset="assets/readme/contrib-v2-light.svg">
  <img alt="Terminal: # Issues and pull requests are welcome. | # Fork, create a feature branch, and make sure CI passes: | $ cargo fmt --check &amp;&amp; cargo clippy -- -D warnings &amp;&amp; cargo test" src="assets/readme/contrib-v2-dark.svg" width="100%">
</picture>
</p>

<br>

<p>
<img alt="05 STATUS. HONEST NUMBERS ONLY." src="assets/readme/divider-status-v2.svg" width="100%">
</p>

<p>
<picture>
  <source media="(prefers-color-scheme: light)" srcset="assets/readme/status-v2-light.svg">
  <img alt="native build: playable, v0.1.0. browser build: wasm, webgl2, lighter tier. headless tests: boss, laser, round transition. touch controls: keyboard only. webgpu backend: cargo feature, off by default" src="assets/readme/status-v2-dark.svg" width="100%">
</picture>
</p>

<p>
<picture>
  <source media="(prefers-color-scheme: light)" srcset="assets/readme/state-v2-light.svg">
  <img alt="STATE: Playable, v0.1.0. CHECKS: fmt + clippy + test on every push. SOUND: Synthesized. M toggles it when paused" src="assets/readme/state-v2-dark.svg" width="100%">
</picture>
</p>

<p>
<picture>
  <source media="(prefers-color-scheme: light)" srcset="assets/readme/ci-v2-light.svg">
  <img alt="Terminal: # with the dead-code allowance the tree currently needs | $ cargo fmt --check &amp;&amp; cargo clippy -- -D warnings -A dead_code &amp;&amp; cargo test" src="assets/readme/ci-v2-dark.svg" width="100%">
</picture>
</p>

<details>
<summary><picture><source media="(prefers-color-scheme: light)" srcset="assets/readme/links-7-v2-light.svg"><img alt="Plain text" src="assets/readme/links-7-v2-dark.svg" width="32%"></picture></summary>

```bash
cargo fmt --check && cargo clippy -- -D warnings && cargo test                # contributing
```

```bash
cargo fmt --check && cargo clippy -- -D warnings -A dead_code && cargo test   # exact CI command
```

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

</details>

<p>
<picture>
  <source media="(prefers-color-scheme: light)" srcset="assets/readme/cinotes-v2-light.svg">
  <img alt="CI workflow: fmt, clippy, tests on Ubuntu. No touch controls yet: The web build needs a keyboard" src="assets/readme/cinotes-v2-dark.svg" width="100%">
</picture>
</p>

<p align="center">
<a href="https://github.com/VelimirMueller/lab-arcade-shooter/actions/workflows/ci.yml"><picture><source media="(prefers-color-scheme: light)" srcset="assets/readme/links-4-v2-light.svg"><img alt="CI workflow (https://github.com/VelimirMueller/lab-arcade-shooter/actions/workflows/ci.yml)" src="assets/readme/links-4-v2-dark.svg" width="32%"></picture></a>
<a href="LICENSE"><picture><source media="(prefers-color-scheme: light)" srcset="assets/readme/links-5-v2-light.svg"><img alt="License (LICENSE)" src="assets/readme/links-5-v2-dark.svg" width="32%"></picture></a>
<a href="https://github.com/VelimirMueller"><picture><source media="(prefers-color-scheme: light)" srcset="assets/readme/links-6-v2-light.svg"><img alt="GitHub (https://github.com/VelimirMueller)" src="assets/readme/links-6-v2-dark.svg" width="32%"></picture></a>
</p>

<p>
<picture>
  <source media="(prefers-color-scheme: light)" srcset="assets/readme/footer-v2-light.svg">
  <img alt="VM. studio / lab · open source · look per vm-brand playbook · MIT © 2026 Velimir Mueller · -- EOF -- insert coin to close --" src="assets/readme/footer-v2-dark.svg" width="100%">
</picture>
</p>

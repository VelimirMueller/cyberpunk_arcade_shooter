use bevy::prelude::*;

/// Placeholder sound system for cyberpunk game
/// In a full implementation, you would load actual audio files
/// This version provides hooks for adding sounds later

#[derive(Resource)]
pub struct AudioManager {
    pub sound_enabled: bool,
    pub volume: f32,
}

impl Default for AudioManager {
    fn default() -> Self {
        Self {
            sound_enabled: true,
            volume: 0.7,
        }
    }
}

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

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AudioManager>()
            .add_systems(Startup, setup_audio);
    }
}

/// Play a sound effect (placeholder implementation)
pub fn play_sound(audio: &AudioManager, effect: SoundEffect) {
    if !audio.sound_enabled {
        return;
    }

    // TODO: Load and play actual sound files
    // For now, we'll just log which sound would play
    match effect {
        SoundEffect::PlayerShoot => {
            info!("🔊 Sound: PlayerShoot (pew_pew.wav)");
        }
        SoundEffect::PlayerHit => {
            info!("🔊 Sound: PlayerHit (damage.wav)");
        }
        SoundEffect::EnemyShoot => {
            info!("🔊 Sound: EnemyShoot (laser.wav)");
        }
        SoundEffect::EnemyHit => {
            info!("🔊 Sound: EnemyHit (hit.wav)");
        }
        SoundEffect::Explosion => {
            info!("🔊 Sound: Explosion (boom.wav)");
        }
        SoundEffect::GameOver => {
            info!("🔊 Sound: GameOver (game_over.wav)");
        }
        SoundEffect::GameWon => {
            info!("🔊 Sound: GameWon (victory.wav)");
        }
        SoundEffect::MenuSelect => {
            info!("🔊 Sound: MenuSelect (select.wav)");
        }
    }
}

/// Set up the audio system
pub fn setup_audio(mut commands: Commands) {
    commands.init_resource::<AudioManager>();
    info!("🔊 Audio system initialized (placeholder - no actual sounds loaded)");
}

/// Toggle sound on/off
pub fn toggle_sound(audio: &mut AudioManager) {
    audio.sound_enabled = !audio.sound_enabled;
    info!("🔊 Sound toggled: {}", if audio.sound_enabled { "ON" } else { "OFF" });
}

/// Example system integration - call this from collision.rs
pub fn on_player_hit(audio: Res<AudioManager>) {
    play_sound(&audio, SoundEffect::PlayerHit);
}

pub fn on_player_shoot(audio: Res<AudioManager>) {
    play_sound(&audio, SoundEffect::PlayerShoot);
}

pub fn on_enemy_hit(audio: Res<AudioManager>) {
    play_sound(&audio, SoundEffect::EnemyHit);
}

pub fn on_enemy_shoot(audio: Res<AudioManager>) {
    play_sound(&audio, SoundEffect::EnemyShoot);
}

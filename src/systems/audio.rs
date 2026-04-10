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
    let data_size = (samples.len() * 4) as u32;
    let file_size = 36 + data_size;

    let mut buf = Vec::with_capacity(file_size as usize + 8);

    // RIFF header
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&file_size.to_le_bytes());
    buf.extend_from_slice(b"WAVE");

    // fmt chunk
    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16u32.to_le_bytes());
    buf.extend_from_slice(&3u16.to_le_bytes()); // IEEE float
    buf.extend_from_slice(&1u16.to_le_bytes()); // mono
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

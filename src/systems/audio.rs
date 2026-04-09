use bevy::prelude::*;
use kira::manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend};
use kira::sound::static_sound::StaticSoundData;
use std::io::Cursor;

#[derive(Clone, Copy, Debug, PartialEq)]
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

pub fn play_sounds(mut audio: NonSendMut<SynthAudio>, mut events: EventReader<SoundEvent>) {
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
            // Low rumble + rising tone: 80→200 Hz sweep, 500ms
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
            // White noise burst + 300→100 Hz sweep, 200ms
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
            // Impact + bass drop: 50 Hz thump + noise, 300ms
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
            // Rising whine: 200→800 Hz sweep, 800ms
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
            // Sustained mid-frequency: 400 Hz tone, 500ms
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
            // Accelerating rumble: 100→400 Hz, 800ms
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
            // Bubble/pop: 600→200 Hz, 100ms
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
            // Sharp crack: noise + 200 Hz, 150ms
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
            // Triumphant chord: 400+500+600 Hz, 800ms
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
            // Deep boom: 40→80 Hz sweep + noise, 400ms
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
            // Sustained 300 Hz tone, 500ms
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
            // Ascending sweep: 100→800 Hz over 0.8s
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
            // Short bright impact: 800→200 Hz, 0.15s
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
            // Descending sweep: 400→80 Hz over 0.5s
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

fn samples_to_sound_data(
    samples: Vec<f32>,
    sample_rate: u32,
) -> Result<StaticSoundData, Box<dyn std::error::Error>> {
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

    let data = StaticSoundData::from_cursor(Cursor::new(buffer.into_inner()))?;

    Ok(data)
}

pub fn toggle_sound(audio: &mut SynthAudio) {
    audio.sound_enabled = !audio.sound_enabled;
    info!(
        "Sound toggled: {}",
        if audio.sound_enabled { "ON" } else { "OFF" }
    );
}

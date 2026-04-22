use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerUpKind {
    // Common
    RepairKit,
    EnergyCell,
    PhaseShift,
    GlitchBlink,
    // Rare
    Shockwave,
    // Ultra-rare
    Laser,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerUpTier {
    Common,
    Rare,
    UltraRare,
}

#[derive(Debug)]
pub struct PowerUpMeta {
    pub kind: PowerUpKind,
    pub tier: PowerUpTier,
    pub color: Color,
    pub display_name: &'static str,
}

pub const CATALOG: &[PowerUpMeta] = &[
    PowerUpMeta {
        kind: PowerUpKind::RepairKit,
        tier: PowerUpTier::Common,
        color: Color::srgb(0.0, 8.0, 2.0),
        display_name: "REPAIR",
    },
    PowerUpMeta {
        kind: PowerUpKind::EnergyCell,
        tier: PowerUpTier::Common,
        color: Color::srgb(0.0, 4.0, 8.0),
        display_name: "ENERGY",
    },
    PowerUpMeta {
        kind: PowerUpKind::PhaseShift,
        tier: PowerUpTier::Common,
        color: Color::srgba(6.0, 6.0, 8.0, 0.7),
        display_name: "PHASE",
    },
    PowerUpMeta {
        kind: PowerUpKind::GlitchBlink,
        tier: PowerUpTier::Common,
        color: Color::srgb(6.0, 0.5, 8.0),
        display_name: "BLINK",
    },
    PowerUpMeta {
        kind: PowerUpKind::Shockwave,
        tier: PowerUpTier::Rare,
        color: Color::srgb(0.0, 8.0, 8.0),
        display_name: "SHOCKWAVE",
    },
    PowerUpMeta {
        kind: PowerUpKind::Laser,
        tier: PowerUpTier::UltraRare,
        color: Color::srgb(8.0, 0.0, 8.0),
        display_name: "LASER",
    },
];

pub fn meta(kind: PowerUpKind) -> &'static PowerUpMeta {
    CATALOG
        .iter()
        .find(|m| m.kind == kind)
        .expect("every PowerUpKind must have a CATALOG entry")
}

/// Pure-function tier selector from a 0..=99 roll.
/// 0-54 = Common (55%), 55-89 = Rare (35%), 90-99 = UltraRare (10%).
pub fn tier_from_roll(n: u8) -> PowerUpTier {
    match n {
        0..=54 => PowerUpTier::Common,
        55..=89 => PowerUpTier::Rare,
        _ => PowerUpTier::UltraRare,
    }
}

pub fn kinds_in_tier(tier: PowerUpTier) -> Vec<PowerUpKind> {
    CATALOG
        .iter()
        .filter(|m| m.tier == tier)
        .map(|m| m.kind)
        .collect()
}

pub fn roll_random_kind() -> PowerUpKind {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let tier_roll: u8 = rng.gen_range(0..100);
    let tier = tier_from_roll(tier_roll);
    let kinds = kinds_in_tier(tier);
    kinds[rng.gen_range(0..kinds.len())]
}

impl PowerUpTier {
    pub fn base_size_px(&self) -> f32 {
        match self {
            PowerUpTier::Common => 14.0,
            PowerUpTier::Rare => 18.0,
            PowerUpTier::UltraRare => 22.0,
        }
    }

    pub fn pulse_hz(&self) -> f32 {
        match self {
            PowerUpTier::Common => 4.0,
            PowerUpTier::Rare => 6.0,
            PowerUpTier::UltraRare => 9.0,
        }
    }

    /// (glow_scale_factor, glow_alpha, glow_color).
    /// glow_color is None for Common (no glow).
    pub fn glow(&self) -> Option<(f32, f32, Color)> {
        match self {
            PowerUpTier::Common => None,
            PowerUpTier::Rare => Some((1.5, 0.35, Color::srgba(8.0, 8.0, 8.0, 0.35))),
            PowerUpTier::UltraRare => Some((1.8, 0.45, Color::srgba(8.0, 6.0, 0.0, 0.45))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_has_a_catalog_entry() {
        // Each PowerUpKind variant must appear in CATALOG.
        let kinds = [
            PowerUpKind::RepairKit,
            PowerUpKind::EnergyCell,
            PowerUpKind::PhaseShift,
            PowerUpKind::GlitchBlink,
            PowerUpKind::Shockwave,
            PowerUpKind::Laser,
        ];
        for kind in kinds {
            // meta() panics if missing, which would fail the test
            let _ = meta(kind);
        }
        assert_eq!(CATALOG.len(), kinds.len());
    }

    #[test]
    fn tier_boundaries() {
        assert_eq!(tier_from_roll(0), PowerUpTier::Common);
        assert_eq!(tier_from_roll(54), PowerUpTier::Common);
        assert_eq!(tier_from_roll(55), PowerUpTier::Rare);
        assert_eq!(tier_from_roll(89), PowerUpTier::Rare);
        assert_eq!(tier_from_roll(90), PowerUpTier::UltraRare);
        assert_eq!(tier_from_roll(99), PowerUpTier::UltraRare);
    }

    #[test]
    fn roll_distribution_within_tolerance() {
        // Over 10_000 rolls, expect ~55% Common / ~35% Rare / ~10% Ultra within ±2%.
        let n = 10_000usize;
        let mut common = 0u32;
        let mut rare = 0u32;
        let mut ultra = 0u32;
        for _ in 0..n {
            match meta(roll_random_kind()).tier {
                PowerUpTier::Common => common += 1,
                PowerUpTier::Rare => rare += 1,
                PowerUpTier::UltraRare => ultra += 1,
            }
        }
        let common_pct = common as f32 / n as f32;
        let rare_pct = rare as f32 / n as f32;
        let ultra_pct = ultra as f32 / n as f32;
        assert!(
            (common_pct - 0.55).abs() < 0.02,
            "common_pct = {} (expected ~0.55)",
            common_pct
        );
        assert!(
            (rare_pct - 0.35).abs() < 0.02,
            "rare_pct = {} (expected ~0.35)",
            rare_pct
        );
        assert!(
            (ultra_pct - 0.10).abs() < 0.02,
            "ultra_pct = {} (expected ~0.10)",
            ultra_pct
        );
    }
}

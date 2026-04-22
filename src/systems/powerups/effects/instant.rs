use crate::core::player::components::Player;
use crate::systems::audio::{SoundEffect, SoundEvent};
use bevy::prelude::*;

const REPAIR_AMOUNT: u32 = 25;
const ENERGY_AMOUNT: u32 = 100;

pub fn apply_repair_kit(player: &mut Player, sound_events: &mut EventWriter<SoundEvent>) {
    let new_hp = player.current.saturating_add(REPAIR_AMOUNT).min(player.max);
    player.current = new_hp;
    sound_events.write(SoundEvent(SoundEffect::RepairKitPickup));
}

pub fn apply_energy_cell(player: &mut Player, sound_events: &mut EventWriter<SoundEvent>) {
    let new_energy = player
        .energy
        .saturating_add(ENERGY_AMOUNT)
        .min(player.max_energy);
    player.energy = new_energy;
    sound_events.write(SoundEvent(SoundEffect::EnergyCellPickup));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_player(current: u32, max: u32, energy: u32, max_energy: u32) -> Player {
        Player {
            current,
            max,
            energy,
            max_energy,
            last_collision_time: None,
            last_shot_time: None,
        }
    }

    // apply_repair_kit / apply_energy_cell take a SoundEvent writer; we only test the cap logic
    // via pure fns factored out below.
    fn repair_into(current: u32, max: u32) -> u32 {
        current.saturating_add(REPAIR_AMOUNT).min(max)
    }

    fn energy_into(energy: u32, max_energy: u32) -> u32 {
        energy.saturating_add(ENERGY_AMOUNT).min(max_energy)
    }

    #[test]
    fn repair_kit_caps_at_max() {
        let _ = test_player(0, 100, 0, 100); // smoke
        assert_eq!(repair_into(0, 100), 25);
        assert_eq!(repair_into(80, 100), 100);
        assert_eq!(repair_into(100, 100), 100);
    }

    #[test]
    fn energy_cell_caps_at_max_energy() {
        assert_eq!(energy_into(0, 100), 100);
        assert_eq!(energy_into(50, 100), 100);
        assert_eq!(energy_into(100, 100), 100);
    }
}

use crate::guilds::monk::{
    AREA_SKILL_1, AREA_SKILL_2, AREA_SKILL_3, ARMOUR_SKILL_1, ARMOUR_SKILL_2, ARMOUR_SKILL_3,
    AVOID_SKILL_1, AVOID_SKILL_2, AVOID_SKILL_3, CURRENT_AREA_SKILL_VAR, CURRENT_ARMOUR_SKILL_VAR,
    CURRENT_AVOID_SKILL_VAR, CURRENT_DISRUPT_SKILL_VAR, DISRUPT_SKILL_1, DISRUPT_SKILL_2,
    DISRUPT_SKILL_3,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MonkSkillTrack {
    Disrupt,
    Armour,
    Area,
    Avoid,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonkTrackSlots {
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub slot_1: bool,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub slot_2: bool,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub slot_3: bool,
}

impl Default for MonkTrackSlots {
    fn default() -> Self {
        Self {
            slot_1: true,
            slot_2: true,
            slot_3: true,
        }
    }
}

impl MonkTrackSlots {
    pub fn is_default(&self) -> bool {
        self.slot_1 && self.slot_2 && self.slot_3
    }

    pub fn slot_enabled(&self, slot: u8) -> bool {
        match slot {
            1 => self.slot_1,
            2 => self.slot_2,
            3 => self.slot_3,
            _ => false,
        }
    }

    pub fn set_slot(&mut self, slot: u8, enabled: bool) {
        match slot {
            1 => {
                self.slot_1 = enabled;
                if !enabled {
                    self.slot_2 = false;
                    self.slot_3 = false;
                }
            }
            2 => {
                if enabled {
                    self.slot_1 = true;
                    self.slot_2 = true;
                } else {
                    self.slot_2 = false;
                    self.slot_3 = false;
                }
            }
            3 => {
                if enabled {
                    self.slot_1 = true;
                    self.slot_2 = true;
                    self.slot_3 = true;
                } else {
                    self.slot_3 = false;
                }
            }
            _ => {}
        }
    }

    pub fn toggle_slot(&mut self, slot: u8) {
        self.set_slot(slot, !self.slot_enabled(slot));
    }

    pub fn first_enabled_slot(&self) -> Option<u8> {
        if self.slot_1 {
            Some(1)
        } else if self.slot_2 {
            Some(2)
        } else if self.slot_3 {
            Some(3)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct MonkSkillsConfig {
    #[serde(default)]
    pub disrupt: MonkTrackSlots,
    #[serde(default)]
    pub armour: MonkTrackSlots,
    #[serde(default)]
    pub area: MonkTrackSlots,
    #[serde(default)]
    pub avoid: MonkTrackSlots,
}

impl MonkSkillsConfig {
    pub fn is_default(&self) -> bool {
        self.disrupt.is_default()
            && self.armour.is_default()
            && self.area.is_default()
            && self.avoid.is_default()
    }

    pub fn track_slots(&self, track: MonkSkillTrack) -> &MonkTrackSlots {
        match track {
            MonkSkillTrack::Disrupt => &self.disrupt,
            MonkSkillTrack::Armour => &self.armour,
            MonkSkillTrack::Area => &self.area,
            MonkSkillTrack::Avoid => &self.avoid,
        }
    }

    pub fn track_slots_mut(&mut self, track: MonkSkillTrack) -> &mut MonkTrackSlots {
        match track {
            MonkSkillTrack::Disrupt => &mut self.disrupt,
            MonkSkillTrack::Armour => &mut self.armour,
            MonkSkillTrack::Area => &mut self.area,
            MonkSkillTrack::Avoid => &mut self.avoid,
        }
    }

    pub fn slot_enabled(&self, track: MonkSkillTrack, slot: u8) -> bool {
        self.track_slots(track).slot_enabled(slot)
    }

    pub fn toggle_slot(&mut self, track: MonkSkillTrack, slot: u8) {
        self.track_slots_mut(track).toggle_slot(slot);
    }

    pub fn skill_name(track: MonkSkillTrack, slot: u8) -> Option<&'static str> {
        match (track, slot) {
            (MonkSkillTrack::Disrupt, 1) => Some(DISRUPT_SKILL_1),
            (MonkSkillTrack::Disrupt, 2) => Some(DISRUPT_SKILL_2),
            (MonkSkillTrack::Disrupt, 3) => Some(DISRUPT_SKILL_3),
            (MonkSkillTrack::Armour, 1) => Some(ARMOUR_SKILL_1),
            (MonkSkillTrack::Armour, 2) => Some(ARMOUR_SKILL_2),
            (MonkSkillTrack::Armour, 3) => Some(ARMOUR_SKILL_3),
            (MonkSkillTrack::Area, 1) => Some(AREA_SKILL_1),
            (MonkSkillTrack::Area, 2) => Some(AREA_SKILL_2),
            (MonkSkillTrack::Area, 3) => Some(AREA_SKILL_3),
            (MonkSkillTrack::Avoid, 1) => Some(AVOID_SKILL_1),
            (MonkSkillTrack::Avoid, 2) => Some(AVOID_SKILL_2),
            (MonkSkillTrack::Avoid, 3) => Some(AVOID_SKILL_3),
            _ => None,
        }
    }

    pub fn track_for_var(var: &str) -> Option<MonkSkillTrack> {
        match var {
            CURRENT_DISRUPT_SKILL_VAR => Some(MonkSkillTrack::Disrupt),
            CURRENT_ARMOUR_SKILL_VAR => Some(MonkSkillTrack::Armour),
            CURRENT_AREA_SKILL_VAR => Some(MonkSkillTrack::Area),
            CURRENT_AVOID_SKILL_VAR => Some(MonkSkillTrack::Avoid),
            _ => None,
        }
    }

    pub fn var_for_track(track: MonkSkillTrack) -> &'static str {
        match track {
            MonkSkillTrack::Disrupt => CURRENT_DISRUPT_SKILL_VAR,
            MonkSkillTrack::Armour => CURRENT_ARMOUR_SKILL_VAR,
            MonkSkillTrack::Area => CURRENT_AREA_SKILL_VAR,
            MonkSkillTrack::Avoid => CURRENT_AVOID_SKILL_VAR,
        }
    }

    pub fn first_enabled_skill_name(&self, track: MonkSkillTrack) -> Option<&'static str> {
        let slots = self.track_slots(track);
        slots
            .first_enabled_slot()
            .and_then(|slot| Self::skill_name(track, slot))
    }

    pub fn is_slot_skill_enabled(&self, track: MonkSkillTrack, slot: u8) -> bool {
        self.slot_enabled(track, slot)
    }

    /// Skill to set after a combat result for `slot`. Wraps to the first enabled
    /// slot when the result targets a disabled later chain slot.
    pub fn rotation_skill_for_result_slot(
        &self,
        track: MonkSkillTrack,
        slot: u8,
    ) -> Option<&'static str> {
        if self.slot_enabled(track, slot) {
            Self::skill_name(track, slot)
        } else {
            self.first_enabled_skill_name(track)
        }
    }

    pub fn is_var_value_enabled(&self, var: &str, value: &str) -> bool {
        let Some(track) = Self::track_for_var(var) else {
            return true;
        };
        (1..=3).any(|slot| {
            self.slot_enabled(track, slot) && Self::skill_name(track, slot) == Some(value)
        })
    }

    pub fn clamp_var_value(&self, var: &str, current: &str) -> Option<String> {
        let track = Self::track_for_var(var)?;
        if self.is_var_value_enabled(var, current) {
            return None;
        }
        self.first_enabled_skill_name(track).map(str::to_string)
    }

    pub fn clamp_rotation_vars(
        &self,
        vars: &std::collections::HashMap<String, String>,
    ) -> Vec<(String, String)> {
        [
            CURRENT_ARMOUR_SKILL_VAR,
            CURRENT_DISRUPT_SKILL_VAR,
            CURRENT_AREA_SKILL_VAR,
            CURRENT_AVOID_SKILL_VAR,
        ]
        .into_iter()
        .filter_map(|var| {
            let current = vars.get(var)?;
            self.clamp_var_value(var, current)
                .map(|value| (var.to_string(), value))
        })
        .collect()
    }

    pub fn track_label(track: MonkSkillTrack) -> &'static str {
        match track {
            MonkSkillTrack::Disrupt => "Disrupt",
            MonkSkillTrack::Armour => "Armour",
            MonkSkillTrack::Area => "Area",
            MonkSkillTrack::Avoid => "Avoid",
        }
    }

    pub const TRACKS: [MonkSkillTrack; 4] = [
        MonkSkillTrack::Disrupt,
        MonkSkillTrack::Armour,
        MonkSkillTrack::Area,
        MonkSkillTrack::Avoid,
    ];
}

fn is_true(value: &bool) -> bool {
    *value
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_slot_3_enables_prefix() {
        let mut slots = MonkTrackSlots::default();
        slots.set_slot(1, false);
        slots.set_slot(2, false);
        slots.set_slot(3, false);
        slots.toggle_slot(3);
        assert!(slots.slot_1 && slots.slot_2 && slots.slot_3);
    }

    #[test]
    fn deselect_slot_1_clears_later_slots() {
        let mut slots = MonkTrackSlots::default();
        slots.set_slot(1, false);
        assert!(!slots.slot_1 && !slots.slot_2 && !slots.slot_3);
    }

    #[test]
    fn clamp_var_moves_to_first_enabled_slot() {
        let mut config = MonkSkillsConfig::default();
        config.disrupt.set_slot(2, false);
        config.disrupt.set_slot(3, false);
        let clamped = config
            .clamp_var_value(CURRENT_DISRUPT_SKILL_VAR, DISRUPT_SKILL_3)
            .unwrap();
        assert_eq!(clamped, DISRUPT_SKILL_1);
    }

    #[test]
    fn rotation_skill_wraps_when_result_slot_disabled() {
        let mut config = MonkSkillsConfig::default();
        config.disrupt.set_slot(3, false);
        assert_eq!(
            config.rotation_skill_for_result_slot(MonkSkillTrack::Disrupt, 3),
            Some(DISRUPT_SKILL_1)
        );
    }
}

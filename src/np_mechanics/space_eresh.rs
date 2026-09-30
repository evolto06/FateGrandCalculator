//! Space Ereshkigal's affection multiplier and level-seven defense bypass.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::NpModifiers;

pub const SERVANT_ID: u32 = 3_300_200;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AffectionScaling {
    pub base: f64,
    pub per_level: f64,
    pub max_level: u8,
    pub ignore_defense_at: u8,
}

impl AffectionScaling {
    pub fn multiplier(&self, level: u8) -> f64 {
        self.base + self.per_level * f64::from(level)
    }

    pub fn is_valid(&self) -> bool {
        self.base.is_finite()
            && self.per_level.is_finite()
            && self.base > 0.0
            && self.per_level > 0.0
            && self.max_level == 10
            && (1..=self.max_level).contains(&self.ignore_defense_at)
    }

    pub fn modifiers(&self, level: u8) -> NpModifiers {
        NpModifiers {
            multiplier: self.multiplier(level),
            ignore_defense: level >= self.ignore_defense_at,
        }
    }
}

/// Accept only the known function, servant target, and constant positive scaling.
pub fn import_scaling(damage: &Value, servant_id: u32) -> Option<AffectionScaling> {
    if servant_id != SERVANT_ID || damage["funcType"] != "damageNpBattlePointPhase" {
        return None;
    }
    let values = damage
        .get("svals")?
        .as_array()
        .filter(|values| values.len() == 5)?;
    let scales: Option<Vec<_>> = values
        .iter()
        .map(|value| {
            let base = value.get("Value2")?.as_f64()? / 1000.0;
            let per_level = value.get("Correction")?.as_f64()? / 1000.0;
            let target = value.get("Target")?.as_u64()?;
            (target == u64::from(SERVANT_ID)
                && base.is_finite()
                && per_level.is_finite()
                && base > 0.0
                && per_level > 0.0)
                .then_some((base, per_level))
        })
        .collect();
    let scales = scales.filter(|scales| scales.iter().all(|scale| *scale == scales[0]))?;
    Some(AffectionScaling {
        base: scales[0].0,
        per_level: scales[0].1,
        max_level: 10,
        ignore_defense_at: 7,
    })
}

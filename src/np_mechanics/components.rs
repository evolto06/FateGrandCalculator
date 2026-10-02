//! Ordered damage functions and explicitly known NP-level / Overcharge values.
//!
//! Reference simulator: chaldea-center/chaldea at
//! daf4e0af5ec6b4ae42f3e9410b26ae143aa3d725:
//! `functions/damage.dart` skips Rate 0 and calculates each function's endpoints;
//! `utils/battle_utils.dart:110` floors each function's damage, before aggregation.
//! `functions/function_executor.dart:153-154` delays release between consecutive
//! damage functions, and :1126-1127 retains alive targets OR CheckDead targets.
//! CheckDead therefore permits continuation; it is not a survival requirement.
//! These values estimate damage to one enemy, without simulating HP, retargeting,
//! buff consumption, ally sacrifice, or death. RNG correlation is not modeled;
//! adding separately floored endpoint damages establishes the possible range.

use super::low_hp::{LowHpBreakdown, LowHpScaling};
use crate::model::AttackerHp;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NpTarget {
    Enemy,
    EnemyAll,
}

fn guaranteed_rates() -> [u16; 5] {
    [1000; 5]
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct NpDamageValues {
    pub multipliers: [f64; 5],
    #[serde(default)]
    pub low_hp: Option<LowHpScaling>,
    #[serde(default = "guaranteed_rates")]
    pub rates: [u16; 5],
    #[serde(default)]
    pub check_dead: [bool; 5],
}

impl NpDamageValues {
    pub fn guaranteed(multipliers: [f64; 5]) -> Self {
        Self {
            multipliers,
            low_hp: None,
            rates: guaranteed_rates(),
            check_dead: [false; 5],
        }
    }

    pub fn is_valid(&self) -> bool {
        self.low_hp
            .as_ref()
            .is_none_or(|scaling| scaling.is_valid_for(&self.multipliers, &self.rates))
            && (0..5).all(|level| {
                let value = self.multipliers[level];
                value.is_finite()
                    && value >= 0.0
                    && matches!(self.rates[level], 0 | 1000)
                    && (self.rates[level] != 0 || value == 0.0)
            })
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct NpDamageComponent {
    #[serde(default)]
    pub target: Option<NpTarget>,
    pub overcharge: [Option<NpDamageValues>; 5],
}

impl NpDamageComponent {
    /// Old snapshots establish only OC1; do not invent the other four rows.
    pub fn legacy(multipliers: [f64; 5]) -> Self {
        Self {
            target: None,
            overcharge: [
                Some(NpDamageValues::guaranteed(multipliers)),
                None,
                None,
                None,
                None,
            ],
        }
    }

    pub fn multiplier(&self, np_level: u8, overcharge: u8) -> Option<f64> {
        let level = np_level.checked_sub(1)? as usize;
        let row = self
            .overcharge
            .get(overcharge.checked_sub(1)? as usize)?
            .as_ref()?;
        match *row.rates.get(level)? {
            0 => Some(0.0),
            1000 => row.multipliers.get(level).copied(),
            _ => None,
        }
    }

    pub fn is_valid(&self) -> bool {
        let Some(first) = &self.overcharge[0] else {
            return false;
        };
        self.overcharge
            .iter()
            .flatten()
            .all(|row| row.low_hp.is_some() == first.low_hp.is_some() && row.is_valid())
    }

    pub fn low_hp_breakdown(
        &self,
        np_level: u8,
        overcharge: u8,
        hp: AttackerHp,
    ) -> Option<LowHpBreakdown> {
        let row = self
            .overcharge
            .get(usize::from(overcharge.checked_sub(1)?))?
            .as_ref()?;
        if !row.is_valid() {
            return None;
        }
        row.low_hp.as_ref()?.breakdown(np_level, hp)
    }

    pub fn effective_multiplier(
        &self,
        np_level: u8,
        overcharge: u8,
        hp: Option<AttackerHp>,
    ) -> Option<f64> {
        let row = self
            .overcharge
            .get(usize::from(overcharge.checked_sub(1)?))?
            .as_ref()?;
        if !row.is_valid() {
            return None;
        }
        if row.low_hp.is_some() {
            self.low_hp_breakdown(np_level, overcharge, hp?)
                .map(|result| result.effective_multiplier)
        } else {
            self.multiplier(np_level, overcharge)
        }
    }
}

/// A selectable row must cover every component and have positive total damage
/// at every NP level. Partial coverage must not omit a missing component.
pub fn components_are_valid(components: &[NpDamageComponent]) -> bool {
    let Some(first) = components.first() else {
        return false;
    };
    if components.iter().any(|component| !component.is_valid())
        || (components.len() > 1
            && (first.target.is_none()
                || components
                    .iter()
                    .any(|component| component.target != first.target)))
    {
        return false;
    }
    (0..5).all(|oc| {
        let known = first.overcharge[oc].is_some();
        components
            .iter()
            .all(|component| component.overcharge[oc].is_some() == known)
            && (!known
                || (1..=5).all(|level| {
                    let total: f64 = components
                        .iter()
                        .map(|component| component.multiplier(level, oc as u8 + 1).unwrap())
                        .sum();
                    total.is_finite() && total > 0.0
                }))
    })
}

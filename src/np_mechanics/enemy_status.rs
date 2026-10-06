//! Manually assumed enemy buff conditions at NP damage time.
//!
//! Chaldea `damage.dart:144-156` checks target buff individualities and assigns
//! Correction/1000 as specificAttackRate. `battle_utils.dart:69,104` multiplies
//! this separately from the NP base rate, before flat damage and endpoint floors.
//! This does not execute status application or infer success from an NP effect.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EnemyStatus {
    Poison,
    SkillSeal,
    Bind,
    DefenseUp,
    Charm,
    Curse,
    Burn,
}

impl EnemyStatus {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Poison => "Poison",
            Self::SkillSeal => "Skill Seal",
            Self::Bind => "Bind",
            Self::DefenseUp => "Defense Up",
            Self::Charm => "Charm",
            Self::Curse => "Curse",
            Self::Burn => "Burn",
        }
    }

    /// Verified positive Atlas Target IDs; negated targets are unsupported.
    pub const fn source_target(self) -> u32 {
        match self {
            Self::Poison => 3011,
            Self::SkillSeal => 3025,
            Self::Bind => 3087,
            Self::DefenseUp => 3058,
            Self::Charm => 3012,
            Self::Curse => 3026,
            Self::Burn => 3015,
        }
    }

    pub const fn includes_ignored_individuality(self) -> bool {
        matches!(self, Self::SkillSeal | Self::Bind | Self::Charm)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EnemyStatusScaling {
    pub condition: EnemyStatus,
    /// Atlas Correction in thousandths, per NP level for one OC row.
    pub source_corrections: [u32; 5],
    /// Includes status buffs whose source script sets IgnoreIndividuality=1.
    /// This flag is distinct from whether a buff can be removed.
    pub include_ignore_individuality: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnemyStatusBreakdown {
    pub condition: EnemyStatus,
    pub active: bool,
    pub base_multiplier: f64,
    pub conditional_multiplier: f64,
    pub applied_multiplier: f64,
    pub effective_multiplier: f64,
}

impl EnemyStatusScaling {
    pub fn is_valid(&self) -> bool {
        self.source_corrections
            .iter()
            .all(|correction| *correction > 0)
            && self.include_ignore_individuality == self.condition.includes_ignored_individuality()
    }

    pub fn correction(&self, np_level: u8) -> Option<f64> {
        if !self.is_valid() {
            return None;
        }
        self.source_corrections
            .get(usize::from(np_level.checked_sub(1)?))
            .map(|correction| f64::from(*correction) / 1000.0)
    }

    pub fn breakdown(
        &self,
        np_level: u8,
        base_multiplier: f64,
        assumption: Option<EnemyStatus>,
    ) -> Option<EnemyStatusBreakdown> {
        if !base_multiplier.is_finite() || base_multiplier < 0.0 {
            return None;
        }
        let conditional_multiplier = self.correction(np_level)?;
        let active = assumption == Some(self.condition);
        let applied_multiplier = if active { conditional_multiplier } else { 1.0 };
        let effective_multiplier = base_multiplier * applied_multiplier;
        effective_multiplier
            .is_finite()
            .then_some(EnemyStatusBreakdown {
                condition: self.condition,
                active,
                base_multiplier,
                conditional_multiplier,
                applied_multiplier,
                effective_multiplier,
            })
    }
}

//! Manual enemy-trait assumption for `damageNpIndividual`.
//!
//! Chaldea at daf4e0af5ec6b4ae42f3e9410b26ae143aa3d725, damage.dart:125-134
//! matches Target against target.getTraits() with checkSignedIndivPartialMatch.
//! Correction/1000 is a separate NP-specific multiplier, before endpoint floors.

use serde::{Deserialize, Serialize};

/// Signed Atlas Target identity. Only audited positive targets are supported;
/// negative conditions have no samples in the audited NA catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TraitCondition {
    pub source_target: i32,
}

impl TraitCondition {
    pub fn from_source_target(source_target: i32) -> Option<Self> {
        let condition = Self { source_target };
        condition.is_valid().then_some(condition)
    }

    pub fn is_valid(self) -> bool {
        trait_label(self.source_target).is_some()
    }

    pub fn label(self) -> &'static str {
        trait_label(self.source_target).unwrap_or("Unknown trait")
    }

    pub const fn is_absence(self) -> bool {
        self.source_target < 0
    }

    pub fn assumption_label(self) -> String {
        format!(
            "Enemy {} the {} trait at NP damage time",
            if self.is_absence() {
                "does not have"
            } else {
                "has"
            },
            self.label()
        )
    }
}

/// Display names retain the exact primary Atlas individuality identity.
fn trait_label(target: i32) -> Option<&'static str> {
    Some(match target {
        1 => "Male",
        2 => "Female",
        103 => "Rider class",
        200 => "Sky attribute",
        201 => "Earth attribute",
        202 => "Human attribute",
        300 => "Lawful",
        301 => "Chaotic",
        303 => "Good",
        304 => "Evil",
        1132 => "Oni",
        1172 => "Threat to Humanity",
        2000 => "Divine",
        2002 => "Dragon",
        2007 => "Saberface",
        2008 => "Weak to Enuma Elish",
        2009 => "Riding",
        2010 => "Arthur",
        2011 => "Sky or Earth Servant",
        2012 => "Brynhild's Beloved",
        2019 => "Demonic",
        2075 => "Saber class Servant",
        2076 => "Super Giant",
        2113 => "King",
        2666 => "Giant",
        2858 => "Standard class Servant",
        2467 => "Weakness Detected",
        3103 => "Non-Protectee of BB Dubai",
        _ => return None,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TraitBonusScaling {
    pub condition: TraitCondition,
    /// Atlas Correction in thousandths, per NP level for one explicit OC row.
    pub source_corrections: [u32; 5],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TraitBonusBreakdown {
    pub condition: TraitCondition,
    pub active: bool,
    pub base_multiplier: f64,
    pub conditional_multiplier: f64,
    pub applied_multiplier: f64,
    pub effective_multiplier: f64,
}

impl TraitBonusScaling {
    pub fn is_valid(&self) -> bool {
        self.condition.is_valid()
            && self
                .source_corrections
                .iter()
                .all(|correction| *correction > 0)
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
        assumption: Option<TraitCondition>,
    ) -> Option<TraitBonusBreakdown> {
        if !base_multiplier.is_finite() || base_multiplier < 0.0 {
            return None;
        }
        let conditional_multiplier = self.correction(np_level)?;
        let active = assumption == Some(self.condition);
        let applied_multiplier = if active { conditional_multiplier } else { 1.0 };
        let effective_multiplier = base_multiplier * applied_multiplier;
        effective_multiplier
            .is_finite()
            .then_some(TraitBonusBreakdown {
                condition: self.condition,
                active,
                base_multiplier,
                conditional_multiplier,
                applied_multiplier,
                effective_multiplier,
            })
    }
}

//! NP-specific import classification and damage modifiers.
//!
//! Known but unsupported mechanics stay distinct from ordinary base NP damage.

pub mod defense_pierce;
pub mod space_eresh;

use crate::loader::NoblePhantasmRecord;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpDamageKind {
    Standard,
    TraitBase,
    DefensePierce,
    SpaceEresh,
    HpRatioLow,
    IndividualSum,
    StateIndividualFix,
    MultipleComponents,
    Rarity,
    AndOr,
    Unknown,
}

/// Classifies Atlas damage functions without implying that every kind is supported.
pub fn classify(function_type: &str, servant_id: u32) -> NpDamageKind {
    match function_type {
        "damageNp" => NpDamageKind::Standard,
        "damageNpIndividual" => NpDamageKind::TraitBase,
        "damageNpPierce" => NpDamageKind::DefensePierce,
        "damageNpBattlePointPhase" if servant_id == space_eresh::SERVANT_ID => {
            NpDamageKind::SpaceEresh
        }
        "damageNpHpratioLow" => NpDamageKind::HpRatioLow,
        "damageNpIndividualSum" => NpDamageKind::IndividualSum,
        "damageNpStateIndividualFix" => NpDamageKind::StateIndividualFix,
        "damageNpRare" => NpDamageKind::Rarity,
        "damageNpAndOrCheckIndividuality" => NpDamageKind::AndOr,
        _ => NpDamageKind::Unknown,
    }
}

/// The importer passes only damage functions, retaining multi-component rejection.
pub fn classify_functions(function_types: &[&str], servant_id: u32) -> NpDamageKind {
    match function_types {
        [] => NpDamageKind::Unknown,
        [function_type] => classify(function_type, servant_id),
        _ => NpDamageKind::MultipleComponents,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NpModifiers {
    pub multiplier: f64,
    pub ignore_defense: bool,
}

/// Modifiers apply to this NP's damage, never to later cards or the Extra attack.
pub fn modifiers(np: &NoblePhantasmRecord, affection_level: u8) -> NpModifiers {
    let mut modifiers = np.affection.as_ref().map_or(
        NpModifiers {
            multiplier: 1.0,
            ignore_defense: false,
        },
        |scaling| scaling.modifiers(affection_level),
    );
    modifiers.ignore_defense |= np.defense_pierce;
    modifiers
}

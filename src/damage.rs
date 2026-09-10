//! The platform-independent calculation core.
//!
//! This module deliberately implements only a small, documented starting point.
//! Extend the input and calculation stages as FGO-specific rules are verified.

/// The type of command card used for an attack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardType {
    Buster,
    Arts,
    Quick,
}

impl CardType {
    /// Base card multiplier used by this early calculator model.
    pub const fn base_multiplier(self) -> f64 {
        match self {
            Self::Buster => 1.5,
            Self::Arts => 1.0,
            Self::Quick => 0.8,
        }
    }
}

/// All values needed for one basic, non-critical normal-card calculation.
#[derive(Debug, Clone, Copy)]
pub struct DamageInput {
    pub attack: u32,
    pub card_type: CardType,
    /// Additive attack buff, expressed as a decimal: `0.20` means +20%.
    pub attack_buff: f64,
    /// Additive card-type buff, expressed as a decimal: `0.30` means +30%.
    pub card_buff: f64,
    /// Enemy defense modifier, expressed as a decimal: `0.10` means 10% defense.
    pub enemy_defense: f64,
    pub class_multiplier: f64,
    pub attribute_multiplier: f64,
}

/// A transparent result suitable for a UI breakdown.
#[derive(Debug, Clone, Copy)]
pub struct DamageResult {
    pub base_card_multiplier: f64,
    pub attack_multiplier: f64,
    pub card_buff_multiplier: f64,
    pub defense_multiplier: f64,
    pub class_multiplier: f64,
    pub attribute_multiplier: f64,
    pub damage_before_random: f64,
    pub minimum_damage: u32,
    pub maximum_damage: u32,
}

/// Calculates an intentionally small normal-card damage model.
///
/// The random range is 90% through 110% of damage before randomness. Values
/// are floored at the final step. This is a starting point, not yet a complete
/// implementation of every in-game rounding and modifier rule.
pub fn calculate(input: DamageInput) -> DamageResult {
    let base_card_multiplier = input.card_type.base_multiplier();
    let attack_multiplier = 1.0 + input.attack_buff;
    let card_buff_multiplier = 1.0 + input.card_buff;
    let defense_multiplier = (1.0 - input.enemy_defense).max(0.0);

    let damage_before_random = input.attack as f64
        * base_card_multiplier
        * attack_multiplier
        * card_buff_multiplier
        * defense_multiplier
        * input.class_multiplier
        * input.attribute_multiplier;

    DamageResult {
        base_card_multiplier,
        attack_multiplier,
        card_buff_multiplier,
        defense_multiplier,
        class_multiplier: input.class_multiplier,
        attribute_multiplier: input.attribute_multiplier,
        damage_before_random,
        minimum_damage: (damage_before_random * 0.9).floor() as u32,
        maximum_damage: (damage_before_random * 1.1).floor() as u32,
    }
}

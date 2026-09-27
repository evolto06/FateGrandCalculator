//! The platform-independent calculation core.
//!
//! This module covers one non-critical first-position command card, without
//! chain bonuses, special damage, flat damage, or Noble Phantasms.
use crate::model::{CardType, ClassType};

const ATTACK_RATE: f64 = 0.23;
const MIN_RANDOM_MODIFIER: f64 = 0.9;
const MAX_RANDOM_MODIFIER: f64 = 1.099;

/// Converts a UI entry such as `20` for 20% to the formula's `0.20` modifier.
pub fn percent_to_modifier(percent: f64) -> f64 {
    percent / 100.0
}

/// All values needed for one basic, non-critical normal-card calculation.
#[derive(Debug, Clone, Copy)]
pub struct DamageInput {
    pub attack: u32,
    pub card_type: CardType,
    pub attacker_class: ClassType,
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
    pub first_card_bonus: f64,
    pub card_damage_multiplier: f64,
    pub class_attack_multiplier: f64,
    pub attack_defense_multiplier: f64,
    pub card_buff_multiplier: f64,
    pub class_multiplier: f64,
    pub attribute_multiplier: f64,
    pub damage_before_random: f64,
    pub minimum_damage: u32,
    pub maximum_damage: u32,
}

/// Calculates a first-position, non-critical command card using the relevant
/// terms of FGO's damage formula.
///
/// The game's random modifier is an integer from 900 through 1099 per 1000.
/// Damage is floored after applying the random modifier. This scoped model does
/// not cover card chains, later card positions, Noble Phantasms, or special buffs.
pub fn calculate(input: DamageInput) -> DamageResult {
    let base_card_multiplier = input.card_type.base_multiplier();
    let first_card_bonus = input.card_type.first_card_bonus();
    let class_attack_multiplier = input.attacker_class.class_default_multiplier();
    let card_buff_multiplier = (1.0 + input.card_buff).max(0.0);
    let attack_defense_multiplier = (1.0 + input.attack_buff - input.enemy_defense).max(0.0);
    let card_damage_multiplier = first_card_bonus + base_card_multiplier * card_buff_multiplier;

    let damage_before_random = input.attack as f64
        * ATTACK_RATE
        * card_damage_multiplier
        * class_attack_multiplier
        * input.class_multiplier
        * input.attribute_multiplier
        * attack_defense_multiplier;

    DamageResult {
        base_card_multiplier,
        first_card_bonus,
        card_damage_multiplier,
        class_attack_multiplier,
        attack_defense_multiplier,
        card_buff_multiplier,
        class_multiplier: input.class_multiplier,
        attribute_multiplier: input.attribute_multiplier,
        damage_before_random,
        minimum_damage: (damage_before_random * MIN_RANDOM_MODIFIER).floor() as u32,
        maximum_damage: (damage_before_random * MAX_RANDOM_MODIFIER).floor() as u32,
    }
}

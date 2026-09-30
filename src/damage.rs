//! The platform-independent calculation core.
//!
//! Non-critical command cards and base NP damage, including ordered three-card
//! selections. Every card and Extra attack has its own range.
use crate::model::{CardType, ClassType};
use crate::np_mechanics::{self, defense_pierce::effective_defense};

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

/// Decimal modifiers (0.20 means +20%). Criticals and automatic skill effects are outside this model.
#[derive(Debug, Clone, Copy, Default)]
pub struct TurnBuffs {
    pub attack_buff: f64,
    pub buster_buff: f64,
    pub arts_buff: f64,
    pub quick_buff: f64,
    pub np_damage_buff: f64,
    pub enemy_defense: f64,
}

#[derive(Debug, Clone)]
pub struct TurnDamageResult {
    pub cards: [DamageResult; 3],
    pub extra: Option<DamageResult>,
    pub notes: Vec<String>,
}

/// Each result is damage to one enemy, with no aggregate damage result.
/// Atlas formula: https://apps.atlasacademy.io/fgo-docs/deeper/battle/damage.html
pub fn calculate_turn(
    servant: &crate::loader::ServantRecord,
    selection: &crate::model::TurnSelection,
    buffs: TurnBuffs,
    enemy_class: ClassType,
    enemy_attribute: crate::model::AttributeType,
) -> Result<TurnDamageResult, String> {
    use crate::model::SelectedCard;
    selection.validate(servant)?;
    if [
        buffs.attack_buff,
        buffs.buster_buff,
        buffs.arts_buff,
        buffs.quick_buff,
        buffs.np_damage_buff,
        buffs.enemy_defense,
    ]
    .iter()
    .any(|value| !value.is_finite())
    {
        return Err("Damage modifiers must be finite numbers.".into());
    }
    let colors = selection
        .slots
        .map(|card| card.card_type(servant).expect("validated card"));
    let color_chain = colors.iter().all(|color| *color == colors[0]);
    let mighty_chain = colors.contains(&CardType::Buster)
        && colors.contains(&CardType::Arts)
        && colors.contains(&CardType::Quick);
    let first_bonus = if mighty_chain || colors[0] == CardType::Buster {
        0.5
    } else {
        0.0
    };
    let buster_chain = color_chain && colors[0] == CardType::Buster;
    let mut notes = Vec::new();
    let cards = std::array::from_fn(|position| {
        let color = colors[position];
        let (np_multiplier, np_buff, position_multiplier, bonus, flat, ignore_defense) =
            match selection.slots[position] {
                SelectedCard::Normal(_) => (
                    1.0,
                    1.0,
                    1.0 + 0.2 * position as f64,
                    first_bonus,
                    if buster_chain {
                        servant.attack as f64 * 0.2
                    } else {
                        0.0
                    },
                    false,
                ),
                SelectedCard::NoblePhantasm(index) => {
                    let np = &servant.noble_phantasms[index];
                    notes.extend(np.notes.iter().cloned());
                    let modifiers = np_mechanics::modifiers(np, selection.affection_level);
                    (
                        np.multipliers[selection.np_level as usize - 1] * modifiers.multiplier,
                        (1.0 + buffs.np_damage_buff).max(0.001),
                        1.0,
                        0.0,
                        0.0,
                        modifiers.ignore_defense,
                    )
                }
            };
        let card_buff = match color {
            CardType::Buster => buffs.buster_buff,
            CardType::Arts => buffs.arts_buff,
            CardType::Quick => buffs.quick_buff,
            CardType::Extra => 0.0,
        };
        turn_card(
            servant,
            TurnBuffs {
                enemy_defense: effective_defense(buffs.enemy_defense, ignore_defense),
                ..buffs
            },
            enemy_class,
            enemy_attribute,
            color.base_multiplier() * position_multiplier,
            bonus,
            card_buff,
            np_multiplier * np_buff,
            flat,
        )
    });
    // All three selected cards belong to the same servant, so every valid selection is a Brave Chain.
    let extra = turn_card(
        servant,
        buffs,
        enemy_class,
        enemy_attribute,
        1.0,
        first_bonus,
        0.0,
        if color_chain { 3.5 } else { 2.0 },
        0.0,
    );
    Ok(TurnDamageResult {
        cards,
        extra: Some(extra),
        notes,
    })
}

#[allow(clippy::too_many_arguments)]
fn turn_card(
    servant: &crate::loader::ServantRecord,
    buffs: TurnBuffs,
    enemy_class: ClassType,
    enemy_attribute: crate::model::AttributeType,
    base: f64,
    first: f64,
    card_buff: f64,
    special_multiplier: f64,
    flat: f64,
) -> DamageResult {
    let card_buff_multiplier = (1.0 + card_buff).max(0.0);
    let card_damage_multiplier = first + base * card_buff_multiplier;
    let class_attack_multiplier = servant.class.class_default_multiplier();
    let class_multiplier = servant.class.affinity_against(enemy_class);
    let attribute_multiplier = servant.attribute.affinity_against(enemy_attribute);
    let attack_defense_multiplier = (1.0 + buffs.attack_buff - buffs.enemy_defense).max(0.0);
    let damage_before_random = servant.attack as f64
        * ATTACK_RATE
        * card_damage_multiplier
        * class_attack_multiplier
        * class_multiplier
        * attribute_multiplier
        * attack_defense_multiplier
        * special_multiplier;
    DamageResult {
        base_card_multiplier: base,
        first_card_bonus: first,
        card_damage_multiplier,
        class_attack_multiplier,
        attack_defense_multiplier,
        card_buff_multiplier,
        class_multiplier,
        attribute_multiplier,
        damage_before_random,
        minimum_damage: (damage_before_random * MIN_RANDOM_MODIFIER + flat).floor() as u32,
        maximum_damage: (damage_before_random * MAX_RANDOM_MODIFIER + flat).floor() as u32,
    }
}

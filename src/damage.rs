//! The platform-independent calculation core.
//!
//! Non-critical command cards and base NP damage, including ordered three-card
//! selections. Every card and Extra attack has its own range.
use crate::loader::ServantRecord;
use crate::model::{
    AttributeType, CardType, CheckedCard, ClassType, NpComponentFactors, TurnSelection,
};
use crate::np_mechanics::{self, NpModifiers, defense_pierce::effective_defense};

const ATTACK_RATE: f64 = 0.23;
const MIN_RANDOM_MODIFIER: f64 = 0.9;
const MAX_RANDOM_MODIFIER: f64 = 1.099;
// Atlas npdamage buff-action cap: +500%, before the formula's 1 + buff term.
const MAX_NP_DAMAGE_BUFF: f64 = 5.0;

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
    /// Post-random flat addition (normal Buster-chain cards only).
    pub flat_damage_addition: f64,
    /// Extra attack factor, already included in `damage_before_random`.
    pub extra_attack_multiplier: f64,
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
/// It shares [`card_damage`] with [`calculate_turn`].
pub fn calculate(input: DamageInput) -> DamageResult {
    card_damage(
        Attacker {
            attack: input.attack,
            class_attack_multiplier: input.attacker_class.class_default_multiplier(),
            class_multiplier: input.class_multiplier,
            attribute_multiplier: input.attribute_multiplier,
        },
        CardTerms {
            base: input.card_type.base_multiplier(),
            first: input.card_type.first_card_bonus(),
            card_buff: input.card_buff,
            attack_buff: input.attack_buff,
            enemy_defense: input.enemy_defense,
            special: 1.0,
            flat: 0.0,
        },
    )
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

impl TurnBuffs {
    fn validate(&self) -> Result<(), String> {
        let values = [
            self.attack_buff,
            self.buster_buff,
            self.arts_buff,
            self.quick_buff,
            self.np_damage_buff,
            self.enemy_defense,
        ];
        if values.iter().all(|value| value.is_finite()) {
            Ok(())
        } else {
            Err("Damage modifiers must be finite numbers.".into())
        }
    }

    fn card_buff(&self, color: CardType) -> f64 {
        match color {
            CardType::Buster => self.buster_buff,
            CardType::Arts => self.arts_buff,
            CardType::Quick => self.quick_buff,
            CardType::Extra => 0.0,
        }
    }

    /// The NP damage term `1 + buff`, with the buff capped at Atlas's +500%.
    fn np_damage_multiplier(&self) -> f64 {
        (1.0 + self.np_damage_buff.min(MAX_NP_DAMAGE_BUFF)).max(0.001)
    }
}

#[derive(Debug, Clone)]
pub struct TurnDamageResult {
    pub cards: [DamageResult; 3],
    /// Ordered NP function results for each slot; normal-card slots are empty.
    pub np_components: [Vec<DamageResult>; 3],
    pub extra: Option<DamageResult>,
    pub notes: Vec<String>,
}

/// Each result is damage to one enemy, with no aggregate damage result.
/// Atlas formula: https://apps.atlasacademy.io/fgo-docs/deeper/battle/damage.html
pub fn calculate_turn(
    servant: &ServantRecord,
    selection: &TurnSelection,
    buffs: TurnBuffs,
    enemy_class: ClassType,
    enemy_attribute: AttributeType,
) -> Result<TurnDamageResult, String> {
    let checked = selection.check(servant)?;
    buffs.validate()?;
    let attacker = Attacker::new(servant, enemy_class, enemy_attribute);
    let chain = Chain::new(checked.cards.each_ref().map(CheckedCard::card_type));

    let mut notes = Vec::new();
    let mut np_components: [Vec<DamageResult>; 3] = std::array::from_fn(|_| Vec::new());
    let cards = std::array::from_fn(|position| match &checked.cards[position] {
        CheckedCard::Normal(color) => normal_card(attacker, buffs, &chain, *color, position),
        CheckedCard::NoblePhantasm { np, components } => {
            notes.extend(np.notes.iter().cloned());
            let modifiers = np_mechanics::modifiers(np, selection.affection_level);
            let (total, parts) = np_card(attacker, buffs, np.card_type, modifiers, components);
            np_components[position] = parts;
            total
        }
    });

    Ok(TurnDamageResult {
        cards,
        np_components,
        extra: Some(extra_attack(attacker, buffs, &chain)),
        notes,
    })
}

/// Chain-wide bonuses that depend only on the three cards' colors.
struct Chain {
    first_bonus: f64,
    color_chain: bool,
    buster_chain: bool,
}

impl Chain {
    fn new(colors: [CardType; 3]) -> Self {
        let color_chain = colors.iter().all(|color| *color == colors[0]);
        let mighty_chain = colors.contains(&CardType::Buster)
            && colors.contains(&CardType::Arts)
            && colors.contains(&CardType::Quick);
        Self {
            first_bonus: if mighty_chain || colors[0] == CardType::Buster {
                0.5
            } else {
                0.0
            },
            color_chain,
            buster_chain: color_chain && colors[0] == CardType::Buster,
        }
    }
}

fn normal_card(
    attacker: Attacker,
    buffs: TurnBuffs,
    chain: &Chain,
    color: CardType,
    position: usize,
) -> DamageResult {
    card_damage(
        attacker,
        CardTerms {
            base: color.base_multiplier() * (1.0 + 0.2 * position as f64),
            first: chain.first_bonus,
            card_buff: buffs.card_buff(color),
            attack_buff: buffs.attack_buff,
            enemy_defense: buffs.enemy_defense,
            special: 1.0,
            flat: if chain.buster_chain {
                attacker.attack as f64 * 0.2
            } else {
                0.0
            },
        },
    )
}

/// Returns the NP's total and one result per damage function. Each function is
/// floored on its own before summing; flooring the combined multiplier would
/// overstate endpoints for some inputs.
fn np_card(
    attacker: Attacker,
    buffs: TurnBuffs,
    color: CardType,
    modifiers: NpModifiers,
    components: &[NpComponentFactors],
) -> (DamageResult, Vec<DamageResult>) {
    let terms = CardTerms {
        base: color.base_multiplier(),
        first: 0.0,
        card_buff: buffs.card_buff(color),
        attack_buff: buffs.attack_buff,
        enemy_defense: effective_defense(buffs.enemy_defense, modifiers.ignore_defense),
        special: 1.0,
        flat: 0.0,
    };
    let np_damage = buffs.np_damage_multiplier();
    let parts: Vec<_> = components
        .iter()
        .map(|factors| {
            card_damage(
                attacker,
                CardTerms {
                    special: factors.multiplier
                        * modifiers.multiplier
                        * factors.enemy_status
                        * factors.trait_bonus
                        * np_damage,
                    ..terms
                },
            )
        })
        .collect();

    let mut total = card_damage(attacker, terms);
    total.damage_before_random = parts.iter().map(|part| part.damage_before_random).sum();
    total.flat_damage_addition = parts.iter().map(|part| part.flat_damage_addition).sum();
    total.minimum_damage = parts
        .iter()
        .fold(0_u32, |sum, part| sum.saturating_add(part.minimum_damage));
    total.maximum_damage = parts
        .iter()
        .fold(0_u32, |sum, part| sum.saturating_add(part.maximum_damage));
    (total, parts)
}

fn extra_attack(attacker: Attacker, buffs: TurnBuffs, chain: &Chain) -> DamageResult {
    // All three selected cards belong to the same servant, so every valid selection is a Brave Chain.
    let extra_attack_multiplier = if chain.color_chain { 3.5 } else { 2.0 };
    let mut extra = card_damage(
        attacker,
        CardTerms {
            base: 1.0,
            first: chain.first_bonus,
            card_buff: 0.0,
            attack_buff: buffs.attack_buff,
            enemy_defense: buffs.enemy_defense,
            special: extra_attack_multiplier,
            flat: 0.0,
        },
    );
    extra.extra_attack_multiplier = extra_attack_multiplier;
    extra
}

/// Attacker-side terms shared by every card the servant plays this turn.
#[derive(Debug, Clone, Copy)]
struct Attacker {
    attack: u32,
    class_attack_multiplier: f64,
    class_multiplier: f64,
    attribute_multiplier: f64,
}

impl Attacker {
    fn new(
        servant: &ServantRecord,
        enemy_class: ClassType,
        enemy_attribute: AttributeType,
    ) -> Self {
        Self {
            attack: servant.attack,
            class_attack_multiplier: servant.class.class_default_multiplier(),
            class_multiplier: servant.class.affinity_against(enemy_class),
            attribute_multiplier: servant.attribute.affinity_against(enemy_attribute),
        }
    }
}

/// Terms that vary from card to card.
#[derive(Debug, Clone, Copy)]
struct CardTerms {
    /// Card color multiplier, including any position bonus.
    base: f64,
    /// First-card bonus added outside the card buff multiplier.
    first: f64,
    card_buff: f64,
    attack_buff: f64,
    enemy_defense: f64,
    /// NP rate, NP-specific corrections or the Extra attack factor.
    special: f64,
    /// Flat damage added after the random modifier.
    flat: f64,
}

/// The single implementation of the damage formula.
fn card_damage(attacker: Attacker, card: CardTerms) -> DamageResult {
    let card_buff_multiplier = (1.0 + card.card_buff).max(0.0);
    let card_damage_multiplier = card.first + card.base * card_buff_multiplier;
    let attack_defense_multiplier = (1.0 + card.attack_buff - card.enemy_defense).max(0.0);
    let damage_before_random = attacker.attack as f64
        * ATTACK_RATE
        * card_damage_multiplier
        * attacker.class_attack_multiplier
        * attacker.class_multiplier
        * attacker.attribute_multiplier
        * attack_defense_multiplier
        * card.special;
    DamageResult {
        base_card_multiplier: card.base,
        first_card_bonus: card.first,
        card_damage_multiplier,
        class_attack_multiplier: attacker.class_attack_multiplier,
        attack_defense_multiplier,
        card_buff_multiplier,
        class_multiplier: attacker.class_multiplier,
        attribute_multiplier: attacker.attribute_multiplier,
        flat_damage_addition: card.flat,
        extra_attack_multiplier: 1.0,
        damage_before_random,
        minimum_damage: (damage_before_random * MIN_RANDOM_MODIFIER + card.flat).floor() as u32,
        maximum_damage: (damage_before_random * MAX_RANDOM_MODIFIER + card.flat).floor() as u32,
    }
}

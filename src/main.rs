use fate_grand_calculator::damage::{CardType, DamageInput, calculate};

fn main() {
    let result = calculate(DamageInput {
        attack: 12_000,
        card_type: CardType::Buster,
        attack_buff: 0.20,
        card_buff: 0.30,
        enemy_defense: 0.10,
        class_multiplier: 2.0,
        attribute_multiplier: 1.0,
    });

    println!("FateGrandCalculator");
    println!(
        "Damage range: {}–{} (before random: {:.2})",
        result.minimum_damage, result.maximum_damage, result.damage_before_random
    );
}

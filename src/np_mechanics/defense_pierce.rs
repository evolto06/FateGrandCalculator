//! Defense piercing ignores Defense Up and retains the damage bonus from Defense Down.

pub fn effective_defense(enemy_defense: f64, ignore_defense: bool) -> f64 {
    if ignore_defense {
        enemy_defense.min(0.0)
    } else {
        enemy_defense
    }
}
